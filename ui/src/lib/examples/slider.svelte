<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { Slider } from "$lib/components/ui/slider/index.js";

	let value = $state(75);
</script>

<Slider type="single" bind:value max={100} step={1} class="mx-auto w-full max-w-xs" />
