import { expect, test } from '@playwright/test';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { unzipSync, strFromU8 } from 'fflate';

test('every lesson and all 23 API functions render at shareable static URLs',async({page,request})=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto('/');await expect(page.locator('.message-link')).toHaveCount(readdirSync('content/mail').length);
  await expect(page.locator('.list-heading')).toContainText('Learning path');
  const links=await page.locator('.message-link').evaluateAll(links=>links.map(a=>a.getAttribute('href')!));
  const headers:string[]=[];
  for(const href of links){
    expect((await request.get(href)).ok()).toBe(true);
    await page.goto(href);await expect(page.locator('.email > h1')).toBeVisible();
    await expect(page.locator('.email-signoff')).toContainText(await page.locator('.email-sender strong').innerText());
    headers.push(...await page.locator('.email-prose h2').allTextContents());
  }
  const spec=readFileSync('../src/api.lua','utf8');
  for(const [,name] of spec.matchAll(/^function vanguard\.(\w+)\(/gm))expect(headers).toContain(`vanguard.${name}`);
  await page.goto('/docs/briefing/');
  expect(await page.locator('.email-prose a[href^="/docs/"]').count()).toBeGreaterThanOrEqual(42);
  expect(errors).toEqual([]);
});

test('VanMail themes persist, blocks highlight and copy exact source',async({page,context})=>{
  await context.grantPermissions(['clipboard-read','clipboard-write']);
  await page.goto('/docs/examples/solo/');
  await expect(page.locator('.mail-account')).toHaveCount(0);
  await page.getByRole('button',{name:'Switch to dark mode'}).click();
  await expect(page.locator('html')).toHaveClass(/dark/);
  await page.reload();await expect(page.locator('html')).toHaveClass(/dark/);
  for(const width of [1440,390]){
    await page.setViewportSize({width,height:1000});
    await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.screenshot({path:`../.impeccable/review/vanmail-dark-${width}.png`,fullPage:true});
  }
  await expect(page.locator('.hljs-keyword').first()).toBeVisible();
  const source=await page.locator('.code-block code').last().textContent();
  await page.locator('.copy-code').last().click();
  expect(await page.evaluate(()=>navigator.clipboard.readText())).toBe(source);
  await page.getByRole('button',{name:'Switch to light mode'}).click();
  await expect(page.locator('html')).not.toHaveClass(/dark/);
});

test('single-file VanMail works offline from file URL with navigation and themes',async({page,context})=>{
  expect(readdirSync('out')).toEqual(['VanMail.html']);
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  const requests:string[]=[];page.on('request',r=>{if(r.url().startsWith('http'))requests.push(r.url());});
  await context.setOffline(true);
  await page.goto(pathToFileURL(resolve('out/VanMail.html')).href);
  await expect(page.locator('.email > h1')).toContainText('READ FIRST');
  await expect(page.locator('.message-link')).toHaveCount(readdirSync('content/mail').length);
  await expect(page.locator('.app-rail, .mail-status, .list-footer, .folder-note, .list-section-label')).toHaveCount(0);
  await expect(page.getByText('Read at your own pace')).toHaveCount(0);
  // Every link in the starter table stays in this file, including native new tabs.
  const starterLinks = await page.locator('.email-prose table a').evaluateAll(links=>links.map(a=>a.getAttribute('href')!));
  expect(starterLinks.length).toBe(5);
  for(const href of starterLinks){
    expect(href).toMatch(/^#\/docs\//);
    await page.goto(pathToFileURL(resolve('out/VanMail.html')).href);
    await page.locator(`.email-prose table a[href="${href}"]`).first().click();
    await expect(page.locator('.email > h1')).toBeVisible();
    expect(new URL(page.url()).hash).toBe(href);
  }
  const popupPromise=context.waitForEvent('page');
  await page.getByRole('link',{name:'Open email in new tab'}).click();
  const popup=await popupPromise;
  await expect(popup.locator('.email > h1')).toContainText('Tutorial 2');
  await popup.close();
  // The ZIP is embedded, so code and topology paths work without the repository.
  const zipDownload=page.waitForEvent('download');
  await page.getByRole('link',{name:'Download example files'}).click();
  const zip=await zipDownload;
  const files=unzipSync(readFileSync((await zip.path())!));
  for(const path of ['vanguard/agents/basic-defender.lua','vanguard/agents/patrol.lua','vanguard/networks/tutorial-1.toml','vanguard/networks/Network Agent.md','network.toml']){
    expect(strFromU8(files[path])).toBe(readFileSync(`../${path}`,'utf8'));
  }
  await page.goto(pathToFileURL(resolve('out/VanMail.html')).href+'#/docs/networks/agent/');
  const fileDownload=page.waitForEvent('download');
  await page.getByRole('link',{name:'Download Network Agent.md',exact:true}).click();
  const file=await fileDownload;
  expect(readFileSync((await file.path())!,'utf8')).toBe(readFileSync('../vanguard/networks/Network Agent.md','utf8'));
  await page.goto(pathToFileURL(resolve('out/VanMail.html')).href);
  await page.locator('.email-prose a[href="#/docs/tutorial/one/"]').first().click();
  await expect(page.locator('.email > h1')).toContainText('Tutorial 1');
  await expect(page.locator('.hljs-attr').first()).toBeVisible();
  await page.getByRole('button',{name:'Switch to dark mode'}).click();
  await expect(page.locator('html')).toHaveClass(/dark/);
  await page.reload();await expect(page.locator('.email > h1')).toContainText('Tutorial 1');
  await expect(page.locator('html')).toHaveClass(/dark/);
  await page.getByRole('searchbox',{name:'Search all documentation'}).fill('dbgguard');
  await expect(page.locator('.message-link')).not.toHaveCount(0);
  expect(errors).toEqual([]);expect(requests).toEqual([]);
  await context.setOffline(false);
});

test('folders, search, read state, stars and new-tab links work',async({page,context})=>{
  await page.goto('/');
  await page.getByRole('button',{name:'Taking action',exact:false}).click();
  await expect(page.locator('.message-link')).toHaveCount(5);
  await page.keyboard.press('Control+k');const search=page.getByRole('searchbox',{name:'Search all documentation'});
  await expect(search).toBeFocused();await search.fill('vanguard.await_any');
  await expect(page.locator('.message-link')).not.toHaveCount(0);
  await page.locator('.message-link').filter({hasText:'Your job receipts and where to file them'}).click();
  await expect(page.locator('.email > h1')).toHaveText('Your job receipts and where to file them');
  await page.getByRole('button',{name:'Star',exact:true}).click();
  await page.getByRole('button',{name:'Starred',exact:false}).first().click();
  await expect(page.locator('.message-link')).toHaveCount(1);
  await page.reload();
  await page.getByRole('button',{name:'Starred',exact:false}).first().click();
  await expect(page.locator('.message-link')).toHaveCount(1);
  await page.getByRole('button',{name:'Mark unread',exact:true}).click();
  await page.getByRole('button',{name:'Unread',exact:false}).click();
  await expect(page.locator('.message-row.unread').filter({hasText:'Your job receipts and where to file them'})).toHaveCount(1);
  const popupPromise=context.waitForEvent('page');
  await page.getByRole('link',{name:'Open email in new tab'}).click();
  const popup=await popupPromise;await popup.waitForLoadState();
  await expect(popup.locator('.email > h1')).toHaveText('Your job receipts and where to file them');await popup.close();
  await search.fill('no-such-lesson');await expect(page.getByText('No matching emails')).toBeVisible();
  await page.getByRole('button',{name:'Clear search'}).click();
  await page.keyboard.press('Meta+k');await expect(search).toBeFocused();
});

test('inbox and long emails fit desktop and phone; mobile search and navigation stay usable',async({page})=>{
  for(const viewport of [{width:1440,height:1000},{width:390,height:844}]){
    await page.setViewportSize(viewport);await page.goto('/');
    await expect(page.locator('.message-list')).toBeVisible();
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.screenshot({path:`../.impeccable/review/mail-inbox-${viewport.width}.png`,fullPage:true});
    await page.goto('/docs/network/');await expect(page.locator('.email > h1')).toHaveText('Two routers. One cable. Start there.');
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.screenshot({path:`../.impeccable/review/mail-reading-${viewport.width}.png`,fullPage:true});
    if(viewport.width===390){
		await expect(page.getByRole('button',{name:'Mark unread',exact:true})).toBeVisible();
		await expect(page.getByRole('button',{name:'Star',exact:true})).toBeVisible();
      await expect(page.locator('.message-list')).not.toBeVisible();
      await page.getByRole('searchbox',{name:'Search all documentation'}).fill('vanguard.can_deploy');
      await expect(page.locator('.message-list')).toBeVisible();
      await page.locator('.message-link').filter({hasText:'“Finished” was not my question'}).click();
      await expect(page.locator('.reading-pane')).toBeVisible();
      await expect(page.locator('.email > h1')).toHaveText('“Finished” was not my question');
      await page.getByRole('link',{name:'Inbox',exact:true}).click();
      await expect(page.locator('.message-list')).toBeVisible();
      await page.getByRole('button',{name:'Show folders'}).click();
      await page.getByRole('button',{name:'Quick reference',exact:false}).click();
      await expect(page.locator('.message-link')).toHaveCount(2);
    }
  }
});
