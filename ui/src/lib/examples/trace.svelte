<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import FileCodeIcon from '@lucide/svelte/icons/file-code';
	import * as Trace from '$lib/components/ui/trace/index.js';
</script>
<div class="w-full space-y-4">
	<p class="text-xs text-muted-foreground">Illustrative agent trace</p>
	<Trace.Root>
		<Trace.Line><Trace.Icon><FileCodeIcon /></Trace.Icon><Trace.Title>Loaded agent.lua</Trace.Title></Trace.Line>
		<Trace.Collapsible>
			<Trace.CollapsibleTrigger><Trace.Line><Trace.Icon><CheckIcon /></Trace.Icon><Trace.Title>Sorted 5 values</Trace.Title></Trace.Line></Trace.CollapsibleTrigger>
			<Trace.CollapsibleContent>1, 2, 3, 4, 5</Trace.CollapsibleContent>
		</Trace.Collapsible>
		<Trace.Line><Trace.Icon><CheckIcon /></Trace.Icon><Trace.Title>Agent completed</Trace.Title></Trace.Line>
	</Trace.Root>
</div>
