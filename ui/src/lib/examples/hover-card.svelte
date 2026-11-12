<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as HoverCard from "$lib/components/ui/hover-card/index.js";
	import { buttonVariants } from "$lib/components/ui/button/index.js";
</script>

<HoverCard.Root openDelay={10} closeDelay={100}>
	<HoverCard.Trigger class={buttonVariants({ variant: "link" })}>Hover Here</HoverCard.Trigger>
	<HoverCard.Content class="flex w-64 flex-col gap-0.5">
		<div class="font-semibold">@sveltejs</div>
		<div>Cybernetically enhanced web apps – created and maintained by @vercel.</div>
		<div class="mt-1 text-xs text-muted-foreground">Joined December 2021</div>
	</HoverCard.Content>
</HoverCard.Root>
