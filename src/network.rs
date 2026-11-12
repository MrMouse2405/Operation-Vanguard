//! Explicit, versioned TOML topology. Agent selection belongs to the CLI only.
use std::{collections::{BTreeMap, BTreeSet}, fs, path::PathBuf};
use serde::Deserialize;
use crate::sim::{Config, Node};

#[derive(Clone)]
pub struct RunFiles {
    pub script: PathBuf,
    pub network: PathBuf,
    pub dbg: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Network {
    #[serde(default)]
    mission: Option<crate::scoring::Mission>,
    version: u32,
    #[serde(default)]
    simulation: Settings,
    routers: Vec<Router>,
    lans: Vec<Lan>,
    #[serde(default)]
    workstations: Vec<Host>,
    #[serde(default)]
    servers: Vec<Host>,
    #[serde(default)]
    connections: Vec<Connection>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Router { id: u64 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lan { id: u64, router: u64 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Host { id: u64, lan: u64, #[serde(default)] compromised: bool }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Connection { from: u64, to: u64 }

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    seed: Option<u64>, tick_limit: Option<u64>, tick_ms: Option<u64>,
    fuel_per_tick: Option<u64>, instruction_quantum: Option<u64>,
    memory_limit: Option<usize>, max_payload_bytes: Option<usize>, max_handlers: Option<usize>,
    native_call_cost: Option<u64>, task_admission_cost: Option<u64>,
    infection_intervals: Option<u64>, attack_slots: Option<usize>,
}

impl Network {
    fn into_config(self) -> Result<Config, String> {
        if self.version != 1 { return Err("network version must be 1".into()); }
        let mut config = Config::default();
        macro_rules! settings { ($($field:ident),*) => {$(if let Some(value) = self.simulation.$field { config.$field = value; })*}; }
        settings!(seed, tick_ms, fuel_per_tick, instruction_quantum, memory_limit,
            max_payload_bytes, max_handlers, native_call_cost, task_admission_cost,
            infection_intervals, attack_slots);
        if let Some(limit) = self.simulation.tick_limit { config.tick_limit = (limit != 0).then_some(limit); }
        let routers: BTreeSet<_> = self.routers.iter().map(|r| r.id).collect();
        if routers.len() != self.routers.len() { return Err("duplicate router ID".into()); }
        let mut lans = BTreeMap::new();
        let mut owners = BTreeSet::new();
        for lan in self.lans {
            if lan.id == 0 || lans.insert(lan.id, lan.router).is_some() { return Err("LAN IDs must be positive and unique".into()); }
            if !routers.contains(&lan.router) { return Err(format!("LAN {} references unknown router {}", lan.id, lan.router)); }
            if !owners.insert(lan.router) { return Err(format!("router {} must own exactly one LAN", lan.router)); }
        }
        if owners != routers { return Err("every router must own exactly one LAN".into()); }
        let mut nodes: Vec<_> = routers.iter().map(|id| {let mut n=Node::new(*id,"ROUTER",*id);n.lan_id=*lans.iter().find(|(_,router)|**router==*id).unwrap().0;n}).collect();
        let mut infected = Vec::new();
        config.hosts_per_lan = 0; // Explicit membership, never a topology generator.
        config.server_count = self.servers.len();
        for (kind, hosts) in [("WORKSTATION", self.workstations), ("SERVER", self.servers)] {
            for host in hosts {
                let router = *lans.get(&host.lan).ok_or_else(|| format!("host {} references unknown LAN {}", host.id, host.lan))?;
                if host.compromised {
                    if kind != "WORKSTATION" { return Err(format!("server {} cannot start compromised", host.id)); }
                    infected.push(host.id);
                }
                let mut node=Node::new(host.id,kind,router);node.lan_id=host.lan;nodes.push(node);
            }
        }
        config.router_count = routers.len();
        config.nodes = Some(nodes);
        config.router_edges = Some(self.connections.into_iter().map(|c| [c.from, c.to]).collect());
        config.initial_compromised = Some(infected); // Empty means clean, not an implicit infection.
        let eligible=config.tick_limit.is_some() && config.server_count>0 && config.nodes.as_ref().unwrap().iter().any(|n|n.kind=="WORKSTATION");
        let mut mission=self.mission.or_else(||eligible.then(crate::scoring::Mission::default));
        if let Some(m)=&mut mission {m.expand(&config);}
        config.mission=mission;
        config.validate()?;
        Ok(config)
    }
}

pub fn parse(text: &str) -> Result<Config, String> {
    toml::from_str::<Network>(text).map_err(|error| error.to_string())?.into_config()
}

impl RunFiles {
    pub fn load(&self) -> Result<Config, String> {
        let text = fs::read_to_string(&self.network).map_err(|e| format!("{}: {e}", self.network.display()))?;
        let mut config = parse(&text).map_err(|e| format!("{}: {e}", self.network.display()))?;
        config.dbg = self.dbg;
        let script = fs::canonicalize(&self.script).map_err(|e| format!("{}: {e}", self.script.display()))?;
        if script.extension().is_none_or(|ext| ext != "lua") || !script.is_file() {
            return Err(format!("{}: --script must name a Lua source file", script.display()));
        }
        config.script_root = script.parent().unwrap().to_string_lossy().into_owned();
        config.script = Some(script.file_name().unwrap().to_str().ok_or("script filename must be UTF-8")?.into());
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SMALL: &str = "version=1\nrouters=[{id=1},{id=2}]\nlans=[{id=10,router=1},{id=20,router=2}]\nworkstations=[{id=1001,lan=10,compromised=true}]\nservers=[{id=2001,lan=20}]\nconnections=[{from=1,to=2}]\n";
    #[test]
    fn explicit_topology_and_infection_are_preserved_without_agent_defaults() {
        let config = parse(SMALL).unwrap();
        assert!(config.script.is_none());
        assert_eq!(config.initial_compromised, Some(vec![1001]));
        assert_eq!(config.nodes.as_ref().unwrap().len(), 4);
        assert_eq!(config.nodes.as_ref().unwrap().iter().find(|n| n.id==2001).unwrap().router_id, 2);
        assert_eq!(parse(&SMALL.replace("compromised=true", "compromised=false")).unwrap().initial_compromised, Some(vec![]));
    }
    #[test]
    fn invalid_networks_and_script_overrides_are_rejected() {
        for bad in [SMALL.replace("id=2001,lan=20", "id=2001,lan=20,compromised=true"),
            SMALL.replace("lan=20", "lan=99"), SMALL.replace("from=1,to=2", "from=1,to=99"),
            SMALL.replace("connections=[{from=1,to=2}]", "connections=[]"),
            SMALL.replace("id=2001", "id=1001"), format!("script='ostrich.lua'\n{SMALL}"),
            SMALL.replace("{id=1}", "{id=1,compromised=true}")] {
            assert!(parse(&bad).is_err(), "accepted {bad}");
        }
    }
}
