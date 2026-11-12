<script lang="ts">
	import * as Table from '$lib/components/ui/table/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Input } from '$lib/components/ui/input/index.js';
	import ArrowUpDownIcon from '@lucide/svelte/icons/arrow-up-down';
	let query = $state('');
	let ascending = $state(true);
	const rows = [{ name: 'explorer.lua', fuel: 755 }, { name: 'pathfinder.lua', fuel: 1240 }, { name: 'sorter.lua', fuel: 430 }];
	let filtered = $derived(rows.filter(row => row.name.includes(query.toLowerCase())).toSorted((a, b) => ascending ? a.fuel - b.fuel : b.fuel - a.fuel));
</script>
<div class="w-full space-y-3">
	<Input aria-label="Filter demo agents" placeholder="Filter demo agents…" bind:value={query} />
	<Table.Root><Table.Caption>Illustrative results · {filtered.length} rows</Table.Caption><Table.Header><Table.Row><Table.Head>Agent</Table.Head><Table.Head><Button variant="ghost" size="sm" onclick={() => ascending = !ascending}>Fuel <ArrowUpDownIcon class="size-3" /></Button></Table.Head></Table.Row></Table.Header>
		<Table.Body>{#each filtered as row (row.name)}<Table.Row><Table.Cell>{row.name}</Table.Cell><Table.Cell>{row.fuel}</Table.Cell></Table.Row>{:else}<Table.Row><Table.Cell colspan={2}>No matching agents.</Table.Cell></Table.Row>{/each}</Table.Body>
	</Table.Root>
</div>
