<script lang="ts">
	import { onMount, type Component } from 'svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import CheckIcon from '@lucide/svelte/icons/check';
	import ArrowUpRightIcon from '@lucide/svelte/icons/arrow-up-right';
	import type { GalleryEntry } from '$lib/gallery.js';
	let { entry }: { entry: GalleryEntry } = $props();
	let element: HTMLElement;
	let Preview = $state<Component | null>(null);
	let error = $state('');
	let code = $state(false);
	let copied = $state(false);
	async function load() {
		error = '';
		try { Preview = (await entry.load()).default; }
		catch { error = 'Could not load this example.'; }
	}
	onMount(() => {
		const observer = new IntersectionObserver(([event]) => {
			if (event.isIntersecting) { void load(); observer.disconnect(); }
		}, { rootMargin: '400px' });
		observer.observe(element);
		return () => observer.disconnect();
	});
	async function copy() {
		try { await navigator.clipboard.writeText(entry.source); copied = true; }
		catch { error = 'Clipboard unavailable. Select and copy the code directly.'; }
	}
</script>

<article id={entry.id} bind:this={element} class="demo-article">
	<header class="mb-3 flex items-center justify-between gap-3">
		<h2 class="text-sm font-semibold"><a href={'#' + entry.id}>{entry.name}</a></h2>
		<div class="flex items-center gap-1">
			<Button variant="ghost" size="sm" class="h-7 text-xs" aria-pressed={!code} onclick={() => code = false}>Preview</Button>
			<Button variant="ghost" size="sm" class="h-7 text-xs" aria-pressed={code} onclick={() => code = true}>Code</Button>
			<a class="text-muted-foreground hover:text-foreground rounded p-1" href={'https://shadcn-svelte.com/docs/components/' + entry.id} target="_blank" rel="noreferrer" aria-label={entry.name + ' documentation (opens in new tab)'}><ArrowUpRightIcon class="size-4" /></a>
		</div>
	</header>
	<div class="demo-surface cut-surface cut-large relative border border-transparent" class:show-code={code}>
		{#if code}
			<Button variant="outline" size="icon-sm" class="absolute top-3 right-3" aria-label="Copy example code" onclick={copy}>{#if copied}<CheckIcon class="size-4" />{:else}<CopyIcon class="size-4" />{/if}</Button>
			<pre class="max-h-[520px] overflow-auto p-6 pt-14 text-xs leading-6"><code>{entry.source}</code></pre>
		{:else if Preview}
			<div class="demo-content"><Preview /></div>
		{:else if error}
			<div class="p-8 text-sm"><p role="alert">{error}</p><Button class="mt-3" variant="outline" onclick={load}>Retry</Button></div>
		{:else}
			<div class="w-full space-y-3 p-8" aria-label={'Loading ' + entry.name}><Skeleton class="h-5 w-2/3" /><Skeleton class="h-4 w-full" /><Skeleton class="h-4 w-4/5" /></div>
		{/if}
	</div>
</article>
