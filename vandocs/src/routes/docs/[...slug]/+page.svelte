<script lang="ts">
  import Mailbox from '$lib/Mailbox.svelte';
  let { data } = $props();
</script>
<Mailbox slug={data.slug} />
