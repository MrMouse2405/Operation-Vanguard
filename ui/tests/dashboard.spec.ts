import { expect, test } from '@playwright/test';
import { mkdir } from 'node:fs/promises';

test.beforeEach(async ({ request }) => {
	await request.post('/api/command', { data: { schema_version: 1, request_id: crypto.randomUUID(), command: 'Restart' } });
});

test('live controls and real barrier stepping retain state and reread on restart', async ({ page, request }) => {
	const errors: string[] = []; page.on('pageerror', e => errors.push(e.message));
	await page.goto('/'); await expect(page.getByRole('status')).toHaveText('Live');
	await page.getByRole('button', { name: /Start/ }).click();
	await expect(page.getByTestId('simulation-tick')).not.toHaveText('0');
	await page.getByRole('button', { name: /Pause/ }).click();
	await expect(page.getByRole('button', { name: /Resume/ })).toBeEnabled();
	const tick = Number(await page.getByTestId('simulation-tick').innerText());
	await page.waitForTimeout(250); await expect(page.getByTestId('simulation-tick')).toHaveText(String(tick));
	await page.getByRole('button', { name: '+1 tick', exact: true }).click();
	await expect(page.getByTestId('simulation-tick')).toHaveText(String(tick+1));
	await page.getByRole('spinbutton', { name: 'Number of ticks to advance' }).fill('7');
	await page.getByRole('button', { name: /Forward/ }).click();
	await expect(page.getByTestId('simulation-tick')).toHaveText(String(tick+8));
	const s = await request.get('/api/snapshot').then(r => r.json());
	expect(s.control).toBe('PAUSED');expect(s.enemy_updates).toBe(tick+8);
	expect(s.jobs.some((j:any) => j.action==='SCAN' && j.state==='COMPLETED')).toBe(true);
	expect(s.agents).toHaveLength(4);
	expect(s.score.enabled).toBe(true);expect(s.score.horizon).toBe(1000);
	expect(s.score.total).toBeGreaterThan(0);expect(s.score.services).toHaveLength(32);
	await page.getByRole('button', { name: /Restart/ }).click();await expect(page.getByTestId('simulation-tick')).toHaveText('0');
	await expect(page.getByRole('button', { name: /Start/ })).toBeEnabled();expect(errors).toEqual([]);
});

test('map, inspector and dock expose real logs, messages, topics, jobs and stats', async ({ page }) => {
	await page.goto('/');await expect(page.getByRole('status')).toHaveText('Live');
	await page.getByRole('button', { name: /Forward/ }).click();await expect(page.getByTestId('simulation-tick')).toHaveText('10');
	await page.getByRole('textbox', { name: 'Search all hosts' }).fill('1001');
	await page.locator('.host-row').first().click();
	await expect(page.locator('.selected-node')).toContainText('1001');
	await expect(page.getByRole('button', { name: 'Fit topology' })).toBeEnabled();
	await expect(page.locator('.map-bar')).toContainText('CONNECTIONS');
	await page.getByRole('tab', { name: /^Output/ }).click();
	await page.getByRole('combobox', { name: 'Filter Vanguard agents' }).selectOption('101');
	await expect(page.locator('.event-source').first()).toContainText('[101:');
	await expect(page.locator('.event-source').first()).toHaveAttribute('style', /color:/);
	const nested = page.locator('.event-row details details').first();await nested.locator(':scope > summary').click();await expect(nested).toHaveAttribute('open','');
	await page.getByRole('searchbox', { name: 'Search telemetry' }).fill('no-such-output');await expect(page.getByText('No output events recorded for this filter.')).toBeVisible();
	await page.getByRole('searchbox', { name: 'Search telemetry' }).clear();
	await page.getByRole('tab', { name: /^Messages/ }).click();await expect(page.locator('.message-route').first()).toBeVisible();
	await page.getByRole('tab', { name: /^Pub \/ Sub/ }).click();const topic=page.locator('.topic-button').first();await topic.click();
	await expect(page.getByRole('dialog')).toContainText('threats');await page.keyboard.press('Escape');await expect(topic).toBeFocused();
	await page.getByRole('tab', { name: /^Jobs/ }).click();await page.getByRole('button', { name: /Inspect job/ }).first().click();await expect(page.getByRole('dialog')).toBeVisible();await page.keyboard.press('Escape');
	await page.getByRole('tab', { name: /^Statistics/ }).click();await expect(page.locator('.stats-table')).toBeVisible();
	await page.getByRole('button', { name: 'Discovered', exact: true }).click();await page.getByRole('combobox', { name: 'Discovered Vanguard filter' }).selectOption('101');
	await expect(page.locator('.observation').first()).toBeVisible();await expect(page.getByText('Recorded observations only.', { exact:false })).toBeVisible();
	await page.getByRole('button', { name: 'True state', exact: true }).click();await page.getByRole('tab', { name: /^Output/ }).click();
	await mkdir('../.impeccable/review',{recursive:true});await page.screenshot({path:'../.impeccable/review/command-workspace-live.png',fullPage:true});
	await page.getByRole('button', { name: 'Collapse telemetry' }).click();await expect(page.locator('.dock-content')).not.toBeVisible();await page.getByRole('button', { name: 'Expand telemetry' }).click();await expect(page.locator('.dock-content')).toBeVisible();
});

test('socket reconnect, sequence gap and stale-run rejection preserve the projection', async ({ page, request }) => {
	await page.addInitScript(() => { const Native=window.WebSocket;(window as any).__sockets=[];window.WebSocket=class extends Native{constructor(url:string|URL,protocols?:string|string[]){super(url,protocols);(window as any).__sockets.push(this);}}; });
	await page.goto('/');await expect(page.getByRole('status')).toHaveText('Live');
	await page.getByRole('button',{name:'+1 tick',exact:true}).click();await expect(page.getByTestId('simulation-tick')).toHaveText('1');
	const s=await request.get('/api/snapshot').then(r=>r.json());
	await page.evaluate(()=>{for(const ws of (window as any).__sockets)ws.close();});
	await expect.poll(()=>page.evaluate(()=>(window as any).__sockets.filter((s:WebSocket)=>s.readyState===WebSocket.OPEN).length)).toBeGreaterThan(0);
	await expect(page.getByRole('status')).toHaveText('Live');await expect(page.getByTestId('simulation-tick')).toHaveText('1');
	await page.evaluate(s=>{(window as any).__sockets.at(-1).dispatchEvent(new MessageEvent('message',{data:JSON.stringify({schema_version:1,type:'update',run_id:s.run_id,seq:s.seq+50,base_seq:s.seq+49,delta:{tick:999,seq:s.seq+50}})}));},s);
	await expect(page.getByTestId('simulation-tick')).toHaveText('1');
	await page.getByRole('button',{name:/Restart/}).click();await expect(page.getByTestId('simulation-tick')).toHaveText('0');
	await page.evaluate(s=>{(window as any).__sockets.at(-1).dispatchEvent(new MessageEvent('message',{data:JSON.stringify({schema_version:1,type:'update',run_id:s.run_id,seq:s.seq+100,snapshot:{...s,tick:999}})}));},s);
	await expect(page.getByTestId('simulation-tick')).toHaveText('0');
});

test('prototype layout fits desktop and narrow widths; component route is removed', async ({ page, request }) => {
	await page.goto('/');await expect(page.getByRole('status')).toHaveText('Live');
	for(const viewport of [{width:1440,height:960},{width:390,height:844}]){
		await page.setViewportSize(viewport);await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
		await page.screenshot({path:`../.impeccable/review/command-workspace-${viewport.width}.png`,fullPage:true});
	}
	await page.getByRole('tab',{name:'Statistics',exact:true}).click();
	await expect(page.getByRole('combobox',{name:'Filter Vanguard agents'})).toBeVisible();
	await page.getByRole('combobox',{name:'Filter Vanguard agents'}).selectOption('101');
	await page.screenshot({path:'../.impeccable/review/explorer-mobile-dock.png',fullPage:true});
	expect((await request.get('/components')).status()).toBe(404);
});

test('explorer search, LAN drill-down, server shortcuts and keyboard navigation use live hosts', async ({page})=>{
	await page.goto('/');await expect(page.getByRole('status')).toHaveText('Live');
	await expect(page.locator('.lan-tile')).toHaveCount(4);
	await page.locator('.lan-tile').first().click();await expect(page.getByRole('button',{name:'Fit topology'})).toBeVisible();
	await page.keyboard.press('Escape');await expect(page.locator('.lan-tile')).toHaveCount(4);
	await page.getByRole('button',{name:'All servers',exact:true}).click();await expect(page.locator('.lan-table tbody tr')).toHaveCount(2);
	await page.locator('.lan-table tbody button').first().click();
	await expect(page.locator('.node-table')).toContainText('1017');
	await page.locator('.node-table tbody button').first().click();await expect(page.locator('.selected-node')).toContainText('Server');
	await page.getByRole('button',{name:'Close node inspector'}).click();await expect(page.locator('.score-brief')).toBeVisible();
	await expect(page.getByRole('button',{name:'Close node inspector'})).toHaveCount(0);
	await page.getByRole('button',{name:'Infected servers',exact:true}).click();await expect(page.getByText('No matching LANs. Try an ID or reset filters.')).toBeVisible();
	await page.getByRole('button',{name:'Reset filters',exact:true}).click();await page.keyboard.press('Control+k');await expect(page.getByRole('textbox',{name:'Search all hosts'})).toBeFocused();
	await page.getByRole('textbox',{name:'Search all hosts'}).fill('1001');await expect(page.locator('.host-row')).toHaveCount(1);
	await page.locator('.host-row').click();await expect(page.locator('.selected-node')).toContainText('1001');
});

test('large snapshot renders bounded overview, table and LAN detail',async({page})=>{
	const nodes:any[]=[];const agents:any[]=[];
	for(let r=1;r<=32;r++){
		nodes.push({id:r,type:'ROUTER',router_id:r,lan_id:r,power:'ONLINE',compromised:false,isolated:false,policy:'DEFAULT_ALLOW',exceptions:[]});agents.push({id:100+r,router_id:r,known:[r],observations:[]});
		for(let i=0;i<64;i++)nodes.push({id:1001+(r-1)*64+i,type:i%16===0?'SERVER':'WORKSTATION',router_id:r,lan_id:r,power:'ONLINE',compromised:false,isolated:false,policy:'DEFAULT_ALLOW',exceptions:[]});
	}
	const snapshot={schema_version:1,run_id:'large-fixture',seq:1,tick:0,control:'READY',scenario:{tick_limit:1000},nodes,agents,links:[],sessions:[],jobs:[],events:[],topics:[],history:{first_seq:1,evicted:0},score:{enabled:false,outcome:'PRACTICE'}};
	let push=()=>{};
	await page.routeWebSocket('**/api/ws',ws=>{push=()=>ws.send(JSON.stringify({schema_version:1,type:'snapshot',run_id:snapshot.run_id,seq:snapshot.seq,tick:0,snapshot}));push();});
	await page.goto('/');await expect(page.locator('.lan-tile')).toHaveCount(32);
	await page.getByRole('button',{name:'Table',exact:true}).click();await expect(page.locator('.lan-table tbody tr')).toHaveCount(32);
	await page.screenshot({path:'../.impeccable/review/explorer-lan-table.png',fullPage:true});
	await page.locator('.lan-table tbody button').first().click();
	await expect(page.locator('.node-table tbody tr')).toHaveCount(64);
	await expect(page.getByRole('button',{name:'Table',exact:true})).toHaveClass(/active/);
	await expect(page.getByRole('navigation',{name:'Network location'})).toContainText('LAN 1');
	await page.screenshot({path:'../.impeccable/review/explorer-lan-node-table.png',fullPage:true});
	await page.getByRole('button',{name:'Topology',exact:true}).click();await expect(page.locator('.scope-bar')).toContainText('64 hosts');
	await expect(page.locator('.cytoscape canvas').first()).toBeVisible();
	await page.getByRole('button',{name:'Fit topology'}).click();
	await page.screenshot({path:'../.impeccable/review/explorer-large.png',fullPage:true});
	// A selected host on a later inventory page must also be on that LAN's map page.
	snapshot.nodes=[nodes[0],...Array.from({length:160},(_,i)=>({id:1001+i,type:'WORKSTATION',router_id:1,lan_id:1,power:'ONLINE',compromised:false,isolated:false,policy:'DEFAULT_ALLOW',exceptions:[]}))];snapshot.agents=[agents[0]];snapshot.seq=2;push();
	await page.getByRole('button',{name:'Table',exact:true}).click();
	await page.getByRole('button',{name:'Next',exact:true}).click();
	await page.locator('.node-table tbody button').first().click();
	await expect(page.locator('.selected-node')).toContainText('1081');
	await page.getByRole('button',{name:'Topology',exact:true}).click();
	await expect(page.locator('.inventory-pager')).toContainText('81–160');
	await expect(page.locator('.cytoscape canvas').first()).toBeVisible();
	await page.screenshot({path:'../.impeccable/review/explorer-selected-page.png',fullPage:true});
});
