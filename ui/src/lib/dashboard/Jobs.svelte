<script lang="ts">
	import type { Snapshot, Job } from './types';
	import { progress, sourceColor } from './types';
	import Modal from './Modal.svelte';
	import JsonTree from './JsonTree.svelte';
	let { snapshot, agent = '', compact = false }: { snapshot: Snapshot | null; agent?: string; compact?: boolean } = $props();
	let stateFilter = $state('ALL');
	let selectedId = $state<string | number | null>(null);
	let open = $state(false);
	let jobs = $derived((snapshot?.jobs ?? []).filter(j => (!agent || String(j.agent_id) === agent) && (compact ? j.state === 'RUNNING' : stateFilter === 'ALL' || j.state === stateFilter)).toSorted((a, b) => Number(b.state === 'RUNNING') - Number(a.state === 'RUNNING') || b.started_at - a.started_at || String(a.id).localeCompare(String(b.id))));
	let selected = $derived(snapshot?.jobs.find(j => j.id === selectedId));
	let related = $derived(snapshot?.events.filter(e => selected && (String(e.data?.task_id ?? e.data?.job_id ?? '') === String(selected.id) || String(e.data?.id ?? '') === String(selected.id))) ?? []);
	function inspect(job: Job) { selectedId = job.id; open = true; }
</script>
<div class="jobs-tools">{#if compact}<span>RUNNING engine tasks</span>{:else}<label>State <select aria-label="Job state" bind:value={stateFilter}><option value="ALL">All states</option><option>RUNNING</option><option>COMPLETED</option><option>FAILED</option></select></label>{/if}<span class="muted">{jobs.length} {compact ? 'tasks' : 'jobs'}</span></div>
<div class="table-scroll" class:compact>
	<table class="jobs-table"><thead><tr><th>Task / action</th><th>Vanguard / target</th><th>State / progress</th><th>Fuel</th></tr></thead><tbody>
		{#each jobs as job (`${snapshot?.run_id}-${job.id}`)}
			<tr><td><button class="job-link" onclick={() => inspect(job)} aria-label={`Inspect job ${job.id}`}><span class="muted">#{job.id}</span> {job.action}</button><span class="cell-note" style:color={sourceColor(`${job.agent_id}:${job.source ?? '<unknown>'}`)}>{job.source ?? '<unknown>'}</span></td>
			<td>A{job.agent_id} / R{job.router_id}<span class="cell-note">{job.target == null ? 'Own router' : `Node ${job.target}`}</span></td>
			<td><span class:failed={job.state === 'FAILED'}>{job.state}</span><progress value={progress(job, snapshot?.tick ?? 0)} max="100" aria-label={`Job ${job.id} progress`}></progress><span class="cell-note">{job.state === 'RUNNING' ? `${Math.max(0, (snapshot?.tick ?? 0) - job.started_at)} / ${job.duration} ticks` : `T${job.started_at} → T${job.finished_at ?? '—'}`}</span></td><td>{job.fuel_spent ?? '—'}</td></tr>
		{:else}<tr><td colspan="4" class="empty">No {stateFilter === 'ALL' ? '' : stateFilter.toLowerCase() + ' '}{compact ? 'tasks' : 'jobs'} in this view.</td></tr>{/each}
	</tbody></table>
</div>
<Modal bind:open title={selected ? `Job #${selected.id} · ${selected.action}` : 'Job details'}>
	{#if selected}<dl class="data-list"><dt>Vanguard / execution router</dt><dd>A{selected.agent_id} / R{selected.router_id}</dd><dt>State</dt><dd>{selected.state}</dd><dt>Admitted</dt><dd>Tick {selected.started_at}</dd><dt>Duration</dt><dd>{selected.duration} ticks</dd><dt>Finished</dt><dd>{selected.finished_at ?? 'Still running'}</dd><dt>Fuel charged on admission</dt><dd>{selected.fuel_spent ?? 'Unavailable'}</dd></dl>
		<h3>Descriptor</h3><JsonTree value={selected.descriptor ?? { action: selected.action, ...(selected.target == null ? {} : { target: selected.target }), ...(selected.peer == null ? {} : { peer: selected.peer }) }} />
		<h3>Result / failure</h3>{#if selected.failure}<p class="failed">{selected.failure}</p>{:else if selected.result != null}<JsonTree value={selected.result} />{:else}<p class="muted">{selected.state === 'RUNNING' ? 'Result available at the completion barrier.' : 'No result recorded.'}</p>{/if}
		<h3>Related events · retained window</h3>{#each related as event (event.seq)}<div class="event-row"><span class="muted">T{event.tick} · {event.kind}</span><JsonTree value={event.data} /></div>{:else}<p class="muted">No related events in the retained window.</p>{/each}
	{:else}<p class="muted">This job is no longer in the current run.</p>{/if}
</Modal>
