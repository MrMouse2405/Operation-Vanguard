<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { Skeleton } from "$lib/components/ui/skeleton/index.js";
</script>

<div class="flex items-center gap-4">
	<Skeleton class="size-12 rounded-full" />
	<div class="space-y-2">
		<Skeleton class="h-4 w-[250px]" />
		<Skeleton class="h-4 w-[200px]" />
	</div>
</div>
