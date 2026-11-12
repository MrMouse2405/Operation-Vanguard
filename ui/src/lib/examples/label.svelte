<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
</script>

<div class="flex gap-2">
	<Checkbox id="terms" />
	<Label for="terms">Accept terms and conditions</Label>
</div>
