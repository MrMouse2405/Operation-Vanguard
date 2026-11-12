<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as Dialog from "$lib/components/ui/dialog/index.js";
	import * as Field from "$lib/components/ui/field/index.js";
	import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
</script>

<Dialog.Root>
	<form>
		<Dialog.Trigger type="button" class={buttonVariants({ variant: "outline" })}>
			Open Dialog
		</Dialog.Trigger>
		<Dialog.Content class="sm:max-w-sm">
			<Dialog.Header>
				<Dialog.Title>Edit profile</Dialog.Title>
				<Dialog.Description>
					Make changes to your profile here. Click save when you&apos;re done.
				</Dialog.Description>
			</Dialog.Header>
			<Field.Group>
				<Field.Field>
					<Label for="name-1">Name</Label>
					<Input id="name-1" name="name" defaultValue="Pedro Duarte" />
				</Field.Field>
				<Field.Field>
					<Label for="username-1">Username</Label>
					<Input id="username-1" name="username" defaultValue="@peduarte" />
				</Field.Field>
			</Field.Group>
			<Dialog.Footer>
				<Dialog.Close type="button" class={buttonVariants({ variant: "outline" })}>
					Cancel
				</Dialog.Close>
				<Button type="submit">Save changes</Button>
			</Dialog.Footer>
		</Dialog.Content>
	</form>
</Dialog.Root>
