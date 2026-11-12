<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import ArrowUpIcon from "@lucide/svelte/icons/arrow-up";
	import { Button } from "$lib/components/ui/button/index.js";
</script>

<div class="flex flex-wrap items-center gap-2 md:flex-row">
	<Button variant="outline">Button</Button>
	<Button variant="outline" size="icon" aria-label="Submit">
		<ArrowUpIcon />
	</Button>
</div>
