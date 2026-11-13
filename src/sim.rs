//! Deterministic, barrier-committed simulation. Lua execution lives outside this module.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[serde(skip)]
    pub dbg: bool,
    pub mission: Option<crate::scoring::Mission>,
    pub router_count: usize,
    pub hosts_per_lan: usize,
    pub server_count: usize,
    pub seed: u64,
    pub tick_limit: Option<u64>,
    pub tick_ms: u64,
    pub fuel_per_tick: u64,
    pub instruction_quantum: u64,
    pub memory_limit: usize,
    pub max_payload_bytes: usize,
    pub max_handlers: usize,
    pub native_call_cost: u64,
    pub task_admission_cost: u64,
    #[serde(skip)]
    pub script_root: String,
    #[serde(skip)]
    pub script: Option<String>,
    pub nodes: Option<Vec<Node>>,
    pub router_edges: Option<Vec<[u64; 2]>>,
    pub initial_compromised: Option<Vec<u64>>,
    pub infection_intervals: u64,
    pub attack_slots: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { dbg: false, mission: None, router_count: 4, hosts_per_lan: 4, server_count: 2, seed: 1,
            tick_limit: Some(1000), tick_ms: 0, fuel_per_tick: 100_000,
            instruction_quantum: 100, memory_limit: 16 * 1024 * 1024,
            max_payload_bytes: 16384, max_handlers: 64, native_call_cost: 10,
            task_admission_cost: 100, script_root: String::new(), script: None,
            nodes: None, router_edges: None, initial_compromised: None,
            infection_intervals: 14, attack_slots: 2 }
    }
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if self.router_count == 0 || self.router_count > 1000 || self.hosts_per_lan > 10000 || self.server_count > 10000 {
            return Err("INVALID_ARGUMENT: invalid topology size".into());
        }
        if self.fuel_per_tick == 0 || self.instruction_quantum == 0 || self.instruction_quantum > self.fuel_per_tick
            || self.memory_limit == 0 || self.max_payload_bytes == 0 || self.max_handlers == 0
            || self.infection_intervals == 0 || self.attack_slots == 0
            || self.native_call_cost.checked_add(self.task_admission_cost).is_none_or(|n| n > self.fuel_per_tick) {
            return Err("INVALID_ARGUMENT: invalid scenario resource limits".into());
        }
        let maximum_native=self.max_payload_bytes.checked_mul(6).and_then(|n|n.checked_add(1024)).and_then(|n|u64::try_from(n).ok()).and_then(|n|n.checked_add(self.native_call_cost));
        if maximum_native.is_none_or(|n|n>self.fuel_per_tick){return Err("INVALID_ARGUMENT: maximum atomic serialization/string charge must fit fuel_per_tick (6*max_payload_bytes+1024+native_call_cost)".into());}
        let nodes = self.topology();
        let mut ids = BTreeSet::new();
        for node in &nodes {
            if node.id == 0 || node.id > 9_007_199_254_740_991 || !ids.insert(node.id)
                || !["ROUTER", "WORKSTATION", "SERVER"].contains(&node.kind.as_str()) {
                return Err("INVALID_ARGUMENT: duplicate, invalid node ID or type".into());
            }
        }
        let routers: BTreeSet<_> = nodes.iter().filter(|n| n.kind == "ROUTER").map(|n| n.id).collect();
        if routers.is_empty() { return Err("INVALID_ARGUMENT: no routers".into()); }
        if routers.len()!=self.router_count {return Err("INVALID_ARGUMENT: router_count does not match explicit topology".into());}
        if routers.iter().any(|router|*router>9_007_199_254_740_891){return Err("INVALID_ARGUMENT: router ID leaves no exact-integer space for its agent ID".into());}
        if nodes.iter().any(|n|n.compromised&&n.kind!="WORKSTATION") {return Err("INVALID_ARGUMENT: only workstations may start compromised".into());}
        for n in &nodes {
            if !routers.contains(&n.router_id) || (n.kind == "ROUTER" && n.router_id != n.id) {
                return Err("INVALID_ARGUMENT: invalid LAN ownership".into());
            }
        }
        let edges=self.edges(&nodes);
        let mut unique_edges=BTreeSet::new();
        for [a,b] in &edges {
            if a == b || !routers.contains(a) || !routers.contains(b) {
                return Err("INVALID_ARGUMENT: invalid router edge".into());
            }
            if !unique_edges.insert(((*a).min(*b),(*a).max(*b))) {return Err("INVALID_ARGUMENT: duplicate router edge".into());}
        }
        let mut reached=BTreeSet::from([*routers.first().unwrap()]);
        loop {let previous=reached.len();for [a,b] in &edges{if reached.contains(a){reached.insert(*b);}if reached.contains(b){reached.insert(*a);}}if reached.len()==previous{break;}}
        if reached!=routers {return Err("INVALID_ARGUMENT: router graph must be connected".into());}
        for id in self.initial_ids(&nodes) {
            if !nodes.iter().any(|n| n.id == id && n.kind == "WORKSTATION") {
                return Err("INVALID_ARGUMENT: initial infection requires a workstation".into());
            }
        }
        if let Some(mission)=&self.mission {mission.validate(self)?;}
        Ok(())
    }
    pub(crate) fn topology(&self) -> Vec<Node> {
        if let Some(nodes) = &self.nodes { return nodes.clone(); }
        let mut nodes = Vec::new();
        let mut host = 1001;
        for r in 1..=self.router_count as u64 {
            nodes.push(Node::new(r, "ROUTER", r));
            for _ in 0..self.hosts_per_lan { nodes.push(Node::new(host, "WORKSTATION", r)); host += 1; }
        }
        for i in 0..self.server_count {
            nodes.push(Node::new(host, "SERVER", (i % self.router_count) as u64 + 1)); host += 1;
        }
        nodes
    }
    fn edges(&self, nodes: &[Node]) -> Vec<[u64;2]> {
        if let Some(edges) = &self.router_edges { return edges.clone(); }
        let rs: Vec<_> = nodes.iter().filter(|n| n.kind == "ROUTER").map(|n| n.id).collect();
        let mut edges = BTreeSet::new();
        if rs.len() > 1 { for i in 0..rs.len() { edges.insert([rs[i].min(rs[(i+1)%rs.len()]), rs[i].max(rs[(i+1)%rs.len()])]); } }
        edges.into_iter().collect()
    }
    fn initial_ids(&self, nodes: &[Node]) -> Vec<u64> {
        self.initial_compromised.clone().unwrap_or_else(|| {let explicit:Vec<_>=nodes.iter().filter(|n|n.compromised).map(|n|n.id).collect();if explicit.is_empty(){nodes.iter().filter(|n| n.kind == "WORKSTATION").map(|n|n.id).min().map(|id|vec![id]).unwrap_or_default()}else{explicit}})
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Node {
    pub id: u64,
    #[serde(rename="type", alias="kind")]
    pub kind: String,
    pub router_id: u64,
    pub lan_id: u64,
    pub power: String,
    pub compromised: bool,
    pub policy: String,
    pub exceptions: BTreeSet<u64>,
    pub isolated: bool,
}
impl Default for Node { fn default() -> Self { Self::new(0, "WORKSTATION", 0) } }
impl Node {
    pub fn new(id: u64, kind: &str, router_id: u64) -> Self {
        Self { id, kind: kind.into(), router_id, lan_id: router_id, power: "ONLINE".into(), compromised: false,
            policy: "DEFAULT_ALLOW".into(), exceptions: BTreeSet::new(), isolated: false }
    }
    fn allows(&self, peer: u64) -> bool {
        !self.isolated && if self.policy == "DEFAULT_ALLOW" { !self.exceptions.contains(&peer) } else { self.exceptions.contains(&peer) }
    }
}
#[derive(Serialize, Clone, Debug)]
pub struct AgentState {
    pub id: u64, pub router_id: u64, pub fuel: u64, pub last_fuel: u64,
    pub known: BTreeSet<u64>, pub observations: Vec<Value>,
}
#[derive(Serialize, Clone, Debug)]
pub struct Job {
    pub id: u64, pub agent_id: u64, pub router_id: u64, pub source: String,
    pub action: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub target: Option<u64>,
    pub peer: Option<u64>, pub state: String, pub started_at: u64,
    #[serde(skip_serializing_if="Option::is_none")]
    pub finished_at: Option<u64>,
    pub fuel_spent: u64, pub duration: u64, pub result: Option<Value>, pub failure: Option<String>,
    #[serde(skip)]
    entered: bool,
    #[serde(skip)]
    changed: bool,
    #[serde(skip)]
    local_sequence: u64,
}
#[derive(Serialize, Clone, Debug)]
pub struct Event { pub sequence: u64, pub tick: u64, pub kind: String, pub agent_id: Option<u64>, pub source: String, pub data: Value }
#[derive(Serialize, Clone, Debug)]
pub struct Session { pub id: u64, pub source: u64, pub target: u64, pub opened_at: u64 }

const ACTIONS: [(&str,u64);19] = [
    ("SCAN",2),("TRAFFIC_SCAN",4),("HOST_SCAN",3),("CONNECTION_SCAN",3),("FULL_HOST_SCAN",8),("SHIELD_STATUS",1),
    ("SHUTDOWN",1),("STARTUP",3),("RESTORE",20),("SET_DEFAULT_ALLOW",1),("SET_DEFAULT_DENY",1),
    ("BLOCK_PEER",1),("ALLOW_PEER",1),("SET_DEFAULT_ALLOW_ON_HOST",1),("SET_DEFAULT_DENY_ON_HOST",1),
    ("BLOCK_PEER_ON_HOST",1),("ALLOW_PEER_ON_HOST",1),("ISOLATE_HOST",1),("RELEASE_HOST",1)];
pub fn action_registry() -> Value {
    Value::Object(ACTIONS.into_iter().map(|(a,d)| (a.into(),json!(d))).collect())
}
fn readonly(a: &str) -> bool { matches!(a,"SCAN"|"TRAFFIC_SCAN"|"HOST_SCAN"|"CONNECTION_SCAN"|"FULL_HOST_SCAN"|"SHIELD_STATUS") }
fn err(label: &str) -> String { format!("{label}: request rejected") }

/// Static, sorted-neighbour BFS trees. O(R²) storage rather than storing a full
/// O(R)-length path per router pair. Policies are always checked live.
struct Routing {
    routers: Vec<u64>,
    index: BTreeMap<u64,usize>,
    parents: Vec<Vec<usize>>,
    neighbours: BTreeMap<u64,Vec<u64>>,
}
impl Routing {
    fn new(routers:Vec<u64>,edges:&BTreeSet<(u64,u64)>)->Self {
        let index:BTreeMap<_,_>=routers.iter().enumerate().map(|(i,id)|(*id,i)).collect();
        let mut neighbours:BTreeMap<_,Vec<_>>=routers.iter().map(|id|(*id,Vec::new())).collect();
        for &(a,b) in edges {neighbours.get_mut(&a).unwrap().push(b);neighbours.get_mut(&b).unwrap().push(a);}
        for peers in neighbours.values_mut(){peers.sort_unstable();}
        let mut parents=vec![vec![usize::MAX;routers.len()];routers.len()];
        for source in 0..routers.len(){
            parents[source][source]=source;let mut queue=VecDeque::from([source]);
            while let Some(current)=queue.pop_front(){
                for peer in &neighbours[&routers[current]]{
                    let next=index[peer];
                    if parents[source][next]==usize::MAX {parents[source][next]=current;queue.push_back(next);}
                }
            }
        }
        Self{routers,index,parents,neighbours}
    }
}

pub struct Core {
    pub score: Option<crate::scoring::Score>,
    pub config: Config, pub tick: u64, pub nodes: BTreeMap<u64,Node>,
    pub agents: BTreeMap<u64,AgentState>, pub jobs: BTreeMap<u64,Job>,
    pub events: VecDeque<Event>, pub sequence: u64, pub sessions: BTreeMap<u64,Session>,
    edges: BTreeSet<(u64,u64)>, history: BTreeMap<u64,BTreeSet<(u64,u64)>>,
    submissions: BTreeMap<u64,u64>, attacks: BTreeMap<(u64,u64),u64>,
    rng: u64, session_sequence: u64, task_sequence:u64,
    pub enemy_updates:u64, pub attack_attempts:u64, pub successful_connections:u64,
    host_attempts:BTreeMap<u64,u64>, host_connections:BTreeMap<u64,u64>,
    routing:Routing,
    hosts_by_router:BTreeMap<u64,Vec<u64>>,
    running_by_router:BTreeMap<u64,BTreeSet<u64>>,
    mutation_locks:BTreeMap<u64,u64>,
    exposure_by_target:BTreeMap<u64,u64>,
    session_by_pair:BTreeMap<(u64,u64),u64>,
    sessions_by_host:BTreeMap<u64,BTreeSet<u64>>,
}
impl Core {
    pub fn new(config: Config) -> Result<Self,String> {
        config.validate()?;
        let mut ns = config.topology();
        ns.sort_by_key(|n|n.id);
        let initial: BTreeSet<_> = config.initial_ids(&ns).into_iter().collect();
        // Scenario nodes describe topology; all dynamic state starts from the rules.
        for n in &mut ns { let lan=n.lan_id; *n = Node::new(n.id, &n.kind, n.router_id); n.lan_id=lan; n.compromised = initial.contains(&n.id); }
        let edges:BTreeSet<_> = config.edges(&ns).into_iter().map(|[a,b]| (a.min(b),a.max(b))).collect();
        let routing=Routing::new(ns.iter().filter(|n|n.kind=="ROUTER").map(|n|n.id).collect(),&edges);
        let mut hosts_by_router=BTreeMap::<_,Vec<_>>::new();
        for node in ns.iter().filter(|n|n.kind!="ROUTER"){hosts_by_router.entry(node.router_id).or_default().push(node.id);}
        let mut agents = BTreeMap::new();
        for (index, n) in ns.iter().filter(|n| n.kind == "ROUTER").enumerate() {
            let id = 101 + index as u64;
            agents.insert(id, AgentState { id,router_id:n.id,fuel:0,last_fuel:0,known:BTreeSet::from([n.id]),observations:vec![] });
        }
        let rng = if config.seed == 0 { 0x9e3779b97f4a7c15 } else { config.seed };
        let scored=config.mission.as_ref().is_some_and(|m|m.enabled);
        let mut core=Self {score:None,config,tick:0,nodes:ns.into_iter().map(|n|(n.id,n)).collect(),agents,jobs:BTreeMap::new(),events:VecDeque::new(),sequence:0,
            sessions:BTreeMap::new(),session_by_pair:BTreeMap::new(),sessions_by_host:BTreeMap::new(),edges,history:BTreeMap::new(),submissions:BTreeMap::new(),attacks:BTreeMap::new(),rng,session_sequence:0,task_sequence:0,enemy_updates:0,attack_attempts:0,successful_connections:0,host_attempts:BTreeMap::new(),host_connections:BTreeMap::new(),routing,hosts_by_router,running_by_router:BTreeMap::new(),mutation_locks:BTreeMap::new(),exposure_by_target:BTreeMap::new()};
        if scored {core.score=Some(crate::scoring::Score::new(&core));}
        Ok(core)
    }
    pub fn emit(&mut self, kind: &str, agent_id: Option<u64>, source: &str, data: Value) {
        self.sequence += 1;
        self.events.push_back(Event {sequence:self.sequence,tick:self.tick,kind:kind.into(),agent_id,source:source.into(),data});
        while self.events.len() > 2000 { self.events.pop_front(); }
    }
    fn descriptor(&self, agent: u64, value: &Value, admission: bool) -> Result<(String,Option<u64>,Option<u64>,u64),String> {
        let state = self.agents.get(&agent).ok_or_else(||err("UNKNOWN_AGENT"))?;
        let obj = value.as_object().ok_or_else(||err("INVALID_ARGUMENT"))?;
        if obj.keys().any(|k| !["action","target","peer"].contains(&k.as_str())) { return Err(err("INVALID_ARGUMENT")); }
        let action = obj.get("action").and_then(Value::as_str).ok_or_else(||err("INVALID_ARGUMENT"))?;
        let duration = ACTIONS.iter().find(|(a,_)| *a == action).map(|(_,d)|*d).ok_or_else(||err("UNKNOWN_ACTION"))?;
        let target = obj.get("target").map(|v| v.as_u64().filter(|n|*n>0).ok_or_else(||err("INVALID_ARGUMENT"))).transpose()?;
        let peer = obj.get("peer").map(|v| v.as_u64().filter(|n|*n>0).ok_or_else(||err("INVALID_ARGUMENT"))).transpose()?;
        let host_action = matches!(action,"HOST_SCAN"|"CONNECTION_SCAN"|"FULL_HOST_SCAN"|"SHUTDOWN"|"STARTUP"|"RESTORE"|"ISOLATE_HOST"|"RELEASE_HOST") || action.ends_with("_ON_HOST");
        if (host_action && target.is_none()) || (!host_action && action != "SHIELD_STATUS" && target.is_some())
            || (action.starts_with("BLOCK_PEER") || action.starts_with("ALLOW_PEER")) != peer.is_some() { return Err(err("INVALID_ARGUMENT")); }
        if let Some(id) = target {
            // Unknown IDs do not expose existence or ownership of undiscovered nodes.
            if !state.known.contains(&id) { return Err(err("UNDISCOVERED")); }
            let node = self.nodes.get(&id).ok_or_else(||err("UNKNOWN_NODE"))?;
            if node.kind == "ROUTER" { return Err(err("INVALID_TARGET_TYPE")); }
            if node.router_id != state.router_id { return Err(err("OUT_OF_SCOPE")); }
            if action != "SHIELD_STATUS" {
                if matches!(node.power.as_str(),"STARTING"|"RESTORING") { return Err(err("TARGET_BUSY")); }
                if matches!(action,"HOST_SCAN"|"CONNECTION_SCAN"|"FULL_HOST_SCAN") && node.power != "ONLINE" { return Err(err("OFFLINE")); }
            }
        }
        if let Some(p) = peer {
            if p == target.unwrap_or(state.router_id) { return Err(err("INVALID_ARGUMENT")); }
            if !state.known.contains(&p) { return Err(err("UNDISCOVERED")); }
            if !self.nodes.contains_key(&p) { return Err(err("UNKNOWN_NODE")); }
        }
        if admission {
            if self.running_by_router.get(&state.router_id).map_or(0,BTreeSet::len) >= 5 { return Err(err("TASK_SLOTS_FULL")); }
            if !readonly(action) && self.mutation_locks.contains_key(&target.unwrap_or(state.router_id)) { return Err(err("TARGET_BUSY")); }
        }
        Ok((action.into(),target,peer,duration))
    }
    pub fn api(&mut self, agent: u64, method: &str, args: Value) -> Result<Value,String> {
        if method!="deploy" {return self.api_read(agent,method,args);}
        let state=self.agents.get(&agent).ok_or_else(||err("UNKNOWN_AGENT"))?;
        let router=state.router_id;
        let args=args.as_array().filter(|args|args.len()==1).ok_or_else(||err("INVALID_ARGUMENT"))?;
        let (action,target,peer,duration)=self.descriptor(agent,&args[0],true)?;
        let seq=self.submissions.entry(router).or_default();*seq+=1;
        self.task_sequence=self.task_sequence.checked_add(1).filter(|v|*v<=9_007_199_254_740_991).ok_or_else(||err("LIMIT_EXCEEDED"))?;
        let id=self.task_sequence;
        self.running_by_router.entry(router).or_default().insert(id);
        if !readonly(&action){self.mutation_locks.insert(target.unwrap_or(router),id);}
        let job=Job{id,agent_id:agent,router_id:router,source:String::new(),action,target,peer,state:"RUNNING".into(),started_at:self.tick,finished_at:None,fuel_spent:self.config.task_admission_cost,duration,result:None,failure:None,entered:false,changed:false,local_sequence:*seq};
        self.jobs.insert(id,job);
        self.emit("task_admitted",Some(agent),"core",json!({"task_id":id}));
        Ok(json!(id))
    }
    /// Queries read committed world state and the calling router's reservations.
    /// Separate shared reads let independent Lua VMs convert results concurrently.
    pub fn api_read(&self, agent: u64, method: &str, args: Value) -> Result<Value,String> {
        let state = self.agents.get(&agent).ok_or_else(||err("UNKNOWN_AGENT"))?;
        let router = state.router_id;
        let args = args.as_array().ok_or_else(||err("INVALID_ARGUMENT"))?;
        if method == "can_deploy" && args.len() != 1 { return Ok(json!([false,"INVALID_ARGUMENT"])); }
        let zero = matches!(method,"self"|"router"|"others"|"ticks"|"slots"|"tasks");
        if (zero && !args.is_empty()) || (!zero && method != "fuel" && args.len()!=1) || (method=="fuel" && args.len()>1) { return Err(err("INVALID_ARGUMENT")); }
        let first = args.first().unwrap_or(&Value::Null);
        match method {
            "self" => Ok(json!(agent)), "router" => Ok(json!(router)), "ticks" => Ok(json!(self.tick)),
            "others" => Ok(Value::Object(self.agents.values().filter(|a|a.id!=agent).map(|a|(a.id.to_string(),json!(a.router_id))).collect())),
            "fuel" => {
                let id = if args.is_empty() {agent} else {first.as_u64().ok_or_else(||err("INVALID_ARGUMENT"))?};
                let a = self.agents.get(&id).ok_or_else(||err("UNKNOWN_AGENT"))?;
                Ok(json!(if id==agent {a.fuel} else {a.last_fuel}))
            }
            "slots" => {let n=self.running_by_router.get(&router).map_or(0,BTreeSet::len); Ok(json!({"capacity":5,"running":n,"available":5-n}))},
            "tasks" => Ok(json!(self.running_by_router.get(&router).map(|ids|ids.iter().copied().collect::<Vec<_>>()).unwrap_or_default())),
            "cost"|"can_deploy" => {
                let parsed = self.descriptor(agent,first,method!="cost");
                if method=="can_deploy" { return Ok(match parsed {Ok(_)=>json!([true]),Err(e)=>json!([false,e.split(':').next().unwrap()])}); }
                let (_,_,_,duration)=parsed?;
                Ok(json!({"fuel":self.config.task_admission_cost,"ticks":duration,"exact":true,"quoted_at":self.tick}))
            }
            "has_task"|"is_finished"|"is_ready"|"status"|"result" => {
                let j=first.as_u64().and_then(|id|self.jobs.get(&id)).filter(|j|j.router_id==router);
                if method=="has_task" {return Ok(json!(j.is_some()));}
                if method=="is_finished" {return Ok(json!(j.is_some_and(|j|j.state!="RUNNING")));}
                if method=="is_ready" {return Ok(json!(j.is_some_and(|j|j.state=="COMPLETED")));}
                let j=j.ok_or_else(||err("UNKNOWN_TASK"))?;
                if method=="status" { let mut v=serde_json::to_value(j).unwrap(); v.as_object_mut().unwrap().remove("result"); return Ok(v); }
                if let Some(f)=&j.failure {return Err(f.clone());}
                j.result.clone().ok_or_else(||err("TASK_NOT_FINISHED"))
            }
            _ => Err(err("INVALID_ARGUMENT")),
        }
    }
    /// Sorted-neighbour BFS picks a stable shortest route. Same-LAN traffic has no router hop.
    pub fn route(&self, source: u64, target: u64) -> Option<Vec<u64>> {
        let a=self.nodes.get(&source)?; let b=self.nodes.get(&target)?;
        if a.kind=="ROUTER" || b.kind=="ROUTER" {return None;}
        if a.router_id==b.router_id {return Some(vec![]);}
        let source=self.routing.index[&a.router_id];let mut current=self.routing.index[&b.router_id];
        if self.routing.parents[source][current]==usize::MAX{return None;}
        let mut path=vec![b.router_id];
        while current!=source {current=self.routing.parents[source][current];path.push(self.routing.routers[current]);}
        path.reverse();Some(path)
    }
    pub fn traffic_allowed(&self, source: u64, target: u64) -> bool {
        let (Some(a),Some(b))=(self.nodes.get(&source),self.nodes.get(&target)) else {return false;};
        if source==target || a.kind=="ROUTER" || b.kind=="ROUTER" || a.power!="ONLINE" || b.power!="ONLINE" || !a.allows(target) || !b.allows(source) {return false;}
        if a.router_id==b.router_id{return true;}
        let start=self.routing.index[&a.router_id];let mut current=self.routing.index[&b.router_id];
        if self.routing.parents[start][current]==usize::MAX{return false;}
        loop {
            let router=&self.nodes[&self.routing.routers[current]];
            if !router.allows(source)||!router.allows(target){return false;}
            if current==start{return true;}
            current=self.routing.parents[start][current];
        }
    }
    pub fn communicate(&mut self, source:u64,target:u64) -> Option<u64> {
        if !self.traffic_allowed(source,target) {return None;}
        for r in self.route(source,target)? {self.history.entry(r).or_default().insert((source.min(target),source.max(target)));}
        if let Some(id)=self.session_by_pair.get(&(source,target)) {return Some(*id);}
        self.session_sequence+=1;let id=self.session_sequence;
        self.session_by_pair.insert((source,target),id);
        for host in [source,target]{self.sessions_by_host.entry(host).or_default().insert(id);}
        self.sessions.insert(id,Session{id,source,target,opened_at:self.tick});Some(id)
    }
    fn remove_session(&mut self,id:u64){
        if let Some(session)=self.sessions.remove(&id){
            self.session_by_pair.remove(&(session.source,session.target));
            for host in [session.source,session.target]{if let Some(ids)=self.sessions_by_host.get_mut(&host){ids.remove(&id);if ids.is_empty(){self.sessions_by_host.remove(&host);}}}
        }
    }
    fn close_host_sessions(&mut self,host:u64,inbound_only:bool){
        let ids=self.sessions_by_host.get(&host).into_iter().flatten().copied().filter(|id|!inbound_only||self.sessions[id].target==host).collect::<Vec<_>>();
        for id in ids{self.remove_session(id);}
    }
    fn connections(&self,target:u64) -> Value {
        let mut records:Vec<_>=self.sessions_by_host.get(&target).into_iter().flatten().map(|id|{let s=&self.sessions[id];if s.source==target{(s.target,s.opened_at,"OUTBOUND")}else{(s.source,s.opened_at,"INBOUND")}}).collect();
        records.sort();json!(records.into_iter().map(|(peer,opened_at,direction)|json!({"peer":peer,"opened_at":opened_at,"direction":direction})).collect::<Vec<_>>())
    }
    fn mutation(&mut self,j:&mut Job) -> Result<Value,String> {
        let node=self.nodes.get_mut(&j.target.unwrap_or(j.router_id)).ok_or_else(||err("UNKNOWN_NODE"))?;
        let before=node.clone();
        match j.action.as_str() {
            "SHUTDOWN"=>node.power="OFFLINE".into(),
            "STARTUP"=>node.power="ONLINE".into(),
            "RESTORE"=>{node.power="ONLINE".into();node.compromised=false;},
            "ISOLATE_HOST"=>node.isolated=true,"RELEASE_HOST"=>node.isolated=false,
            a if a.starts_with("SET_DEFAULT_")=>{let policy=if a.starts_with("SET_DEFAULT_ALLOW") {"DEFAULT_ALLOW"}else{"DEFAULT_DENY"};if node.policy!=policy {node.policy=policy.into();node.exceptions.clear();}},
            a if a.starts_with("BLOCK_PEER") || a.starts_with("ALLOW_PEER")=>{
                let p=j.peer.ok_or_else(||err("INVALID_ARGUMENT"))?;
                if a.starts_with("BLOCK_PEER")== (node.policy=="DEFAULT_ALLOW") {node.exceptions.insert(p);}else{node.exceptions.remove(&p);}
            }
            _=>return Err(err("UNKNOWN_ACTION")),
        }
        let changed=j.changed || before.power!=node.power || before.compromised!=node.compromised || before.policy!=node.policy || before.exceptions!=node.exceptions || before.isolated!=node.isolated;
        Ok(json!({"changed":changed}))
    }
    fn scan(&mut self,j:&Job) -> Result<Value,String> {
        if let Some(id)=j.target {
            let n=&self.nodes[&id];
            if j.action!="SHIELD_STATUS" {match n.power.as_str() {"OFFLINE"=>return Err(err("OFFLINE")),"STARTING"|"RESTORING"=>return Err(err("TARGET_BUSY")),_=>{}}}
        }
        let mut discovered=BTreeSet::new();
        let data=match j.action.as_str() {
            "SCAN"=>{
                let hosts:Vec<_>=self.hosts_by_router.get(&j.router_id).into_iter().flatten().map(|id|{let n=&self.nodes[id];discovered.insert(n.id);json!({"id":n.id,"type":n.kind})}).collect();
                let routers:BTreeSet<_>=self.routing.neighbours[&j.router_id].iter().copied().collect();
                discovered.extend(&routers);json!({"hosts":hosts,"routers":routers})
            }
            "TRAFFIC_SCAN"=>{let pairs:Vec<_>=self.history.get(&j.router_id).into_iter().flatten().map(|&(a,b)|{discovered.extend([a,b]);json!({"a":a,"b":b})}).collect();json!({"pairs":pairs})},
            "SHIELD_STATUS"=>{let n=&self.nodes[&j.target.unwrap_or(j.router_id)];json!({"policy":n.policy,"exceptions":n.exceptions,"isolated":n.isolated})},
            "HOST_SCAN"=>{let n=&self.nodes[&j.target.unwrap()];json!({"power":n.power,"assessment":if n.compromised {"COMPROMISED"}else if self.exposure_by_target.contains_key(&n.id) {"SUSPICIOUS"}else{"CLEAN"}})},
            "CONNECTION_SCAN"|"FULL_HOST_SCAN"=>{
                let id=j.target.unwrap();let con=self.connections(id);
                for entry in con.as_array().unwrap() {discovered.insert(entry["peer"].as_u64().unwrap());}
                if j.action=="FULL_HOST_SCAN" {let n=&self.nodes[&id];json!({"power":n.power,"compromised":n.compromised,"connections":con})}else{json!({"connections":con})}
            }
            _=>return Err(err("UNKNOWN_ACTION")),
        };
        let mut data=data;data["observed_at"]=json!(self.tick);
        let a=self.agents.get_mut(&j.agent_id).unwrap();a.known.extend(discovered);
        Ok(data)
    }
    fn random(&mut self)->u64 {let mut x=self.rng;x^=x<<13;x^=x>>7;x^=x<<17;self.rng=x;x}
    fn interval(&mut self) {
        self.enemy_updates+=1;
        let invalid:Vec<_>=self.sessions.values().filter(|s|!self.traffic_allowed(s.source,s.target)).map(|s|s.id).collect();
        for id in invalid {self.remove_session(id);}
        let old=std::mem::take(&mut self.attacks);
        let valid:BTreeMap<_,_>=old.iter().filter(|((s,t),_)|self.nodes[s].compromised&&self.nodes[s].power=="ONLINE"&&!self.nodes[t].compromised&&self.traffic_allowed(*s,*t)).map(|(pair,age)|(*pair,*age)).collect();
        for (&(source,target),&age) in &old {if !valid.contains_key(&(source,target)) {
            if let Some(id)=self.session_by_pair.get(&(source,target)).copied(){self.remove_session(id);}
            self.emit("exposure_reset",None,"enemy",json!({"source":source,"target":target,"previous_age":age,"reason":"session interrupted"}));
        }}
        let mut claimed:BTreeSet<u64>=valid.keys().map(|&(_,target)|target).collect();
        let mut active_by_source=BTreeMap::<_,Vec<_>>::new();
        for &(source,target) in valid.keys(){active_by_source.entry(source).or_default().push(target);}
        // Capture sources before infections: a newly infected host cannot spread in this interval.
        let sources:Vec<_>=self.nodes.values().filter(|n|n.compromised&&n.power=="ONLINE"&&n.kind!="ROUTER").map(|n|n.id).collect();
        let mut infected=BTreeSet::new();
        // A single sorted boundary inventory; target permission stays live and
        // claimed hosts are excluded per source. Preserve RNG candidate order.
        let eligible:Vec<_>=self.nodes.values().filter(|n|n.kind!="ROUTER"&&!n.compromised&&n.power=="ONLINE"&&!n.isolated).map(|n|n.id).collect();
        for source in sources {
            let mut selected=active_by_source.remove(&source).unwrap_or_default();
            selected.truncate(self.config.attack_slots);
            // Existing sessions already occupy their slots. Avoid a full-world
            // candidate pass unless this source actually needs a replacement.
            if selected.len()<self.config.attack_slots && !self.nodes[&source].isolated {
                let mut candidates:Vec<_>=eligible.iter().copied().filter(|id|!claimed.contains(id)&&self.traffic_allowed(source,*id)).collect();
                while selected.len()<self.config.attack_slots&&!candidates.is_empty() {let i=(self.random() % candidates.len() as u64) as usize;let target=candidates.remove(i);claimed.insert(target);selected.push(target);}
            }
            for target in selected {
                self.attack_attempts+=1;
                *self.host_attempts.entry(source).or_default()+=1;
                if self.communicate(source,target).is_some() {
                    self.successful_connections+=1;
                    *self.host_connections.entry(source).or_default()+=1;
                    let progress=old.get(&(source,target)).copied().unwrap_or(0)+1;
                    if progress>=self.config.infection_intervals {infected.insert(target);}else{self.attacks.insert((source,target),progress);}
                }
            }
        }
        for target in infected {self.nodes.get_mut(&target).unwrap().compromised=true;self.close_host_sessions(target,true);self.emit("infection",None,"enemy",json!({"target":target}));}
        self.exposure_by_target.clear();
        for (&(_,target),&age) in &self.attacks {self.exposure_by_target.entry(target).and_modify(|current|*current=(*current).max(age)).or_insert(age);}
    }
    fn finish(&mut self,mut j:Job,outcome:Result<Value,String>) {
        if let Some(ids)=self.running_by_router.get_mut(&j.router_id){ids.remove(&j.id);}
        if !readonly(&j.action){self.mutation_locks.remove(&j.target.unwrap_or(j.router_id));}
        j.finished_at=Some(self.tick);
        match outcome {
            Ok(data)=>{
                j.state="COMPLETED".into();
                let mut result=json!({"task_id":j.id,"action":j.action,"router_id":j.router_id,"started_at":j.started_at,"finished_at":self.tick,"fuel_spent":j.fuel_spent,"data":data});
                if let Some(t)=j.target {result["target"]=json!(t);}
                if readonly(&j.action) {let mut observation=result.clone();observation["observed_at"]=json!(self.tick);observation["source"]=json!(j.source);self.agents.get_mut(&j.agent_id).unwrap().observations.push(observation);}
                j.result=Some(result);
            }
            Err(f)=>{j.state="FAILED".into();j.failure=Some(f);if matches!(j.action.as_str(),"STARTUP"|"RESTORE") {self.nodes.get_mut(&j.target.unwrap()).unwrap().power="OFFLINE".into();}}
        }
        self.emit("task_finished",Some(j.agent_id),&j.source,json!({"task_id":j.id,"state":j.state,"failure":j.failure}));
        self.jobs.insert(j.id,j);
    }
    pub fn barrier(&mut self)->Result<(),String> {
        if self.score.as_ref().is_some_and(|s|s.defeated_at.is_some() || s.verified_at.is_some()) || (self.score.is_some() && self.config.tick_limit.is_some_and(|h|self.tick>=h)) {return Err("INVALID_ARGUMENT: scored match is finished".into());}
        self.tick=self.tick.checked_add(1).ok_or_else(||err("LIMIT_EXCEEDED"))?;
        let mut running:Vec<_>=self.running_by_router.values().flatten().map(|id|{let j=&self.jobs[id];(j.router_id,j.local_sequence,j.id)}).collect();
        running.sort();
        let mut scans=vec![];
        // Mutations use router ID then local sequence, independent of global IDs.
        for (_,_,id) in running {
            let mut j=self.jobs.remove(&id).unwrap();
            if !j.entered && matches!(j.action.as_str(),"STARTUP"|"RESTORE") {
                let n=self.nodes.get_mut(&j.target.unwrap()).unwrap();
                j.changed=if j.action=="RESTORE" {true}else{n.power!="ONLINE"};
                if j.action=="RESTORE" {n.power="RESTORING".into();}else if n.power=="OFFLINE" {n.power="STARTING".into();}
                // Recovery interrupts any partial infection, even if it completes this boundary.
                let interrupted=if n.power!="ONLINE"{Some(n.id)}else{None};
                if let Some(host)=interrupted{self.close_host_sessions(host,false);}
                j.entered=true;
            }
            if self.tick-j.started_at>=j.duration {
                if readonly(&j.action) {scans.push(j);}else{let outcome=self.mutation(&mut j);self.finish(j,outcome);}
            }else{self.jobs.insert(id,j);}
        }
        self.interval();
        for j in scans {let outcome=self.scan(&j);self.finish(j,outcome);}
        for agent in self.agents.values_mut() {agent.last_fuel=agent.fuel;}
        if let Some(mut score)=self.score.take() {score.observe(self);self.score=Some(score);}
        Ok(())
    }
    pub(crate) fn has_exposure(&self)->bool { !self.attacks.is_empty() }
    pub fn snapshot(&self)->Value {
        let mut scenario=serde_json::to_value(&self.config).unwrap();scenario["tick_cost"]=action_registry();
        scenario["rng_state"]=json!(self.rng.to_string());
        scenario["seed"]=json!(self.config.seed.to_string());
        let mut links:Vec<_>=self.edges.iter().map(|&(source,target)|json!({"source":source,"target":target})).collect();
        links.extend(self.nodes.values().filter(|n|n.kind!="ROUTER").map(|n|json!({"source":n.router_id,"target":n.id})));
        let nodes:Vec<_>=self.nodes.values().map(|node|{let mut value=serde_json::to_value(node).unwrap();let age=self.exposure_by_target.get(&node.id).copied().unwrap_or(0);value["exposure"]=if age>0{json!({"age":age,"started_at":self.tick.saturating_sub(age)+1})}else{Value::Null};value["infection"]=json!(if node.kind=="ROUTER"{"IMMUNE"}else if node.compromised{"COMPROMISED"}else if age>0{"IN_DANGER"}else{"HEALTHY"});value["health"]=if node.kind=="ROUTER"{Value::Null}else if node.compromised{json!(0)}else if age>0{json!((5*(self.config.infection_intervals-age)).div_ceil(self.config.infection_intervals).clamp(1,5))}else{json!(5)};value}).collect();
        let sessions:Vec<_>=self.sessions.values().map(|s|{let mut value=serde_json::to_value(s).unwrap();value["route"]=json!(self.route(s.source,s.target));value}).collect();
        json!({"tick":self.tick,"scenario":scenario,"nodes":nodes,"links":links,
            "score":self.score.as_ref().map(|s|s.snapshot(self)).unwrap_or(json!({"enabled":false,"outcome":"PRACTICE"})),
            "sessions":sessions,"jobs":self.jobs.values().collect::<Vec<_>>(),
            "enemy_updates":self.enemy_updates,"attack_attempts":self.attack_attempts,"successful_connections":self.successful_connections,
            "host_activity":self.host_attempts.iter().map(|(host,attempts)|json!({"host_id":host,"attack_attempts":attempts,"successful_connections":self.host_connections.get(host).copied().unwrap_or(0)})).collect::<Vec<_>>(),
            "history":{"first_seq":self.events.front().map_or(self.sequence+1,|e|e.sequence),"evicted":self.sequence.saturating_sub(self.events.len() as u64)},
            "agents":self.agents.values().collect::<Vec<_>>(),"events":self.events})
    }
}

#[cfg(test)]
#[path="sim_tests.rs"]
mod tests;
