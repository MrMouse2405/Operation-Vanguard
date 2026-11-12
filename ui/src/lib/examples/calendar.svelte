<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import { getLocalTimeZone, today, type CalendarDate } from "@internationalized/date";
	import { Calendar } from "$lib/components/ui/calendar/index.js";

	let value = $state<CalendarDate | undefined>(today(getLocalTimeZone()));
</script>

<Calendar type="single" bind:value class="rounded-lg border" captionLayout="dropdown" />
