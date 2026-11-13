import { expect,test } from '@playwright/test';

test('topology keeps labels separated and camera stable, with router jobs and default discovered perspective',async({page},testInfo)=>{
	const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
	const routers=[123,486,884].map((id,i)=>({id,type:'ROUTER',router_id:id,lan_id:3057+i,power:'ONLINE',compromised:false}));
	const hosts=Array.from({length:14},(_,i)=>({id:200795+i,type:i===0?'SERVER':'WORKSTATION',router_id:i<10?123:i<12?486:884,power:i===0?'OFFLINE':'ONLINE',isolated:i===0,compromised:i%2===0,exposure:i%2?{age:2}:null}));
	const snapshot:any={compact:true,schema_version:1,run_id:'labels',seq:1,tick:7,control:'PAUSED',nodes:[...routers,...hosts],agents:routers.map((r,i)=>({id:104+i,router_id:r.id,known:[r.id],observations:[],observation_count:1})),jobs:[{id:1,agent_id:104,router_id:123,target:200795,action:'HOST_SCAN',state:'RUNNING',started_at:6,duration:3}],sessions:[{id:1,source:200795,target:200805,route:[123,486],opened_at:5},{id:2,source:200796,target:200807,route:[123,884],opened_at:6}],links:[],events:[],topics:[],scenario:{tick_limit:100,seed:1},score:{enabled:false,outcome:'PRACTICE'},history:{first_seq:1,evicted:0}};
	let send:(value:any)=>void=()=>{};
	await page.routeWebSocket('**/api/ws*',ws=>{send=e=>ws.send(JSON.stringify(e));send({schema_version:1,type:'snapshot',run_id:snapshot.run_id,seq:1,tick:7,snapshot});});
	await page.route('**/api/details?*',route=>{const q=new URL(route.request().url()).searchParams;return route.fulfill({json:{run_id:'labels',tick:7,total:1,items:q.get('kind')==='job'?[{...snapshot.jobs[0],result:{assessment:'CLEAN'}}]:[{task_id:2,action:'SCAN',router_id:123,observed_at:7,data:{hosts:[]}}]}});});
	await page.goto('/');await page.locator('.lan-tile').first().click();
	const graph=page.locator('.cytoscape');await expect.poll(()=>graph.evaluate((el:any)=>!!el._cyreg?.cy?.nodes().length)).toBe(true);
	const overlaps=await graph.evaluate((el:any)=>{const cy=el._cyreg.cy;const nodes=cy.nodes().not('.lan').toArray();const collisions:string[]=[];for(let i=0;i<nodes.length;i++)for(let j=i+1;j<nodes.length;j++){const a=nodes[i].boundingBox({includeLabels:true,includeOverlays:false}),b=nodes[j].boundingBox({includeLabels:true,includeOverlays:false});if(a.x1<b.x2&&a.x2>b.x1&&a.y1<b.y2&&a.y2>b.y1)collisions.push(`${nodes[i].id()}:${nodes[j].id()}`);}return collisions;});expect(overlaps).toEqual([]);
	await graph.evaluate((el:any)=>{const cy=el._cyreg.cy;cy.zoom(1.1);cy.pan({x:35,y:40});cy.getElementById('123').emit('tap');});
	await expect(page.getByLabel('Router running jobs')).toContainText('Host 200795');
	await page.getByRole('button',{name:'Inspect job 1',exact:true}).click();await expect(page.getByRole('dialog')).toContainText('CLEAN');await page.getByRole('dialog').getByRole('button',{name:'Close',exact:true}).click();
	await page.getByRole('button',{name:'Discovered',exact:true}).click();await expect(page.getByLabel('Discovered Vanguard filter')).toHaveValue('104');await expect(page.locator('.observation')).toContainText('SCAN');
	const before=await graph.evaluate((el:any)=>({zoom:el._cyreg.cy.zoom(),pan:el._cyreg.cy.pan()}));
	snapshot.seq=2;snapshot.tick=8;snapshot.sessions.push({id:3,source:200797,target:200806,route:[123,486],opened_at:8});
	send({schema_version:1,type:'snapshot',run_id:'labels',seq:2,tick:8,snapshot});await expect(page.getByTestId('simulation-tick')).toHaveText('8');
	expect(await graph.evaluate((el:any)=>({zoom:el._cyreg.cy.zoom(),pan:el._cyreg.cy.pan()}))).toEqual(before);
	await page.screenshot({path:testInfo.outputPath('topology-labels-desktop.png')});
	await page.setViewportSize({width:390,height:844});await graph.scrollIntoViewIfNeeded();await page.getByRole('button',{name:'Fit topology'}).click();await page.screenshot({path:testInfo.outputPath('topology-labels-phone.png'),fullPage:true});
	expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);expect(errors).toEqual([]);
});
