<script lang="ts">
	import * as CodeBlock from '$lib/components/ui/code-block/index.js';
	const code = 'local values = {5, 2, 4, 1, 3}\ntable.sort(values)\nprint(table.concat(values, ", "))';
</script>
<CodeBlock.Root class="w-full">
	<CodeBlock.Header>
		<CodeBlock.Title>agent.lua</CodeBlock.Title>
		<CodeBlock.Actions><CodeBlock.CopyButton value={code} /></CodeBlock.Actions>
	</CodeBlock.Header>
	<CodeBlock.Content {code} wrap lineNumbers />
</CodeBlock.Root>
