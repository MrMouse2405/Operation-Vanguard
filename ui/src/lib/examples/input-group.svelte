<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import SearchIcon from "@lucide/svelte/icons/search";
	import * as InputGroup from "$lib/components/ui/input-group/index.js";
</script>

<InputGroup.Root class="max-w-xs">
	<InputGroup.Input placeholder="Search..." />
	<InputGroup.Addon>
		<SearchIcon />
	</InputGroup.Addon>
	<InputGroup.Addon align="inline-end">12 results</InputGroup.Addon>
</InputGroup.Root>
