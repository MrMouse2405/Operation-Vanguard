<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	let selected = $state('Overview');
</script>
<Sidebar.Provider class="min-h-64 w-full overflow-hidden rounded-md border" style="--sidebar-width: 11rem;">
	<Sidebar.Root collapsible="none" class="min-h-64">
		<Sidebar.Header class="px-4 py-3 font-semibold">Workspace</Sidebar.Header>
		<Sidebar.Content>
			<Sidebar.Group><Sidebar.GroupLabel>Demo navigation</Sidebar.GroupLabel>
				<Sidebar.Menu>
					{#each ['Overview', 'Agents', 'Settings'] as label (label)}
						<Sidebar.MenuItem><Sidebar.MenuButton isActive={selected === label} onclick={() => selected = label}>{label}</Sidebar.MenuButton></Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.Group>
		</Sidebar.Content>
	</Sidebar.Root>
	<div class="grid flex-1 place-items-center p-4 text-sm">{selected}</div>
</Sidebar.Provider>
