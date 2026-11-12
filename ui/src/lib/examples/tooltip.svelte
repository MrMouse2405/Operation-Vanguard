<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as Tooltip from "$lib/components/ui/tooltip/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
</script>

<Tooltip.Provider>
	<Tooltip.Root>
		<Tooltip.Trigger class={buttonVariants({ variant: "outline" })}>Hover</Tooltip.Trigger>
		<Tooltip.Content>
			<p>Add to library</p>
		</Tooltip.Content>
	</Tooltip.Root>
</Tooltip.Provider>
