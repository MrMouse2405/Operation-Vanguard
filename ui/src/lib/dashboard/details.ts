export async function loadDetails(runId:string,kind:string,id:number,signal:AbortSignal,offset=0,target?:number){
	const query=new URLSearchParams({run_id:runId,kind,id:String(id),offset:String(offset),limit:'50'});
	if(target!==undefined)query.set('target',String(target));
	const response=await fetch(`/api/details?${query}`,{signal});if(!response.ok)throw new Error(`Details unavailable (${response.status}).`);
	const value=await response.json();if(value.error||value.run_id!==runId)throw new Error(value.error==='STALE_RUN'?'The match restarted. Reopen this inspection.':value.error??'Run changed while loading details.');return value as {run_id:string;tick:number;total:number;items:any[]};
}
