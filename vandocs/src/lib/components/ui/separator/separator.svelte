<script lang="ts">
  import { Separator as SeparatorPrimitive } from 'bits-ui';
  import { cn } from '$lib/utils';
  let { class:className, ...rest }: SeparatorPrimitive.RootProps = $props();
</script>
<SeparatorPrimitive.Root data-slot="separator" class={cn('shrink-0 bg-border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px',className)} {...rest}/>
