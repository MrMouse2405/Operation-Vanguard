<script lang="ts">
	import { untrack } from 'svelte';
	import JsonTree from './JsonTree.svelte';
	let { value, name = '', depth = 0 }: { value: any; name?: string; depth?: number } = $props();
	let structured = $derived(value !== null && typeof value === 'object');
	let expanded = $state(untrack(() => depth === 0));
	let page = $state(0);
	const pageSize = 100;
	let size = $derived(structured ? (Array.isArray(value) ? value.length : Object.keys(value).length) : 0);
	let entries = $derived(structured && expanded ? Object.entries(value).slice(page * pageSize, (page + 1) * pageSize) : []);
	let kind = $derived(value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value);
	$effect(() => { if (page * pageSize >= size) page = 0; });
</script>
{#if structured && depth < 20}
	<details class="json-tree" bind:open={expanded}>
		<summary>{#if name}<span class="json-key">{name}: </span>{/if}<span class="json-type">{kind} [{size}]</span></summary>
		{#if expanded}<div class="json-children">{#each entries as [key, child] (key)}<JsonTree value={child} name={key} depth={depth + 1} />{/each}{#if !size}<span class="muted">empty {kind}</span>{/if}</div>
		{#if size > pageSize}<div class="table-pagination" aria-label={`${name || kind} entries`}><button disabled={page === 0} onclick={() => page--}>Previous entries</button><span>{page * pageSize + 1}–{Math.min((page + 1) * pageSize, size)} / {size}</span><button disabled={(page + 1) * pageSize >= size} onclick={() => page++}>Next entries</button></div>{/if}{/if}
	</details>
{:else}
	<div class="json-leaf">{#if name}<span class="json-key">{name}: </span>{/if}<span class="json-type">{kind}</span> <span class:json-string={kind === 'string'}>{structured ? '[maximum display depth]' : JSON.stringify(value) ?? 'unavailable'}</span></div>
{/if}
