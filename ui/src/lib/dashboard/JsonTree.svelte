<script lang="ts">
	import JsonTree from './JsonTree.svelte';
	let { value, name = '', depth = 0 }: { value: any; name?: string; depth?: number } = $props();
	let structured = $derived(value !== null && typeof value === 'object');
	let entries = $derived(structured ? Object.entries(value) : []);
	let kind = $derived(value === null ? 'null' : Array.isArray(value) ? 'array' : typeof value);
</script>
{#if structured && depth < 20}
	<details class="json-tree" open={depth === 0}>
		<summary>{#if name}<span class="json-key">{name}: </span>{/if}<span class="json-type">{kind} [{entries.length}]</span></summary>
		<div class="json-children">{#each entries as [key, child] (key)}<JsonTree value={child} name={key} depth={depth + 1} />{/each}{#if !entries.length}<span class="muted">empty {kind}</span>{/if}</div>
	</details>
{:else}
	<div class="json-leaf">{#if name}<span class="json-key">{name}: </span>{/if}<span class="json-type">{kind}</span> <span class:json-string={kind === 'string'}>{structured ? '[maximum display depth]' : JSON.stringify(value) ?? 'unavailable'}</span></div>
{/if}
