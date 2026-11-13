import type { Snapshot } from './types';
import { inventory } from './inventory';
export const flowColors={task:'#90b8fc',exposure:'#f8c176',compromised:'#ff7782',connection:'#54dfb3'};
export type FlowKind=keyof typeof flowColors;
export interface Flow {id:string;source:number;target:number;kind:FlowKind;label:string;opened:number;}
export function networkFlows(snapshot:Snapshot|null):Flow[]{
	if(!snapshot)return [];
	const nodes=inventory(snapshot.nodes).byId;
	const flows:Flow[]=snapshot.jobs.filter(j=>j.state==='RUNNING'&&nodes.has(j.target??j.router_id)).map(j=>({id:`task-${j.id}`,source:j.router_id,target:j.target??j.router_id,kind:'task',label:`A${j.agent_id} · ${j.action.replaceAll('_',' ')}`,opened:j.started_at}));
	for(const session of snapshot.sessions){
		const source=nodes.get(session.source),target=nodes.get(session.target);if(!source||!target)continue;
		const kind=source.compromised&&target.exposure?'exposure':source.compromised||target.compromised?'compromised':'connection';
		flows.push({id:`session-${session.id}`,source:source.id,target:target.id,kind,label:kind==='exposure'?'Active exposure':kind==='compromised'?'Compromised endpoint':'Live connection',opened:session.opened_at});
	}
	return flows;
}
