<script lang="ts">
	import { onMount } from 'svelte';
	import { Play, Pause, StepForward, RotateCcw, Info, ChevronDown, ChevronUp, Shield, Server } from '@lucide/svelte';
	import { createConnection } from '$lib/dashboard/connection.svelte';
	import Explorer from '$lib/dashboard/Explorer.svelte';
	import Vanguard from '$lib/dashboard/Vanguard.svelte';
	import Jobs from '$lib/dashboard/Jobs.svelte';
	import Stats from '$lib/dashboard/Stats.svelte';
	import Modal from '$lib/dashboard/Modal.svelte';
	import JsonTree from '$lib/dashboard/JsonTree.svelte';
	import DebugViews from '$lib/dashboard/DebugViews.svelte';
	import PaneResize from '$lib/dashboard/PaneResize.svelte';
	import '$lib/dashboard/dashboard.css';
	import '$lib/dashboard/explorer-fixes.css';
	const runtime = createConnection();
	let snapshot = $derived(runtime.snapshot);
	let dock = $state('Activity');
	let agent = $state('');
	let collapsed = $state(true);
	let dockHeight = $state(260);
	let viewportHeight = $state(1000);
	let settings = $state(false);
	let showDebug = $state(false);
	let count = $state(10);
	let inputError = $state('');
	let hostState = $state('all');
	let hostType = $state('all');
	let hosts = $derived(snapshot?.nodes.filter(n => n.type !== 'ROUTER') ?? []);
	let danger = $derived(hosts.filter(n => !n.compromised && n.exposure != null).length);
	let compromised = $derived(hosts.filter(n => n.compromised).length);
	let servers = $derived(hosts.filter(n => n.type === 'SERVER'));
	let controlState = $derived(snapshot?.control ?? 'CONNECTING');
	let canStep = $derived(!!snapshot && ['READY', 'PAUSED'].includes(controlState) && !runtime.pending);
	const tabs = ['Activity', 'Jobs', 'Output', 'Messages', 'Pub / Sub', 'Statistics'];
	onMount(() => {
		try{const saved=Number(localStorage.getItem('vanguard-dock-height'));if(saved>=120&&saved<=1000)dockHeight=saved;}catch{/* Optional preference. */}
		runtime.connect(); return runtime.destroy;
	});
	function resizeDock(value:number){dockHeight=value;try{localStorage.setItem('vanguard-dock-height',String(value));}catch{/* Optional. */}}
	function step(ticks: number) { if (!Number.isInteger(ticks) || ticks < 1 || ticks > 1000) { inputError = 'Enter a whole number from 1 to 1000.'; return; } inputError = ''; void runtime.send('StepTicks', ticks); }
	function filter(state: string, type = 'all') { hostState=state;hostType=type; }
	function pacing(value: string) {
		const seconds = value.trim() ? Number(value) : NaN;
		if (!Number.isFinite(seconds) || seconds < 0 || seconds > 3600) { inputError = 'Seconds/tick must be between 0 and 3600. Use 0 for full speed.'; return; }
		inputError = ''; void runtime.send('SetTickPacing', undefined, seconds);
	}
</script>

<svelte:head><title>Vanguard · Network operations</title><meta name="description" content="Explore a live network defence match and your agent's evidence." /></svelte:head>
<svelte:window bind:innerHeight={viewportHeight}/>
<div class="dashboard-shell explorer-shell" class:debug-open={showDebug&&snapshot?.debug?.enabled}>
	<header class="command-header">
		<a class="brand" href="/" aria-label="Operation Vanguard"><Shield size={28} class="brand-shield" /><span>OPERATION VANGUARD<small>NETWORK OPERATIONS / EXPLORER</small></span></a>
		<div class="run-label">LOCAL EXERCISE<small>SEED {snapshot?.scenario.seed ?? '—'} · {snapshot?.agents.length ?? 0} VMS</small></div>
		<div class="command-controls"><span class="tick-label">TICK <b data-testid="simulation-tick">{snapshot?.tick ?? 0}</b> / {snapshot?.scenario.tick_limit ?? '∞'}</span><span class="control-state" class:running={controlState === 'RUNNING'}>{controlState}</span>
			{#if controlState === 'RUNNING'}<button class="primary" disabled={!!runtime.pending} onclick={() => runtime.send('Pause')}><Pause size={15} /> Pause</button>{:else if controlState === 'PAUSED'}<button class="primary" disabled={!!runtime.pending} onclick={() => runtime.send('Resume')}><Play size={15} /> Resume</button>{:else}<button class="primary" disabled={controlState !== 'READY' || !!runtime.pending} onclick={() => runtime.send('Start')}><Play size={15} /> Start</button>{/if}
			<button disabled={!canStep} onclick={() => step(1)} aria-label="+1 tick">+1</button><input class="tick-input" aria-label="Number of ticks to advance" type="number" min="1" max="1000" step="1" bind:value={count} disabled={!canStep} /><button disabled={!canStep} onclick={() => step(count)}><StepForward size={15} /> Forward</button>
			<label class="pacing-control" title="Minimum seconds per tick, including computation. 0 = full speed. Applies to continuous playback.">s/tick <input aria-label="Seconds per tick" type="number" min="0" max="3600" step="0.05" value={snapshot?.seconds_per_tick ?? 0} disabled={!snapshot || !!runtime.pending} onchange={e => pacing(e.currentTarget.value)} /></label>
			<button class="icon-button" aria-label="Restart" disabled={!snapshot || !!runtime.pending} onclick={() => runtime.send('Restart')}><RotateCcw size={16} /></button><button class="icon-button" aria-label="Scenario information" onclick={() => settings = true}><Info size={16} /></button>
		</div>
	</header>
	{#if runtime.error || inputError}<div class="runtime-error" role="alert">{inputError || runtime.error}</div>{/if}
	{#if snapshot?.debug?.enabled}<button class="debug-toggle" aria-expanded={showDebug} onclick={()=>showDebug=!showDebug}>{showDebug?'Hide':'Compare'} agent drawings · dbgguard</button>{#if showDebug}<DebugViews {snapshot}/>{/if}{/if}
	<section class="host-summary" aria-label="Filter by host state">
		<button class:active={hostState==='all'&&hostType==='all'} onclick={()=>filter('all')}><b>{hosts.length}</b><span>TOTAL<br/>HOSTS</span></button>
		<button class:active={hostState==='COMPROMISED'&&hostType==='all'} onclick={()=>filter('COMPROMISED')}><b class="red">{compromised}</b><span>COMPROMISED<small>Review incidents</small></span></button>
		<button class:active={hostState==='IN_DANGER'} onclick={()=>filter('IN_DANGER')}><b class="amber">{danger}</b><span>IN DANGER<small>Active exposure</small></span></button>
		<button class:active={hostState==='HEALTHY'} onclick={()=>filter('HEALTHY')}><b class="green">{hosts.length-compromised-danger}</b><span>HEALTHY<small>No active exposure</small></span></button>
		<button class:active={hostType==='SERVER'&&hostState==='all'} onclick={()=>filter('all','SERVER')}><Server size={17} class="blue"/><b class="blue">{servers.length}</b><span>SERVERS<small>Browse inventory</small></span></button>
		<button class:active={hostType==='SERVER'&&hostState==='COMPROMISED'} onclick={()=>filter('COMPROMISED','SERVER')}><b class="red">{servers.filter(n=>n.compromised).length}</b><span>INFECTED SERVERS<small>of {servers.length} servers</small></span></button>
		<div class="summary-score"><b>{snapshot?.score.enabled ? `${snapshot.score.total.toFixed(2)} / 1,000` : 'PRACTICE'}</b><span>{snapshot?.score.outcome ?? 'CONNECTING'}</span></div>
	</section>
	<Explorer {snapshot} bind:hostState bind:hostType onscope={router=>{agent=router===null?'':String(snapshot?.agents.find(a=>a.router_id===router)?.id??'');}}>
		{#snippet children()}
		{#if !collapsed}<PaneResize label="Resize telemetry" axis="y" reverse value={Math.min(dockHeight,Math.max(120,viewportHeight-500))} min={120} max={Math.max(120,viewportHeight-500)} onresize={resizeDock} onreset={()=>resizeDock(260)}/>{/if}
		<section class="telemetry-dock" class:collapsed style:height={collapsed?undefined:`${Math.min(dockHeight,Math.max(120,viewportHeight-500))}px`} aria-label="Vanguard telemetry"><div class="dock-bar"><div class="dock-tabs" role="tablist" aria-label="Telemetry views">{#each tabs as tab}<button role="tab" aria-selected={dock === tab} onclick={() => {dock=tab;collapsed=false;}}>{tab}</button>{/each}</div><label class="dock-agent"><span class="sr-only">Filter Vanguard agents</span><select aria-label="Filter Vanguard agents" bind:value={agent}><option value="">All Vanguards</option>{#each snapshot?.agents ?? [] as a}<option value={String(a.id)}>A{a.id} · R{a.router_id}</option>{/each}</select></label><button class="icon-button" aria-label={collapsed ? 'Expand telemetry' : 'Collapse telemetry'} aria-expanded={!collapsed} onclick={() => collapsed = !collapsed}>{#if collapsed}<ChevronUp size={16}/>{:else}<ChevronDown size={16}/>{/if}</button></div>
		{#if !collapsed}<div class="dock-content" role="tabpanel" aria-label={dock}>{#if dock === 'Statistics'}<Stats {snapshot} {agent} />{:else if dock === 'Jobs'}<Jobs {snapshot} {agent} />{:else if dock==='Activity'}<div class="activity-events">{#each (snapshot?.events??[]).filter(e=>!agent||String(e.agent_id)===agent).slice(-60).toReversed() as event (event.seq)}<details class="activity-event"><summary><span>T{event.tick}</span><span>{event.agent_id ? `A${event.agent_id}` : event.source}</span><b>{event.kind.replaceAll('_',' ')}</b></summary><JsonTree value={event.data}/></details>{:else}<p class="empty">No activity yet. Start or step the simulation.</p>{/each}</div>{:else}<Vanguard {snapshot} tab={dock === 'Pub / Sub' ? 'PubSub' : dock} filter={agent} hideTabs />{/if}</div>{/if}
		</section>
		{/snippet}
	</Explorer>
	<footer class="status-bar"><span><span class="live-dot"></span><span role="status">{runtime.status}</span> · {controlState} · {snapshot?.run_id ?? 'Connecting'}</span><span>{snapshot?.agents.length??0} immune routers / {hosts.length} hosts · {snapshot?.history.evicted??0} events evicted</span></footer>
</div>
<Modal bind:open={settings} title="Match rules"><p>One agent, private router VMs. Score: 600 service + 250 security + 150 recovery. Verified full recovery ends the match immediately; recovery points are provisional until the match ends. Early victory credits unused deadline ticks at the final service and clean-host fractions.</p><JsonTree value={snapshot?.scenario ?? {}} /><button onclick={() => runtime.send('GetSnapshot')}>Refresh snapshot</button></Modal>
