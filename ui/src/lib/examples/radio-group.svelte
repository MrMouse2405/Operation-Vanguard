<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as RadioGroup from "$lib/components/ui/radio-group/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
</script>

<RadioGroup.Root value="comfortable" class="w-fit">
	<div class="flex items-center gap-3">
		<RadioGroup.Item value="default" id="r1" />
		<Label for="r1">Default</Label>
	</div>
	<div class="flex items-center gap-3">
		<RadioGroup.Item value="comfortable" id="r2" />
		<Label for="r2">Comfortable</Label>
	</div>
	<div class="flex items-center gap-3">
		<RadioGroup.Item value="compact" id="r3" />
		<Label for="r3">Compact</Label>
	</div>
</RadioGroup.Root>
