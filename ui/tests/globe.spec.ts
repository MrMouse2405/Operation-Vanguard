import { expect,test } from '@playwright/test';

test('globe and topology show directional live tasks and sessions, with usable inspection and motion controls',async({page},testInfo)=>{
	const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
	let send:(value:any)=>void=()=>{};
	const nodes=Array.from({length:12},(_,i)=>({id:i<4?i+1:1001+i-4,type:i<4?'ROUTER':'WORKSTATION',router_id:i<4?i+1:Math.floor((i-4)/2)+1,power:'ONLINE',compromised:i===4,exposure:i===6?{age:2}:null,exceptions:[]}));
	const snapshot={schema_version:1,run_id:'globe-test',seq:1,tick:7,control:'RUNNING',nodes,agents:[1,2,3,4].map(id=>({id:100+id,router_id:id,known:[],observations:[]})),jobs:[{id:1,agent_id:101,router_id:1,target:1002,action:'HOST_SCAN',state:'RUNNING',started_at:6,duration:3},{id:2,agent_id:102,router_id:2,target:1003,action:'RESTORE',state:'COMPLETED',started_at:1,duration:5}],sessions:[{id:1,source:1001,target:1003,route:[1,2],opened_at:5},{id:2,source:1005,target:1007,route:[3,4],opened_at:6}],links:[],events:[],topics:[],scenario:{tick_limit:100,seed:1},score:{enabled:false,outcome:'PRACTICE'},history:{first_seq:1,evicted:0}};
	await page.routeWebSocket('**/api/ws*',ws=>{send=e=>ws.send(JSON.stringify(e));send({schema_version:1,type:'snapshot',run_id:snapshot.run_id,seq:1,tick:7,snapshot});});
	await page.goto('/');await page.getByRole('button',{name:'Globe',exact:true}).click();
	await expect(page.getByRole('region',{name:'Global network communications'})).toBeVisible();
	await expect(page.locator('.globe-flow')).toHaveCount(3);await expect(page.getByText('Illustrative LAN locations',{exact:false})).toBeVisible();
	const canvas=page.locator('.globe-stage canvas');await expect(canvas).toBeVisible();
	const first=await canvas.evaluate((c:HTMLCanvasElement)=>c.toDataURL());
	await expect.poll(()=>canvas.evaluate((c:HTMLCanvasElement)=>c.toDataURL())).not.toBe(first);
	await page.getByRole('button',{name:'Pause globe rotation'}).click();
	await page.getByRole('combobox',{name:'Globe flow type'}).selectOption('task');await expect(page.locator('.globe-flow')).toHaveCount(1);
	await page.locator('.globe-flow').getByRole('button',{name:'Host 1002'}).click();await expect(page.locator('.selected-node')).toContainText('1002');
	await expect(page.getByRole('button',{name:'Globe',exact:true})).toHaveAttribute('aria-pressed','true');
	await page.getByRole('combobox',{name:'Globe flow type'}).selectOption('all');
	await page.getByRole('combobox',{name:'Globe LAN'}).selectOption('1');await expect(page.locator('.globe-flow')).toHaveCount(2);
	await canvas.focus();await page.keyboard.press('ArrowLeft');await page.keyboard.press('+');
	await page.screenshot({path:testInfo.outputPath('globe-desktop.png')});
	await page.getByRole('button',{name:'Topology',exact:true}).click();await expect(page.locator('.map-bar')).toContainText('1 tasks');await expect(page.locator('.map-bar')).toContainText('1 connections');await expect(page.locator('.cytoscape canvas').first()).toBeVisible();
	await page.getByRole('tab',{name:'Jobs',exact:true}).click();await expect(page.getByRole('tab',{name:'Tasks',exact:true})).toHaveCount(0);
	await page.getByRole('combobox',{name:'Filter Vanguard agents'}).selectOption('');
	await page.getByRole('button',{name:/^Running/}).click();await expect(page.locator('.jobs-table tbody tr')).toHaveCount(1);
	await page.getByRole('button',{name:/^Completed/}).click();await expect(page.locator('.jobs-table tbody tr')).toHaveCount(1);
	// Record patch finishes a task and appends its observation, without resending
	// the unchanged nodes, task history or scenario arrays.
	send({schema_version:1,type:'update',run_id:snapshot.run_id,seq:2,base_seq:1,tick:8,patch:{fields:{tick:{set:8},seq:{set:2},control:{set:'PAUSED'},jobs:{key:'id',changes:[{id:1,patch:{fields:{state:{set:'COMPLETED'},finished_at:{set:8}},removed:[]}}],removed:[]},agents:{key:'id',changes:[{id:101,patch:{fields:{observations:{append:[{task_id:1,action:'HOST_SCAN',target:1002,observed_at:8,data:{assessment:'CLEAN'}}]}},removed:[]}}],removed:[]}},removed:[]}});
	await expect(page.getByTestId('simulation-tick')).toHaveText('8');await expect(page.locator('.jobs-table tbody tr')).toHaveCount(2);
	await page.getByRole('button',{name:'Globe',exact:true}).click();await expect(page.locator('.globe-flow')).toHaveCount(1);
	await page.waitForTimeout(150);const paused=await canvas.evaluate((c:HTMLCanvasElement)=>c.toDataURL());await page.waitForTimeout(150);expect(await canvas.evaluate((c:HTMLCanvasElement)=>c.toDataURL())===paused).toBe(true);
	await page.emulateMedia({reducedMotion:'reduce'});await page.setViewportSize({width:390,height:844});await canvas.scrollIntoViewIfNeeded();await page.waitForTimeout(100);await page.screenshot({path:testInfo.outputPath('globe-mobile.png'),fullPage:true});
	expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);expect(errors).toEqual([]);
});
