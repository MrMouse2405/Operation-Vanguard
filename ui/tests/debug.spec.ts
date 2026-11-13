import { expect, test } from '@playwright/test';

test('debug perspectives compare notes and metadata with bounded responsive drawings', async({page,request})=>{
  const live=await request.get('/api/snapshot').then(r=>r.json());
  const nodes=Object.fromEntries(Array.from({length:90},(_,i)=>[String(i+1),{id:String(i+1),info:{assessment:'UNKNOWN',observed_at:7}}]));
  const view={updated_at:7,nodes,edges:{link:{id:'link',from:'1',to:'2',info:{kind:'observed'}}},info:{explanation:'observed only'}};
  const snapshot={...live,debug:{enabled:true,agents:{101:{discovery:view,assessments:{...view,updated_at:9,info:{explanation:'fresh assessment'}}},102:{discovery:view}}}};
  await page.route('**/api/snapshot',route=>route.fulfill({json:snapshot}));
  await page.routeWebSocket('**/api/ws*',ws=>{ws.send(JSON.stringify({schema_version:1,type:'snapshot',run_id:snapshot.run_id,seq:snapshot.seq,tick:snapshot.tick,snapshot}));});
  await page.goto('/');
  await page.getByRole('button',{name:/Compare agent drawings/}).click();
  await expect(page.getByRole('region',{name:'Debug network comparison'})).toBeVisible();
  await expect(page.getByRole('combobox',{name:'Left debug view'})).toHaveValue('101/discovery');
  await page.getByRole('combobox',{name:'Right debug view'}).selectOption('101/assessments');
  await expect(page.locator('.debug-pane').nth(1)).toContainText('fresh assessment');
  await expect(page.locator('.debug-pane').first().locator('circle')).toHaveCount(80);
  await page.locator('.debug-pane').first().getByRole('button',{name:'Next',exact:true}).click();
  await expect(page.locator('.debug-pane').first().locator('circle')).toHaveCount(10);
  const leftPane=page.locator('.debug-pane').first();
  const before=(await leftPane.boundingBox())!.width;
  await page.getByRole('separator',{name:'Resize debug perspectives',exact:true}).focus();
  await page.keyboard.press('ArrowRight');
  await expect.poll(async()=>Math.round((await leftPane.boundingBox())!.width)).toBe(Math.round(before+10));
  const comparison=page.getByRole('region',{name:'Debug network comparison'});
  const height=(await comparison.boundingBox())!.height;
  await page.getByRole('separator',{name:'Resize agent drawings',exact:true}).focus();
  await page.keyboard.press('ArrowDown');
  await expect.poll(async()=>Math.round((await comparison.boundingBox())!.height)).toBe(Math.round(height+10));
  for(const width of [1440,390]){
    await page.setViewportSize({width,height:1000});
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.screenshot({path:`../.impeccable/review/debug-comparison-${width}.png`,fullPage:true});
  }
});
