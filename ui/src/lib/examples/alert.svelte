<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
	import InfoIcon from "@lucide/svelte/icons/info";
	import * as Alert from "$lib/components/ui/alert/index.js";
</script>

<div class="grid w-full max-w-md items-start gap-4">
	<Alert.Root>
		<CircleCheckIcon />
		<Alert.Title>Payment successful</Alert.Title>
		<Alert.Description>
			Your payment of $29.99 has been processed. A receipt has been sent to your email address.
		</Alert.Description>
	</Alert.Root>
	<Alert.Root>
		<InfoIcon />
		<Alert.Title>New feature available</Alert.Title>
		<Alert.Description>
			We've added dark mode support. You can enable it in your account settings.
		</Alert.Description>
	</Alert.Root>
</div>
