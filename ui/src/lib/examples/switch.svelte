<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { Label } from "$lib/components/ui/label/index.js";
	import { Switch } from "$lib/components/ui/switch/index.js";
</script>

<div class="flex items-center space-x-2">
	<Switch id="airplane-mode" />
	<Label for="airplane-mode">Airplane Mode</Label>
</div>
