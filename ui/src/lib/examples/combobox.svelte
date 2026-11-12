<script lang="ts">
	import * as Popover from '$lib/components/ui/popover/index.js';
	import * as Command from '$lib/components/ui/command/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	let open = $state(false);
	let value = $state('');
</script>
<Popover.Root bind:open>
	<Popover.Trigger>
		{#snippet child({ props })}<Button {...props} variant="outline" role="combobox" aria-expanded={open} class="w-56 justify-between">{value || 'Select a framework'}<ChevronsUpDownIcon class="size-4" /></Button>{/snippet}
	</Popover.Trigger>
	<Popover.Content class="w-56 p-0">
		<Command.Root><Command.Input placeholder="Search frameworks…" /><Command.List><Command.Empty>No framework found.</Command.Empty><Command.Group>
			{#each ['SvelteKit', 'Astro', 'Next.js', 'Nuxt'] as framework (framework)}
				<Command.Item value={framework} onSelect={() => { value = framework; open = false; }}>{framework}</Command.Item>
			{/each}
		</Command.Group></Command.List></Command.Root>
	</Popover.Content>
</Popover.Root>
