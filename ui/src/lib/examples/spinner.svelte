<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as Item from "$lib/components/ui/item/index.js";
	import { Spinner } from "$lib/components/ui/spinner/index.js";
</script>

<div class="flex w-full max-w-xs flex-col gap-4 [--radius:1rem]">
	<Item.Root variant="muted">
		<Item.Media>
			<Spinner />
		</Item.Media>
		<Item.Content>
			<Item.Title class="line-clamp-1">Processing payment...</Item.Title>
		</Item.Content>
		<Item.Content class="flex-none justify-end">
			<span class="text-sm tabular-nums">$100.00</span>
		</Item.Content>
	</Item.Root>
</div>
