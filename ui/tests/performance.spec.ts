import { expect, test } from '@playwright/test';

test('10,000-node live dashboard keeps hidden details, tables and update bursts bounded', async ({ page }, testInfo) => {
	const errors:string[]=[];page.on('pageerror',error=>errors.push(error.message));
	const nodes:any[]=[];const agents:any[]=[];
	for(let router=1;router<=40;router++){
		nodes.push({id:router,type:'ROUTER',router_id:router,lan_id:router,power:'ONLINE',compromised:false,exceptions:[]});
		agents.push({id:100+router,router_id:router,known:[router],observations:[]});
		for(let host=0;host<249;host++)nodes.push({id:10001+(router-1)*249+host,type:host<2?'SERVER':'WORKSTATION',router_id:router,lan_id:router,power:'ONLINE',compromised:host===2,exceptions:[]});
	}
	const services=Array.from({length:29640},(_,index)=>({client:10003+index%247,server:10001,weight:2,working:true}));
	const jobs=Array.from({length:10000},(_,index)=>({id:index+1,agent_id:101+index%40,router_id:1+index%40,action:'HOST_SCAN',state:'COMPLETED',started_at:index,finished_at:index+3,duration:3,result:{data:{assessment:'CLEAN'}}}));
	const sessions=Array.from({length:120},(_,index)=>({id:index+1,source:10001+index,target:19960,opened_at:0,route:[1,40]}));
	const snapshot={schema_version:1,run_id:'performance-10000',seq:1,tick:0,control:'READY',nodes,agents,jobs,scenario:{nodes,mission:{services}},score:{enabled:false,outcome:'PRACTICE',services},links:[],sessions,events:[],topics:[],history:{first_seq:1,evicted:0}};
	let push:(value:any)=>void=()=>{};let unexpectedCommands=0;
	await page.routeWebSocket('**/api/ws',socket=>{
		push=value=>socket.send(JSON.stringify(value));
		socket.onMessage(()=>unexpectedCommands++);
		push({schema_version:1,type:'snapshot',run_id:snapshot.run_id,seq:1,tick:0,snapshot});
	});
	const started=Date.now();await page.goto('/');await expect(page.locator('.lan-tile')).toHaveCount(40);
	const initialMs=Date.now()-started;
	const initialElements=await page.locator('*').count();expect(initialElements).toBeLessThan(5000);
	expect(await page.locator('dialog .json-tree').count()).toBe(0);
	const searchStart=Date.now();await page.getByRole('textbox',{name:'Search all hosts'}).fill('19960');
	await expect(page.locator('.host-row')).toHaveCount(1);const searchMs=Date.now()-searchStart;
	await page.locator('.host-row').click();await expect(page.locator('.selected-node')).toContainText('19960');
	await expect(page.locator('.cytoscape canvas').first()).toBeVisible();
	await expect(page.locator('.live-connection')).toHaveCount(100);
	await page.getByRole('button',{name:'Next connections'}).click();await expect(page.locator('.live-connection')).toHaveCount(20);
	await page.getByRole('button',{name:'Scenario information'}).click();
	const dialog=page.getByRole('dialog');await expect(dialog).toBeVisible();
	await dialog.locator('.json-key').filter({hasText:/^nodes:/}).locator('..').click();
	await expect(dialog.getByText('1–100 / 10000',{exact:true})).toBeVisible();
	await dialog.getByRole('button',{name:'Next entries',exact:true}).click();
	await expect(dialog.getByText('101–200 / 10000',{exact:true})).toBeVisible();
	expect(await page.locator('*').count()).toBeLessThan(7000);
	await page.keyboard.press('Escape');expect(await page.locator('dialog .json-tree').count()).toBe(0);
	await page.getByRole('tab',{name:'Jobs',exact:true}).click();
	await expect(page.locator('.jobs-table tbody tr')).toHaveCount(100);
	await page.getByRole('button',{name:'Next jobs'}).click();
	await expect(page.getByRole('navigation',{name:'Network location'})).toContainText('LAN 40');
	// Every delta must reduce against the latest wire sequence even when only
	// the most recent projection is painted. No false gap/resync is permitted.
	for(let tick=1;tick<=50;tick++)push({schema_version:1,type:'update',run_id:snapshot.run_id,seq:tick+1,base_seq:tick,delta:{tick,seq:tick+1}});
	await expect(page.getByTestId('simulation-tick')).toHaveText('50');expect(unexpectedCommands).toBe(0);
	await expect(page.locator('[aria-label="Job pages"]')).toContainText('101–200 / 250');
	await expect(page.locator('.live-connection')).toHaveCount(20);
	expect(await page.locator('*').count()).toBeLessThan(8000);expect(errors).toEqual([]);
	await page.setViewportSize({width:390,height:844});
	await expect(page.locator('.explorer-inspector')).toHaveCSS('overflow-y','auto');
	expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
	await testInfo.attach('large-network-metrics',{body:JSON.stringify({initialMs,searchMs,initialElements}),contentType:'application/json'});
});
