use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64,Ordering};

static NEXT:AtomicU64=AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(script:&str)->Self{
        let path=std::env::current_dir().unwrap().join(format!(".vanguard-test-{}-{}",std::process::id(),NEXT.fetch_add(1,Ordering::Relaxed)));
        fs::create_dir_all(&path).unwrap();fs::write(path.join("app.lua"),script).unwrap();Self(path)
    }
    fn config(&self,routers:usize)->Config{
        let mut config=Config::default();config.router_count=routers;
        config.script_root=self.0.to_string_lossy().into_owned();config.script=Some("app.lua".into());config
    }
}
impl Drop for Fixture {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn events(runtime:&Runtime,kind:&str)->Vec<Value>{runtime.snapshot()["events"].as_array().unwrap().iter().filter(|e|e["kind"]==kind).cloned().collect()}

#[test]
fn debug_views_are_opt_in_write_only_namespaced_and_transactional(){
    let fixture=Fixture::new(r#"
        if not dbgguard then print('disabled'); return end
        dbgguard.view('map')
        dbgguard.node('map',vanguard.router(),{label='mine'})
        dbgguard.node('map','peer',{assessment='UNKNOWN'})
        dbgguard.edge('map','link',vanguard.router(),'peer',{confidence='low'})
        assert(not pcall(dbgguard.edge,'map','bad','absent','peer',{}))
        assert(not pcall(dbgguard.node,'absent',1,{}))
        dbgguard.note('map',{tick=vanguard.ticks()})
        dbgguard.view('temporary')
        dbgguard.node('temporary',1,{})
        dbgguard.node('temporary',2,{})
        dbgguard.edge('temporary','x',1,2,{})
        dbgguard.remove_node('temporary',2)
        dbgguard.remove_edge('temporary','missing')
        dbgguard.clear('temporary')
    "#);
    let mut off=Runtime::new(fixture.config(2),"off".into()).unwrap();off.step().unwrap();
    assert_eq!(off.snapshot()["debug"],json!({"enabled":false,"agents":{}}));
    let mut config=fixture.config(2);config.dbg=true;
    let mut on=Runtime::new(config.clone(),"on".into()).unwrap();
    let mut repeat=Runtime::new(config,"repeat".into()).unwrap();
    for _ in 0..3{on.step().unwrap();repeat.step().unwrap();assert_eq!(on.state_hash(),repeat.state_hash());}
    assert!(events(&on,"runtime_error").is_empty());
    let snapshot=on.snapshot();
    let agents=snapshot["debug"]["agents"].as_object().unwrap();assert_eq!(agents.len(),2);
    for (agent,views) in agents{
        assert_eq!(views["map"]["nodes"].as_object().unwrap().len(),2);
        assert_eq!(views["map"]["edges"].as_object().unwrap().len(),1);
        assert!(views["temporary"]["nodes"].as_object().unwrap().is_empty());
        // Invented drawing IDs do not become discovered simulation IDs.
        let known=&on.core.borrow().agents[&agent.parse::<u64>().unwrap()].known;
        assert_eq!(known.len(),1);
    }
}

#[test]
fn teaching_agents_run_on_both_tutorials_without_runtime_errors(){
    for network in ["tutorial-1.toml","tutorial-2.toml"] {
        for script in ["examples/sensing.lua","examples/shields.lua","examples/recovery.lua","examples/solo.lua","examples/early-warning.lua","examples/leader.lua","examples/debug-map.lua","examples/debug-connections.lua","basic-defender.lua","coordinated-defender.lua","advanced-defender.lua"] {
            let files=crate::network::RunFiles{script:format!("vanguard/agents/{script}").into(),network:format!("vanguard/networks/{network}").into(),dbg:true};
            let mut runtime=Runtime::new(files.load().unwrap(),"teaching".into()).unwrap();
            for _ in 0..100{runtime.step().unwrap();if runtime.control=="FINISHED"{break;}}
            assert!(events(&runtime,"runtime_error").is_empty(),"{script}, {network}: {:?}",events(&runtime,"runtime_error"));
            if script=="examples/recovery.lua" {assert!(runtime.core.borrow().nodes[&1001].isolated);assert!(!runtime.core.borrow().nodes[&1001].compromised);}
            if script=="examples/debug-map.lua" {assert!(runtime.snapshot()["debug"]["agents"]["101"]["assessments"]["nodes"]["1001"]["info"]["observed_at"].as_u64().is_some());}
        }
    }
}

#[test]
fn cross_lan_warning_discovers_and_blocks_a_remote_source(){
    let text="version=1\nrouters=[{id=1},{id=2}]\nlans=[{id=1,router=1},{id=2,router=2}]\nworkstations=[{id=1001,lan=1,compromised=true}]\nservers=[{id=2001,lan=2}]\nconnections=[{from=1,to=2}]\n[mission]\nenabled=false\n";
    let fixture=Fixture::new(&fs::read_to_string("vanguard/agents/examples/early-warning.lua").unwrap());
    let mut config=crate::network::parse(text).unwrap();let files=fixture.config(2);config.script=files.script;config.script_root=files.script_root;
    let mut runtime=Runtime::new(config,"warning".into()).unwrap();
    for _ in 0..30{runtime.step().unwrap();}
    assert!(events(&runtime,"runtime_error").is_empty());
    let core=runtime.core.borrow();
    assert!(core.nodes[&2].exceptions.contains(&1001));
    assert!(core.agents[&102].known.contains(&1001));
    assert!(core.nodes[&1001].isolated);
}

#[test]
fn new_mail_lua_snippets_execute_in_the_real_sandbox(){
    for name in ["vendor","operation","lua-basics","lua-tables","lua-functions","debug-api"] {
        let text=fs::read_to_string(format!("vandocs/content/mail/{name}.md")).unwrap();
        for (index,part) in text.split("```lua\n").skip(1).enumerate(){
            let code=part.split("```").next().unwrap();let fixture=Fixture::new(code);
            let mut config=fixture.config(2);config.dbg=true;
            let mut runtime=Runtime::new(config,"snippet".into()).unwrap();
            for _ in 0..8{runtime.step().unwrap();}
            assert!(events(&runtime,"runtime_error").is_empty(),"{name} block {index}: {:?}",events(&runtime,"runtime_error"));
        }
    }
}

#[test]
fn independent_vms_and_busy_loop_barriers_are_replayable(){
    let fixture=Fixture::new("private=(private or 0)+1; assert(private==1); print(vanguard.self(),vanguard.ticks()); while true do end");
    for routers in [4,5]{
        let config=fixture.config(routers);
        let mut a=Runtime::new(config.clone(),"a".into()).unwrap();
        let mut slower=config;slower.tick_ms+=123;
        let mut b=Runtime::new(slower,"b".into()).unwrap();
        assert_eq!(a.vms.len(),routers);
        for _ in 0..5{
            a.step().unwrap();b.step().unwrap();
            assert_eq!(a.state_hash(),b.state_hash());
            for vm in a.vms.values(){assert_eq!(vm.budget.remaining.get(),0);}
        }
        assert_eq!(events(&a,"log").len(),routers);
        assert!(events(&a,"runtime_error").is_empty());
        assert!(a.snapshot()["agents"].as_array().unwrap().iter().all(|a|a["last_fuel"]==a["allowance"]));
    }
}

#[test]
fn direct_messages_response_barriers_and_handler_isolation(){
    let fixture=Fixture::new(r#"
        vanguard.rx(function(sender,m,meta)
            print('delivery', meta)
            if m.kind=='ping' then
                vanguard.tx(sender,{kind='pong'})
                error('intentional handler error after accepted send')
            end
        end)
        if vanguard.self()==101 then vanguard.tx(102,{kind='ping'}) end
        while true do end
    "#);
    let mut runtime=Runtime::new(fixture.config(4),"test".into()).unwrap();
    runtime.step().unwrap();
    assert!(events(&runtime,"message").iter().all(|e|e["data"]["outcome"]!="delivered"));
    runtime.step().unwrap();runtime.step().unwrap();
    let delivered=events(&runtime,"message").into_iter().filter(|e|e["data"]["outcome"]=="delivered").collect::<Vec<_>>();
    assert_eq!(delivered.len(),2);
    assert_eq!(delivered[0]["data"]["delivered_at"],1);
    assert_eq!(delivered[1]["data"]["delivered_at"],2);
    assert_eq!(events(&runtime,"runtime_error").len(),1);
    assert!(runtime.vms.values().all(|vm|!vm.runnable.is_empty()));
}

#[test]
fn publications_use_start_of_tick_membership_and_missing_handlers_drop(){
    let fixture=Fixture::new(r#"
        vanguard.sub('topic',function(sender,topic,payload,meta) print('pub',meta) end)
        local first=vanguard.pub('topic',{stage=0})
        assert(first.recipient_count==0)
        vanguard.tx(vanguard.self()==101 and 102 or 101,{drop=true})
        vanguard.wait(1)
        local second=vanguard.pub('topic',{stage=1})
        assert(second.recipient_count==3)
    "#);
    let mut runtime=Runtime::new(fixture.config(4),"test".into()).unwrap();
    for _ in 0..3{runtime.step().unwrap();}
    assert!(events(&runtime,"runtime_error").is_empty(),"{:?}",events(&runtime,"runtime_error"));
    assert_eq!(events(&runtime,"message").iter().filter(|e|e["data"]["outcome"]=="dropped").count(),4);
    assert_eq!(events(&runtime,"publication").iter().filter(|e|e["data"]["outcome"]=="delivered").count(),12);
}

#[test]
fn native_admission_gate_resumes_once_without_duplicate_tasks(){
    let fixture=Fixture::new(r#"
        local a=vanguard.deploy({action=vanguard.sense.SCAN})
        local b=vanguard.deploy({action=vanguard.sense.SCAN})
        local result=vanguard.await(b)
        assert(result.finished_at==vanguard.status(b).started_at+2)
        print(a,b,result)
    "#);
    let mut config=fixture.config(4);config.fuel_per_tick=5000;config.task_admission_cost=4000;config.max_payload_bytes=256;config.native_call_cost=5;
    let mut runtime=Runtime::new(config,"test".into()).unwrap();
    for _ in 0..6{runtime.step().unwrap();}
    let snapshot=runtime.snapshot();let jobs=snapshot["jobs"].as_array().unwrap();
    assert_eq!(jobs.len(),8);
    for agent in snapshot["agents"].as_array().unwrap(){let id=agent["id"].as_u64().unwrap();let owned=jobs.iter().filter(|j|j["agent_id"]==id).collect::<Vec<_>>();assert_eq!(owned.len(),2);assert!(owned[1]["started_at"].as_u64().unwrap()>owned[0]["started_at"].as_u64().unwrap());}
    assert!(events(&runtime,"runtime_error").is_empty());
}

#[test]
fn payload_validation_and_structured_logs_are_bounded_and_nonfatal(){
    let fixture=Fixture::new(r#"
        local other=vanguard.self()==101 and 102 or 101
        local cyclic={};cyclic.me=cyclic
        for _,value in ipairs({cyclic,{[1]='x',bad='y'},setmetatable({},{})}) do
            local ok,err=pcall(vanguard.tx,other,value)
            assert(not ok and string.find(err,'INVALID_ARGUMENT'))
        end
        local bad=setmetatable({},{__tostring=function() error('do not call me') end})
        print('structured', {a=1,b=true},cyclic,bad,function()end)
        assert(io==nil and os==nil and debug==nil and coroutine==nil and load==nil)
        assert(('ab'):rep(2)=='abab')
        local ok=pcall(function() return ('x'):rep(100000000) end);assert(not ok)
        local order='';for k in pairs({z=1,a=2,m=3}) do order=order..k end;assert(order=='amz')
    "#);
    let mut runtime=Runtime::new(fixture.config(4),"test".into()).unwrap();
    for _ in 0..3{runtime.step().unwrap();}
    assert!(events(&runtime,"runtime_error").is_empty(),"{:?}",events(&runtime,"runtime_error"));
    assert_eq!(events(&runtime,"log").len(),4);
    assert!(events(&runtime,"log").iter().any(|e|e.to_string().contains("cycle")));
}

#[test]
fn staged_modules_are_consistent_and_restart_rereads_disk(){
    let fixture=Fixture::new("print(require('module'))");
    fs::write(fixture.0.join("module.lua"),"return 'before'").unwrap();
    let config=fixture.config(4);let mut old=Runtime::new(config.clone(),"old".into()).unwrap();
    fs::write(fixture.0.join("module.lua"),"return 'after'").unwrap();
    old.step().unwrap();let mut new=Runtime::new(config.clone(),"new".into()).unwrap();new.step().unwrap();
    assert!(events(&old,"log")[0].to_string().contains("before"));
    assert!(events(&new,"log")[0].to_string().contains("after"));
    fs::write(fixture.0.join("module.lua"),"syntax error !").unwrap();
    assert!(Runtime::new(config,"failed".into()).is_err());
    old.step().unwrap();assert_eq!(old.core.borrow().tick,2);
}

#[test]
fn actual_memory_exhaustion_is_controlled_and_other_routers_keep_running(){
    let fixture=Fixture::new("if vanguard.self()==101 then local t={};while true do t[#t+1]={1,2,3,4,5,6,7,8} end else print('healthy teammate') end");
    let mut config=fixture.config(4);config.memory_limit=512*1024;
    let mut runtime=Runtime::new(config,"memory".into()).unwrap();
    for _ in 0..10{runtime.step().unwrap();}
    let errors=events(&runtime,"runtime_error");assert_eq!(errors.len(),1);assert_eq!(errors[0]["agent_id"],101);
    assert!(errors[0]["data"]["error"].as_str().unwrap().contains("memory"));
    assert_eq!(events(&runtime,"log").len(),3);
    assert!(runtime.snapshot()["agents"].as_array().unwrap().iter().all(|a|a["peak_memory_bytes"].as_u64().unwrap()<=512*1024));
}

#[test]
fn default_defender_and_completed_jobs_replay_identically(){
    let files=crate::network::RunFiles{script:"vanguard/agents/defender.lua".into(),network:"network.toml".into(),dbg:false};
    let config=files.load().unwrap();
    let mut a=Runtime::new(config.clone(),"a".into()).unwrap();
    let mut config=config;config.tick_ms=813;
    let mut b=Runtime::new(config,"b".into()).unwrap();
    for _ in 0..45{a.step().unwrap();b.step().unwrap();assert_eq!(a.state_hash(),b.state_hash());}
    assert!(events(&a,"runtime_error").is_empty());
    assert!(a.snapshot()["jobs"].as_array().unwrap().iter().any(|j|j["action"]=="RESTORE"&&j["state"]=="COMPLETED"));
}
