//! Public mission rules and deterministic, barrier-only score accounting.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use crate::sim::{Config, Core};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Mission {
    pub enabled: bool,
    pub recovery_ticks: u64,
    pub outage_ticks: u64,
    pub service_threshold_percent: u64,
    pub services: Vec<Service>,
}
impl Default for Mission {
    fn default() -> Self { Self { enabled: true, recovery_ticks: 14, outage_ticks: 30, service_threshold_percent: 90, services: vec![] } }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service { pub client: u64, pub server: u64, #[serde(default="one")] pub weight: u64 }
fn one() -> u64 { 1 }

impl Mission {
    pub fn validate(&self, config: &Config) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        if config.tick_limit.is_none_or(|h| h==0 || h>1_000_000_000) || self.recovery_ticks==0 || self.outage_ticks==0
            || !(1..=100).contains(&self.service_threshold_percent) {
            return Err("INVALID_ARGUMENT: scored missions need a finite deadline (1..1000000000), positive recovery/outage windows and threshold 1..100".into());
        }
        let nodes=config.topology();
        let kinds:std::collections::BTreeMap<_,_>=nodes.iter().map(|n|(n.id,n.kind.as_str())).collect();
        if self.services.is_empty() { return Err("INVALID_ARGUMENT: a scored mission needs service connections".into()); }
        let mut pairs=std::collections::BTreeSet::new();
        for s in &self.services {
            if s.weight==0 || s.weight>1_000_000 || !pairs.insert((s.client,s.server))
                || kinds.get(&s.client)!=Some(&"WORKSTATION")
                || kinds.get(&s.server)!=Some(&"SERVER") {
                return Err("INVALID_ARGUMENT: services need unique workstation/server pairs and weights 1..1000000".into());
            }
        }
        if nodes.iter().filter(|n|n.kind=="SERVER").any(|n|!self.services.iter().any(|s|s.server==n.id)) {
            return Err("INVALID_ARGUMENT: every critical server needs a required client".into());
        }
        Ok(())
    }
    pub fn expand(&mut self, config: &Config) {
        if !self.enabled || !self.services.is_empty() { return; }
        let nodes=config.topology();
        for client in nodes.iter().filter(|n|n.kind=="WORKSTATION") {
            for server in nodes.iter().filter(|n|n.kind=="SERVER") {
                self.services.push(Service { client:client.id,server:server.id,weight:1 });
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Score {
    service_sum: u64,
    clean_sum: u64,
    pub recovery_streak: u64,
    pub outage_streak: u64,
    pub verified_at: Option<u64>,
    pub defeated_at: Option<u64>,
    service_ppm: u64,
    clean_ppm: u64,
    operational_ppm: u64,
    working: Vec<bool>,
}
const SCALE: u64=1_000_000;
fn fraction(a:u64,b:u64)->u64 { if b==0 {0}else{((a as u128*SCALE as u128)/b as u128) as u64} }
impl Score {
    pub fn new(core:&Core)->Self {
        let mut score=Self::default();score.observe(core);
        score.service_sum=0;score.clean_sum=0;score.recovery_streak=0;score.outage_streak=0;
        score.verified_at=None;score.defeated_at=None;score
    }
    pub fn observe(&mut self, core:&Core) {
        if self.defeated_at.is_some() { return; }
        let mission=core.config.mission.as_ref().unwrap();
        self.working=mission.services.iter().map(|s|!core.nodes[&s.client].compromised && !core.nodes[&s.server].compromised && core.traffic_allowed(s.client,s.server)).collect();
        let total:u64=mission.services.iter().map(|s|s.weight).sum();
        let served:u64=mission.services.iter().zip(&self.working).filter(|(_,ok)|**ok).map(|(s,_)|s.weight).sum();
        let mut blocked_hosts=std::collections::BTreeSet::new();
        let mut serving_servers=std::collections::BTreeSet::new();
        for (service,working) in mission.services.iter().zip(&self.working) {
            if *working {serving_servers.insert(service.server);}else{blocked_hosts.insert(service.client);blocked_hosts.insert(service.server);}
        }
        self.service_ppm=fraction(served,total);
        let hosts:Vec<_>=core.nodes.values().filter(|n|n.kind!="ROUTER").collect();
        self.clean_ppm=fraction(hosts.iter().filter(|n|!n.compromised).count() as u64,hosts.len() as u64);
        self.operational_ppm=fraction(hosts.iter().filter(|n|!n.compromised && n.power=="ONLINE" && !n.isolated
            && !blocked_hosts.contains(&n.id)).count() as u64,hosts.len() as u64);
        let servers_ready=core.nodes.values().filter(|n|n.kind=="SERVER").all(|n|serving_servers.contains(&n.id));
        let recovered=self.clean_ppm==SCALE && !core.has_exposure() && servers_ready
            && (served as u128)*100 >= (total as u128)*mission.service_threshold_percent as u128;
        self.recovery_streak=if recovered {self.recovery_streak+1}else{0};
        if !recovered {self.verified_at=None;}
        if self.recovery_streak>=mission.recovery_ticks && self.verified_at.is_none(){self.verified_at=Some(core.tick);}
        self.outage_streak=if served==0 {self.outage_streak+1}else{0};
        self.service_sum+=self.service_ppm;
        self.clean_sum+=self.clean_ppm;
        if self.outage_streak>=mission.outage_ticks {self.defeated_at=Some(core.tick);}
    }
    pub fn snapshot(&self, core:&Core)->Value {
        let mission=core.config.mission.as_ref().unwrap();
        let horizon=core.config.tick_limit.unwrap();
        let service=((60_000u128*self.service_sum as u128)/(horizon as u128*SCALE as u128)) as u64;
        let security=((25_000u128*self.clean_sum as u128)/(horizon as u128*SCALE as u128)) as u64;
        let recovery=if self.defeated_at.is_some(){0}else{15_000*self.operational_ppm/SCALE};
        let outcome=if self.defeated_at.is_some(){"DEFEAT"}else if core.tick>=horizon {if self.verified_at.is_some(){"VICTORY"}else{"INCOMPLETE"}}else if self.verified_at.is_some(){"RECOVERED"}else if self.recovery_streak>0{"VERIFYING"}else{"ACTIVE"};
        json!({"enabled":true,"outcome":outcome,"total":(service+security+recovery) as f64/100.,
            "service":service as f64/100.,"security":security as f64/100.,"recovery":recovery as f64/100.,
            "service_fraction":self.service_ppm as f64/SCALE as f64,"clean_fraction":self.clean_ppm as f64/SCALE as f64,
            "operational_fraction":self.operational_ppm as f64/SCALE as f64,"recovery_streak":self.recovery_streak,
            "outage_streak":self.outage_streak,"verified_at":self.verified_at,"defeated_at":self.defeated_at,
            "horizon":horizon,"service_sum":self.service_sum,"clean_sum":self.clean_sum,
            "services":mission.services.iter().enumerate().map(|(i,s)|json!({"client":s.client,"server":s.server,"weight":s.weight,"working":self.working.get(i).copied().unwrap_or(false)})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn core(horizon:u64)->Core {
        let mut config=crate::network::parse(&format!("version=1\nrouters=[{{id=1}},{{id=2}}]\nlans=[{{id=10,router=1}},{{id=20,router=2}}]\nworkstations=[{{id=1001,lan=10}},{{id=1002,lan=20}}]\nservers=[{{id=2001,lan=20}}]\nconnections=[{{from=1,to=2}}]\n[simulation]\ntick_limit={horizon}\n")).unwrap();
        config.mission.as_mut().unwrap().outage_ticks=3;
        Core::new(config).unwrap()
    }
    #[test]
    fn perfect_match_requires_stable_recovery_and_scores_1000() {
        let mut c=core(20);
        assert_eq!(c.nodes[&1001].lan_id,10);
        for _ in 0..13 {c.barrier().unwrap();}
        assert_eq!(c.snapshot()["score"]["outcome"],"VERIFYING");
        c.barrier().unwrap();assert_eq!(c.snapshot()["score"]["verified_at"],14);
        for _ in 14..20 {c.barrier().unwrap();}
        let s=c.snapshot()["score"].clone();
        assert_eq!(s["outcome"],"VICTORY");assert_eq!(s["total"],1000.0);
        assert!(c.barrier().is_err());
    }
    #[test]
    fn filters_and_power_remove_service_without_creating_traffic() {
        let mut c=core(10);
        c.nodes.get_mut(&1).unwrap().policy="DEFAULT_DENY".into();
        c.barrier().unwrap();let s=c.snapshot()["score"].clone();
        // Same-LAN client still reaches its server; cross-LAN client is blocked.
        assert_eq!(s["service_fraction"],0.5);assert_eq!(s["clean_fraction"],1.0);
        assert_eq!(s["service"],30.0);assert!(c.sessions.is_empty());assert_eq!(c.successful_connections,0);
        c.nodes.get_mut(&2001).unwrap().power="OFFLINE".into();
        for _ in 0..3 {c.barrier().unwrap();}
        let s=c.snapshot()["score"].clone();assert_eq!(s["outcome"],"DEFEAT");assert_eq!(s["defeated_at"],4);
        assert_eq!(s["service"],30.0);assert_eq!(s["security"],100.0);assert_eq!(s["recovery"],0.0);
    }
    #[test]
    fn isolated_offline_infection_is_not_eradicated_and_recovery_can_be_lost() {
        let mut c=core(40);
        for _ in 0..14 {c.barrier().unwrap();}
        assert_eq!(c.snapshot()["score"]["outcome"],"RECOVERED");
        let n=c.nodes.get_mut(&1001).unwrap();n.compromised=true;n.isolated=true;n.power="OFFLINE".into();
        c.barrier().unwrap();let s=c.snapshot()["score"].clone();
        assert_eq!(s["verified_at"],Value::Null);assert_eq!(s["recovery_streak"],0);
        assert!(s["clean_fraction"].as_f64().unwrap()<1.0);assert_eq!(s["service_fraction"],0.5);
        for _ in 15..40 {c.barrier().unwrap();}
        assert_eq!(c.snapshot()["score"]["outcome"],"INCOMPLETE");
    }
    #[test]
    fn outage_window_resets_and_weights_are_respected() {
        let mut c=core(30);
        c.config.mission.as_mut().unwrap().services[0].weight=3;
        c.nodes.get_mut(&1001).unwrap().isolated=true;c.barrier().unwrap();
        assert_eq!(c.snapshot()["score"]["service_fraction"],0.25);
        c.nodes.get_mut(&2001).unwrap().isolated=true;
        c.barrier().unwrap();c.barrier().unwrap();assert_eq!(c.score.as_ref().unwrap().outage_streak,2);
        c.nodes.get_mut(&2001).unwrap().isolated=false;c.barrier().unwrap();
        assert_eq!(c.score.as_ref().unwrap().outage_streak,0);
    }
    #[test]
    fn invalid_missions_are_rejected_and_practice_is_unscored() {
        let base="version=1\nrouters=[{id=1}]\nlans=[{id=1,router=1}]\nworkstations=[{id=1001,lan=1}]\nservers=[{id=2001,lan=1}]\n";
        for suffix in ["[mission]\noutage_ticks=0", "[mission]\nservices=[{client=1,server=2001}]", "[mission]\nservices=[{client=1001,server=2001,weight=0}]", "[simulation]\ntick_limit=0\n[mission]\nenabled=true"] {
            assert!(crate::network::parse(&format!("{base}{suffix}")).is_err());
        }
        let c=crate::network::parse(&format!("{base}[mission]\nenabled=false")).unwrap();
        assert_eq!(Core::new(c).unwrap().snapshot()["score"]["outcome"],"PRACTICE");
    }
    #[test]
    fn runtime_stops_at_defeat_and_restart_starts_fresh_accounting() {
        let files=crate::network::RunFiles{script:"vanguard/agents/ostrich.lua".into(),network:"network.toml".into(),dbg:false};
        let config=files.load().unwrap();
        let mut run=crate::runtime::Runtime::new(config.clone(),"scoring-a".into()).unwrap();
        for n in run.core.borrow_mut().nodes.values_mut().filter(|n|n.kind=="SERVER"){n.power="OFFLINE".into();}
        for _ in 0..30 {run.step().unwrap();}
        assert_eq!(run.control,"FINISHED");assert_eq!(run.snapshot()["score"]["outcome"],"DEFEAT");
        let hash=run.state_hash();run.step().unwrap();assert_eq!(hash,run.state_hash());
        let fresh=crate::runtime::Runtime::new(config,"scoring-b".into()).unwrap();
        assert_eq!(fresh.snapshot()["score"]["service_sum"],0);assert_eq!(fresh.snapshot()["score"]["defeated_at"],Value::Null);
    }
}
