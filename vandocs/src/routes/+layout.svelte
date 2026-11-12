<script lang="ts">
  import './mail.css';
  import { ModeWatcher } from 'mode-watcher';
  let { children } = $props();
</script>
<ModeWatcher defaultMode="light" />
{@render children()}
