<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  let { ref=$bindable(null), value=$bindable(''), class:className, ...rest }: HTMLInputAttributes & { ref?: HTMLInputElement|null } = $props();
</script>
<input bind:this={ref} bind:value data-slot="input" class={cn('flex h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-sm disabled:cursor-not-allowed disabled:opacity-50',className)} {...rest}/>
