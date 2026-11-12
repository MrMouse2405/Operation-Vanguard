<script lang="ts">
  import { onMount } from 'svelte';
  import { ModeWatcher } from 'mode-watcher';
  import Mailbox from '../src/lib/Mailbox.svelte';
  let slug = $state<string | undefined>(undefined);
  function navigate() {
    const route = decodeURIComponent(location.hash.slice(1));
    slug = route.startsWith('/docs/') ? route.slice(6).replace(/\/$/, '') : undefined;
  }
  onMount(() => { navigate(); });
</script>
<svelte:window onhashchange={navigate}/>
<ModeWatcher defaultMode="light" />
<Mailbox {slug} offline/>
