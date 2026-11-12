<script lang="ts">
	import * as MessageScroller from '$lib/components/ui/message-scroller/index.js';
	import { Button } from '$lib/components/ui/button/index.js';
	let count = $state(8);
</script>
<div class="w-full space-y-3">
	<MessageScroller.Provider>
		<MessageScroller.Root class="relative h-56 overflow-hidden rounded-md border">
			<MessageScroller.Viewport class="h-full">
				<MessageScroller.Content class="space-y-3 p-4">
					{#each Array.from({ length: count }, (_, i) => i + 1) as line (line)}
						<MessageScroller.Item><p class="rounded-md bg-muted p-3 text-sm">Demo output message {line}</p></MessageScroller.Item>
					{/each}
				</MessageScroller.Content>
			</MessageScroller.Viewport>
			<MessageScroller.Button />
		</MessageScroller.Root>
	</MessageScroller.Provider>
	<Button variant="outline" size="sm" onclick={() => count += 1}>Add message</Button>
</div>
