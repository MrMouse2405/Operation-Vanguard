<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Mail, Search, Inbox, Folder, Star, BookOpen, ChevronRight, ChevronDown, ArrowLeft, ArrowRight, ExternalLink, PanelLeftClose, PanelLeftOpen, Link, X, GraduationCap, MailOpen } from '@lucide/svelte';
  import Button from '$lib/components/ui/button/button.svelte';
  import Input from '$lib/components/ui/input/input.svelte';
  import Separator from '$lib/components/ui/separator/separator.svelte';
  import { emails, folders, exerciseDownload } from '$lib/mail';
  import { mode, toggleMode } from 'mode-watcher';
  import { Moon, Sun } from '@lucide/svelte';

  let { slug, offline = false }: { slug?: string; offline?: boolean } = $props();
  const mailHref = (href: string) => offline ? `#${href}` : href;
  let search = $state('');
  let searchInput = $state<HTMLInputElement|null>(null);
  let folder = $state('Learning path');
  let folderOpen = $state(true);
  let mobileInbox = $state(false);
  let read = $state<string[]>([]);
  let starred = $state<string[]>([]);
  let loaded = $state(false);
  let feedback = $state('');
  let shortcut = $state('Ctrl K');
  let selected = $derived(emails.find(email => email.slug === (slug ?? 'briefing')) ?? emails[0]);
  let emailHtml = $derived(offline ? selected.html.replaceAll('href="/docs/', 'href="#/docs/') : selected.html);
  let next = $derived(emails[selected.order]);
  let previous = $derived(emails[selected.order-2]);
  let filtered = $derived(emails.filter(email => {
    const inFolder = folder === 'Learning path' || folder === 'All mail' || (folder === 'Unread' ? !read.includes(email.slug) : folder === 'Starred' ? starred.includes(email.slug) : email.folder === folder);
    const query = search.trim().toLowerCase();
    return (query ? true : inFolder) && (!query || `${email.sender} ${email.subject} ${email.body}`.toLowerCase().includes(query));
  }));
  let title = $derived(search.trim() ? 'Search results' : folder);
  onMount(() => {
    const desktop = window.matchMedia('(min-width: 801px)');
    folderOpen = desktop.matches;
    const resizeFolders = (event: MediaQueryListEvent) => { folderOpen = event.matches; };
    desktop.addEventListener('change', resizeFolders);
    shortcut = /Mac|iPhone|iPad/.test(navigator.platform) ? '⌘ K' : 'Ctrl K';
    try {
      const saved = JSON.parse(localStorage.getItem('vanguard-mail') ?? '{}');
      read = Array.isArray(saved.read) ? saved.read.filter((s:unknown)=>typeof s==='string') : [];
      starred = Array.isArray(saved.starred) ? saved.starred.filter((s:unknown)=>typeof s==='string') : [];
    } catch { /* Mail remains usable when local storage is unavailable. */ }
    loaded = true;
    return () => desktop.removeEventListener('change', resizeFolders);
  });
  $effect(() => {
    const id = selected.slug;
    const ready = loaded;
    if(ready) untrack(() => { if(!read.includes(id)) read = [...read,id]; feedback=''; });
  });
  $effect(() => {
    if(loaded) {
      try { localStorage.setItem('vanguard-mail', JSON.stringify({read,starred})); } catch { /* Optional local preferences. */ }
    }
  });
  function chooseFolder(value:string) { folder=value;search='';mobileInbox=true;if(window.matchMedia('(max-width: 800px)').matches)folderOpen=false; }
  function toggleStar(id:string) { starred = starred.includes(id) ? starred.filter(s=>s!==id) : [...starred,id]; }
  function toggleRead() { read = read.includes(selected.slug) ? read.filter(s=>s!==selected.slug) : [...read,selected.slug]; }
  function focusSearch() { searchInput?.focus();searchInput?.select(); }
  function keys(event:KeyboardEvent) {
    if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k') { event.preventDefault();focusSearch(); }
    if(event.key==='Escape') { search='';searchInput?.blur(); }
  }
  function listKeys(event:KeyboardEvent) {
    if(!['ArrowDown','ArrowUp','Home','End'].includes(event.key)) return;
    const links = Array.from(document.querySelectorAll<HTMLAnchorElement>('.message-link'));
    const index=links.indexOf(document.activeElement as HTMLAnchorElement);
    if(index<0)return;
    event.preventDefault();
    const target=event.key==='Home'?0:event.key==='End'?links.length-1:Math.max(0,Math.min(links.length-1,index+(event.key==='ArrowDown'?1:-1)));
    links[target]?.focus();
  }
  async function copyLink() {
    try { await navigator.clipboard.writeText(offline ? `${location.href.split('#')[0]}#${selected.href}` : new URL(selected.href,location.origin).href);feedback='Link copied'; }
    catch { feedback='Copy the address from “Open in new tab”.'; }
  }
  async function copyCode(event: MouseEvent) {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>('.copy-code');
    if (!button) return;
    const code = button.closest('.code-block')?.querySelector('code')?.textContent ?? '';
    try { await navigator.clipboard.writeText(code); button.textContent='Copied'; feedback='Code copied'; }
    catch { feedback='Clipboard unavailable. Select the code and copy it with Ctrl/Cmd C.'; }
  }
</script>

<svelte:window onkeydown={keys} onclick={copyCode}/>
<svelte:head><title>{slug===undefined?'Inbox':selected.subject} · VanMail</title><meta name="description" content="Operation Vanguard competition briefing, tutorials, Lua examples, interface guide and API reference."/></svelte:head>

<a class="skip-link" href="#email-body">Skip to email</a>
<div class="mail-app" class:reading={slug!==undefined&&!mobileInbox&&!search.trim()} class:folders-hidden={!folderOpen}>
  <header class="app-bar">
    <a class="app-brand" href={mailHref('/')} aria-label="VanMail inbox"><Mail size={24}/><span>Van<b>Mail</b></span></a>
    <div class="mail-search"><Search size={18}/><Input type="search" bind:ref={searchInput} bind:value={search} placeholder="Search mail, topics, or Lua functions" aria-label="Search all documentation"/><button class="search-key" onclick={focusSearch} aria-label="Focus mail search"><kbd>{shortcut}</kbd></button></div>
    <button class="theme-toggle" onclick={toggleMode} aria-label={mode.current === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'} title={mode.current === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}>{#if mode.current === 'dark'}<Sun size={19}/>{:else}<Moon size={19}/>{/if}</button>
  </header>

  <div class="mail-workspace">
    <aside class="folder-pane" aria-label="Documentation folders">
      <div class="folder-heading"><h1>Documentation</h1><Button variant="ghost" size="icon" aria-label="Hide folders" onclick={()=>folderOpen=false}><PanelLeftClose size={16}/></Button></div>
      <button class="learning-button" class:folder-selected={folder==='Learning path'} onclick={()=>chooseFolder('Learning path')}><GraduationCap size={18}/><span>Learning path</span><ChevronRight size={15}/></button>
      <h2>Favourites</h2>
      <nav class="folder-list" aria-label="Mail folders">
        <button class:folder-selected={folder==='All mail'} onclick={()=>chooseFolder('All mail')}><Inbox size={17}/><span>All mail</span><small>{emails.length}</small></button>
        <button class:folder-selected={folder==='Unread'} onclick={()=>chooseFolder('Unread')}><Mail size={17}/><span>Unread</span><small>{emails.filter(e=>!read.includes(e.slug)).length}</small></button>
        <button class:folder-selected={folder==='Starred'} onclick={()=>chooseFolder('Starred')}><Star size={17}/><span>Starred</span><small>{starred.length}</small></button>
      </nav>
      <h2><ChevronDown size={13}/> Your lessons</h2>
      <nav class="folder-list topic-folders" aria-label="Topic folders">
        {#each folders as topic}<button class:folder-selected={folder===topic} onclick={()=>chooseFolder(topic)}><Folder size={17}/><span>{topic}</span><small>{emails.filter(e=>e.folder===topic).length}</small></button>{/each}
      </nav>
    </aside>

    <section class="mail-content" aria-label="Documentation mailbox">
      <div class="mail-toolbar">
        <Button variant="ghost" size="icon" aria-label={folderOpen?'Hide folders':'Show folders'} onclick={()=>folderOpen=!folderOpen}>{#if folderOpen}<PanelLeftClose size={18}/>{:else}<PanelLeftOpen size={18}/>{/if}</Button>
        <span class="toolbar-label">Mail</span><Separator orientation="vertical" class="toolbar-separator"/>
        <Button variant="ghost" size="sm" onclick={toggleRead} aria-label={read.includes(selected.slug)?'Mark unread':'Mark read'} title={read.includes(selected.slug)?'Mark unread':'Mark read'}>{#if read.includes(selected.slug)}<Mail size={16}/><span>Mark unread</span>{:else}<MailOpen size={16}/><span>Mark read</span>{/if}</Button>
        <Button variant="ghost" size="sm" onclick={()=>toggleStar(selected.slug)} aria-label={starred.includes(selected.slug)?'Unstar':'Star'} title={starred.includes(selected.slug)?'Unstar':'Star'} aria-pressed={starred.includes(selected.slug)}><Star size={16} fill={starred.includes(selected.slug)?'currentColor':'none'}/><span>{starred.includes(selected.slug)?'Unstar':'Star'}</span></Button>
        <a class="exercise-download" href={exerciseDownload} download="VanMail-examples.zip">Download example files</a>
      </div>
      <div class="message-columns">
        <section class="message-list" aria-label="Documentation emails">
          <div class="list-heading"><div><h2>{title}</h2><span>{filtered.length} {filtered.length===1?'email':'emails'}{title==='Learning path'?' · suggested order':''}</span></div>{#if search}<Button variant="ghost" size="icon" aria-label="Clear search" onclick={()=>search=''}><X size={17}/></Button>{:else}<BookOpen size={18}/>{/if}</div>
          <!-- Arrow keys move through the normal, keyboard-focusable message links. -->
          <div class="message-scroll" role="group" aria-label="Email list; use arrow keys to move between messages">
            {#each filtered as email (email.slug)}
              <div class="message-row" class:message-selected={selected.slug===email.slug} class:unread={!read.includes(email.slug)}>
                <a class="message-link" href={mailHref(email.href)} onkeydown={listKeys} onclick={()=>{mobileInbox=false;search='';}} aria-current={selected.slug===email.slug?'page':undefined}>
                  <div class="message-from"><strong>{email.sender}</strong><span>{email.minutes} min</span></div>
                  <h3>{email.subject}</h3><p>{email.preview}</p><span class="message-topic">{folder==='Learning path'?`${email.order} of ${emails.length} · `:''}{email.folder}</span>
                </a>
                <button class="row-star" class:is-starred={starred.includes(email.slug)} aria-label={`${starred.includes(email.slug)?'Unstar':'Star'}: ${email.subject}`} aria-pressed={starred.includes(email.slug)} onclick={()=>toggleStar(email.slug)}><Star size={15} fill={starred.includes(email.slug)?'currentColor':'none'}/></button>
              </div>
            {:else}<div class="mail-empty"><Search size={28}/><h3>{search?'No matching emails':folder==='Starred'?'Nothing starred yet':'You’re all caught up'}</h3><p>{search?'Try a topic like “scan” or a function like “vanguard.await”.':folder==='Starred'?'Use the star beside an email to keep it here.':'Open All mail to revisit a lesson.'}</p><Button variant="outline" size="sm" onclick={()=>chooseFolder('All mail')}>Show all mail</Button></div>{/each}
          </div>
        </section>

        <section class="reading-pane" aria-label="Selected email">
          <div class="reading-top"><a class="back-inbox" href={mailHref('/')}><ArrowLeft size={16}/> Inbox</a><span>{selected.folder}</span><a class="popout" href={mailHref(selected.href)} target="_blank" rel="noopener noreferrer" aria-label="Open email in new tab"><ExternalLink size={16}/><span>Open in new tab</span></a></div>
          {#key selected.slug}<div class="reading-scroll" data-email={selected.slug}>
            <article class="email" id="email-body" tabindex="-1">
              <h1>{selected.subject}</h1>
              <div class="email-sender"><span class="sender-avatar" style={`--avatar-color: ${['#0067b8','#6b4c9a','#216e58','#9b4c19'][selected.order%4]}`}>{selected.sender.split(' ')[1].slice(0,2).toUpperCase()}</span><div><strong>{selected.sender}</strong><span>To: You</span></div><span class="lesson-position">Lesson {selected.order} of {emails.length}<small>{selected.minutes} min read</small></span></div>
              <!-- Delegation handles copy buttons rendered from trusted local Markdown. -->
              <div class="email-prose"><p>{selected.greeting}</p>{@html emailHtml}<p class="email-signoff">{selected.signoff}<br/><strong>{selected.sender}</strong></p></div>
              <div class="email-actions"><Button variant="outline" size="sm" onclick={copyLink}><Link size={15}/> Copy email link</Button><span role="status" aria-live="polite">{feedback}</span></div>
              <nav class="lesson-navigation" aria-label="Learning path navigation">{#if previous}<a href={mailHref(previous.href)} class="previous-lesson"><ArrowLeft size={16}/><span>Previous<small>{previous.subject}</small></span></a>{/if}{#if next}<a href={mailHref(next.href)} class="next-lesson"><span>Up next<small>{next.subject}</small></span><ArrowRight size={17}/></a>{:else}<a href={mailHref('/')} class="next-lesson"><span>You made it!<small>Back to your inbox</small></span><Inbox size={17}/></a>{/if}</nav>
            </article>
          </div>{/key}
        </section>
      </div>
    </section>
  </div>
</div>
