<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as Field from "$lib/components/ui/field/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
</script>

<Field.Field>
	<Field.Label for="input-demo-api-key">API Key</Field.Label>
	<Input id="input-demo-api-key" type="password" placeholder="sk-..." />
	<Field.Description>Your API key is encrypted and stored securely.</Field.Description>
</Field.Field>
