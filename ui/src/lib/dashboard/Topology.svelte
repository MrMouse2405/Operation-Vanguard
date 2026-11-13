<script lang="ts">
	import { onMount } from 'svelte';
	import { Minus, Plus, Focus, Network } from '@lucide/svelte';
	import type { Core, ElementDefinition, NodeSingular } from 'cytoscape';
	import type { Snapshot } from './types';
	import { networkFlows,flowColors } from './flows';
	let { snapshot, selected, onselect,scope=null }: { snapshot: Snapshot | null; selected: number | null; onselect: (id: number) => void;scope?:number|null } = $props();
	let container: HTMLDivElement;
	let cy: Core | undefined;
	let ready = $state(false);
	let viewport = $state({width:0,height:0});
	let signature = '';
	let fittedScope='';let activity=$state('all');let activeOnly=$state(false);
	const dataSignatures = new Map<string,string>();
	const icons = new Map<string,string>();
	let renderedIds = $derived(new Set(snapshot?.nodes.map(n=>n.id)??[]));
	let allFlows=$derived(networkFlows(snapshot).filter(f=>renderedIds.has(f.source)&&renderedIds.has(f.target)));
	let flows=$derived(allFlows.filter(f=>activity==='all'||(activity==='task'?f.kind==='task':f.kind!=='task')));
	let activeIds=$derived(new Set(flows.flatMap(f=>[f.source,f.target])));
	const icon = (type: string, color: string) => {
		const key=`${type}:${color}`;let value=icons.get(key);
		if(!value){value='data:image/svg+xml;charset=utf-8,' + encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="${color}" stroke-width="1.8">${type === 'SERVER' ? '<rect x="5" y="3" width="22" height="26" rx="3"/><path d="M10 10h12M10 16h12M10 22h8"/>' : '<rect x="5" y="5" width="22" height="16" rx="2"/><path d="M16 21v6M9 27h14"/>'}</svg>`);icons.set(key,value);}
		return value;
	};
	onMount(() => {
		let disposed = false;
		let resize: ResizeObserver;
		let animation=0,last=0,visible=true;
		const reduced=matchMedia('(prefers-reduced-motion: reduce)');
		const intersection=new IntersectionObserver(([entry])=>visible=entry.isIntersecting);intersection.observe(container);
		function animate(time:number){animation=requestAnimationFrame(animate);if(time-last<50||!visible||document.hidden||reduced.matches||snapshot?.control!=='RUNNING')return;last=time;cy?.edges('.flow').style('line-dash-offset',-(time/35)%24);}
		animation=requestAnimationFrame(animate);
		void import('cytoscape').then(({ default: cytoscape }) => {
			if (disposed) return;
			cy = cytoscape({ container, elements: [], layout: { name: 'preset' }, minZoom: .2, maxZoom: 4, wheelSensitivity: .2, style: [
				{ selector: 'node', style: { label: 'data(label)', width: 30, height: 30, 'background-opacity': 0, 'border-width': 0, color: '#91a9b8', 'font-size': 10, 'font-family': 'monospace', 'text-valign': 'bottom', 'text-margin-y': 9, 'text-wrap': 'wrap', 'text-max-width': 'data(labelWidth)', 'text-background-color': '#0c161e', 'text-background-opacity': .95, 'text-background-padding': '3px', 'min-zoomed-font-size': 7, 'background-image': 'data(icon)', 'background-fit': 'contain' } },
				{ selector: 'node[type="ROUTER"]', style: { shape: 'diamond', 'background-opacity': 1, 'background-color': '#142e37', 'background-image': 'none', 'border-width': 2, 'border-color': '#72a6b8', width: 27, height: 27 } },
				{ selector: 'node.lan', style: { label: 'data(label)', 'background-image': 'none', 'background-opacity': .3, 'background-color': '#111c25', 'border-width': 1, 'border-style': 'dashed', 'border-color': '#324652', padding: '40px', 'text-valign': 'top', 'text-halign': 'center', 'text-margin-x': 0, 'text-margin-y': 18, 'text-wrap': 'none', 'text-background-opacity': 0, 'font-size': 11, 'compound-sizing-wrt-labels': 'include' } },
				{ selector: 'node[isolated=1]', style: { 'border-width': 2, 'border-color': '#ffbf69', 'border-style': 'double' } },
				{ selector: 'node[unavailable=1]', style: { 'background-image-opacity': .45, 'border-width': 1, 'border-style': 'dashed', 'border-color': '#81919f' } },
				{ selector: 'node:selected', style: { 'overlay-opacity': .12, 'overlay-color': '#ffbf69', 'overlay-padding': 8, color: '#edf6fc', 'min-zoomed-font-size': 0 } },
				{ selector: 'edge', style: { width: 1, 'line-color': '#324652', 'curve-style': 'straight', opacity: .75 } },
				{ selector: 'edge.router-link', style: { width: 2, 'line-color': '#456071' } },
				{ selector: 'edge.flow', style: { width: 2, 'line-color': 'data(color)', 'line-style': 'dashed', 'line-dash-pattern': [8,5], 'target-arrow-shape': 'triangle', 'target-arrow-color': 'data(color)', 'curve-style': 'unbundled-bezier', 'control-point-distances': 48, opacity: .9 } },
				{ selector: 'edge.task', style: { width: 2.5, 'control-point-distances': -35 } }
			] });
			cy.on('tap', 'node', event => { const id = Number(event.target.id()); if (Number.isFinite(id)) onselect(id); });
			cy.on('tap','edge',event=>{const edge=event.target;onselect(Number(edge.hasClass('task')?edge.data('source'):edge.data('target')));});
			resize = new ResizeObserver(() => { cy?.resize(); viewport={width:container.clientWidth,height:container.clientHeight}; }); resize.observe(container);viewport={width:container.clientWidth,height:container.clientHeight};ready = true;
		});
		return () => { disposed = true; cancelAnimationFrame(animation);intersection.disconnect();resize?.disconnect(); cy?.destroy(); };
	});
	$effect(() => {
		if (!ready || !cy || !snapshot) return;
		const routers = snapshot.nodes.filter(n => n.type === 'ROUTER').toSorted((a,b) => a.id - b.id);
		const elements: ElementDefinition[] = [];
		const narrow=viewport.width>0&&viewport.width<500;
		const fontSize=narrow?13:10;
		// Reserve a complete label cell, including long IDs. State belongs on a
		// second line so a changing power/isolation flag cannot widen the grid.
		const cellWidth=Math.max(112,Math.ceil(Math.max(...snapshot.nodes.map(n=>String(n.id).length+4),8)*fontSize*.62)+24);
		const rowHeight=112;
		let offset=0;
		for (const router of routers) {
			const members = [router, ...snapshot.nodes.filter(n => n.router_id === router.id && n.type !== 'ROUTER'&&(!activeOnly||activeIds.has(n.id)||n.id===selected))];
			const maxColumns=narrow?Math.max(1,Math.floor((viewport.width-100)/cellWidth)):8;
			const columns=Math.min(maxColumns,Math.max(1,Math.ceil(Math.sqrt(members.length-1))));
			const originX=narrow?0:offset;
			const originY=narrow?offset:0;
			const center = { x: originX+40+(columns-1)*cellWidth/2, y: originY+20 };
			elements.push({ data: { id: `lan-${router.id}`, label: `LAN ${router.lan_id??router.id}  /  A${snapshot.agents.find(a => a.router_id === router.id)?.id ?? '?'}` }, classes: 'lan', selectable: false });
			for (const [i,n] of members.entries()) {
				const color = n.compromised ? '#ff6c78' : n.exposure != null ? '#ffbf69' : '#50e3b1';
				const offset = i-1;
				const position = i===0 ? center : { x: originX+40+(offset%columns)*cellWidth, y: originY+132+Math.floor(offset/columns)*rowHeight };
				const status=[n.isolated?'ISO':'',n.power!=='ONLINE'?n.power:''].filter(Boolean).join(' · ');
				elements.push({ data: { type:n.type, id: String(n.id), parent: `lan-${router.id}`, label: `${n.compromised?'! ':n.exposure?'? ':''}${n.type==='ROUTER'?'R':n.type==='SERVER'?'SRV-':'WS-'}${n.id}${status?`\n${status}`:''}`, labelWidth:cellWidth-16, icon: icon(n.type,color), isolated: n.isolated?1:0, unavailable: n.power!=='ONLINE'?1:0 }, position });
			}
			offset+=narrow?240+Math.ceil((members.length-1)/columns)*rowHeight:Math.max(240,columns*cellWidth+100);
		}
		// LAN fabric implies attachment; avoid drawing a fan of redundant host edges.
		for (const flow of flows) elements.push({ data: { id:flow.id,source:String(flow.source),target:String(flow.target),color:flowColors[flow.kind] },classes:`flow ${flow.kind==='task'?'task':'session'}` });
		const next = `${viewport.width}:${viewport.height}:${cellWidth}:${activeOnly}:${elements.filter(e=>e.position).map(e=>e.data.id).join(',')}`;
		cy.batch(() => {
			const ids = new Set(elements.map(e => e.data.id)); cy!.elements().filter(e => !ids.has(e.id())).remove();
			for(const id of dataSignatures.keys())if(!ids.has(id))dataSignatures.delete(id);
			for (const e of elements) {const id=e.data.id!;const existing=cy!.getElementById(id);const dataSignature=JSON.stringify(e.data);if(existing.length){if(dataSignatures.get(id)!==dataSignature)existing.data(e.data);if(next!==signature&&e.position)existing.position(e.position);}else cy!.add(e);dataSignatures.set(id,dataSignature);}
			cy!.nodes().not('.lan').style('font-size',narrow?13:10);
			cy!.$(':selected').filter(n=>n.id()!==String(selected)).unselect();if(selected!==null)cy!.getElementById(String(selected)).select();
		});
		signature=next;
		// Live membership changes must never overwrite the operator's camera.
		const nextScope=String(scope??routers[0]?.id??'empty');
		if(fittedScope!==nextScope){fittedScope=nextScope;fit();}
	});
	function fit(){
		if(!cy)return;
		cy.fit(undefined,30);
		if(viewport.width>0&&viewport.width<500&&cy.zoom()<.85){
			cy.zoom(.85);
			const focus=(selected===null?cy.nodes().not('.lan').first():cy.getElementById(String(selected))) as NodeSingular;
			if(focus.length){const point=focus.position();cy.pan({x:cy.width()/2-point.x*cy.zoom(),y:72-point.y*cy.zoom()});}
		}
	}
	function zoom(factor:number) { if(cy)cy.zoom({level:cy.zoom()*factor,renderedPosition:{x:cy.width()/2,y:cy.height()/2}}); }
	function focusSelected(){if(cy&&selected!==null){const node=cy.getElementById(String(selected));if(node.length)cy.center(node);}}
</script>
<div class="map-bar"><h2>CONNECTIONS</h2><span class="pill">{snapshot?.nodes.filter(n=>n.type!=='ROUTER').length??0} HOSTS</span><span class="connection-count">{flows.filter(f=>f.kind==='task').length} tasks · {snapshot?.sessions.length??0} connections</span></div>
<div class="topology-tools"><label>Activity <select aria-label="Topology activity" bind:value={activity}><option value="all">Tasks + connections</option><option value="task">Vanguard tasks</option><option value="connection">Host connections</option></select></label><label><input type="checkbox" bind:checked={activeOnly}/> Active hosts only</label><button disabled={selected===null} onclick={focusSelected}>Focus selection</button></div>
<p class="map-hint">{selected===null?'Select a router for its running jobs, or a host for its evidence':'Selection stays in the inspector; Focus selection centers the map'} · Drag to pan · Scroll to zoom · Camera stays put during live updates{#if snapshot?.sessions.some(s=>!renderedIds.has(s.source)||!renderedIds.has(s.target))} · More peers listed in inspector{/if}</p>
<div class="cytoscape" bind:this={container} aria-label="Live LAN topology. Use Node table for keyboard-accessible host navigation." role="img"></div>
<div class="map-footer"><div class="graph-legend"><span><i class="green-dot"></i>Healthy / other connection</span><span><i class="amber-dot"></i>In danger / exposure</span><span><i class="red-dot"></i>Compromised</span><span style="color:#90b8fc"><Network size={12}/> Vanguard task → host</span><span>[ISO] Isolated · dim: unavailable</span><span>Moving arrows = live activity · LAN attachment implied</span></div><div class="map-controls"><button aria-label="Zoom out" onclick={()=>zoom(.8)}><Minus size={14}/></button><button aria-label="Zoom in" onclick={()=>zoom(1.25)}><Plus size={14}/></button><button aria-label="Fit topology" onclick={fit}><Focus size={14}/> Fit</button></div></div>

<style>
	.topology-tools{display:flex;align-items:center;flex-wrap:wrap;gap:9px 14px;padding:10px 16px;border-bottom:1px solid var(--line)}.topology-tools label{display:flex;align-items:center;gap:6px;font:10px monospace;color:var(--muted)}.topology-tools select,.topology-tools button{font-size:10px;padding:6px}.topology-tools input{accent-color:#90b8fc}
</style>
