<script lang="ts">
	import type { Snippet } from 'svelte';
	let { open = $bindable(false), title, children }: { open?: boolean; title: string; children: Snippet } = $props();
	let dialog: HTMLDialogElement;
	const titleId = $props.id();
	let previous: HTMLElement | null = null;
	$effect(() => {
		if (!dialog) return;
		if (open && !dialog.open) { previous = document.activeElement as HTMLElement; dialog.showModal(); }
		else if (!open && dialog.open) { dialog.close(); previous?.focus(); }
	});
	function close() { open = false; }
</script>
<dialog bind:this={dialog} class="dash-dialog cut-surface cut-card" aria-labelledby={titleId} oncancel={close} onclose={() => { open = false; previous?.focus(); }}>
	<header><h2 id={titleId}>{title}</h2><button class="dash-button" onclick={close}>Close</button></header>
	<div class="dialog-body">{@render children()}</div>
</dialog>
