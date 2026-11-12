<script lang="ts">
	import FolderIcon from '@lucide/svelte/icons/folder';
	import * as Empty from '$lib/components/ui/empty/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import { toast } from 'svelte-sonner';
</script>
<Empty.Root>
	<Empty.Header>
		<Empty.Media variant="icon"><FolderIcon /></Empty.Media>
		<Empty.Title>No agents yet</Empty.Title>
		<Empty.Description>Your next experiment starts with a Lua script.</Empty.Description>
	</Empty.Header>
	<Empty.Content><Button onclick={() => toast('Example action', { description: 'Run an agent from the Vanguard CLI.' })}>Create an agent</Button></Empty.Content>
</Empty.Root>
