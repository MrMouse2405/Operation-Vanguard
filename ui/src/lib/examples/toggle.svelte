<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import BookmarkIcon from "@lucide/svelte/icons/bookmark";
	import { Toggle } from "$lib/components/ui/toggle/index.js";
</script>

<Toggle aria-label="Toggle bookmark" size="sm" variant="outline">
	<BookmarkIcon class="group-aria-pressed/toggle:fill-foreground" />
	Bookmark
</Toggle>
