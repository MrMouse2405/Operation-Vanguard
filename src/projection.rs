//! Presentation-only lossless record and append patches; not replay state.
use serde_json::{Value,json};
pub fn between(previous:&Value,next:&Value)->Value {
    if let (Some(a),Some(b))=(previous.as_object(),next.as_object()) {
        let fields:serde_json::Map<_,_>=b.iter().filter(|(key,value)|a.get(*key)!=Some(*value)).map(|(key,value)|(key.clone(),a.get(key).map_or_else(||json!({"set":value}),|old|between(old,value)))).collect();
        return json!({"fields":fields,"removed":a.keys().filter(|key|!b.contains_key(*key)).collect::<Vec<_>>()});
    }
    if let (Some(a),Some(b))=(previous.as_array(),next.as_array()) {
        if b.len()>=a.len() && a.iter().zip(b).all(|(a,b)|a==b){return json!({"append":&b[a.len()..]});}
        let key=["id","sequence","seq","host_id","topic"].into_iter().find(|key|!a.is_empty()&&!b.is_empty()&&a.iter().chain(b).all(|v|v.get(*key).is_some_and(|id|id.is_number()||id.is_string())));
        if a.len()==b.len()&&key.is_none_or(|key|a.iter().zip(b).all(|(a,b)|a[key]==b[key])) {
            let items:serde_json::Map<_,_>=a.iter().zip(b).enumerate().filter(|(_, (a,b))|a!=b).map(|(i,(a,b))|(i.to_string(),between(a,b))).collect();
            return json!({"items":items});
        }
        if let Some(key)=key {
            let old:std::collections::BTreeMap<_,_>=a.iter().map(|v|(v[key].to_string(),v)).collect();
            let new:std::collections::BTreeMap<_,_>=b.iter().map(|v|(v[key].to_string(),v)).collect();
            let order:Vec<_>=a.iter().filter(|v|new.contains_key(&v[key].to_string())).chain(b.iter().filter(|v|!old.contains_key(&v[key].to_string()))).map(|v|&v[key]).collect();
            if old.len()==a.len()&&new.len()==b.len()&&order==b.iter().map(|v|&v[key]).collect::<Vec<_>>() {
                let changes:Vec<_>=b.iter().filter(|v|old.get(&v[key].to_string()).copied()!=Some(*v)).map(|v|json!({"id":v[key],"patch":old.get(&v[key].to_string()).map_or_else(||json!({"set":v}),|before|between(before,v))})).collect();
                return json!({"key":key,"changes":changes,"removed":a.iter().filter(|v|!new.contains_key(&v[key].to_string())).map(|v|&v[key]).collect::<Vec<_>>()});
            }
        }
        // Fixed-layout service pairs and other unkeyed arrays can change a
        // handful of records without replacing their full inventory.
        if a.len()==b.len(){
            let items:serde_json::Map<_,_>=a.iter().zip(b).enumerate().filter(|(_, (a,b))|a!=b).map(|(i,(a,b))|(i.to_string(),between(a,b))).collect();
            return json!({"items":items});
        }
    }
    json!({"set":next})
}
#[cfg(test)]
mod tests {
    use super::*;
    fn apply(value:&Value,patch:&Value)->Value{
        if let Some(value)=patch.get("set"){return value.clone();}
        if let Some(append)=patch["append"].as_array(){let mut result=value.as_array().unwrap().clone();result.extend(append.iter().cloned());return json!(result);}
        if let Some(items)=patch["items"].as_object(){let mut result=value.as_array().unwrap().clone();for (index,change) in items{let i=index.parse::<usize>().unwrap();result[i]=apply(&result[i],change);}return json!(result);}
        if let Some(key)=patch["key"].as_str(){
            let mut result=value.as_array().unwrap().clone();result.retain(|v|!patch["removed"].as_array().unwrap().contains(&v[key]));
            for change in patch["changes"].as_array().unwrap(){if let Some(existing)=result.iter_mut().find(|v|v[key]==change["id"]){*existing=apply(existing,&change["patch"]);}else{result.push(apply(&Value::Null,&change["patch"]));}}return json!(result);
        }
        let mut result=value.as_object().unwrap().clone();for key in patch["removed"].as_array().unwrap(){result.remove(key.as_str().unwrap());}
        for (key,change) in patch["fields"].as_object().unwrap(){result.insert(key.clone(),apply(result.get(key).unwrap_or(&Value::Null),change));}json!(result)
    }
    #[test]
    fn reconstruct_updates_eviction_append_restart_and_reorder(){
        let states=[json!({"nodes":[{"id":1,"power":"ONLINE"},{"id":2,"power":"ONLINE"}],"agents":[{"id":101,"observations":[]}],"events":[{"sequence":1}]}),json!({"nodes":[{"id":1,"power":"OFFLINE"},{"id":2,"power":"ONLINE"}],"agents":[{"id":101,"observations":[{"task":1}]}],"events":[{"sequence":2}],"tick":1}),json!({"nodes":[{"id":2},{"id":3}],"agents":[],"events":[]}),json!({"nodes":[{"id":3},{"id":2}]}),json!({})];
        for pair in states.windows(2){assert_eq!(apply(&pair[0],&between(&pair[0],&pair[1])),pair[1]);}
    }
    #[test]
    fn one_host_update_is_small(){
        let before=json!({"nodes":(0..10000).map(|id|json!({"id":id,"compromised":false})).collect::<Vec<_>>()});let mut after=before.clone();after["nodes"][42]["compromised"]=json!(true);
        let patch=between(&before,&after);assert!(patch.to_string().len()<200);assert_eq!(apply(&before,&patch),after);
    }
}
