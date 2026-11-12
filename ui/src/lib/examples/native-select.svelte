<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as NativeSelect from "$lib/components/ui/native-select/index.js";
</script>

<NativeSelect.Root>
	<NativeSelect.Option value="">Select status</NativeSelect.Option>
	<NativeSelect.Option value="todo">Todo</NativeSelect.Option>
	<NativeSelect.Option value="in-progress">In Progress</NativeSelect.Option>
	<NativeSelect.Option value="done">Done</NativeSelect.Option>
	<NativeSelect.Option value="cancelled">Cancelled</NativeSelect.Option>
</NativeSelect.Root>
