<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";

	const tags = Array.from({ length: 50 }).map((_, i, a) => `v1.2.0-beta.${a.length - i}`);
</script>

<ScrollArea class="h-72 w-48 rounded-md border">
	<div class="p-4">
		<h4 class="mb-4 text-sm leading-none font-medium">Tags</h4>
		{#each tags as tag (tag)}
			<div class="text-sm">
				{tag}
			</div>
			<Separator class="my-2" />
		{/each}
	</div>
</ScrollArea>
