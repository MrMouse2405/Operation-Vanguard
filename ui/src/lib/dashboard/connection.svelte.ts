import { normalizeSnapshot, type Snapshot, type Envelope, type Command } from './types';

export function createConnection() {
	let snapshot = $state<Snapshot | null>(null);
	let status = $state('Connecting');
	let error = $state('');
	let pending = $state<Command | null>(null);
	let socket: WebSocket | null = null;
	let stopped = false;
	let reconnect: ReturnType<typeof setTimeout>;
	let timeout: ReturnType<typeof setTimeout>;
	let poll: ReturnType<typeof setInterval>;
	let attempt = 0;
	let request = '';
	let fallback = false;
	let syncing = false;
	const retired = new Set<string>();
	function receive(envelope: Envelope) {
		if (envelope.schema_version !== 1) { error = 'Unsupported runtime schema. Reload after updating Vanguard.'; return; }
		if (envelope.request_id === request && (envelope.type === 'ack' || envelope.type === 'error')) { pending = null; clearTimeout(timeout); }
		if (envelope.type === 'error') { error = typeof envelope.error === 'string' ? envelope.error : JSON.stringify(envelope.error ?? 'Command failed'); return; }
		if (!envelope.snapshot && !envelope.delta) return;
		if (retired.has(envelope.run_id) && envelope.type !== 'snapshot') return;
		if (snapshot && envelope.run_id !== snapshot.run_id && envelope.type !== 'snapshot') { if (!syncing) { syncing = true; void send('GetSnapshot'); } return; }
		if (snapshot && envelope.run_id === snapshot.run_id) {
			if (envelope.seq < snapshot.seq || (envelope.seq === snapshot.seq && envelope.type !== 'snapshot')) return;
			if (envelope.type === 'update' && envelope.seq > snapshot.seq + 1) { if (!syncing) { syncing = true; void send('GetSnapshot'); } return; }
		}
		try {
			if (envelope.delta && (!snapshot || envelope.base_seq !== snapshot.seq)) { if (!syncing) { syncing = true; void send('GetSnapshot'); } return; }
			const next = normalizeSnapshot(envelope.snapshot ?? { ...snapshot, ...envelope.delta });
			if (snapshot && snapshot.run_id !== next.run_id) retired.add(snapshot.run_id);
			retired.delete(next.run_id);
			snapshot = next; syncing = false; status = fallback ? 'HTTP connected' : 'Live';
		} catch (cause) { error = String(cause); }
	}
	async function getSnapshot() {
		try { const response = await fetch('/api/snapshot'); if (!response.ok) throw new Error(`Snapshot unavailable (${response.status})`); const data = await response.json(); receive(data.type ? data : { schema_version: 1, type: 'snapshot', run_id: data.run_id, seq: data.seq, tick: data.tick, snapshot: data }); }
		catch (cause) { status = 'Disconnected'; error = String(cause); }
	}
	async function send(command: Command, ticks?: number, seconds_per_tick?: number) {
		if (command !== 'GetSnapshot' && pending) return;
		const request_id = globalThis.crypto?.randomUUID?.() ?? `ui-${Date.now()}-${Math.random()}`;
		if (command !== 'GetSnapshot') { request = request_id; pending = command; error = ''; clearTimeout(timeout); timeout = setTimeout(() => { pending = null; error = `${command} was not acknowledged. Request a snapshot and try again.`; }, 10000); }
		const body = { schema_version: 1, request_id, command, ...(ticks === undefined ? {} : { ticks }), ...(seconds_per_tick === undefined ? {} : { seconds_per_tick }) };
		try {
			if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(body));
			else if (fallback) {
				if (command === 'GetSnapshot') { await getSnapshot(); return; }
				const response = await fetch('/api/command', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
				const data = await response.json(); receive(data); if (!response.ok) throw new Error(data.error ?? `Command failed (${response.status})`); await getSnapshot();
			} else throw new Error('Runtime disconnected. Wait for reconnection and try again.');
		} catch (cause) { if (command !== 'GetSnapshot') { pending = null; clearTimeout(timeout); } error = String(cause); syncing = false; }
	}
	function connect() {
		if (stopped) return;
		if (!('WebSocket' in window)) { fallback = true; void getSnapshot(); poll = setInterval(getSnapshot, 1500); return; }
		status = attempt ? 'Reconnecting' : 'Connecting';
		socket = new WebSocket(`${location.protocol === 'https:' ? 'wss:' : 'ws:'}//${location.host}/api/ws`);
		socket.onopen = () => { attempt = 0; error = ''; status = 'Synchronizing'; syncing = false; };
		socket.onmessage = event => { try { receive(JSON.parse(event.data)); } catch { error = 'Could not read runtime update. Request a fresh snapshot.'; } };
		socket.onclose = () => { if (stopped) return; status = 'Reconnecting'; pending = null; clearTimeout(timeout); reconnect = setTimeout(connect, Math.min(15000, 500 * 2 ** attempt++)); };
		socket.onerror = () => { error = 'Connection interrupted; reconnecting automatically.'; };
	}
	function destroy() { stopped = true; clearTimeout(reconnect); clearTimeout(timeout); clearInterval(poll); socket?.close(); }
	return { get snapshot() { return snapshot; }, get status() { return status; }, get error() { return error; }, get pending() { return pending; }, send, connect, destroy };
}
