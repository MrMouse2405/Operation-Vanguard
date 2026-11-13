export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export type RuntimeData = Record<string, any>;
export interface Node extends RuntimeData { id: number; type: string; router_id: number; power: string; compromised: boolean }
export interface Agent extends RuntimeData { id: number; router_id: number; known: number[]; observations: RuntimeData[] }
export interface Job extends RuntimeData { id: number | string; agent_id: number; router_id: number; action: string; state: string; started_at: number; duration: number }
export interface RuntimeEvent extends RuntimeData { seq: number; tick: number; kind: string; agent_id?: number; source: string; data: RuntimeData }
export interface Snapshot {
    debug?: { enabled: boolean; agents: Record<string, Record<string, any>> };
    score: RuntimeData;
	schema_version: number; run_id: string; seq: number; tick: number; control: string;
	seconds_per_tick: number;
	scenario: RuntimeData; nodes: Node[]; links: RuntimeData[]; sessions: RuntimeData[];
	jobs: Job[]; agents: Agent[]; events: RuntimeEvent[]; topics: RuntimeData[];
	enemy_updates: number; history: { first_seq: number; evicted: number };
}
export interface Envelope { schema_version: number; run_id: string; seq: number; tick: number; type: string; request_id?: string; snapshot?: unknown; delta?: Partial<Snapshot>; base_seq?: number; error?: unknown }
export type Command = 'Start' | 'Pause' | 'Resume' | 'Restart' | 'GetSnapshot' | 'StepTicks' | 'SetTickPacing';
const normalizedNodes = new WeakMap<any[], Node[]>();
const normalizedAgents = new WeakMap<any[], Agent[]>();
const normalizedEvents = new WeakMap<any[], RuntimeEvent[]>();
function cachedList<T>(values:any[], cache:WeakMap<any[],T[]>, normalize:(value:any,index:number)=>T):T[] {
	let result=cache.get(values);
	if(!result){result=values.map(normalize);cache.set(values,result);}
	return result;
}
export function normalizeSnapshot(value: any): Snapshot {
	if (!value || typeof value !== 'object' || !Array.isArray(value.nodes) || !value.run_id) throw new Error('Invalid runtime snapshot. Reconnect to request a fresh snapshot.');
	const list = (v: any): any[] => Array.isArray(v) ? v : [];
	return { ...value, schema_version: value.schema_version ?? 1, seq: Number(value.seq ?? 0), tick: Number(value.tick ?? 0),
		control: String(value.control ?? value.state ?? 'READY').toUpperCase(), scenario: value.scenario ?? {}, score: value.score ?? {enabled:false,outcome:'PRACTICE'},
		nodes: cachedList(list(value.nodes), normalizedNodes, n => ({ ...n, type: String(n.type ?? n.node_type ?? 'UNKNOWN').toUpperCase(), shield: n.shield ?? { policy: n.policy, exceptions: n.exceptions, isolated: n.isolated } })),
		links: list(value.links), sessions: list(value.sessions), jobs: list(value.jobs),
		agents: cachedList(list(value.agents), normalizedAgents, a => ({ ...a, known: list(a.known), observations: list(a.observations) })),
		events: cachedList(list(value.events), normalizedEvents, (e, index) => ({ ...e, seq: e.seq ?? e.sequence ?? index, kind: String(e.kind ?? 'unknown').toLowerCase(), source: String(e.source ?? `agent:${e.agent_id ?? 'runtime'}`), data: e.data ?? {} })),
		topics: list(value.topics), enemy_updates: value.enemy_updates ?? 0, history: { first_seq: value.history?.first_seq ?? 0, evicted: value.history?.evicted ?? 0 } };
}
export function sourceColor(source: string): string {
	const normalized = source.replaceAll('\\', '/').replace(/^@/, '').replace(/:\d+(:\d+)?$/, '').toLowerCase();
	let hash = 0; for (const char of normalized) hash = (hash * 31 + char.charCodeAt(0)) | 0;
	return ['var(--source-emerald)', 'var(--source-blue)', 'var(--source-violet)', 'var(--source-amber)', 'var(--source-teal)'][Math.abs(hash) % 5];
}
export function progress(job: Job, tick: number): number {
	const at = job.state === 'RUNNING' ? tick : (job.finished_at ?? job.started_at);
	return Math.max(0, Math.min(100, ((at - job.started_at) / Math.max(1, job.duration)) * 100));
}
