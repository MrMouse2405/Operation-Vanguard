//! One simulation-owner thread serializes controls and commits whole barriers.
//! Socket/render activity only consumes projections; it cannot drive the clock.
use std::{collections::{BTreeMap,VecDeque},sync::{Arc,RwLock,mpsc},thread,time::{Duration,Instant}};
use rocket::{State,Request,http::Status,request::{FromRequest,Outcome},serde::json::Json};
use rocket::tokio::sync::{broadcast,oneshot};
use rocket::futures::{SinkExt,StreamExt};
use serde::Deserialize;
use serde_json::{json,Value};
use crate::{runtime::Runtime,network::RunFiles};

/// Version 1 root fields are stable. Changed scalar/object/array fields replace
/// that whole field; unchanged topology/history arrays need not cross the wire.
mod value_delta {
    use super::*;
    pub type Delta=BTreeMap<String,Value>;
    pub fn between(previous:&Value,next:&Value)->Delta{
        next.as_object().unwrap().iter().filter(|(key,value)|previous.get(*key)!=Some(*value)).map(|(key,value)|(key.clone(),value.clone())).collect()
    }
}

#[derive(Clone,Debug,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {pub schema_version:u32,pub request_id:String,pub command:String,#[serde(default)]pub ticks:Option<u64>,#[serde(default)]pub seconds_per_tick:Option<f64>}

fn tick_delay(seconds_per_tick:f64,elapsed:Duration)->Duration {
    Duration::from_secs_f64(seconds_per_tick).saturating_sub(elapsed)
}
enum RequestMessage {Command(Command,bool,oneshot::Sender<Value>),Snapshot(oneshot::Sender<Value>),Details(Details,oneshot::Sender<Value>),Stop}
#[derive(Clone,Debug)]
pub struct Details {pub kind:String,pub id:u64,pub target:Option<u64>,pub offset:usize,pub limit:usize,pub run_id:String}
#[derive(Clone,Debug)]
struct Published {seq:u64,text:Option<Arc<str>>,records:Option<Arc<str>>,compact:Arc<str>}
#[derive(Clone)]
pub struct Server {sender:mpsc::Sender<RequestMessage>,snapshot:Arc<RwLock<Value>>,events:broadcast::Sender<Published>,full_clients:Arc<std::sync::atomic::AtomicUsize>}

impl Server {
    pub fn launch(files:RunFiles)->Result<Self,String>{
        let (sender,receiver)=mpsc::channel();let (events,_)=broadcast::channel(8);
        let snapshot=Arc::new(RwLock::new(Value::Null));let shared=snapshot.clone();let output=events.clone();
        let full_clients=Arc::new(std::sync::atomic::AtomicUsize::new(0));let clients=full_clients.clone();
        let (ready_tx,ready_rx)=mpsc::sync_channel(1);
        thread::Builder::new().name("vanguard-simulation".into()).spawn(move ||{
            let mut generation=1_u64;
            let mut runtime=match files.load().and_then(|c|Runtime::new(c,format!("run-{generation}"))) {Ok(r)=>r,Err(e)=>{let _=ready_tx.send(Err(e));return;}};
            let mut seq=0;
            let mut seconds_per_tick=runtime.core.borrow().config.tick_ms as f64/1000.0;
            let mut next_tick=Instant::now();
            let mut last_full=None;
            fn projected(runtime:&Runtime,compact:bool,seq:u64,seconds:f64)->Value{let mut value=runtime.projection(compact);value["seq"]=json!(seq);value["seconds_per_tick"]=json!(seconds);value}
            fn envelope(previous:&Value,value:&Value,records:bool)->Value{
                if previous["run_id"]!=value["run_id"]{return json!({"schema_version":1,"run_id":value["run_id"],"seq":value["seq"],"tick":value["tick"],"type":"snapshot","snapshot":value});}
                let mut result=json!({"schema_version":1,"run_id":value["run_id"],"seq":value["seq"],"tick":value["tick"],"type":"update","base_seq":previous["seq"]});
                if records{result["patch"]=crate::projection::between(previous,value);}else{result["delta"]=json!(value_delta::between(previous,value));}result
            }
            let mut publish=|runtime:&Runtime,seconds_per_tick:f64,seq:&mut u64,shared:&RwLock<Value>,events:&broadcast::Sender<Published>|{
                *seq+=1;let value=projected(runtime,true,*seq,seconds_per_tick);
                let mut shared=shared.write().unwrap();let compact=envelope(&shared,&value,true).to_string().into();*shared=value;drop(shared);
                let (text,records)=if clients.load(std::sync::atomic::Ordering::SeqCst)>0{
                    let value=projected(runtime,false,*seq,seconds_per_tick);let empty=Value::Null;let previous=last_full.as_ref().unwrap_or(&empty);
                    let text=Some(envelope(previous,&value,false).to_string().into());let records=Some(envelope(previous,&value,true).to_string().into());last_full=Some(value);(text,records)
                }else{last_full=None;(None,None)};
                let _=events.send(Published{seq:*seq,text,records,compact});
            };
            publish(&runtime,seconds_per_tick,&mut seq,&shared,&output);let _=ready_tx.send(Ok(()));
            let mut last_publish=Instant::now();
            let mut cache=BTreeMap::<String,Value>::new();let mut cache_order=VecDeque::new();let mut cache_bytes=0;
            loop {
                let timeout=if runtime.control=="RUNNING" {next_tick.saturating_duration_since(Instant::now())}else{Duration::from_secs(3600)};
                match receiver.recv_timeout(timeout){
                    Ok(RequestMessage::Stop)|Err(mpsc::RecvTimeoutError::Disconnected)=>break,
                    Err(mpsc::RecvTimeoutError::Timeout)=>{
                        if runtime.control=="RUNNING" {
                            let started=Instant::now();
                            if let Err(error)=runtime.step(){runtime.control="ERROR".into();runtime.core.borrow_mut().emit("runtime_error",None,"<engine>",json!({"error":error,"severity":"error"}));}
                            // Rendering is a projection, not the clock. Run every
                            // barrier, but publish live views at most 10 Hz.
                            if seconds_per_tick>=0.1 || last_publish.elapsed()>=Duration::from_millis(100) || runtime.control!="RUNNING" {
                                publish(&runtime,seconds_per_tick,&mut seq,&shared,&output);last_publish=Instant::now();
                            }
                            next_tick=Instant::now()+tick_delay(seconds_per_tick,started.elapsed());
                        }
                    },
                    Ok(RequestMessage::Snapshot(response))=>{
                        publish(&runtime,seconds_per_tick,&mut seq,&shared,&output);last_publish=Instant::now();
                        let _=response.send(projected(&runtime,false,seq,seconds_per_tick));
                    },
                    Ok(RequestMessage::Details(request,response))=>{let _=response.send(runtime.details(&request));},
                    Ok(RequestMessage::Command(command,compact,response))=>{
                        if let Some(cached)=cache.get(&command.request_id){let _=response.send(cached.clone());continue;}
                        let result=if command.schema_version!=1 {Err("unsupported schema_version".to_string())}
                        else if command.request_id.is_empty()||command.request_id.len()>128 {Err("request_id must be 1..128 bytes".into())}
                        else {match command.command.as_str(){
                            "GetSnapshot"=>Ok(()),
                            "SetTickPacing"=>match command.seconds_per_tick.filter(|s|s.is_finite()&&(0.0..=3600.0).contains(s)) {
                                Some(seconds)=>{seconds_per_tick=seconds;next_tick=Instant::now();Ok(())},
                                None=>Err("seconds_per_tick must be a finite number from 0 to 3600 (0 runs as fast as possible)".into()),
                            },
                            "Start" if runtime.control=="READY"=>{runtime.control="RUNNING".into();next_tick=Instant::now();Ok(())},
                            "Pause" if runtime.control=="RUNNING"=>{runtime.control="PAUSED".into();Ok(())},
                            "Resume" if runtime.control=="PAUSED"=>{runtime.control="RUNNING".into();next_tick=Instant::now();Ok(())},
                            "StepTicks" if matches!(runtime.control.as_str(),"READY"|"PAUSED")=>{
                                match command.ticks.filter(|n|(1..=1000).contains(n)){
                                    None=>Err("StepTicks requires ticks: an integer from 1 to 1000".into()),
                                    Some(count)=>{
                                        runtime.control="PAUSED".into();
                                        let mut result=Ok(());
                                        for _ in 0..count{
                                            if let Err(error)=runtime.step(){runtime.control="ERROR".into();result=Err(error);break;}
                                            if runtime.control=="FINISHED"{break;}
                                        }
                                        result
                                    }
                                }
                            },
                            "Restart"=>{
                                runtime.control="PAUSED".into();
                                match files.load().and_then(|c|Runtime::new(c,format!("run-{}",generation+1))){
                                    Ok(staged)=>{generation+=1;runtime=staged;Ok(())},Err(error)=>Err(format!("Restart staging failed; old run paused: {error}")),
                                }
                            },
                            _=>Err(format!("invalid transition: {} in {}",command.command,runtime.control)),
                        }};
                        publish(&runtime,seconds_per_tick,&mut seq,&shared,&output);last_publish=Instant::now();
                        let value=if compact{shared.read().unwrap().clone()}else{projected(&runtime,false,seq,seconds_per_tick)};
                        let reply=json!({"schema_version":1,"run_id":runtime.run_id,"seq":seq,"tick":value["tick"],"type":if result.is_ok(){if command.command=="GetSnapshot"{"snapshot"}else{"ack"}}else{"error"},"request_id":command.request_id,"command":command.command,"error":result.err(),"snapshot":value});
                        let bytes=serde_json::to_vec(&reply).unwrap().len();
                        while !cache_order.is_empty() && (cache_order.len()>=512 || cache_bytes+bytes>16*1024*1024) {
                            let (id,size)=cache_order.pop_front().unwrap();cache.remove(&id);cache_bytes-=size;
                        }
                        cache_bytes+=bytes;cache_order.push_back((command.request_id.clone(),bytes));cache.insert(command.request_id,reply.clone());let _=response.send(reply);
                    }
                }
            }
        }).map_err(|e|e.to_string())?;
        ready_rx.recv().map_err(|e|e.to_string())??;
        Ok(Self{sender,snapshot,events,full_clients})
    }
    pub fn stop(&self){let _=self.sender.send(RequestMessage::Stop);}
    pub fn snapshot(&self)->Value{self.snapshot.read().unwrap().clone()}
    #[cfg(test)]
    async fn command(&self,command:Command)->Value{
        self.command_mode(command,false).await
    }
    async fn full_snapshot(&self)->Value{
        let (sender,receiver)=oneshot::channel();if self.sender.send(RequestMessage::Snapshot(sender)).is_err(){return Value::Null;}receiver.await.unwrap_or(Value::Null)
    }
    async fn command_mode(&self,command:Command,compact:bool)->Value{
        let (sender,receiver)=oneshot::channel();
        if self.sender.send(RequestMessage::Command(command,compact,sender)).is_err(){return json!({"schema_version":1,"type":"error","error":"simulation owner stopped"});}
        receiver.await.unwrap_or_else(|_|json!({"schema_version":1,"type":"error","error":"simulation owner stopped"}))
    }
}

/// Loopback-only browser controls, including WebSocket handshakes. CLI requests
/// without Origin are supported; browser cross-origin controls are rejected.
pub struct LocalOrigin;
#[rocket::async_trait]
impl<'r> FromRequest<'r> for LocalOrigin {
    type Error=&'static str;
    async fn from_request(request:&'r Request<'_>)->Outcome<Self,Self::Error>{
        let host=request.headers().get_one("Host").unwrap_or("");
        let hostname=host.split(':').next().unwrap_or("");
        if !["127.0.0.1","localhost"].contains(&hostname){return Outcome::Error((Status::Forbidden,"loopback host required"));}
        if let Some(origin)=request.headers().get_one("Origin"){
            let allowed=[format!("http://{host}"),"http://127.0.0.1:5173".into(),"http://localhost:5173".into()];
            if !allowed.iter().any(|allowed|allowed==origin){return Outcome::Error((Status::Forbidden,"control origin is not allowed"));}
        }
        Outcome::Success(Self)
    }
}

#[rocket::get("/api/snapshot?<compact>")]
pub async fn snapshot(server:&State<Server>,compact:Option<bool>)->Json<Value>{Json(if compact==Some(true){server.snapshot()}else{server.full_snapshot().await})}
#[rocket::post("/api/command?<compact>",format="json",data="<command>")]
pub async fn command(_origin:LocalOrigin,server:&State<Server>,command:Json<Command>,compact:Option<bool>)->Json<Value>{Json(server.command_mode(command.into_inner(),compact==Some(true)).await)}
#[rocket::get("/api/details?<kind>&<id>&<target>&<offset>&<limit>&<run_id>")]
pub async fn details(_origin:LocalOrigin,server:&State<Server>,kind:String,id:Option<u64>,target:Option<u64>,offset:Option<usize>,limit:Option<usize>,run_id:String)->Json<Value>{
    let (sender,receiver)=oneshot::channel();let request=Details{kind,id:id.unwrap_or(0),target,offset:offset.unwrap_or(0),limit:limit.unwrap_or(50).clamp(1,100),run_id};
    if server.sender.send(RequestMessage::Details(request,sender)).is_err(){return Json(Value::Null);}Json(receiver.await.unwrap_or(Value::Null))
}
#[rocket::get("/api/ws?<records>&<compact>")]
pub fn websocket(_origin:LocalOrigin,ws:rocket_ws::WebSocket,server:&State<Server>,records:Option<bool>,compact:Option<bool>)->rocket_ws::Channel<'static>{
    let server=server.inner().clone();
    ws.config(rocket_ws::Config{max_message_size:Some(4096),max_frame_size:Some(4096),..Default::default()}).channel(move |mut stream|Box::pin(async move{
        let compact=compact==Some(true);
        struct Client(Option<Arc<std::sync::atomic::AtomicUsize>>);impl Drop for Client{fn drop(&mut self){if let Some(count)=&self.0{count.fetch_sub(1,std::sync::atomic::Ordering::SeqCst);}}}
        let _client=if compact{Client(None)}else{server.full_clients.fetch_add(1,std::sync::atomic::Ordering::SeqCst);Client(Some(server.full_clients.clone()))};
        // Subscribe before projection, then discard events at/below its watermark.
        let mut updates=server.events.subscribe();let snapshot=if compact{server.snapshot()}else{server.full_snapshot().await};let mut watermark=snapshot["seq"].as_u64().unwrap_or(0);
        stream.send(rocket_ws::Message::Text(json!({"schema_version":1,"run_id":snapshot["run_id"],"seq":watermark,"tick":snapshot["tick"],"type":"snapshot","snapshot":snapshot}).to_string())).await?;
        loop {rocket::tokio::select!{
            update=updates.recv()=>{
                let value=match update{Ok(value)=>value,Err(broadcast::error::RecvError::Lagged(_))=>{let s=if compact{server.snapshot()}else{server.full_snapshot().await};let text:Arc<str>=json!({"schema_version":1,"run_id":s["run_id"],"seq":s["seq"],"tick":s["tick"],"type":"snapshot","snapshot":s}).to_string().into();Published{seq:s["seq"].as_u64().unwrap_or(0),records:Some(text.clone()),text:Some(text.clone()),compact:text}},Err(_)=>break};
                if value.seq<=watermark{continue;}watermark=value.seq;
                let text=if compact{Some(value.compact)}else if records==Some(true){value.records}else{value.text};
                if let Some(text)=text{stream.send(rocket_ws::Message::Text(text.to_string())).await?;}
            },
            incoming=stream.next()=>{match incoming{
                Some(Ok(rocket_ws::Message::Text(text)))=>{
                    let reply=match serde_json::from_str::<Command>(&text){Ok(command)=>server.command_mode(command,compact).await,Err(error)=>{let snapshot=server.snapshot();json!({"schema_version":1,"run_id":snapshot["run_id"],"seq":snapshot["seq"],"tick":snapshot["tick"],"type":"error","error":format!("invalid command: {error}")})}};
                    stream.send(rocket_ws::Message::Text(reply.to_string())).await?;
                },Some(Ok(rocket_ws::Message::Ping(bytes)))=>{stream.send(rocket_ws::Message::Pong(bytes)).await?;},Some(Ok(rocket_ws::Message::Close(_)))|None=>break,Some(Err(error))=>return Err(error),_=>{},
            }}
        }}
        Ok(())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    fn cmd(id:&str,command:&str)->Command{Command{schema_version:1,request_id:id.into(),command:command.into(),ticks:None,seconds_per_tick:None}}

    #[test]
    fn pacing_only_waits_for_the_remaining_tick_time(){
        assert_eq!(tick_delay(0.5,Duration::from_millis(120)),Duration::from_millis(380));
        assert_eq!(tick_delay(0.5,Duration::from_millis(750)),Duration::ZERO);
        assert_eq!(tick_delay(0.0,Duration::from_millis(1)),Duration::ZERO);
    }

    #[rocket::async_test]
    async fn pacing_is_validated_published_and_keeps_controls_responsive(){
        let server=Server::launch(RunFiles{script:"vanguard/agents/ostrich.lua".into(),network:"network.toml".into(),dbg:false}).unwrap();
        let mut pacing=cmd("pacing","SetTickPacing");pacing.seconds_per_tick=Some(1.0);
        assert_eq!(server.command(pacing).await["snapshot"]["seconds_per_tick"],1.0);
        server.command(cmd("start-paced","Start")).await;
        let deadline=Instant::now()+Duration::from_secs(2);
        while server.snapshot()["tick"]==0 {assert!(Instant::now()<deadline);rocket::tokio::time::sleep(Duration::from_millis(5)).await;}
        let tick=server.snapshot()["tick"].clone();
        assert_eq!(tick,1,"slow playback must publish its first boundary before waiting for the next tick");
        rocket::tokio::time::sleep(Duration::from_millis(80)).await;
        assert_eq!(server.snapshot()["tick"],tick);
        let before=Instant::now();server.command(cmd("pause-paced","Pause")).await;
        assert!(before.elapsed()<Duration::from_millis(500));
        for (id,seconds) in [("negative",Some(-1.0)),("too-large",Some(3601.0)),("missing",None)] {
            let mut invalid=cmd(id,"SetTickPacing");invalid.seconds_per_tick=seconds;
            assert_eq!(server.command(invalid).await["type"],"error");
            assert_eq!(server.snapshot()["seconds_per_tick"],1.0);
        }
        let mut unlimited=cmd("unlimited","SetTickPacing");unlimited.seconds_per_tick=Some(0.0);
        assert_eq!(server.command(unlimited).await["snapshot"]["seconds_per_tick"],0.0);
        server.stop();
    }

    #[test]
    fn field_delta_reduces_to_the_canonical_projection(){
        let before=json!({"seq":2,"tick":0,"nodes":[{"id":1}],"jobs":[]});
        let after=json!({"seq":3,"tick":1,"nodes":[{"id":1}],"jobs":[{"id":1}]});
        let delta=value_delta::between(&before,&after);assert!(!delta.contains_key("nodes"));
        let mut reduced=before;for (key,value) in delta{reduced[key]=value;}
        assert_eq!(reduced,after);
    }

    #[rocket::async_test]
    async fn stepping_uses_real_barriers_and_is_idempotent() {
        let server=Server::launch(RunFiles{script:"vanguard/agents/ostrich.lua".into(),network:"network.toml".into(),dbg:false}).unwrap();
        let mut step=cmd("step-five","StepTicks");step.ticks=Some(5);
        let reply=server.command(step.clone()).await;
        assert_eq!(reply["snapshot"]["tick"],5);assert_eq!(reply["snapshot"]["enemy_updates"],5);
        assert_eq!(reply["snapshot"]["control"],"PAUSED");assert_eq!(server.command(step).await,reply);
        assert!(reply["snapshot"]["jobs"].as_array().unwrap().is_empty());
        assert!(reply["snapshot"]["nodes"].as_array().unwrap().iter().any(|n|n["exposure"]["age"]==5));
        let mut invalid=cmd("invalid-step","StepTicks");invalid.ticks=Some(0);
        assert_eq!(server.command(invalid).await["type"],"error");assert_eq!(server.snapshot()["tick"],5);
        server.stop();
    }

    #[rocket::async_test]
    async fn controls_pause_resume_idempotency_and_failed_restart(){
        let path=std::env::current_dir().unwrap().join(format!(".server-test-{}.toml",std::process::id()));
        let text=include_str!("../network.toml").replace("tick_ms = 100", "tick_ms = 5");
        std::fs::write(&path,&text).unwrap();
        let server=Server::launch(RunFiles{script:"vanguard/agents/defender.lua".into(),network:path.clone(),dbg:false}).unwrap();
        assert_eq!(server.snapshot()["control"],"READY");
        let first=server.command(cmd("start","Start")).await;assert_eq!(first["type"],"ack");
        assert_eq!(server.command(cmd("start","Start")).await,first);
        let deadline=Instant::now()+Duration::from_secs(3);
        while server.snapshot()["tick"].as_u64().unwrap()<2{assert!(Instant::now()<deadline);rocket::tokio::time::sleep(Duration::from_millis(5)).await;}
        let paused=server.command(cmd("pause","Pause")).await;assert_eq!(paused["snapshot"]["control"],"PAUSED");
        let tick=server.snapshot()["tick"].clone();rocket::tokio::time::sleep(Duration::from_millis(30)).await;assert_eq!(server.snapshot()["tick"],tick);
        assert_eq!(server.command(cmd("resume","Resume")).await["type"],"ack");
        let old_run=server.snapshot()["run_id"].clone();
        std::fs::write(&path,"invalid json").unwrap();
        let failed=server.command(cmd("bad-restart","Restart")).await;
        assert_eq!(failed["type"],"error");assert_eq!(failed["snapshot"]["run_id"],old_run);assert_eq!(failed["snapshot"]["control"],"PAUSED");
        std::fs::write(&path,&text).unwrap();
        let restart=server.command(cmd("restart","Restart")).await;
        assert_eq!(restart["snapshot"]["tick"],0);assert_eq!(restart["snapshot"]["control"],"READY");assert_ne!(restart["snapshot"]["run_id"],old_run);
        assert_eq!(server.command(cmd("snap","GetSnapshot")).await["type"],"snapshot");
        server.stop();std::fs::remove_file(path).unwrap();
    }
}
