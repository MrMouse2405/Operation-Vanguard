import { expect, test } from '@playwright/test';

test('sidebar selection locates a later-page node and renders its connections beyond search and LAN filters',async({page})=>{
  const nodes:any[]=[{id:1,type:'ROUTER',router_id:1,lan_id:1,power:'ONLINE',compromised:false}, {id:2,type:'ROUTER',router_id:2,lan_id:2,power:'ONLINE',compromised:false},
    ...Array.from({length:160},(_,i)=>({id:1001+i,type:'WORKSTATION',router_id:1,lan_id:1,power:'ONLINE',compromised:false,health:5,policy:'DEFAULT_ALLOW',exceptions:[]})),
    {id:2001,type:'SERVER',router_id:2,lan_id:2,power:'ONLINE',compromised:false,health:5,policy:'DEFAULT_ALLOW',exceptions:[]}];
  const snapshot={schema_version:1,run_id:'connections-fixture',seq:1,tick:7,control:'PAUSED',nodes,agents:[{id:101,router_id:1,known:[1],observations:[]}],links:[],sessions:[{id:1,source:1081,target:2001,opened_at:1,route:[1,2]}],jobs:[],events:[],topics:[],scenario:{tick_limit:1000},score:{enabled:false,outcome:'PRACTICE'}};
  await page.routeWebSocket('**/api/ws',ws=>ws.send(JSON.stringify({schema_version:1,type:'snapshot',run_id:snapshot.run_id,seq:1,tick:7,snapshot})));
  await page.goto('/');
  await page.getByRole('button',{name:'Table',exact:true}).click();
  await page.getByRole('button',{name:'Focus host search'}).click();
  const search=page.getByRole('textbox',{name:'Search all hosts'});await expect(search).toBeFocused();
  expect(await search.evaluate(el=>getComputedStyle(el).outlineStyle)).toBe('none');
  await search.fill('1081');await page.locator('.host-row').click();
  await expect(page.locator('.selected-node')).toContainText('1081');
  await expect(page.getByRole('button',{name:'Fit topology'})).toBeVisible();
  await expect(page.locator('.live-connection')).toContainText('Outbound');
  await expect(page.locator('.live-connection')).toContainText('Opened at tick 1 · 6 ticks ago');
  await expect(page.locator('.live-connection')).toContainText('R1 → R2');
  await expect.poll(()=>page.locator('.cytoscape').evaluate((el:any)=>{
    const cy=el._cyreg?.cy;
    return cy?{selected:cy.$(':selected').id(),peer:cy.getElementById('2001').length,edges:cy.edges('.session').length}:null;
  })).toEqual({selected:'1081',peer:1,edges:1});
  await page.screenshot({path:'../.impeccable/review/explorer-connections-1440.png',fullPage:true});
  await page.getByRole('button',{name:'Server 2001'}).click();
  await expect(page.locator('.selected-node')).toContainText('2001');
  await expect(page.locator('.live-connection')).toContainText('Inbound');
  await page.keyboard.press('Control+k');await expect(search).toBeFocused();
  await page.screenshot({path:'../.impeccable/review/explorer-search-focus.png',fullPage:true});
  await page.keyboard.press('Meta+k');await expect(search).toBeFocused();
  await page.setViewportSize({width:390,height:844});
  await expect.poll(()=>page.locator('.cytoscape').evaluate((el:any)=>{
    const cy=el._cyreg?.cy;return cy?cy.zoom():0;
  })).toBeGreaterThanOrEqual(.85);
  await page.screenshot({path:'../.impeccable/review/explorer-connections-390.png',fullPage:true});
});

test('seconds per tick is server-owned, survives reconnect and does not slow manual stepping',async({page,request})=>{
  const command=async(command:string,extra={})=>request.post('/api/command',{data:{schema_version:1,request_id:crypto.randomUUID(),command,...extra}});
  await command('Restart');
  await page.goto('/');await expect(page.getByRole('status')).toHaveText('Live');
  const pacing=page.getByRole('spinbutton',{name:'Seconds per tick'});
  await pacing.fill('0.5');await pacing.press('Tab');
  await expect.poll(()=>request.get('/api/snapshot').then(r=>r.json()).then(s=>s.seconds_per_tick)).toBe(0.5);
  await page.reload();await expect(pacing).toHaveValue('0.5');
  await page.getByRole('button',{name:'+1 tick',exact:true}).click();
  await expect(page.getByTestId('simulation-tick')).toHaveText('1');
  await pacing.fill('0');await pacing.press('Tab');
  await expect.poll(()=>request.get('/api/snapshot').then(r=>r.json()).then(s=>s.seconds_per_tick)).toBe(0);
  await command('SetTickPacing',{seconds_per_tick:0.1});
});
