<script lang="ts">
	import { Calendar } from '$lib/components/ui/calendar/index.js';
	import * as Popover from '$lib/components/ui/popover/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import type { DateValue } from '@internationalized/date';
	let value = $state<DateValue>();
</script>
<Popover.Root>
	<Popover.Trigger>{#snippet child({ props })}<Button {...props} variant="outline"><CalendarIcon class="size-4" />{value?.toString() ?? 'Pick a date'}</Button>{/snippet}</Popover.Trigger>
	<Popover.Content class="w-auto p-0"><Calendar type="single" bind:value captionLayout="dropdown" /></Popover.Content>
</Popover.Root>
