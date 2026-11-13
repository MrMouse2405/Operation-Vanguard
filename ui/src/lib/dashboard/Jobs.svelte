<script lang="ts">
	import type { Snapshot, Job } from './types';
	import { progress, sourceColor } from './types';
	import Modal from './Modal.svelte';
	import JsonTree from './JsonTree.svelte';
	import { loadDetails } from './details';
	let { snapshot, agent = '', compact = false, cards=false }: { snapshot: Snapshot | null; agent?: string; compact?: boolean;cards?:boolean } = $props();
	let stateFilter = $state('ALL');
	let selectedId = $state<string | number | null>(null);
	let open = $state(false);
	let page = $state(0);
	let runId = $derived(snapshot?.run_id);
	let allJobs = $derived((snapshot?.jobs??[]).filter(j=>!agent||String(j.agent_id)===agent));
	let counts = $derived({running:allJobs.filter(j=>j.state==='RUNNING').length,completed:allJobs.filter(j=>j.state==='COMPLETED').length,failed:allJobs.filter(j=>j.state==='FAILED').length});
	let jobs = $derived((snapshot?.jobs ?? []).filter(j => (!agent || String(j.agent_id) === agent) && (compact ? j.state === 'RUNNING' : stateFilter === 'ALL' || j.state === stateFilter)).toSorted((a, b) => Number(b.state === 'RUNNING') - Number(a.state === 'RUNNING') || b.started_at - a.started_at || String(a.id).localeCompare(String(b.id))));
	let selected = $derived(snapshot?.jobs.find(j => j.id === selectedId));
	let isCompact=$derived(!!snapshot?.compact);
	let detail=$state.raw<any>(null);let detailError=$state('');let loading=$state(false);
	let result=$derived(isCompact?detail?.result:selected?.result);
	$effect(()=>{
		const id=selected?.id,state=selected?.state,run=runId;state;detail=null;detailError='';loading=false;
		if(!open||!isCompact||id==null||!run)return;
		const controller=new AbortController();loading=true;
		void loadDetails(run,'job',Number(id),controller.signal).then(value=>{detail=value.items[0];loading=false;}).catch(error=>{if(!controller.signal.aborted){detailError=error.message;loading=false;}});
		return()=>controller.abort();
	});
	let related = $derived(open ? snapshot?.events.filter(e => selected && (String(e.data?.task_id ?? e.data?.job_id ?? '') === String(selected.id) || String(e.data?.id ?? '') === String(selected.id))) ?? [] : []);
	$effect(() => { agent; stateFilter; runId; page = 0; });
	$effect(() => { if (page * 100 >= jobs.length) page = 0; });
	function inspect(job: Job) { selectedId = job.id; open = true; }
</script>
{#if !cards}<div class="jobs-tools">{#if compact}<span>RUNNING engine tasks</span>{:else}<div class="job-state-buttons" role="group" aria-label="Task status filters"><button aria-pressed={stateFilter==='ALL'} onclick={()=>stateFilter='ALL'}>All <b>{allJobs.length}</b></button><button aria-pressed={stateFilter==='RUNNING'} onclick={()=>stateFilter='RUNNING'}><i class="green-dot"></i> Running <b>{counts.running}</b></button><button aria-pressed={stateFilter==='COMPLETED'} onclick={()=>stateFilter='COMPLETED'}>Completed <b>{counts.completed}</b></button><button aria-pressed={stateFilter==='FAILED'} onclick={()=>stateFilter='FAILED'}>Failed <b class:red={counts.failed>0}>{counts.failed}</b></button></div>{/if}<span class="muted">{compact?'Running tasks':'Engine tasks · running first · select a row for results'}</span></div>{/if}
{#if cards}<div class="router-jobs" aria-label="Router running jobs">{#each jobs as job (job.id)}<button class="router-job" onclick={()=>inspect(job)} aria-label={`Inspect job ${job.id}`}><span><strong>{job.action.replaceAll('_',' ').toLowerCase()}</strong><small>#{job.id}</small></span><span>{job.target==null?'Local router':`Host ${job.target}`} · {Math.max(0,job.duration-((snapshot?.tick??0)-job.started_at))} ticks left</span><progress value={progress(job,snapshot?.tick??0)} max="100" aria-label={`Job ${job.id} progress`}></progress></button>{:else}<p class="empty">No running jobs. This router has all five task slots available.</p>{/each}</div>{:else}
<div class="table-scroll" class:compact>
	<table class="jobs-table"><thead><tr><th>Task / action</th><th>Vanguard / target</th><th>State / progress</th><th>Fuel</th></tr></thead><tbody>
		{#each jobs.slice(page * 100, (page + 1) * 100) as job (`${snapshot?.run_id}-${job.id}`)}
			<tr><td><button class="job-link" onclick={() => inspect(job)} aria-label={`Inspect job ${job.id}`}><span class="muted">#{job.id}</span> {job.action}</button><span class="cell-note" style:color={sourceColor(`${job.agent_id}:${job.source ?? '<unknown>'}`)}>{job.source ?? '<unknown>'}</span></td>
			<td>A{job.agent_id} / R{job.router_id}<span class="cell-note">{job.target == null ? 'Own router' : `Node ${job.target}`}</span></td>
			<td><span class:failed={job.state === 'FAILED'}>{job.state}</span><progress value={progress(job, snapshot?.tick ?? 0)} max="100" aria-label={`Job ${job.id} progress`}></progress><span class="cell-note">{job.state === 'RUNNING' ? `${Math.max(0, (snapshot?.tick ?? 0) - job.started_at)} / ${job.duration} ticks` : `T${job.started_at} → T${job.finished_at ?? '—'}`}</span></td><td>{job.fuel_spent ?? '—'}</td></tr>
		{:else}<tr><td colspan="4" class="empty">No {stateFilter === 'ALL' ? '' : stateFilter.toLowerCase() + ' '}{compact ? 'tasks' : 'jobs'} in this view.</td></tr>{/each}
	</tbody></table>
</div>
{/if}
{#if jobs.length > 100}<div class="table-pagination" aria-label="Job pages"><button disabled={page === 0} onclick={() => page--}>Previous jobs</button><span>{page * 100 + 1}–{Math.min((page + 1) * 100, jobs.length)} / {jobs.length}</span><button disabled={(page + 1) * 100 >= jobs.length} onclick={() => page++}>Next jobs</button></div>{/if}
<Modal bind:open title={selected ? `Job #${selected.id} · ${selected.action}` : 'Job details'}>
	{#if selected}<dl class="data-list"><dt>Vanguard / execution router</dt><dd>A{selected.agent_id} / R{selected.router_id}</dd><dt>State</dt><dd>{selected.state}</dd><dt>Admitted</dt><dd>Tick {selected.started_at}</dd><dt>Duration</dt><dd>{selected.duration} ticks</dd><dt>Finished</dt><dd>{selected.finished_at ?? 'Still running'}</dd><dt>Fuel charged on admission</dt><dd>{selected.fuel_spent ?? 'Unavailable'}</dd></dl>
		<h3>Descriptor</h3><JsonTree value={selected.descriptor ?? { action: selected.action, ...(selected.target == null ? {} : { target: selected.target }), ...(selected.peer == null ? {} : { peer: selected.peer }) }} />
		<h3>Result / failure</h3>{#if selected.failure}<p class="failed">{selected.failure}</p>{:else if loading}<p class="muted">Loading task result…</p>{:else if detailError}<p role="alert">{detailError}</p>{:else if result != null}<JsonTree value={result} />{:else}<p class="muted">{selected.state === 'RUNNING' ? 'Result available at the completion barrier.' : 'No result recorded.'}</p>{/if}
		<h3>Related events · retained window</h3>{#each related as event (event.seq)}<div class="event-row"><span class="muted">T{event.tick} · {event.kind}</span><JsonTree value={event.data} /></div>{:else}<p class="muted">No related events in the retained window.</p>{/each}
	{:else}<p class="muted">This job is no longer in the current run.</p>{/if}
</Modal>

<style>
	.router-jobs{display:flex;flex-direction:column;gap:8px}.router-job{display:flex;flex-direction:column;gap:8px;text-align:left;width:100%;padding:11px!important}.router-job>span{display:flex;justify-content:space-between;gap:8px;font:10px monospace;color:var(--muted)}.router-job strong{text-transform:capitalize;color:var(--text);font:600 12px system-ui}.router-job progress{width:100%;height:4px;accent-color:#90b8fc}
	.job-state-buttons{display:flex;flex-wrap:wrap;gap:5px}.job-state-buttons button{display:flex;align-items:center;gap:6px;font-size:10px;padding:6px 8px}.job-state-buttons button[aria-pressed=true]{background:var(--selected,#20333e);border-color:#90b8fc;color:#dcecff}.job-state-buttons b{font:10px monospace;color:var(--muted)}
</style>
