<script lang="ts">
	import type { Snapshot } from './types';
	import JsonTree from './JsonTree.svelte';
	import { Monitor, Server, Network, ArrowUpRight, ArrowDownLeft, MoveRight } from '@lucide/svelte';
	let { snapshot, selected, onselect }: { snapshot: Snapshot | null; selected: number | null; onselect: (id: number) => void } = $props();
	let view = $state('Truth');
	let agent = $state('');
	let connectionPage = $state(0);
	let observationPage = $state(0);
	let runId = $derived(snapshot?.run_id);
	let byId = $derived(new Map((snapshot?.nodes??[]).map(n=>[n.id,n])));
	let node = $derived(selected===null?undefined:byId.get(selected));
	let owner = $derived(snapshot?.agents.find(a => a.router_id === node?.router_id));
	let observer = $derived(snapshot?.agents.find(a => String(a.id) === agent));
	let known = $derived(!!observer && selected !== null && observer.known.includes(selected));
	let connections=$derived(snapshot?.sessions.filter(s=>s.source===selected||s.target===selected||s.route?.includes(selected))??[]);
	function hostName(id:number){const peer=byId.get(id);return `${peer?.type==='ROUTER'?'Router':peer?.type==='SERVER'?'Server':'Host'} ${id}`;}
	let observations = $derived((view==='Discovered'?observer?.observations ?? []:[]).filter(o => {
		const data = o.data ?? {};
		return o.target === selected || (o.target == null && o.router_id === selected)
			|| (o.action === 'SCAN' && ((data.hosts ?? []).some((h:any) => h.id === selected) || (data.routers ?? []).includes(selected)))
			|| (data.pairs ?? []).some((p:any) => p.a === selected || p.b === selected)
			|| (data.connections ?? []).some((c:any) => c.peer === selected);
	}));
	$effect(()=>{selected;agent;runId;connectionPage=0;observationPage=0;});
	$effect(()=>{if(connectionPage*100>=connections.length)connectionPage=0;if(observationPage*50>=observations.length)observationPage=0;});
</script>
{#if node}<div class="selected-node"><div class="node-icon">{#if node.type==='ROUTER'}<Network size={24}/>{:else if node.type==='SERVER'}<Server size={24}/>{:else}<Monitor size={24}/>{/if}</div><div><h3>{node.type === 'ROUTER' ? 'Router' : node.type === 'SERVER' ? 'Server' : 'Workstation'} {node.id}</h3><small>LAN {node.lan_id ?? node.router_id} / ROUTER R{node.router_id}</small></div></div>{/if}
<div class="tabs" aria-label="Node visibility"><button aria-pressed={view === 'Truth'} onclick={() => view = 'Truth'}>True state</button><button aria-pressed={view === 'Discovered'} onclick={() => view = 'Discovered'}>Discovered</button></div>
<div class="node-content">
	{#if view === 'Truth' && node}
		<div class="state-banner" class:danger={node.exposure!=null} class:infected={node.compromised}><strong>● {node.infection === 'IN_DANGER' ? 'IN DANGER' : node.infection ?? (node.type === 'ROUTER' ? 'IMMUNE' : 'HEALTHY')}</strong><small>LIVE TRUTH</small></div>
		<div class="integrity-label">{node.type === 'ROUTER' ? 'Router integrity' : 'Host integrity'}<span>{node.type === 'ROUTER' ? 'N/A' : `${node.health ?? '?'} / 5`}</span></div><div class="integrity" class:danger={node.exposure!=null} class:infected={node.compromised}>{#each [1,2,3,4,5] as segment}<i class:empty={node.type==='ROUTER'||segment>(node.health??0)}></i>{/each}</div>
		<dl class="data-list"><dt>Power</dt><dd class:green={node.power==='ONLINE'}>{node.power}</dd><dt>Policy</dt><dd>{node.policy}</dd><dt>Isolation</dt><dd>{node.type==='ROUTER'?'Not applicable':node.isolated?'Isolated':'Released'}</dd><dt>Exposure</dt><dd>{node.exposure==null?'No active exposure':`${node.exposure.age} / ${snapshot?.scenario.infection_intervals ?? 14} ticks`}</dd><dt>Exceptions</dt><dd>{node.exceptions?.join(', ')||'None'}</dd></dl>
		<section class="inspector-section"><h4>OWNING VANGUARD</h4><span class="owner-tag">A{owner?.id ?? '?'} / R{node.router_id}</span><p class="muted">Truth is operator-only. Agent observations are separate.</p></section>
		<section class="inspector-section connections-section"><h4>CONNECTIONS <span>{connections.length}</span></h4>
			{#each connections.slice(connectionPage*100,(connectionPage+1)*100) as session (session.id)}{@const outbound=session.source===node.id}{@const inbound=session.target===node.id}{@const peer=outbound?session.target:session.source}
				<article class="live-connection"><header>{#if outbound}<ArrowUpRight size={15}/>{:else if inbound}<ArrowDownLeft size={15}/>{:else}<MoveRight size={15}/>{/if}<strong>{outbound?'Outbound':inbound?'Inbound':'Transit'}</strong><span>#{session.id}</span></header>
					{#if outbound||inbound}<button class="connection-peer" onclick={()=>onselect(peer)}>{hostName(peer)}<ArrowUpRight size={13}/></button>{:else}<div class="connection-endpoints"><button onclick={()=>onselect(session.source)}>{hostName(session.source)}</button><MoveRight size={12}/><button onclick={()=>onselect(session.target)}>{hostName(session.target)}</button></div>{/if}
					<p>Opened at tick {session.opened_at} · {Math.max(0,(snapshot?.tick??0)-session.opened_at)} ticks ago</p>
					{#if session.route?.length}<p>Via {session.route.map((id:number)=>`R${id}`).join(' → ')}</p>{:else}<p>Same LAN</p>{/if}
				</article>
			{:else}<p class="empty">No active connections for this node.</p>{/each}
			{#if connections.length>100}<div class="table-pagination" aria-label="Connection pages"><button disabled={connectionPage===0} onclick={()=>connectionPage--}>Previous connections</button><span>{connectionPage*100+1}–{Math.min((connectionPage+1)*100,connections.length)} / {connections.length}</span><button disabled={(connectionPage+1)*100>=connections.length} onclick={()=>connectionPage++}>Next connections</button></div>{/if}
		</section>
	{:else if view === 'Discovered'}
		<label class="observer-label">Vanguard perspective<select bind:value={agent} aria-label="Discovered Vanguard filter"><option value="">Choose Vanguard</option>{#each snapshot?.agents??[] as a}<option value={String(a.id)}>A{a.id} · R{a.router_id}</option>{/each}</select></label><p class="inspector-note">Recorded observations only. Missing fields stay unknown; older scans may be stale.</p>
		{#if !observer}<p class="empty">Choose a Vanguard to inspect its knowledge.</p>{:else if !known}<p class="empty">Undiscovered by A{observer.id}.</p>{:else}{#each observations.slice(observationPage*50,(observationPage+1)*50) as o,index (`${o.task_id}-${index}`)}<article class="observation"><header>{o.action} · T{o.observed_at ?? o.finished_at} · Task #{o.task_id}</header><p class="muted">{Math.max(0,(snapshot?.tick??0)-(o.observed_at??o.finished_at))} ticks ago</p><JsonTree value={o} /></article>{:else}<p class="empty">ID known. No recorded scan details for this node.</p>{/each}{#if observations.length>50}<div class="table-pagination" aria-label="Observation pages"><button disabled={observationPage===0} onclick={()=>observationPage--}>Previous observations</button><span>{observationPage*50+1}–{Math.min((observationPage+1)*50,observations.length)} / {observations.length}</span><button disabled={(observationPage+1)*50>=observations.length} onclick={()=>observationPage++}>Next observations</button></div>{/if}{/if}
	{:else}<p class="empty">Select a node to inspect.</p>{/if}
</div>
