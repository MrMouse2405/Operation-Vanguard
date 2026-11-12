use super::*;

#[test]
fn attacks_have_exclusive_targets_and_incoming_sessions_close_on_infection(){
    let mut c=Core::new(Config{router_count:1,hosts_per_lan:8,server_count:0,initial_compromised:Some(vec![1001,1002]),..Config::default()}).unwrap();
    c.barrier().unwrap();
    let targets:Vec<_>=c.attacks.keys().map(|(_,target)|*target).collect();
    assert_eq!(targets.len(),targets.iter().collect::<BTreeSet<_>>().len());
    assert_eq!(targets.len(),4);
    advance(&mut c,13);
    assert_eq!(c.nodes.values().filter(|n|n.compromised).count(),6);
    assert!(c.sessions.values().all(|session|!c.nodes[&session.target].compromised));
    c.barrier().unwrap();
    assert_eq!(c.attacks.len(),2,"remaining clean hosts are claimed once, regardless of source population");
    assert!(c.attacks.keys().all(|(source,_)|c.attacks.keys().filter(|(s,_)|s==source).count()<=2));
}

#[test]
fn invalid_scenario_graph_and_atomic_limits_fail_validation(){
    for edges in [vec![[1,2]],vec![[1,2],[2,3],[3,4],[4,1],[2,1]]]{
        let c=Config{router_edges:Some(edges),..Config::default()};assert!(c.validate().is_err());
    }
    let c=Config{max_payload_bytes:100_000,..Config::default()};assert!(c.validate().unwrap_err().contains("atomic"));
    let c=Config{initial_compromised:Some(vec![1]),..Config::default()};assert!(c.validate().is_err());
}

fn small() -> Core {
    Core::new(Config {router_count:2,hosts_per_lan:2,server_count:0,
        initial_compromised:Some(vec![]),..Config::default()}).unwrap()
}
fn submit(c:&mut Core,agent:u64,descriptor:Value)->u64 {
    c.api(agent,"deploy",json!([descriptor])).unwrap().as_u64().unwrap()
}
fn advance(c:&mut Core,n:usize) {for _ in 0..n {c.barrier().unwrap();}}
fn discover(c:&mut Core) {
    for a in c.agents.values_mut() {a.known.extend(c.nodes.keys());}
}
fn result(c:&mut Core,agent:u64,id:u64)->Value {c.api(agent,"result",json!([id])).unwrap()}

#[test]
fn duration_is_admission_relative_and_scan_reads_completion_snapshot() {
    let mut c=small(); advance(&mut c,7);discover(&mut c);
    let id=submit(&mut c,101,json!({"action":"HOST_SCAN","target":1001}));
    assert_eq!(c.jobs[&id].started_at,7);
    advance(&mut c,2); assert_eq!(c.jobs[&id].state,"RUNNING");
    c.nodes.get_mut(&1001).unwrap().compromised=true;
    advance(&mut c,1);
    let r=result(&mut c,101,id);
    assert_eq!(r["finished_at"],10);assert_eq!(r["data"]["observed_at"],10);
    assert_eq!(r["data"]["assessment"],"COMPROMISED");
    assert_eq!(c.agents[&101].observations.len(),1);
}

#[test]
fn infection_requires_fourteen_intervals_and_never_recurses() {
    let mut c=Core::new(Config {router_count:1,hosts_per_lan:4,server_count:0,
        attack_slots:1,..Config::default()}).unwrap();
    advance(&mut c,13);
    assert_eq!(c.nodes.values().filter(|n|n.compromised).count(),1);
    advance(&mut c,1);
    assert_eq!(c.nodes.values().filter(|n|n.compromised).count(),2);
    assert!(c.attacks.is_empty(),"newly infected hosts must not spread recursively");
    advance(&mut c,1);
    assert!(c.attacks.values().all(|&p|p==1));
    assert!(!c.nodes[&1].compromised);
}

#[test]
fn scenario_rejects_router_and_server_initial_infection() {
    let mut config=Config::default();config.initial_compromised=Some(vec![1]);
    assert!(Core::new(config.clone()).is_err());
    config.initial_compromised=Some(vec![1017]);assert!(Core::new(config).is_err());
    let mut config=Config::default();config.fuel_per_tick=100;assert!(config.validate().is_err());
    let mut config=Config::default();config.router_edges=Some(vec![[1,999]]);assert!(config.validate().is_err());
}

#[test]
fn same_lan_bypasses_router_but_not_endpoints_and_transit_checks_both_ids() {
    let mut c=small();
    c.nodes.get_mut(&1).unwrap().policy="DEFAULT_DENY".into();
    assert!(c.traffic_allowed(1001,1002));
    assert!(!c.traffic_allowed(1001,1003));
    c.nodes.get_mut(&1).unwrap().exceptions.insert(1001);
    assert!(!c.traffic_allowed(1001,1003),"source allowance is insufficient");
    c.nodes.get_mut(&1).unwrap().exceptions.insert(1003);
    assert!(c.traffic_allowed(1001,1003));
    c.nodes.get_mut(&2).unwrap().exceptions.insert(1003);
    assert!(!c.traffic_allowed(1001,1003));
    c.nodes.get_mut(&1002).unwrap().isolated=true;
    assert!(!c.traffic_allowed(1001,1002));
}

#[test]
fn policy_idempotence_isolation_and_policy_change_preserve_correct_state() {
    let mut c=small();discover(&mut c);
    let block=submit(&mut c,101,json!({"action":"BLOCK_PEER_ON_HOST","target":1001,"peer":1002}));advance(&mut c,1);
    assert_eq!(result(&mut c,101,block)["data"]["changed"],true);
    submit(&mut c,101,json!({"action":"SET_DEFAULT_ALLOW_ON_HOST","target":1001}));advance(&mut c,1);
    assert!(c.nodes[&1001].exceptions.contains(&1002));
    submit(&mut c,101,json!({"action":"ISOLATE_HOST","target":1001}));advance(&mut c,1);
    submit(&mut c,101,json!({"action":"RELEASE_HOST","target":1001}));advance(&mut c,1);
    assert!(c.nodes[&1001].exceptions.contains(&1002));
    submit(&mut c,101,json!({"action":"SET_DEFAULT_DENY_ON_HOST","target":1001}));advance(&mut c,1);
    assert!(c.nodes[&1001].exceptions.is_empty());
}

#[test]
fn interruption_discards_partial_infection_and_sessions_but_preserves_history() {
    let mut c=small();c.nodes.get_mut(&1001).unwrap().compromised=true;
    c.config.attack_slots=1;
    // Force the only available target onto the other LAN.
    c.nodes.get_mut(&1002).unwrap().isolated=true;c.nodes.get_mut(&1004).unwrap().isolated=true;
    advance(&mut c,8);assert_eq!(c.attacks[&(1001,1003)],8);
    assert!(c.history[&1].contains(&(1001,1003)));
    c.nodes.get_mut(&1003).unwrap().isolated=true;advance(&mut c,1);
    assert!(c.attacks.is_empty());assert!(c.sessions.is_empty());
    c.nodes.get_mut(&1003).unwrap().isolated=false;advance(&mut c,13);
    assert!(!c.nodes[&1003].compromised);advance(&mut c,1);assert!(c.nodes[&1003].compromised);
    assert!(c.history[&1].contains(&(1001,1003)));
}

#[test]
fn discovery_scope_slots_and_locks_are_local_and_atomic() {
    let mut c=small();
    let e=c.api(101,"deploy",json!([{"action":"HOST_SCAN","target":1001}])).unwrap_err();assert!(e.starts_with("UNDISCOVERED:"));
    let scan=submit(&mut c,101,json!({"action":"SCAN"}));advance(&mut c,2);
    assert_eq!(result(&mut c,101,scan)["data"]["routers"],json!([2]));
    assert!(c.agents[&101].known.contains(&1001));assert!(!c.agents[&101].known.contains(&1003));
    c.agents.get_mut(&101).unwrap().known.insert(1003);
    assert!(c.api(101,"deploy",json!([{"action":"HOST_SCAN","target":1003}])).unwrap_err().starts_with("OUT_OF_SCOPE:"));
    let restore=submit(&mut c,101,json!({"action":"RESTORE","target":1001}));
    assert!(c.api(101,"deploy",json!([{"action":"ISOLATE_HOST","target":1001}])).unwrap_err().starts_with("TARGET_BUSY:"));
    for _ in 0..4 {submit(&mut c,101,json!({"action":"SCAN"}));}
    assert_eq!(c.api(101,"slots",json!([])).unwrap()["available"],0);
    assert!(c.api(101,"deploy",json!([{"action":"SCAN"}])).unwrap_err().starts_with("TASK_SLOTS_FULL:"));
    assert!(c.api(101,"cost",json!([{"action":"SCAN"}])).is_ok());
    assert_eq!(c.jobs.len(),6);assert_eq!(c.api(102,"has_task",json!([restore])).unwrap(),false);
    assert!(c.api(102,"status",json!([restore])).unwrap_err().starts_with("UNKNOWN_TASK:"));
    assert_eq!(c.api(101,"can_deploy",json!([{"action":"SCAN","window_ticks":4}])).unwrap(),json!([false,"INVALID_ARGUMENT"]));
}

#[test]
fn mutations_then_enemy_then_scans_and_terminal_failures_are_data() {
    let mut c=small();discover(&mut c);
    let scan=submit(&mut c,101,json!({"action":"HOST_SCAN","target":1001}));
    advance(&mut c,2);
    submit(&mut c,101,json!({"action":"SHUTDOWN","target":1001}));
    advance(&mut c,1);
    assert_eq!(c.jobs[&scan].state,"FAILED");
    assert!(c.api(101,"status",json!([scan])).unwrap()["failure"].as_str().unwrap().starts_with("OFFLINE:"));
    assert!(c.api(101,"result",json!([scan])).unwrap_err().starts_with("OFFLINE:"));
    assert_eq!(c.api(101,"is_finished",json!([scan])).unwrap(),true);
    assert_eq!(c.api(101,"is_ready",json!([scan])).unwrap(),false);
    let status=submit(&mut c,101,json!({"action":"SHIELD_STATUS","target":1001}));advance(&mut c,1);
    assert!(result(&mut c,101,status)["data"]["exceptions"].is_array());
}

#[test]
fn restore_and_startup_transitions_preserve_policy_and_infection_as_specified() {
    let mut c=small();discover(&mut c);
    {let n=c.nodes.get_mut(&1001).unwrap();n.compromised=true;n.isolated=true;n.exceptions.insert(1002);}
    submit(&mut c,101,json!({"action":"SHUTDOWN","target":1001}));advance(&mut c,1);
    let startup=submit(&mut c,101,json!({"action":"STARTUP","target":1001}));advance(&mut c,1);
    assert_eq!(c.nodes[&1001].power,"STARTING");advance(&mut c,2);
    assert_eq!(c.nodes[&1001].power,"ONLINE");assert!(c.nodes[&1001].compromised);
    assert_eq!(result(&mut c,101,startup)["data"]["changed"],true);
    let restore=submit(&mut c,101,json!({"action":"RESTORE","target":1001}));advance(&mut c,1);
    assert_eq!(c.nodes[&1001].power,"RESTORING");advance(&mut c,18);
    assert!(c.nodes[&1001].compromised);advance(&mut c,1);
    assert_eq!(c.jobs[&restore].state,"COMPLETED");assert!(!c.nodes[&1001].compromised);
    assert!(c.nodes[&1001].isolated);assert!(c.nodes[&1001].exceptions.contains(&1002));
}

#[test]
fn traffic_history_is_unique_sorted_and_discovery_does_not_grant_remote_control() {
    let mut c=small();
    c.communicate(1001,1003);c.communicate(1003,1001);c.communicate(1001,1002);
    let scan=submit(&mut c,101,json!({"action":"TRAFFIC_SCAN"}));advance(&mut c,4);
    assert_eq!(result(&mut c,101,scan)["data"]["pairs"],json!([{"a":1001,"b":1003}]));
    assert!(c.agents[&101].known.contains(&1003));
    assert!(c.api(101,"deploy",json!([{"action":"RESTORE","target":1003}])).unwrap_err().starts_with("OUT_OF_SCOPE:"));
    submit(&mut c,101,json!({"action":"BLOCK_PEER","peer":1003}));advance(&mut c,1);
    assert_eq!(c.sessions.len(),1); // same-LAN session survives router filtering
    assert_eq!(c.history[&1].len(),1);
}

#[test]
fn registry_and_snapshot_are_complete_and_replay_is_deterministic() {
    assert_eq!(action_registry().as_object().unwrap().len(),19);
    for (a,t) in ACTIONS {assert_eq!(action_registry()[a],t);}
    let mut a=Core::new(Config::default()).unwrap();let mut b=Core::new(Config::default()).unwrap();
    advance(&mut a,40);advance(&mut b,40);assert_eq!(a.snapshot(),b.snapshot());
    for field in ["nodes","links","sessions","jobs","agents","events"] {assert!(a.snapshot()[field].is_array());}
    for i in 0..2100 {a.emit("test",None,"test",json!(i));}
    assert_eq!(a.events.len(),2000);
}

#[test]
fn routing_tie_breaks_by_sorted_router_ids() {
    let mut config=Config::default();
    config.router_edges=Some(vec![[1,4],[4,3],[1,2],[2,3]]);
    let c=Core::new(config).unwrap();assert_eq!(c.route(1001,1009),Some(vec![1,2,3]));
}

#[test]
fn exact_scan_observes_infection_from_its_completion_interval() {
    let mut c=small();discover(&mut c);
    c.nodes.get_mut(&1003).unwrap().compromised=true;
    c.nodes.get_mut(&1002).unwrap().isolated=true;
    c.nodes.get_mut(&1004).unwrap().isolated=true;
    advance(&mut c,6);
    let id=submit(&mut c,101,json!({"action":"FULL_HOST_SCAN","target":1001}));
    advance(&mut c,7);assert!(!c.nodes[&1001].compromised);
    advance(&mut c,1);
    let data=result(&mut c,101,id)["data"].clone();
    assert_eq!(data["observed_at"],14);assert_eq!(data["compromised"],true);
    assert!(data["connections"].as_array().unwrap().is_empty(),"incoming attack ends when compromise commits");
}
