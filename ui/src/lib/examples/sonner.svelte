<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { toast } from "svelte-sonner";
	import { Button } from "$lib/components/ui/button/index.js";
</script>

<Button
	variant="outline"
	onclick={() =>
		toast("Event has been created", {
			description: "Sunday, December 03, 2023 at 9:00 AM",
			action: {
				label: "Undo",
				onClick: () => console.info("Undo"),
			},
		})}
>
	Show Toast
</Button>
