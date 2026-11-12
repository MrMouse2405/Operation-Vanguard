//! Sequential router VMs and deterministic synchronization. Lua never owns the
//! world: bindings see a committed snapshot plus their router's reservations.
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::Path,
    rc::Rc,
};

use mlua::chunk::ChunkMode;
use mlua::{Function, HookTriggers, Lua, LuaOptions, MultiValue, StdLib, Table, Thread, Value as LValue, VmState};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::sim::{self, Config, Core};

const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_PENDING: usize = 100_000;
const MAX_SAFE_ID: u64 = 9_007_199_254_740_991;

#[derive(Clone)]
struct Handler { function: Function, source: String }
#[derive(Default)]
struct Registrations { direct: Option<Handler>, topics: BTreeMap<String, Handler>, discarded:BTreeSet<String> }
#[derive(Clone, Debug)]
struct Message {
    id: u64, sender: u64, source: String, topic: Option<String>,
    payload: Value, sent_at: u64, recipients: Vec<u64>,
}
#[derive(Default)]
struct Bus {
    next_id: u64,
    outgoing: Vec<Message>,
    membership: BTreeMap<String, BTreeSet<u64>>,
    synchronized_fuel: BTreeMap<u64, u64>,
    topics: BTreeMap<String, Value>,
}

#[derive(Default)]
struct Budget { remaining: Cell<u64>, total: Cell<u64>, quantum: Cell<u64> }
impl Budget {
    fn charge(&self, amount: u64) -> bool {
        if self.remaining.get() < amount { return false; }
        self.remaining.set(self.remaining.get() - amount);
        self.total.set(self.total.get().saturating_add(amount));
        true
    }
}

enum Wait { Runnable, Tick(u64), Task(u64), Any(Vec<u64>) }
struct Coroutine {
    thread: Thread, source: String, wait: Wait, initial: Option<MultiValue>,
    message: Option<(u64,u64)>,
}
struct Vm {
    id: u64, lua: Lua, budget: Rc<Budget>, handlers: Rc<RefCell<Registrations>>,
    runnable: VecDeque<Coroutine>, pending: VecDeque<Message>, errors: u64, last_fuel:u64,
    memory:crate::memory::AllocationMeter,
}

/// A consistent disk source set. Every dependency is staged before VM creation.
pub struct Staged {
    sources: BTreeMap<String, String>,
    entry: String,
}
impl Staged {
    pub fn load(config: &Config) -> Result<Self, String> {
        let entry=config.script.as_ref().ok_or("no agent supplied; use --script agent.lua --network network.toml")?;
        let root = fs::canonicalize(&config.script_root)
            .map_err(|e| format!("script root '{}': {e}", config.script_root))?;
        let mut sources = BTreeMap::new();
        fn visit(root: &Path, path: &Path, sources: &mut BTreeMap<String,String>) -> Result<(),String> {
            let mut paths = fs::read_dir(path).map_err(|e| e.to_string())?
                .map(|entry| entry.map(|e| e.path())).collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
            paths.sort();
            for path in paths {
                if path.file_name().is_some_and(|n| {let n=n.to_string_lossy();n.starts_with('.')||matches!(n.as_ref(),"node_modules"|"target"|"out")}) {continue;}
                if path.symlink_metadata().map_err(|e| e.to_string())?.file_type().is_symlink() {
                    return Err(format!("symlinks are not allowed in staged script root: {}", path.display()));
                }
                if path.is_dir() { visit(root, &path, sources)?; }
                else if path.extension().is_some_and(|e| e == "lua") {
                    if sources.len() >= 256 { return Err("LIMIT_EXCEEDED: at most 256 staged Lua files".into()); }
                    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                    if bytes.len() > MAX_SOURCE_BYTES { return Err(format!("{}: source exceeds 1 MiB", path.display())); }
                    let name = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                    sources.insert(name, String::from_utf8(bytes).map_err(|e| e.to_string())?);
                }
            }
            Ok(())
        }
        visit(&root,&root,&mut sources)?;
        for (name,source) in &sources{if source.len() as u64+config.native_call_cost>config.fuel_per_tick{return Err(format!("{name}: staged module charge exceeds a full tick allowance"));}}
        if !sources.contains_key(entry) { return Err(format!("entry '{entry}' is absent from script root {}",root.display())); }
        Ok(Self { sources, entry:entry.clone() })
    }
}

fn lua_error(message: impl Into<String>) -> mlua::Error { mlua::Error::runtime(message.into()) }
fn positive(value: Option<&LValue>) -> mlua::Result<u64> {
    match value {
        Some(LValue::Integer(n)) if *n > 0 && (*n as u64) <= MAX_SAFE_ID => Ok(*n as u64),
        _ => Err(lua_error("INVALID_ARGUMENT: expected a positive integer")),
    }
}
fn topic(value: Option<&LValue>) -> mlua::Result<String> {
    let Some(LValue::String(s)) = value else { return Err(lua_error("INVALID_ARGUMENT: expected topic string")); };
    let s = s.to_str()?.to_string();
    if s.is_empty() || s.len()>128 { return Err(lua_error("INVALID_ARGUMENT: topic must be 1..128 bytes")); }
    Ok(s)
}
fn from_json(lua: &Lua, value: &Value) -> mlua::Result<LValue> {
    Ok(match value {
        Value::Null => LValue::Nil,
        Value::Bool(v) => LValue::Boolean(*v),
        Value::Number(v) => if let Some(v)=v.as_i64() { LValue::Integer(v) } else { LValue::Number(v.as_f64().unwrap()) },
        Value::String(v) => LValue::String(lua.create_string(v)?),
        Value::Array(values) => { let table=lua.create_table()?; for (i,v) in values.iter().enumerate() { table.raw_set(i+1,from_json(lua,v)?)?; } LValue::Table(table) },
        Value::Object(values) => { let table=lua.create_table()?; for (k,v) in values { table.raw_set(k.as_str(),from_json(lua,v)?)?; } LValue::Table(table) },
    })
}

/// Strict copied JSON tree. Traversal bounds apply before allocating output.
fn to_json(value: &LValue, max: usize) -> mlua::Result<Value> {
    fn walk(value:&LValue, seen:&mut BTreeSet<usize>, size:&mut usize,max:usize,depth:usize)->mlua::Result<Value> {
        *size += 8;
        if *size>max || depth>16 { return Err(lua_error("PAYLOAD_TOO_LARGE: bounded payload exceeded")); }
        Ok(match value {
            LValue::Boolean(v)=>json!(v),
            LValue::Integer(v)=>json!(v),
            LValue::Number(v) if v.is_finite()=>json!(v),
            LValue::String(v)=>{ *size+=v.as_bytes().len(); if *size>max { return Err(lua_error("PAYLOAD_TOO_LARGE: string exceeded bound")); } json!(v.to_str()?.as_ref()) },
            LValue::Table(t)=>{
                if t.metatable().is_some() { return Err(lua_error("INVALID_ARGUMENT: payload metatables are not allowed")); }
                let ptr=t.to_pointer() as usize;
                if !seen.insert(ptr) { return Err(lua_error("INVALID_ARGUMENT: cyclic payload")); }
                let mut array=BTreeMap::new(); let mut object=BTreeMap::new();
                for pair in t.clone().pairs::<LValue,LValue>() {
                    let (key,value)=pair?;
                    match key {
                        LValue::Integer(i) if i>0=> {array.insert(i,walk(&value,seen,size,max,depth+1)?);},
                        LValue::String(k)=> { *size+=k.as_bytes().len(); object.insert(k.to_str()?.to_string(),walk(&value,seen,size,max,depth+1)?); },
                        _=>return Err(lua_error("INVALID_ARGUMENT: payload keys must be dense integer indices or strings")),
                    }
                }
                seen.remove(&ptr);
                if !array.is_empty() {
                    if !object.is_empty() || array.keys().copied().ne(1..=array.len() as i64) { return Err(lua_error("INVALID_ARGUMENT: mixed or sparse payload table")); }
                    Value::Array(array.into_values().collect())
                } else { json!(object) }
            },
            _=>return Err(lua_error("INVALID_ARGUMENT: unsupported payload value")),
        })
    }
    walk(value,&mut BTreeSet::new(),&mut 0,max,0)
}

fn log_value(value:&LValue,seen:&mut BTreeSet<usize>,remaining:&mut usize,depth:usize)->Value {
    if *remaining==0 || depth>6 { return json!({"type":"truncated","value":"…"}); }
    *remaining-=1;
    match value {
        LValue::Nil=>json!({"type":"nil","value":null}),
        LValue::Boolean(v)=>json!({"type":"boolean","value":v}),
        LValue::Integer(v)=>json!({"type":"number","value":v}),
        LValue::Number(v)=>json!({"type":"number","value":if v.is_finite(){json!(v)}else{json!(v.to_string())}}),
        LValue::String(v)=>json!({"type":"string","value":v.to_string_lossy().chars().take(1024).collect::<String>()}),
        LValue::Table(t)=>{
            let ptr=t.to_pointer() as usize;
            if !seen.insert(ptr) {return json!({"type":"cycle","value":"<cycle>"});}
            let mut entries=BTreeMap::new();
            let mut ordered=BTreeMap::new();
            for (index,(k,v)) in t.clone().pairs::<LValue,LValue>().flatten().enumerate() {
                if index>=64 {seen.remove(&ptr);return json!({"type":"truncated","value":"<table exceeds 64 entries>"});}
                let key=match k {LValue::String(s)=>format!("string:{}",s.to_string_lossy()),LValue::Integer(n)=>format!("integer:{n}"),LValue::Boolean(b)=>format!("boolean:{b}"),_=>continue};
                ordered.insert(key,v);
            }
            for (key,value) in ordered.into_iter().take(64) {
                entries.insert(key,log_value(&value,seen,remaining,depth+1));
            }
            seen.remove(&ptr);
            json!({"type":"table","value":entries})
        },
        other=>json!({"type":other.type_name(),"value":format!("<{}>",other.type_name())}),
    }
}

fn source(lua:&Lua,fallback:&str)->String {
    for level in 0..16 {
        if let Some(Some(name))=lua.inspect_stack(level,|d|d.source().source.map(|s|s.into_owned())) {
            let name=name.trim_start_matches('@');
            if name.ends_with(".lua") && name!="sandbox.lua" && name!="table.lua" { return name.to_string(); }
        }
    }
    fallback.to_string()
}

impl Vm {
    fn new(id:u64,_router:u64,core:Rc<RefCell<Core>>,bus:Rc<RefCell<Bus>>,debug:Rc<RefCell<crate::debug::Store>>,staged:&Staged)->Result<Self,String> {
        let config=core.borrow().config.clone();
        let lua=Lua::new_with(StdLib::TABLE|StdLib::STRING|StdLib::MATH|StdLib::COROUTINE,LuaOptions::default()).map_err(|e|e.to_string())?;
        lua.set_memory_limit(config.memory_limit).map_err(|e|e.to_string())?;
        let memory=crate::memory::AllocationMeter::install(&lua).map_err(|e|e.to_string())?;
        let env=lua.create_table().map_err(|e|e.to_string())?;
        let globals=lua.globals();
        for name in ["assert","error","pcall","xpcall","type","select","tonumber","tostring","rawequal","rawget","rawset","setmetatable","getmetatable"] {
            env.set(name,globals.get::<LValue>(name).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        }
        // ipairs is implemented in Lua to keep iteration yieldable and metered.
        let ipairs:Function=lua.load("return function(t) local function step(t,i) i=i+1; if t[i]~=nil then return i,t[i] end end; return step,t,0 end").eval().map_err(|e|e.to_string())?;
        env.set("ipairs",ipairs).map_err(|e|e.to_string())?;
        let library=lua.create_table().map_err(|e|e.to_string())?;
        for name in ["assert","type","select"] {library.set(name,globals.get::<LValue>(name).unwrap()).unwrap();}
        let table:Table=lua.load(include_str!("table.lua")).set_name("@table.lua").set_environment(library).eval().map_err(|e|e.to_string())?;
        env.set("table",table).map_err(|e|e.to_string())?;
        env.set("_G",env.clone()).map_err(|e|e.to_string())?;
        // GC callbacks and weak tables make execution depend on allocator/GC
        // timing. Copy and seal permitted metamethods so later table mutation
        // cannot enable either facility after this check.
        let setmetatable:Function=lua.load("local setter,next=...; return function(t,mt) if mt==nil then return setter(t,nil) end; if type(mt)~='table' then error('INVALID_ARGUMENT: expected metatable',2) end; local copy={};for k,v in next,mt do if k=='__gc' or k=='__close' or k=='__mode' then error('INVALID_ARGUMENT: GC-dependent metatables are disabled',2) end;copy[k]=v end;copy.__metatable='sealed agent metatable';return setter(t,copy) end").set_environment(env.clone()).call((globals.get::<Function>("setmetatable").unwrap(),globals.get::<Function>("next").unwrap())).map_err(|e|e.to_string())?;
        env.set("setmetatable",setmetatable).unwrap();
        let stable_tostring:Function=lua.load("local raw=...; return function(v) local k=type(v); if k=='table' then local mt=getmetatable(v); if type(mt)=='table' and type(mt.__tostring)=='function' then return mt.__tostring(v) end; return '<table>' elseif k=='function' or k=='thread' or k=='userdata' then return '<'..k..'>' end; return raw(v) end").set_environment(env.clone()).call(globals.get::<Function>("tostring").unwrap()).map_err(|e|e.to_string())?;
        env.set("tostring",stable_tostring).unwrap();
        let budget=Rc::new(Budget::default());
        let handlers=Rc::new(RefCell::new(Registrations::default()));
        let b=budget.clone(); let h=handlers.clone(); let c=core.clone(); let bs=bus.clone(); let e=env.clone();
        let sources=staged.sources.clone();
        let raw_strings:Table=globals.get("string").map_err(|e|e.to_string())?;
        let string_functions=["len","sub","byte","char","lower","upper","reverse","rep","find"].into_iter().map(|name|Ok((name.to_string(),raw_strings.get::<Function>(name)?))).collect::<mlua::Result<BTreeMap<_,_>>>().map_err(|e|e.to_string())?;
        let invoke=lua.create_function(move |lua,(method,args):(String,Table)| {
            let operation=||->mlua::Result<MultiValue>{
            let n=args.get::<usize>("n")?;
            if n>64 {return Err(lua_error("INVALID_ARGUMENT: too many arguments"));}
            let args=(1..=n).map(|i|args.raw_get::<LValue>(i)).collect::<mlua::Result<Vec<_>>>()?;
            let source=source(lua,"<unknown>");
            let base=config.native_call_cost;
            let payload=if method=="tx" || method=="pub" {Some(to_json(args.get(1).unwrap_or(&LValue::Nil),config.max_payload_bytes))}else{None};
            let mut cost=base;
            let debug_args=if method.starts_with("dbg_") {
                if !config.dbg {return Err(lua_error("INVALID_ARGUMENT: dbgguard requires --dbg"));}
                let values=args.iter().map(|v|to_json(v,config.max_payload_bytes)).collect::<mlua::Result<Vec<_>>>()?;
                let bytes=serde_json::to_vec(&values).unwrap().len();
                if bytes>config.max_payload_bytes {return Err(lua_error("PAYLOAD_TOO_LARGE: debug call exceeds payload budget"));}
                cost+=bytes as u64;
                Some(values)
            }else{None};
            if method=="module" {if let Some(LValue::String(name))=args.first(){let path=format!("{}.lua",name.to_string_lossy().replace('.',"/"));cost+=sources.get(&path).map_or(0,|s|s.len() as u64);}}
            if method=="tonumber" {if let Some(LValue::String(value))=args.first(){if value.as_bytes().len()>config.max_payload_bytes{return Err(lua_error("PAYLOAD_TOO_LARGE: number parsing input bound"));}cost+=value.as_bytes().len() as u64;}}
            if let Some(Ok(payload))=&payload {cost+=serde_json::to_vec(payload).unwrap().len() as u64;}
            let log=if method=="print" {
                let mut remaining=256;
                let values=args.iter().take(32).map(|v|log_value(v,&mut BTreeSet::new(),&mut remaining,0)).collect::<Vec<_>>();
                if serde_json::to_vec(&values).unwrap().len()>config.max_payload_bytes {
                    Some(vec![json!({"type":"truncated","value":"<print payload exceeded configured byte bound>"})])
                }else{Some(values)}
            }else{None};
            if let Some(values)=&log{cost+=serde_json::to_vec(values).unwrap().len() as u64;}
            // Check descriptors without creating reservations. Failed admissions
            // pay only the ordinary call cost; success pays admission once.
            let descriptor=if matches!(method.as_str(),"deploy"|"cost"|"can_deploy") {
                Some(to_json(args.first().unwrap_or(&LValue::Nil),config.max_payload_bytes))
            }else{None};
            if method=="deploy" {
                if let Some(Ok(desc))=&descriptor {
                    if c.borrow_mut().api(id,"cost",json!([desc])).is_ok() {
                        let ready=c.borrow_mut().api(id,"can_deploy",json!([desc])).unwrap_or(json!(false));
                        let can=ready.as_bool().unwrap_or_else(||ready.get(0).and_then(Value::as_bool).unwrap_or(false));
                        if can {cost+=config.task_admission_cost;}
                    }
                }
            }
            if method=="string" {
                if args.iter().any(|value|matches!(value,LValue::String(s) if s.as_bytes().len()>config.max_payload_bytes)){return Err(lua_error("PAYLOAD_TOO_LARGE: native string input bound"));}
                cost+=args.iter().map(|v|if let LValue::String(s)=v {s.as_bytes().len() as u64}else{1}).sum::<u64>();
                if matches!(args.first(),Some(LValue::String(s)) if s.to_string_lossy()=="rep") {
                    let count=positive(args.get(2)).unwrap_or(0);
                    let len=if let Some(LValue::String(s))=args.get(1){s.as_bytes().len() as u64}else{0};
                    let separator=if let Some(LValue::String(s))=args.get(3){s.as_bytes().len() as u64}else{0};
                    let size=len.saturating_mul(count).saturating_add(separator.saturating_mul(count.saturating_sub(1)));
                    if size>config.max_payload_bytes as u64 {return Err(lua_error("PAYLOAD_TOO_LARGE: string output bound"));}
                    cost+=size;
                }
            }
            if cost>config.fuel_per_tick {return Err(lua_error("LIMIT_EXCEEDED: atomic native charge exceeds a tick allowance"));}
            if !b.charge(cost) {return Ok(MultiValue::from_vec(vec![LValue::Boolean(false)]));}
            let result:Vec<LValue>=match method.as_str() {
                name if name.starts_with("dbg_")=>{
                    let tick=c.borrow().tick;
                    let value=debug.borrow_mut().call(id,tick,&name[4..],debug_args.as_ref().unwrap()).map_err(lua_error)?;
                    vec![from_json(lua,&value)?]
                },
                "print"=>{c.borrow_mut().emit("log",Some(id),&source,json!({"severity":"info","values":log.unwrap()}));vec![]},
                "rx"=>{match args.first(){Some(LValue::Nil)|None=>h.borrow_mut().direct=None,Some(LValue::Function(f))=>h.borrow_mut().direct=Some(Handler{function:f.clone(),source:source.clone()}),_=>return Err(lua_error("INVALID_ARGUMENT: rx expects function or nil"))};vec![LValue::Boolean(true)]},
                "sub"=>{let t=topic(args.first())?;let Some(LValue::Function(f))=args.get(1)else{return Err(lua_error("INVALID_ARGUMENT: sub expects handler function"));};h.borrow_mut().topics.insert(t.clone(),Handler{function:f.clone(),source:source.clone()});c.borrow_mut().emit("subscription",Some(id),&source,json!({"topic":t,"subscribed":true}));vec![LValue::Boolean(true)]},
                "unsub"=>{let t=topic(args.first())?;let mut handlers=h.borrow_mut();handlers.topics.remove(&t);handlers.discarded.insert(t.clone());c.borrow_mut().emit("subscription",Some(id),&source,json!({"topic":t,"subscribed":false}));vec![LValue::Boolean(true)]},
                "tx"|"pub"=>{
                    let payload=payload.unwrap()?;
                    let tick=c.borrow().tick; let mut bus=bs.borrow_mut();
                    let (recipient,t,recipients)=if method=="tx" {
                        let recipient=positive(args.first())?;
                        if recipient==id {return Err(lua_error("INVALID_ARGUMENT: self delivery is not supported"));}
                        if !c.borrow().agents.contains_key(&recipient) {return Err(lua_error("UNKNOWN_AGENT: recipient does not exist"));}
                        (Some(recipient),None,vec![recipient])
                    } else {let t=topic(args.first())?;let recipients=bus.membership.get(&t).map(|s|s.iter().copied().filter(|v|*v!=id).collect()).unwrap_or_default();(None,Some(t),recipients)};
                    bus.next_id=bus.next_id.checked_add(1).filter(|v|*v<=MAX_SAFE_ID).ok_or_else(||lua_error("LIMIT_EXCEEDED: message ID space exhausted"))?;
                    let message=Message{id:bus.next_id,sender:id,source:source.clone(),topic:t.clone(),payload,sent_at:tick,recipients:recipients.clone()};
                    c.borrow_mut().emit(if t.is_some(){"publication"}else{"message"},Some(id),&source,json!({"message_id":message.id,"sender":id,"recipient":recipient,"topic":t,"payload":message.payload,"sent_at":tick,"recipients":recipients,"outcome":"accepted"}));
                    if let Some(t)=&t {let entry=bus.topics.entry(t.clone()).or_insert(json!({"topic":t,"publication_count":0,"last_tick":tick}));let count=entry["publication_count"].as_u64().unwrap_or(0)+1;entry["publication_count"]=json!(count);entry["last_tick"]=json!(tick);}
                    let ack=if method=="tx"{json!({"message_id":message.id,"sent_at":tick})}else{json!({"message_id":message.id,"sent_at":tick,"recipient_count":recipients.len()})};
                    bus.outgoing.push(message);
                    vec![from_json(lua,&ack)?]
                },
                "fuel"=>{let who=if args.is_empty()||matches!(args[0],LValue::Nil){id}else{positive(args.first())?};if !c.borrow().agents.contains_key(&who){return Err(lua_error("UNKNOWN_AGENT: fuel agent does not exist"));}vec![LValue::Integer(if who==id{b.total.get()}else{*bs.borrow().synchronized_fuel.get(&who).unwrap_or(&0)} as i64)]},
                "has_task"|"is_finished"|"is_ready" if positive(args.first()).is_err()=>vec![LValue::Boolean(false)],
                "validate_wait"=>{positive(args.first())?;vec![LValue::Boolean(true)]},
                "validate_await_any"=>{
                    let value=to_json(args.first().unwrap_or(&LValue::Nil),config.max_payload_bytes)?;
                    let Some(values)=value.as_array()else{return Err(lua_error("INVALID_ARGUMENT: await_any requires a dense nonempty list"));};
                    if values.is_empty(){return Err(lua_error("INVALID_ARGUMENT: await_any requires a nonempty list"));}
                    let mut ids=BTreeSet::new();
                    for value in values {let Some(task)=value.as_u64().filter(|v|*v>0)else{return Err(lua_error("INVALID_ARGUMENT: invalid task ID"));};if c.borrow_mut().api(id,"has_task",json!([task])).ok()!=Some(json!(true)){return Err(lua_error("UNKNOWN_TASK: task not owned by this router"));}ids.insert(task);}
                    vec![from_json(lua,&json!(ids))?]
                },
                "module"=>{
                    let Some(LValue::String(name))=args.first()else{return Err(lua_error("INVALID_ARGUMENT: require expects module name"));};
                    let name=name.to_str()?;
                    if name.is_empty()||!name.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_'||b==b'.')||name.contains("..") {return Err(lua_error("INVALID_ARGUMENT: invalid module path"));}
                    let path=format!("{}.lua",name.replace('.',"/"));
                    let text=sources.get(&path).ok_or_else(||lua_error(format!("INVALID_ARGUMENT: module '{path}' was not staged")))?;
                    vec![LValue::Function(lua.load(text).set_name(format!("@{path}")).set_mode(ChunkMode::Text).set_environment(e.clone()).into_function()?)]
                },
                "string"=>{
                    let Some(LValue::String(name))=args.first()else{return Err(lua_error("INVALID_ARGUMENT: string operation"));};
                    let name=name.to_str()?.to_string();
                    if !["len","sub","byte","char","lower","upper","reverse","rep","find"].contains(&name.as_str()){return Err(lua_error("INVALID_ARGUMENT: unsupported string operation"));}
                    let f=string_functions.get(&name).unwrap();
                    let mut values=args[1..].to_vec();
                    if name=="find" {while values.len()<3{values.push(LValue::Nil);}values.truncate(3);values.push(LValue::Boolean(true));}
                    if name=="byte"&&values.len()>2 {if let (Some(LValue::Integer(a)),Some(LValue::Integer(z)))=(values.get(1),values.get(2)){if z.saturating_sub(*a)>1024{return Err(lua_error("LIMIT_EXCEEDED: byte output bound"));}}}
                    f.call::<MultiValue>(MultiValue::from_vec(values))?.into_vec()
                },
                "tonumber"=>{let function:Function=lua.globals().get("tonumber")?;function.call::<MultiValue>(MultiValue::from_vec(args))?.into_vec()},
                _=>{
                    let args=if let Some(desc)=descriptor {match desc {Ok(v)=>json!([v]),Err(err)=>{if method=="can_deploy"{return Ok(MultiValue::from_vec(vec![LValue::Boolean(true),LValue::Boolean(false),LValue::String(lua.create_string("INVALID_ARGUMENT")?)]));}return Err(err);}}} else {
                        Value::Array(args.iter().map(|v|if matches!(v,LValue::Nil){Ok(Value::Null)}else{to_json(v,config.max_payload_bytes)}).collect::<mlua::Result<_>>()?)
                    };
                    let result=c.borrow_mut().api(id,&method,args).map_err(lua_error)?;
                    if method=="deploy" {if let Some(task)=result.as_u64(){if let Some(job)=c.borrow_mut().jobs.get_mut(&task){job.source=source.clone();}}}
                    if method=="others" {let map=lua.create_table()?;if let Some(values)=result.as_object(){for (agent,router) in values {map.raw_set(agent.parse::<u64>().map_err(|_|lua_error("invalid engine agent ID"))?,from_json(lua,router)?)?;}}vec![LValue::Table(map)]}
                    else if method=="can_deploy" {if let Some(values)=result.as_array(){values.iter().map(|v|from_json(lua,v)).collect::<mlua::Result<_>>()?}else{vec![from_json(lua,&result)?]}}else{vec![from_json(lua,&result)?]}
                }
            };
            let mut values=vec![LValue::Boolean(true)];values.extend(result);Ok(MultiValue::from_vec(values))
            };
            match operation(){
                Ok(values)=>Ok(values),
                Err(error)=>{
                    let text=match error{mlua::Error::RuntimeError(text)=>text,other=>format!("INVALID_ARGUMENT: {other}")};
                    Ok(MultiValue::from_vec(vec![LValue::String(lua.create_string(text)?)]))
                }
            }
        }).map_err(|e|e.to_string())?;
        let coroutine:Table=globals.get("coroutine").map_err(|e|e.to_string())?;
        let native_table:Table=globals.get("table").map_err(|e|e.to_string())?;
        let factory:Function=lua.load(include_str!("sandbox.lua")).set_name("@sandbox.lua").set_environment(env.clone()).into_function().map_err(|e|e.to_string())?;
        let (api,pairs,next,require,call):(Table,Function,Function,Function,Function)=factory.call((invoke,coroutine.get::<Function>("yield").unwrap(),native_table.get::<Function>("pack").unwrap(),native_table.get::<Function>("unpack").unwrap(),globals.get::<Function>("next").unwrap(),globals.get::<Function>("type").unwrap(),globals.get::<Function>("error").unwrap(),globals.get::<Function>("tostring").unwrap())).map_err(|e|e.to_string())?;
        api.set("VERSION","1.3").unwrap();api.set("MAX_TASKS",5).unwrap();
        for (name,values) in [
            ("node_type",vec!["ROUTER","WORKSTATION","SERVER"]),
            ("power",vec!["ONLINE","OFFLINE","STARTING","RESTORING"]),
            ("health",vec!["CLEAN","SUSPICIOUS","COMPROMISED"]),
            ("task_state",vec!["RUNNING","COMPLETED","FAILED"]),
            ("policy",vec!["DEFAULT_ALLOW","DEFAULT_DENY"]),
            ("sense",vec!["SCAN","TRAFFIC_SCAN","HOST_SCAN","CONNECTION_SCAN","FULL_HOST_SCAN","SHIELD_STATUS"]),
            ("act",vec!["SHUTDOWN","STARTUP","RESTORE"]),
            ("shield",vec!["SET_DEFAULT_ALLOW","SET_DEFAULT_DENY","BLOCK_PEER","ALLOW_PEER","SET_DEFAULT_ALLOW_ON_HOST","SET_DEFAULT_DENY_ON_HOST","BLOCK_PEER_ON_HOST","ALLOW_PEER_ON_HOST","ISOLATE_HOST","RELEASE_HOST"]),
            ("error",vec!["INVALID_ARGUMENT","UNKNOWN_ACTION","UNKNOWN_AGENT","UNKNOWN_NODE","UNDISCOVERED","OUT_OF_SCOPE","INVALID_TARGET_TYPE","OFFLINE","TARGET_BUSY","TASK_SLOTS_FULL","UNKNOWN_TASK","TASK_NOT_FINISHED","PAYLOAD_TOO_LARGE","LIMIT_EXCEEDED"]),
        ]{let constants=lua.create_table().unwrap();for value in values{constants.set(value,value).unwrap();}api.set(name,constants).unwrap();}
        api.set("tick_cost",from_json(&lua,&sim::action_registry()).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        env.set("vanguard",api).unwrap();env.set("pairs",pairs).unwrap();env.set("next",next).unwrap();env.set("require",require).unwrap();
        let print:Function=lua.load("local call=...; return function(...) return call('print',...) end").call(call.clone()).map_err(|e|e.to_string())?;
        env.set("print",print).unwrap();
        if core.borrow().config.dbg {
            let dbg=lua.create_table().map_err(|e|e.to_string())?;
            for name in ["view","node","edge","note","clear","remove_node","remove_edge"] {
                let f:Function=lua.load("local call,name=...; return function(...) return call(name,...) end").call((call.clone(),format!("dbg_{name}"))).map_err(|e|e.to_string())?;
                dbg.set(name,f).map_err(|e|e.to_string())?;
            }
            env.set("dbgguard",dbg).map_err(|e|e.to_string())?;
        }
        let tonumber:Function=lua.load("local call=...;return function(...) return call('tonumber',...) end").call(call.clone()).map_err(|e|e.to_string())?;
        env.set("tonumber",tonumber).unwrap();
        let strings=lua.create_table().unwrap();
        for name in ["len","sub","byte","char","lower","upper","reverse","rep","find"] {
            let f:Function=lua.load("local call,name=...; return function(...) return call('string',name,...) end").call((call.clone(),name)).map_err(|e|e.to_string())?;
            strings.set(name,f).unwrap();
        }
        // The Lua string metatable points at the original library table. Replace
        // that table too so ('x'):rep(...) cannot bypass bounded wrappers.
        let original:Table=globals.get("string").unwrap();
        let keys=original.clone().pairs::<LValue,LValue>().map(|p|p.map(|(k,_)|k)).collect::<mlua::Result<Vec<_>>>().unwrap();
        for key in keys {original.raw_set(key,LValue::Nil).unwrap();}
        for pair in strings.clone().pairs::<LValue,LValue>(){let (key,value)=pair.unwrap();original.raw_set(key,value).unwrap();}
        env.set("string",strings).unwrap();
        let math=lua.create_table().unwrap();let raw_math:Table=globals.get("math").unwrap();
        for name in ["abs","ceil","floor","min","max","sqrt","fmod","huge","pi","tointeger","type"] {math.set(name,raw_math.get::<LValue>(name).unwrap()).unwrap();}
        env.set("math",math).unwrap();
        // All dependency syntax is checked during staging, never halfway through a run.
        for (name,text) in &staged.sources {lua.load(text).set_name(format!("@{name}")).set_mode(ChunkMode::Text).set_environment(env.clone()).into_function().map_err(|e|e.to_string())?;}
        let mut vm=Self{id,lua,budget,handlers,runnable:VecDeque::new(),pending:VecDeque::new(),errors:0,last_fuel:0,memory};
        let name=&staged.entry;
        let function=vm.lua.load(&staged.sources[name]).set_name(format!("@{name}")).set_mode(ChunkMode::Text).set_environment(env.clone()).into_function().map_err(|e|e.to_string())?;
        vm.spawn(function,name.clone(),MultiValue::new(),None)?;
        Ok(vm)
    }
    fn spawn(&mut self,function:Function,source:String,args:MultiValue,message:Option<(u64,u64)>)->Result<(),String>{
        let thread=self.lua.create_thread(function).map_err(|e|e.to_string())?;
        let b=self.budget.clone();
        thread.set_hook(HookTriggers::new().every_nth_instruction(1),move |_,_|{
            if !b.charge(1){return Ok(VmState::Yield);}
            let q=b.quantum.get().saturating_sub(1);b.quantum.set(q);
            Ok(if q==0||b.remaining.get()==0{VmState::Yield}else{VmState::Continue})
        }).map_err(|e|e.to_string())?;
        self.runnable.push_back(Coroutine{thread,source,wait:Wait::Runnable,initial:Some(args),message});Ok(())
    }
}

pub struct Runtime {
    pub core: Rc<RefCell<Core>>,
    bus: Rc<RefCell<Bus>>,
    debug: Rc<RefCell<crate::debug::Store>>,
    vms: BTreeMap<u64,Vm>,
    pub control: String,
    pub run_id: String,
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
impl Runtime {
    pub fn new(config:Config,run_id:String)->Result<Self,String>{
        config.validate()?;
        let staged=Staged::load(&config)?;
        let core=Rc::new(RefCell::new(Core::new(config)?));let bus=Rc::new(RefCell::new(Bus::default()));
        let agents=core.borrow().agents.iter().map(|(id,a)|(*id,a.router_id)).collect::<Vec<_>>();
        let mut vms=BTreeMap::new();
        let debug=Rc::new(RefCell::new(crate::debug::Store::default()));
        for (id,router) in agents {vms.insert(id,Vm::new(id,router,core.clone(),bus.clone(),debug.clone(),&staged)?);}
        Ok(Self{core,bus,debug,vms,control:"READY".into(),run_id})
    }
    pub fn step(&mut self)->Result<(),String>{
        if self.control=="FINISHED" {return Ok(());}
        let tick=self.core.borrow().tick;let config=self.core.borrow().config.clone();
        for vm in self.vms.values_mut() {
            let before=vm.budget.total.get();vm.budget.remaining.set(config.fuel_per_tick);
            // Capture recipient handlers when capacity permits delivery, never at send.
            let mut deferred=VecDeque::new();
            while let Some(message)=vm.pending.pop_front(){
                let active=vm.runnable.iter().filter(|c|c.message.is_some()).count();
                if active>=config.max_handlers {deferred.push_back(message);continue;}
                let handler=if let Some(topic)=&message.topic {vm.handlers.borrow().topics.get(topic).cloned()}else{vm.handlers.borrow().direct.clone()};
                let outcome=if handler.is_some(){"delivered"}else{"dropped"};
                self.core.borrow_mut().emit(if message.topic.is_some(){"publication"}else{"message"},Some(message.sender),&message.source,json!({"message_id":message.id,"sender":message.sender,"recipient":vm.id,"topic":message.topic,"payload":message.payload,"sent_at":message.sent_at,"delivered_at":tick,"outcome":outcome}));
                if let Some(handler)=handler{
                    let metadata=json!({"message_id":message.id,"sent_at":message.sent_at,"delivered_at":tick});
                    let mut args=vec![LValue::Integer(message.sender as i64)];if let Some(topic)=&message.topic{args.push(LValue::String(vm.lua.create_string(topic).map_err(|e|e.to_string())?));}args.push(from_json(&vm.lua,&message.payload).map_err(|e|e.to_string())?);args.push(from_json(&vm.lua,&metadata).map_err(|e|e.to_string())?);
                    vm.spawn(handler.function,handler.source,MultiValue::from_vec(args),Some((message.sender,message.id)))?;
                }
            }
            vm.pending=deferred;
            let mut idle=0;
            while !vm.runnable.is_empty() && vm.budget.remaining.get()>0 {
                let mut coroutine=vm.runnable.pop_front().unwrap();
                let ready=match &coroutine.wait {
                    Wait::Runnable=>true,Wait::Tick(until)=>tick>=*until,
                    Wait::Task(task)=>self.core.borrow_mut().api(vm.id,"is_finished",json!([task]))?==json!(true),
                    Wait::Any(tasks)=>tasks.iter().any(|task|self.core.borrow_mut().api(vm.id,"is_finished",json!([task])).ok()==Some(json!(true))),
                };
                if !ready {vm.runnable.push_back(coroutine);idle+=1;if idle>=vm.runnable.len(){break;}continue;}
                idle=0;vm.budget.quantum.set(config.instruction_quantum);
                let args=coroutine.initial.take().unwrap_or_default();
                match coroutine.thread.resume::<MultiValue>(args){
                    Ok(values) if coroutine.thread.is_resumable()=>{
                        coroutine.wait=match values.front(){
                            Some(LValue::String(s)) if s.to_string_lossy()=="fuel"=>Wait::Tick(tick+1),
                            Some(LValue::String(s)) if s.to_string_lossy()=="wait"=>Wait::Tick(positive(values.get(1)).map_err(|e|e.to_string())?),
                            Some(LValue::String(s)) if s.to_string_lossy()=="await"=>Wait::Task(positive(values.get(1)).map_err(|e|e.to_string())?),
                            Some(LValue::String(s)) if s.to_string_lossy()=="await_any"=>{let value=to_json(values.get(1).unwrap(),config.max_payload_bytes).map_err(|e|e.to_string())?;Wait::Any(value.as_array().unwrap().iter().map(|v|v.as_u64().unwrap()).collect())},
                            _=>Wait::Runnable,
                        };vm.runnable.push_back(coroutine);
                    },
                    Ok(_)=>{},
                    Err(error)=>{vm.errors+=1;self.core.borrow_mut().emit("runtime_error",Some(vm.id),&coroutine.source,json!({"severity":"error","error":error.to_string(),"sender":coroutine.message.map(|m|m.0),"message_id":coroutine.message.map(|m|m.1)}));},
                }
            }
            vm.last_fuel=vm.budget.total.get()-before;
            let discarded=std::mem::take(&mut vm.handlers.borrow_mut().discarded);
            vm.pending.retain(|message| {
                if message.topic.as_ref().is_some_and(|topic|discarded.contains(topic)){
                    self.core.borrow_mut().emit("publication",Some(message.sender),&message.source,json!({"message_id":message.id,"sender":message.sender,"recipient":vm.id,"topic":message.topic,"outcome":"dropped","reason":"unsubscribed while pending"}));false
                }else{true}
            });
            let mut core=self.core.borrow_mut();if let Some(agent)=core.agents.get_mut(&vm.id){agent.last_fuel=vm.last_fuel;agent.fuel=vm.budget.total.get();}
        }
        self.core.borrow_mut().barrier()?;
        let mut bus=self.bus.borrow_mut();
        let mut outgoing=std::mem::take(&mut bus.outgoing);outgoing.sort_by_key(|m|(m.sender,m.id));
        for message in outgoing {for recipient in &message.recipients {if let Some(vm)=self.vms.get_mut(recipient){vm.pending.push_back(message.clone());}}}
        let pending=self.vms.values().map(|vm|vm.pending.len()).sum::<usize>();if pending>MAX_PENDING{return Err("LIMIT_EXCEEDED: accepted pending message resources exhausted; run stopped".into());}
        bus.membership.clear();
        for (id,vm) in &self.vms {for topic in vm.handlers.borrow().topics.keys(){bus.membership.entry(topic.clone()).or_default().insert(*id);}bus.synchronized_fuel.insert(*id,vm.budget.total.get());}
        if config.tick_limit.is_some_and(|limit|self.core.borrow().tick>=limit) || self.core.borrow().score.as_ref().is_some_and(|s|s.defeated_at.is_some() || s.verified_at.is_some()){self.control="FINISHED".into();}
        Ok(())
    }
    pub fn snapshot(&self)->Value{
        let mut snapshot=self.core.borrow().snapshot();
        snapshot["schema_version"]=json!(1);snapshot["run_id"]=json!(self.run_id);snapshot["control"]=json!(self.control);
        snapshot["seq"]=json!(self.core.borrow().sequence);
        if let Some(agents)=snapshot["agents"].as_array_mut(){for agent in agents {if let Some(vm)=agent["id"].as_u64().and_then(|id|self.vms.get(&id)){agent["last_fuel"]=json!(vm.last_fuel);agent["memory_bytes"]=json!(vm.memory.current());agent["peak_memory_bytes"]=json!(vm.memory.peak());agent["coroutines"]=json!(vm.runnable.len());agent["backlog"]=json!(vm.pending.len());agent["runtime_errors"]=json!(vm.errors);agent["allowance"]=json!(self.core.borrow().config.fuel_per_tick);}}}
        let bus=self.bus.borrow();let mut topics=bus.topics.clone();for (topic,subscribers) in &bus.membership {let entry=topics.entry(topic.clone()).or_insert(json!({"topic":topic,"publication_count":0,"last_tick":null}));entry["subscribers"]=json!(subscribers);}
        snapshot["topics"]=json!(topics.into_values().collect::<Vec<_>>());
        snapshot["debug"]=json!({"enabled":self.core.borrow().config.dbg,"agents":self.debug.borrow().0});snapshot
    }
    /// Semantic replay excludes presentation speed and VM allocator diagnostics.
    pub fn state_hash(&self)->String{
        let mut value=self.snapshot();value.as_object_mut().unwrap().remove("run_id");value.as_object_mut().unwrap().remove("seq");value.as_object_mut().unwrap().remove("control");
        value.as_object_mut().unwrap().remove("debug");
        if let Some(scenario)=value["scenario"].as_object_mut(){scenario.remove("tick_ms");}
        if let Some(agents)=value["agents"].as_array_mut(){for a in agents {for key in ["memory_bytes","peak_memory_bytes"]{a.as_object_mut().unwrap().remove(key);}}}
        value["scheduler"]=json!(self.vms.values().map(|vm|json!({"agent":vm.id,"fuel":vm.budget.total.get(),"remaining":vm.budget.remaining.get(),"coroutines":vm.runnable.iter().map(|c|json!({"source":c.source,"message":c.message,"wait":match &c.wait{Wait::Runnable=>json!({"kind":"runnable"}),Wait::Tick(t)=>json!({"tick":t}),Wait::Task(t)=>json!({"task":t}),Wait::Any(t)=>json!({"any":t})}})).collect::<Vec<_>>(),"pending":vm.pending.iter().map(|m|json!({"id":m.id,"sender":m.sender,"topic":m.topic,"payload":m.payload,"sent_at":m.sent_at})).collect::<Vec<_>>(),"subscriptions":vm.handlers.borrow().topics.keys().collect::<Vec<_>>(),"direct_handler":vm.handlers.borrow().direct.is_some()})).collect::<Vec<_>>());
        format!("{:x}",Sha256::digest(serde_json::to_vec(&value).unwrap()))
    }
}
