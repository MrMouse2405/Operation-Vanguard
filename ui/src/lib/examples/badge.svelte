<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { Badge } from "$lib/components/ui/badge/index.js";
</script>

<div class="flex w-full flex-wrap justify-center gap-2">
	<Badge>Badge</Badge>
	<Badge variant="secondary">Secondary</Badge>
	<Badge variant="destructive">Destructive</Badge>
	<Badge variant="outline">Outline</Badge>
</div>
