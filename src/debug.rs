//! Write-only, bounded agent drawings. Never reads simulation truth.
use std::collections::BTreeMap;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Clone, Default, Serialize)]
pub struct View {
    pub updated_at: u64,
    pub nodes: BTreeMap<String, Value>,
    pub edges: BTreeMap<String, Value>,
    pub info: Value,
}
#[derive(Default)]
pub struct Store(pub BTreeMap<u64, BTreeMap<String, View>>);

fn key(value: Option<&Value>) -> Result<String, String> {
    let text = match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) if n.as_u64().is_some_and(|n| n > 0) => n.to_string(),
        _ => return Err("INVALID_ARGUMENT: debug IDs must be nonempty strings or positive integers".into()),
    };
    if text.is_empty() || text.len() > 128 { return Err("INVALID_ARGUMENT: debug ID must be 1..128 bytes".into()); }
    Ok(text)
}
impl Store {
    pub fn call(&mut self, agent: u64, tick: u64, method: &str, args: &[Value]) -> Result<Value, String> {
        let count = match method {"view"|"clear"=>1,"note"|"remove_node"|"remove_edge"=>2,"node"=>3,"edge"=>5,_=>return Err("INVALID_ARGUMENT: unknown debug method".into())};
        if args.len()!=count {return Err("INVALID_ARGUMENT: wrong dbgguard argument count".into());}
        let name=key(args.first())?;
        let mut views=self.0.get(&agent).cloned().unwrap_or_default();
        if !views.contains_key(&name) && method!="view" {return Err("INVALID_ARGUMENT: create the debug view first".into());}
        let view=views.entry(name).or_default();
        match method {
            "view"=>{},
            "node"=>{let id=key(args.get(1))?;view.nodes.insert(id.clone(),json!({"id":id,"info":args[2]}));},
            "edge"=>{
                let id=key(args.get(1))?;let from=key(args.get(2))?;let to=key(args.get(3))?;
                if !view.nodes.contains_key(&from)||!view.nodes.contains_key(&to){return Err("INVALID_ARGUMENT: draw both debug endpoints first".into());}
                view.edges.insert(id.clone(),json!({"id":id,"from":from,"to":to,"info":args[4]}));
            },
            "note"=>view.info=args[1].clone(),
            "clear"=>*view=View::default(),
            "remove_node"=>{let id=key(args.get(1))?;view.nodes.remove(&id);view.edges.retain(|_,e|e["from"]!=id&&e["to"]!=id);},
            "remove_edge"=>{view.edges.remove(&key(args.get(1))?);},
            _=>unreachable!(),
        }
        view.updated_at=tick;
        let nodes:usize=views.values().map(|v|v.nodes.len()).sum();
        let edges:usize=views.values().map(|v|v.edges.len()).sum();
        if views.len()>8 || nodes>4096 || edges>8192 || serde_json::to_vec(&views).unwrap().len()>1_048_576 {
            return Err("LIMIT_EXCEEDED: debug limit per agent: 8 views, 4096 nodes, 8192 edges, 1 MiB".into());
        }
        self.0.insert(agent,views);
        Ok(json!(true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn view_and_payload_store_limits_reject_without_destroying_prior_views() {
        let mut store=Store::default();
        for n in 1..=8 {store.call(101,0,"view",&[json!(n)]).unwrap();}
        assert!(store.call(101,1,"view",&[json!(9)]).unwrap_err().starts_with("LIMIT_EXCEEDED"));
        assert_eq!(store.0[&101].len(),8);
        let before=serde_json::to_value(&store.0).unwrap();
        assert!(store.call(101,2,"note",&[json!(1),json!("x".repeat(1_048_576))]).unwrap_err().starts_with("LIMIT_EXCEEDED"));
        assert_eq!(serde_json::to_value(&store.0).unwrap(),before);
        // The quota belongs to an agent, not the whole competition.
        store.call(102,2,"view",&[json!(9)]).unwrap();
    }
}
