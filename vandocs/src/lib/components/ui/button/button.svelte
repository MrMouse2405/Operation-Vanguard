<script lang="ts" module>
  import { tv } from 'tailwind-variants';
  export const buttonVariants = tv({
    base: 'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
    variants: {
      variant: { default: 'bg-primary text-primary-foreground hover:bg-primary/90', outline: 'border border-input bg-background hover:bg-accent', ghost: 'hover:bg-accent hover:text-accent-foreground' },
      size: { default: 'h-9 px-4 py-2', sm: 'h-8 rounded-md px-3 text-xs', icon: 'size-9' }
    },
    defaultVariants: { variant: 'default', size: 'default' }
  });
</script>
<script lang="ts">
  import type { HTMLButtonAttributes, HTMLAnchorAttributes } from 'svelte/elements';
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';
  let { class: className, variant = 'default', size = 'default', href, children, ...rest }: (HTMLButtonAttributes & HTMLAnchorAttributes) & { variant?: 'default'|'outline'|'ghost'; size?: 'default'|'sm'|'icon'; children?: Snippet } = $props();
</script>
{#if href}<a data-slot="button" class={cn(buttonVariants({variant,size}),className)} {href} {...rest}>{@render children?.()}</a>
{:else}<button type="button" data-slot="button" class={cn(buttonVariants({variant,size}),className)} {...rest}>{@render children?.()}</button>{/if}
