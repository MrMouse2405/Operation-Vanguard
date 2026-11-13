//! Track the actual high-water mark of each Lua allocator, including temporary
//! allocations made and released inside one native call. Forwarding to mlua's
//! original allocator preserves its configured memory limit.
use std::{ffi::c_void,sync::atomic::{AtomicUsize,Ordering}};
use mlua::{Lua,ffi};

struct State {
    original:ffi::lua_Alloc,
    // Opaque allocator userdata belonging to this Lua instance. Lua's `send`
    // support permits moving the whole VM between workers, never concurrent use.
    userdata:usize,
    current:AtomicUsize,
    peak:AtomicUsize,
}

unsafe extern "C" fn allocate(userdata:*mut c_void,ptr:*mut c_void,old:usize,new:usize)->*mut c_void {
    // Lua invokes this synchronously on its current worker. The Box remains at a
    // stable address until the original allocator is restored in Drop.
    let state=unsafe{&*(userdata as *const State)};
    let result=unsafe{(state.original)(state.userdata as *mut c_void,ptr,old,new)};
    if new==0 || !result.is_null(){
        let old=if ptr.is_null(){0}else{old};
        let current=state.current.load(Ordering::Relaxed).saturating_sub(old).saturating_add(new);
        state.current.store(current,Ordering::Relaxed);state.peak.fetch_max(current,Ordering::Relaxed);
    }
    result
}

pub struct AllocationMeter {lua:Lua,state:Box<State>}
impl AllocationMeter {
    pub fn install(lua:&Lua)->mlua::Result<Self>{
        let current=lua.used_memory();let mut original=None;
        unsafe{lua.exec_raw::<()>((),|raw|{let mut userdata=std::ptr::null_mut();let allocator=ffi::lua_getallocf(raw,&mut userdata);original=Some((allocator,userdata));})?;}
        let (allocator,userdata)=original.unwrap();
        let mut meter=Self{lua:lua.clone(),state:Box::new(State{original:allocator,userdata:userdata as usize,current:AtomicUsize::new(current),peak:AtomicUsize::new(current)})};
        let pointer=&mut *meter.state as *mut State as *mut c_void;
        unsafe{lua.exec_raw::<()>((),|raw|ffi::lua_setallocf(raw,allocate,pointer))?;}
        Ok(meter)
    }
    pub fn current(&self)->usize{self.state.current.load(Ordering::Relaxed)}
    pub fn peak(&self)->usize{self.state.peak.load(Ordering::Relaxed)}
}
impl Drop for AllocationMeter {
    fn drop(&mut self){
        // Retain an owning Lua clone until after restoration. mlua can then
        // recognize and destroy its own allocator userdata normally.
        let _=unsafe{self.lua.exec_raw::<()>((),|raw|ffi::lua_setallocf(raw,self.state.original,self.state.userdata as *mut c_void))};
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn peak_survives_collection_and_allocator_limit_still_applies(){
        let lua=Lua::new();lua.set_memory_limit(512*1024).unwrap();
        let meter=AllocationMeter::install(&lua).unwrap();
        let initial=meter.current();lua.load("local t={};for i=1,5000 do t[i]=i end").exec().unwrap();
        let peak=meter.peak();assert!(peak>initial);
        lua.gc_collect().unwrap();assert!(meter.current()<peak);assert_eq!(meter.peak(),peak);
        assert!(lua.load("local t={};for i=1,1000000 do t[i]=i end").exec().is_err());
        assert!(meter.peak()<=512*1024);
    }
}
