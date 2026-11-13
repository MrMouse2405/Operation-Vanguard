<script lang="ts">
	import { Search, Network, Monitor, Server, Table2, Globe2, ChevronRight, ChevronLeft, X } from '@lucide/svelte';
	import type { Snippet } from 'svelte';
	import { untrack, tick } from 'svelte';
	import { onMount } from 'svelte';
	import type { Snapshot, Node } from './types';
	import Topology from './Topology.svelte';
	import Globe from './Globe.svelte';
	import { inventory } from './inventory';
	import NodeInspector from './NodeInspector.svelte';
	import PaneResize from './PaneResize.svelte';
	let {snapshot,hostState=$bindable('all'),hostType=$bindable('all'),children,onscope}:{snapshot:Snapshot|null;hostState?:string;hostType?:string;children:Snippet;onscope?:(router:number|null)=>void}=$props();
	let search=$state('');let searchInput:HTMLInputElement;
	let shortcut=$state('Ctrl K');
	let viewport=$state(1440);let layoutWidth=$state(1440);
	let leftWidth=$state(258);let rightWidth=$state(300);
	let leftHeight=$state(430);let rightHeight=$state(450);
	let leftSize=$derived(Math.max(200,Math.min(leftWidth,layoutWidth-(viewport>1000?Math.max(240,rightWidth):0)-334)));
	let rightSize=$derived(Math.max(240,Math.min(rightWidth,layoutWidth-leftSize-334)));
	onMount(()=>{
		shortcut=/Mac|iPhone|iPad/.test(navigator.platform)?'⌘ K':'Ctrl K';
		try{const saved=JSON.parse(localStorage.getItem('vanguard-explorer-panes')??'{}');
			if(Number.isFinite(saved.left))leftWidth=saved.left;
			if(Number.isFinite(saved.right))rightWidth=saved.right;
		}catch{/* Layout preferences are optional. */}
	});
	function saveWidths(){try{localStorage.setItem('vanguard-explorer-panes',JSON.stringify({left:leftWidth,right:rightWidth}));}catch{/* Optional. */}}
	let scope=$state<number|null>(null);let selected=$state<number|null>(null);
	let view=$state('map');let side=$state('hierarchy');let sidePage=$state(0);let tablePage=$state(0);let lanPage=$state(0);
	let index=$derived(inventory(snapshot?.nodes??[]));
	let hosts=$derived(index.hosts);
	let routers=$derived(index.routers);
	let byRouter=$derived(index.byRouter);
	let byId=$derived(index.byId);
	let agentsByRouter=$derived(new Map((snapshot?.agents??[]).map(a=>[a.router_id,a])));
	let query=$derived(search.trim().toLowerCase());
	let searchLabels=$derived(new Map(hosts.map(n=>[n.id,`${n.id} ${n.type==='SERVER'?'srv':'ws'}-${n.id} LAN ${n.lan_id??n.router_id} R${n.router_id} A${agentsByRouter.get(n.router_id)?.id}`.toLowerCase()])));
	const stateOf=(n:Node)=>n.compromised?'COMPROMISED':n.exposure?'IN_DANGER':'HEALTHY';
	let matches=$derived(hosts.filter(n=>(hostState==='all'||(hostState==='risk'?stateOf(n)!=='HEALTHY':stateOf(n)===hostState))&&(hostType==='all'||n.type===hostType)&&(!query||searchLabels.get(n.id)!.includes(query))));
	let matchingIds=$derived(new Set(matches.map(n=>n.id)));
	let matchingCounts=$derived.by(()=>{const counts=new Map<number,number>();for(const n of matches)counts.set(n.router_id,(counts.get(n.router_id)??0)+1);return counts;});
	let scoped=$derived(scope===null?matches:matches.filter(n=>n.router_id===scope));
	let local=$derived(scope===null?hosts:(byRouter.get(scope)??[]));
	let selectedNode=$derived(selected===null?undefined:byId.get(selected));
	let matchingRouters=$derived(routers.filter(r=>!query&&hostState==='all'&&hostType==='all'||matchingCounts.has(r.id)));
	let visibleLans=$derived(matchingRouters.slice(lanPage*48,lanPage*48+48));
	let graphSnapshot=$derived.by(()=>{
		if(!snapshot||scope===null||view!=='map')return null;
		const sessions=snapshot.sessions.filter(s=>s.source===selected||s.target===selected||s.route?.includes(selected)||byId.get(s.source)?.router_id===scope||byId.get(s.target)?.router_id===scope);
		const page=scoped.slice(tablePage*80,tablePage*80+80);
		const peerIds=new Set(sessions.flatMap(s=>[s.source,s.target]));
		for(const task of snapshot.jobs){if(task.state==='RUNNING'&&task.router_id===scope&&task.target!=null)peerIds.add(task.target);}
		const pageIds=new Set(page.map(n=>n.id));
		const peers=[...peerIds].filter(id=>!pageIds.has(id)).map(id=>byId.get(id)).filter((n):n is Node=>!!n).toSorted((a,b)=>a.id-b.id).slice(0,80);
		const members=[...page,...peers];
		const routerIds=new Set([scope,...members.map(n=>n.router_id)]);
		const nodes=[...routers.filter(n=>routerIds.has(n.id)),...members.filter(n=>n.type!=='ROUTER')];
		return {...snapshot,nodes,links:[],sessions};
	});
	$effect(()=>{search;hostState;hostType;scope;sidePage=0;tablePage=0;lanPage=0;});
	$effect(()=>{const router=scope;untrack(()=>onscope?.(router));});
	$effect(()=>{if(hostType==='SERVER'){scope=null;selected=null;view='table';side='results';search='';}});
	$effect(()=>{if(snapshot&&scope!==null&&!routers.some(r=>r.id===scope)){scope=null;selected=null;}});
	function select(id:number){const n=byId.get(id);if(!n)return;selected=id;if(view==='map'){scope=n.router_id;void tick().then(()=>{if(selected===id&&view==='map'){const index=scoped.findIndex(n=>n.id===id);if(index>=0)tablePage=Math.floor(index/80);}});}}
	function locate(id:number){const n=byId.get(id);if(!n)return;if(view!=='globe')view='map';if(!matchingIds.has(id)&&n.type!=='ROUTER'){reset();}select(id);}
	function showMap(){view='map';if(selected!==null)select(selected);}
	function enter(router:number){scope=router;selected=null;tablePage=0;}
	function sidebarLAN(router:number){if(view!=='globe')view='map';enter(router);}
	function reset(){search='';hostState='all';hostType='all';tablePage=sidePage=0;}
	function servers(infected=false){reset();scope=null;hostType='SERVER';hostState=infected?'COMPROMISED':'all';view='table';side='results';selected=null;}
	function focusSearch(){searchInput?.focus();searchInput?.select();}
	function keys(e:KeyboardEvent){if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'&&!document.querySelector('dialog[open]')){e.preventDefault();focusSearch();}if(e.key==='Escape'&&!document.querySelector('dialog[open]')){if(selected!==null)selected=null;else scope=null;}}
	const countBad=(nodes:Node[])=>nodes.filter(n=>n.compromised).length;
	const countDanger=(nodes:Node[])=>nodes.filter(n=>!n.compromised&&n.exposure).length;
</script>
<svelte:window onkeydown={keys} bind:innerWidth={viewport}/>
<div class="exploration-layout resizable-explorer" bind:clientWidth={layoutWidth} style={`--left-pane:${leftSize}px;--right-pane:${rightSize}px;--left-height:${leftHeight}px;--right-height:${rightHeight}px`}>
	<aside class="network-explorer" aria-label="Network explorer">
		<div class="explorer-head"><h2>Network explorer <button class="search-shortcut" aria-label="Focus host search" aria-keyshortcuts="Control+k Meta+k" onclick={focusSearch}><kbd>{shortcut}</kbd></button></h2><label class="explorer-search"><Search size={15}/><input bind:this={searchInput} bind:value={search} placeholder="Find ID, hostname, LAN…" aria-label="Search all hosts"/></label><div class="explorer-filters"><select aria-label="Host state" bind:value={hostState}><option value="all">All states</option><option value="risk">All at risk</option><option value="COMPROMISED">Compromised</option><option value="IN_DANGER">In danger</option><option value="HEALTHY">Healthy</option></select><select aria-label="Host type" bind:value={hostType}><option value="all">All host types</option><option value="WORKSTATION">Workstations</option><option value="SERVER">Servers</option></select></div><div class="server-shortcuts"><button onclick={()=>servers()}><Server size={13}/> All servers</button><button class="red" onclick={()=>servers(true)}>Infected servers</button></div><div class="filter-caption"><span>{matches.length} matching hosts</span><button onclick={reset}>Reset filters</button></div></div>
		<div class="explorer-tabs"><button class:active={side==='hierarchy'&&!search} onclick={()=>{side='hierarchy';search='';}}>Hierarchy</button><button class:active={side==='results'||!!search} onclick={()=>side='results'}>Node results</button></div>
		<div class="explorer-list">{#if side==='hierarchy'&&!search}<h3>NETWORK / {routers.length} LANS</h3>{#each visibleLans as router (router.id)}{@const nodes=byRouter.get(router.id)??[]}<button class="lan-row" class:active={scope===router.id} onclick={()=>sidebarLAN(router.id)}><Network size={15}/><span>LAN {router.lan_id??router.id}</span><span class="lan-counts"><b class="red">{countBad(nodes)||'·'}</b><b class="amber">{countDanger(nodes)||'·'}</b><small>{matchingCounts.get(router.id)??0}/{nodes.length}</small></span></button>{/each}{:else}{#each matches.slice(sidePage*40,sidePage*40+40) as node (node.id)}<button class="host-row" class:active={selected===node.id} onclick={()=>locate(node.id)}>{#if node.type==='SERVER'}<Server size={16}/>{:else}<Monitor size={16}/>{/if}<span><strong>{node.type==='SERVER'?'srv':'ws'}-{node.id}</strong><small>LAN {node.lan_id??node.router_id} · {stateOf(node).replaceAll('_',' ')}</small></span><ChevronRight size={12}/></button>{:else}<p class="empty">No matching hosts. Try an ID or reset filters.</p>{/each}{/if}</div>
		<div class="explorer-pagination"><span>{side==='hierarchy'&&!search ? `${matchingRouters.length} LANs · ${matchingRouters.length} routers` : `${sidePage*40+1}–${Math.min((sidePage+1)*40,matches.length)} / ${matches.length}`}</span><button aria-label="Previous node results" disabled={side==='hierarchy'&&!search?lanPage===0:sidePage===0} onclick={()=>side==='hierarchy'&&!search?lanPage--:sidePage--}><ChevronLeft size={14}/></button><button aria-label="Next node results" disabled={side==='hierarchy'&&!search?(lanPage+1)*48>=matchingRouters.length:(sidePage+1)*40>=matches.length} onclick={()=>side==='hierarchy'&&!search?lanPage++:sidePage++}><ChevronRight size={14}/></button></div>
	</aside>
	<PaneResize label="Resize network explorer" axis={viewport<=650?'y':'x'} value={viewport<=650?leftHeight:leftSize} min={viewport<=650?330:200} max={viewport<=650?900:Math.max(200,layoutWidth-(viewport>1000?rightSize:0)-334)} onresize={value=>{if(viewport<=650)leftHeight=value;else{leftWidth=value;saveWidths();}}} onreset={()=>{leftWidth=258;leftHeight=430;saveWidths();}}/>
	<main class="explorer-center">
		<div class="explorer-viewbar"><nav aria-label="Network location"><button onclick={()=>{scope=null;selected=null;}}>Network</button><ChevronRight size={13}/><b>{scope===null?'All LANs':`LAN ${routers.find(r=>r.id===scope)?.lan_id??scope}`}</b></nav><div class="view-toggle" role="group" aria-label="Network view"><button class:active={view==='map'} aria-pressed={view==='map'} onclick={showMap}><Network size={14}/> Topology</button><button class:active={view==='table'} aria-pressed={view==='table'} onclick={()=>view='table'}><Table2 size={14}/> Table</button><button class:active={view==='globe'} aria-pressed={view==='globe'} onclick={()=>view='globe'}><Globe2 size={14}/> Globe</button></div></div>
		<div class="scope-bar"><span><b>{scope===null?`${routers.length} LANs`:`${local.length} hosts`}</b><small>{scope===null?'Select a LAN to explore':`R${scope} / A${snapshot?.agents.find(a=>a.router_id===scope)?.id??'?'}`}</small></span><span class="blue">{local.filter(n=>n.type==='SERVER').length} servers · {countBad(local.filter(n=>n.type==='SERVER'))} infected</span>{#if scope!==null}<button onclick={()=>{scope=null;selected=null;}}>All LANs</button>{/if}</div>
		<div class="explorer-visual" id="network-map" tabindex="-1">
			{#if view==='globe'}<Globe {snapshot} {scope} {selected} {matchingIds} onselect={select} onscope={router=>{scope=router;selected=null;}}/>
			{:else if view==='table'&&scope===null}<div class="node-table-wrap"><table class="node-table lan-table"><thead><tr><th>LAN / ROUTER</th><th>HOSTS</th><th>MATCHING</th><th>SERVERS</th><th>COMPROMISED</th><th>IN DANGER</th></tr></thead><tbody>{#each visibleLans as router (router.id)}{@const nodes=byRouter.get(router.id)??[]}<tr onclick={()=>enter(router.id)}><td><button onclick={()=>enter(router.id)}><Network size={14}/> LAN {router.lan_id??router.id} / R{router.id}</button></td><td>{nodes.length}</td><td>{matchingCounts.get(router.id)??0}</td><td>{nodes.filter(n=>n.type==='SERVER').length}</td><td class:red={countBad(nodes)>0}>{countBad(nodes)}</td><td class:amber={countDanger(nodes)>0}>{countDanger(nodes)}</td></tr>{:else}<tr><td colspan="6" class="empty">No matching LANs. Try an ID or reset filters.</td></tr>{/each}</tbody></table></div>
			{:else if view==='table'}<div class="node-table-wrap"><table class="node-table"><thead><tr><th>HOST / ID</th><th>STATE</th><th>LAN / ROUTER</th><th>TYPE</th><th>POWER</th></tr></thead><tbody>{#each scoped.slice(tablePage*80,tablePage*80+80) as node (node.id)}<tr class:selected={selected===node.id}><td><button onclick={()=>select(node.id)}>{#if node.type==='SERVER'}<Server size={14}/>{:else}<Monitor size={14}/>{/if} {node.type==='SERVER'?'srv':'ws'}-{node.id}</button></td><td class:red={node.compromised} class:amber={!!node.exposure&&!node.compromised} class:green={!node.compromised&&!node.exposure}>{stateOf(node).replaceAll('_',' ')}{node.isolated?' / ISO':''}</td><td>{node.lan_id??node.router_id} / R{node.router_id}</td><td>{node.type}</td><td>{node.power}</td></tr>{:else}<tr><td colspan="5" class="empty">No matching hosts in this scope.</td></tr>{/each}</tbody></table></div>
			{:else if scope===null}<div class="health-map"><div class="health-map-title"><h2>Network health map</h2><p>Aggregates show host health, not physical routing links.</p></div><div class="lan-grid">{#each visibleLans as router (router.id)}{@const nodes=byRouter.get(router.id)??[]}{@const bad=countBad(nodes)}{@const danger=countDanger(nodes)}<button class="lan-tile" class:dim={!matchingCounts.has(router.id)} onclick={()=>enter(router.id)}><strong><Network size={15}/> LAN {router.lan_id??router.id}</strong><span>{nodes.length} hosts · R{router.id}</span><span class="blue">SRV {nodes.filter(n=>n.type==='SERVER').length} / !{countBad(nodes.filter(n=>n.type==='SERVER'))}</span><span class="health-strip"><i style={`flex:${nodes.length-bad-danger}`}></i><i class="exposed" style={`flex:${danger}`}></i><i class="infected" style={`flex:${bad}`}></i></span><small><span class="red">{bad} compromised</span> · <span class="amber">{danger} exposed</span></small></button>{/each}</div>{#if routers.length>48}<div class="table-pagination"><button disabled={lanPage===0} onclick={()=>lanPage--}>Previous LANs</button><button disabled={(lanPage+1)*48>=routers.length} onclick={()=>lanPage++}>Next LANs</button></div>{/if}</div>
			{:else}<div class="lan-detail"><Topology snapshot={graphSnapshot} {scope} selected={selectedNode?.router_id===scope?selected:null} onselect={select}/></div>{/if}
		</div>
		<div class="explorer-context"><span>{view==='globe'?'Live tasks and host sessions · arrows show direction':scope===null?`${matchingRouters.length} matching LANs · select a LAN to explore hosts`:`${scoped.length} matching hosts · ${view==='map'?'bounded LAN detail':'80 rows per page'}`}</span>{#if view==='globe'}<span>Illustrative geography</span>{:else if scope!==null}<div class="inventory-pager"><button disabled={tablePage===0} onclick={()=>tablePage--}>Previous</button><span>{scoped.length?tablePage*80+1:0}–{Math.min((tablePage+1)*80,scoped.length)}</span><button disabled={(tablePage+1)*80>=scoped.length} onclick={()=>tablePage++}>Next</button></div>{:else if view==='table'}<div class="inventory-pager"><button disabled={lanPage===0} onclick={()=>lanPage--}>Previous</button><span>{matchingRouters.length?lanPage*48+1:0}–{Math.min((lanPage+1)*48,matchingRouters.length)}</span><button disabled={(lanPage+1)*48>=matchingRouters.length} onclick={()=>lanPage++}>Next</button></div>{:else}<span>{matchingRouters.length} aggregates / {matches.length} hosts</span>{/if}</div>
		{@render children()}
	</main>
	<PaneResize label="Resize match score and inspector" axis={viewport<=1000?'y':'x'} value={viewport<=1000?rightHeight:rightSize} min={viewport<=1000?180:240} max={viewport<=1000?1000:Math.max(240,layoutWidth-leftSize-334)} reverse={viewport>1000} onresize={value=>{if(viewport<=1000)rightHeight=value;else{rightWidth=value;saveWidths();}}} onreset={()=>{rightWidth=300;rightHeight=450;saveWidths();}}/>
	<aside class="inspector explorer-inspector" aria-label="Node inspector">
		{#if selected!==null}<div class="panel-heading"><h2>NODE INSPECTOR</h2><button class="icon-button" aria-label="Close node inspector" onclick={()=>selected=null}><X size={15}/></button></div><NodeInspector {snapshot} {selected} onselect={locate}/>{:else}<div class="match-overview">
		<section class="score-brief"><h3>Match score <span>{snapshot?.score.outcome??'CONNECTING'}</span></h3>{#if snapshot?.score.enabled}<strong>{snapshot.score.total.toFixed(2)} <small>/ 1,000</small></strong><dl><dt>Service</dt><dd>{snapshot.score.service.toFixed(2)} / 600</dd><dt>Security</dt><dd>{snapshot.score.security.toFixed(2)} / 250</dd><dt>Recovery (provisional)</dt><dd>{snapshot.score.recovery.toFixed(2)} / 150</dd><dt>Working service</dt><dd>{Math.round(snapshot.score.service_fraction*100)}%</dd><dt>Recovery window</dt><dd>{snapshot.score.recovery_streak} / {snapshot.scenario.mission?.recovery_ticks}</dd><dt>Total outage</dt><dd>{snapshot.score.outage_streak} / {snapshot.scenario.mission?.outage_ticks}</dd></dl>{:else}<p>Unscored practice network.</p>{/if}</section>
		</div>{/if}
	</aside>
</div>
