<script lang="ts">
	import { tick } from 'svelte';
	import type { Snapshot } from './types';
	import { sourceColor } from './types';
	import JsonTree from './JsonTree.svelte';
	import Jobs from './Jobs.svelte';
	import Modal from './Modal.svelte';
	let { snapshot, tab = 'Output', filter, hideTabs = false }: { snapshot: Snapshot | null; tab?: string; filter?: string; hideTabs?: boolean } = $props();
	const tabs = ['Output', 'Messages', 'PubSub', 'Tasks'];
	let filters = $state<Record<string, string>>({ Output: '', Messages: '', PubSub: '', Tasks: '' });
	let topicName = $state('');
	let topicOpen = $state(false);
	let source = $state('');
	let limit = $state(100);
	let search = $state('');
	let severity = $state('all');
	let locked = $state(false);
	let lockedSequence = $state(0);
	let scroller: HTMLDivElement;
	let agent = $derived(filter ?? filters[tab]);
	let events = $derived((snapshot?.events ?? []).filter(e => {
		const kind = e.kind.toLowerCase();
		if (locked && e.seq > lockedSequence) return false;
		if (search && !JSON.stringify(e).toLowerCase().includes(search.toLowerCase())) return false;
		if (tab === 'Output' && severity !== 'all' && (e.data.severity ?? (kind.includes('error') ? 'error' : 'info')) !== severity) return false;
		if (tab === 'Output') return ['log', 'output', 'error', 'runtime_error', 'handler_error'].includes(kind) && (!agent || String(e.agent_id) === agent) && (!source || e.source === source);
		if (tab === 'Messages') return (kind.includes('message') || kind === 'tx' || kind === 'rx') && (!agent || String(e.data.sender ?? e.agent_id) === agent || String(e.data.recipient) === agent);
		return (kind.includes('pub') || kind.includes('sub') || kind === 'topic') && (!agent || String(e.agent_id ?? e.data.sender) === agent || String(e.data.recipient) === agent || (e.data.recipients ?? []).map(String).includes(agent));
	}));
	let visible = $derived(events.slice(-limit));
	let sources = $derived([...new Set((snapshot?.events ?? []).filter(e => ['log', 'output', 'error', 'runtime_error', 'handler_error'].includes(e.kind)).map(e => e.source))].toSorted());
	let topics = $derived((snapshot?.topics ?? []).filter(t => !agent || (t.subscribers ?? []).map(String).includes(agent) || (snapshot?.events ?? []).some(e => e.data.topic === t.topic && (String(e.agent_id ?? e.data.sender) === agent || String(e.data.recipient) === agent || (e.data.recipients ?? []).map(String).includes(agent)))));
	let topic = $derived(snapshot?.topics.find(t => t.topic === topicName));
	let publications = $derived(events.filter(e => e.data.topic === topicName));
	$effect(() => { const latest = visible.at(-1)?.seq; if (!locked && latest != null) void tick().then(() => { if (!locked && scroller) scroller.scrollTop = scroller.scrollHeight; }); });
	function tabKeys(event: KeyboardEvent) {
		if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
		event.preventDefault(); const index = tabs.indexOf(tab); tab = event.key === 'Home' ? tabs[0] : event.key === 'End' ? tabs.at(-1)! : tabs[(index + (event.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length];
		document.getElementById(`vanguard-tab-${tab}`)?.focus();
	}
</script>
{#if !hideTabs}<div class="tabs" role="tablist" aria-label="Vanguard views" tabindex="-1" onkeydown={tabKeys}>
	{#each tabs as name}<button id={`vanguard-tab-${name}`} role="tab" aria-selected={tab === name} aria-controls="vanguard-view" tabindex={tab === name ? 0 : -1} onclick={() => tab = name}>{name}</button>{/each}
</div>
<div class="view-tools"><label>Vanguard <select bind:value={filters[tab]} aria-label={`${tab} Vanguard filter`}><option value="">All Vanguards</option>{#each snapshot?.agents ?? [] as a (a.id)}<option value={String(a.id)}>A{a.id} · router {a.router_id}</option>{/each}</select></label>{#if tab === 'Output'}<label>Source <select bind:value={source} aria-label="Output source filter"><option value="">All sources</option>{#each sources as name}<option value={name}>{name}</option>{/each}</select></label>{/if}</div>
{/if}
{#if tab !== 'Tasks'}<div class="view-tools log-tools"><label>Search <input type="search" bind:value={search} aria-label="Search telemetry" placeholder="Search retained events" /></label>{#if tab === 'Output'}<label>Severity <select bind:value={severity} aria-label="Output severity filter"><option value="all">All</option><option value="info">Info</option><option value="error">Error</option></select></label>{#if hideTabs}<label>Source <select bind:value={source} aria-label="Output source filter"><option value="">All sources</option>{#each sources as name}<option value={name}>{name}</option>{/each}</select></label>{/if}{/if}<button class="dash-button" aria-pressed={locked} onclick={() => { locked = !locked; lockedSequence = snapshot?.events.at(-1)?.seq ?? 0; }}>{locked ? 'Resume live log' : 'Lock log scroll'}</button></div>{/if}
<div class="vanguard-view" bind:this={scroller} id="vanguard-view" role="tabpanel" aria-label={tab} tabindex="0">
	{#if tab === 'Tasks'}<Jobs {snapshot} {agent} compact />
	{:else}
		<div class="retention muted">Retained events {snapshot?.history.first_seq ?? '—'} onward · {snapshot?.history.evicted ?? 0} evicted{#if events.length > limit} · {events.length - limit} earlier matching events hidden{/if}</div>
		{#if tab === 'PubSub'}
			<div class="topic-list">{#each topics as t (t.topic)}<button class="topic-button" onclick={() => { topicName = t.topic; topicOpen = true; }}><strong>{t.topic}</strong><span>{t.subscribers?.length ?? 0} subscribers · {t.publication_count ?? 0} publications</span><span class="muted">Last publication: {t.last_tick == null ? 'none' : `T${t.last_tick}`}</span></button>{:else}<p class="empty">No topic membership recorded for this filter.</p>{/each}</div>
		{/if}
		<div class="event-stream" aria-label={`${tab} events`}>
			{#each visible as event (`${snapshot?.run_id}-${event.seq}`)}
				<article class="event-row" class:event-error={event.kind.includes('error')}>
					<header><span class="muted">T{event.tick} <span class="event-seq">#{event.seq}</span></span><span class="event-source" style:color={sourceColor(`${event.agent_id ?? event.data.sender}:${event.source}`)}>[{event.agent_id ?? event.data.sender ?? '—'}:{event.source}]</span><span class="event-kind">{event.data.severity ?? event.kind}</span></header>
					{#if tab === 'Messages'}<p class="message-route">A{event.data.sender ?? event.agent_id ?? '?'} → A{event.data.recipient ?? '?'} <span class="muted">#{event.data.message_id ?? '—'} · {event.data.outcome ?? 'pending'} · sent T{event.data.sent_at ?? event.tick}{event.data.delivered_at == null ? '' : ` / delivered T${event.data.delivered_at}`}</span></p><JsonTree value={event.data.payload ?? event.data.message ?? event.data} />
					{:else if tab === 'Output' && Array.isArray(event.data.values)}{#each event.data.values as value, index}<JsonTree {value} name={String(index + 1)} />{/each}
					{:else}<JsonTree value={event.data} />{/if}
				</article>
			{:else}<p class="empty">{snapshot ? `No ${tab.toLowerCase()} events recorded for this filter.` : 'Waiting for the runtime snapshot.'}</p>{/each}
		</div>
		{#if events.length > limit}<button class="dash-button more-events" onclick={() => limit = Math.min(500, limit + 100)} disabled={limit >= 500}>{limit >= 500 ? 'Display bounded to latest 500 matches' : 'Show 100 earlier events'}</button>{/if}
	{/if}
</div>
<Modal bind:open={topicOpen} title={`Topic · ${topicName}`}>
	{#if topic}<dl class="data-list"><dt>Subscribers (committed at barrier)</dt><dd>{topic.subscribers?.map((id: number) => `A${id}`).join(', ') || 'None'}</dd><dt>Publications</dt><dd>{topic.publication_count ?? 0}</dd><dt>Last tick</dt><dd>{topic.last_tick ?? 'No publications'}</dd></dl>{/if}
	<h3>Topic events · retained window</h3>{#each publications as event (event.seq)}<article class="event-row"><header>T{event.tick} · A{event.agent_id ?? event.data.sender ?? '?'} · {event.kind}</header><JsonTree value={event.data} /></article>{:else}<p class="muted">No topic events retained.</p>{/each}
</Modal>
