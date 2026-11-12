<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import BoldIcon from "@lucide/svelte/icons/bold";
	import ItalicIcon from "@lucide/svelte/icons/italic";
	import UnderlineIcon from "@lucide/svelte/icons/underline";
	import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
</script>

<ToggleGroup.Root variant="outline" type="multiple">
	<ToggleGroup.Item value="bold" aria-label="Toggle bold">
		<BoldIcon />
	</ToggleGroup.Item>
	<ToggleGroup.Item value="italic" aria-label="Toggle italic">
		<ItalicIcon />
	</ToggleGroup.Item>
	<ToggleGroup.Item value="strikethrough" aria-label="Toggle strikethrough">
		<UnderlineIcon />
	</ToggleGroup.Item>
</ToggleGroup.Root>
