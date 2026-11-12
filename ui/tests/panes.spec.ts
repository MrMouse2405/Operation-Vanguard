import { expect, test } from '@playwright/test';

test('dashboard panes resize with pointer and keyboard, persist, and retain score/inspection controls',async({page})=>{
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Live');
  await expect(page.getByText('SITUATION BRIEF',{exact:true})).toHaveCount(0);
  await expect(page.getByRole('button',{name:'Close node inspector'})).toHaveCount(0);
  await expect(page.getByRole('heading',{name:/Match score/})).toBeVisible();
  const drag=async(name:string,x:number,y:number)=>{
    const separator=page.getByRole('separator',{name,exact:true});
    const box=(await separator.boundingBox())!;
    await page.mouse.move(box.x+box.width/2,box.y+box.height/2);
    await page.mouse.down();await page.mouse.move(box.x+box.width/2+x,box.y+box.height/2+y,{steps:8});await page.mouse.up();
  };
  const left=page.getByRole('complementary',{name:'Network explorer'});
  const right=page.getByRole('complementary',{name:'Node inspector'});
  const leftBefore=(await left.boundingBox())!.width;
  await drag('Resize network explorer',70,0);
  await expect.poll(async()=>Math.round((await left.boundingBox())!.width)).toBe(Math.round(leftBefore+70));
  const rightBefore=(await right.boundingBox())!.width;
  await drag('Resize match score and inspector',-60,0);
  await expect.poll(async()=>Math.round((await right.boundingBox())!.width)).toBe(Math.round(rightBefore+60));
  await page.getByRole('separator',{name:'Resize match score and inspector',exact:true}).focus();
  await page.keyboard.press('ArrowLeft');
  await expect.poll(async()=>Math.round((await right.boundingBox())!.width)).toBe(Math.round(rightBefore+70));
  await page.reload();
  await expect.poll(async()=>Math.round((await left.boundingBox())!.width)).toBe(Math.round(leftBefore+70));
  await page.getByRole('tab',{name:'Activity',exact:true}).click();
  const telemetry=page.getByRole('region',{name:'Vanguard telemetry'});
  const dockBefore=(await telemetry.boundingBox())!.height;
  await drag('Resize telemetry',0,-70);
  await expect.poll(async()=>Math.round((await telemetry.boundingBox())!.height)).toBe(Math.round(dockBefore+70));
  await page.getByRole('button',{name:'Collapse telemetry'}).click();
  await page.getByRole('button',{name:'Expand telemetry'}).click();
  await expect.poll(async()=>Math.round((await telemetry.boundingBox())!.height)).toBe(Math.round(dockBefore+70));
  await page.screenshot({path:'../.impeccable/review/resizable-dashboard-1440.png',fullPage:true});
  for(const width of [900,390]){
    await page.setViewportSize({width,height:1000});
    await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  }
  await expect(page.getByRole('separator',{name:'Resize network explorer',exact:true})).toHaveAttribute('aria-orientation','horizontal');
  await page.screenshot({path:'../.impeccable/review/resizable-dashboard-390.png',fullPage:true});
});
