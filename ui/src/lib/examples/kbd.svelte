<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as Kbd from "$lib/components/ui/kbd/index.js";
</script>

<div class="flex flex-col items-center gap-4">
	<Kbd.Group>
		<Kbd.Root>⌘</Kbd.Root>
		<Kbd.Root>⇧</Kbd.Root>
		<Kbd.Root>⌥</Kbd.Root>
		<Kbd.Root>⌃</Kbd.Root>
	</Kbd.Group>
	<Kbd.Group>
		<Kbd.Root>Ctrl</Kbd.Root>
		<span>+</span>
		<Kbd.Root>B</Kbd.Root>
	</Kbd.Group>
</div>
