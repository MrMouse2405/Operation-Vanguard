import type { Node } from './types';
interface Inventory {nodes:Node[];byId:Map<number,Node>;hosts:Node[];routers:Node[];byRouter:Map<number,Node[]>}
const cache=new WeakMap<Node[],Inventory>();let previous:WeakRef<Inventory>|undefined;
/** One shared index per immutable inventory; only changed LAN buckets rebuild. */
export function inventory(nodes:Node[]):Inventory{
	const cached=cache.get(nodes);if(cached)return cached;
	const before=previous?.deref();let result:Inventory;
	if(before&&nodes.length===before.nodes.length&&nodes.every((n,i)=>n.id===before.nodes[i].id)){
		const byId=new Map(before.byId),dirty=new Set<number>();let hostsChanged=false,routersChanged=false;
		for(let i=0;i<nodes.length;i++){const node=nodes[i],old=before.nodes[i];if(node===old)continue;byId.set(node.id,node);dirty.add(node.router_id);dirty.add(old.router_id);hostsChanged||=node.type!=='ROUTER'||old.type!=='ROUTER';routersChanged||=node.type==='ROUTER'||old.type==='ROUTER';}
		const byRouter=new Map(before.byRouter);for(const router of dirty)byRouter.set(router,[]);
		for(const node of nodes)if(node.type!=='ROUTER'&&dirty.has(node.router_id))byRouter.get(node.router_id)!.push(node);
		result={nodes,byId,byRouter,hosts:hostsChanged?nodes.filter(n=>n.type!=='ROUTER'):before.hosts,routers:routersChanged?nodes.filter(n=>n.type==='ROUTER'):before.routers};
	}else{
		const byId=new Map<number,Node>(),byRouter=new Map<number,Node[]>(),hosts:Node[]=[],routers:Node[]=[];
		for(const node of nodes){byId.set(node.id,node);if(node.type==='ROUTER'){routers.push(node);continue;}hosts.push(node);const bucket=byRouter.get(node.router_id)??[];bucket.push(node);byRouter.set(node.router_id,bucket);}
		result={nodes,byId,byRouter,hosts,routers};
	}
	cache.set(nodes,result);previous=new WeakRef(result);return result;
}
