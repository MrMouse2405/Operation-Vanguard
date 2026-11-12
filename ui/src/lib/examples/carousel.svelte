<!-- Adapted from shadcn-svelte 3c4152d7d3904e76f6f756ec8ffca7c9ec72f977; MIT, see THIRD_PARTY.md. -->
<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import * as Carousel from "$lib/components/ui/carousel/index.js";
</script>

<Carousel.Root class="w-full max-w-[12rem] sm:max-w-xs">
	<Carousel.Content>
		{#each Array(5), i}
			<Carousel.Item>
				<div class="p-1">
					<Card.Root>
						<Card.Content class="flex aspect-square items-center justify-center p-6">
							<span class="text-4xl font-semibold">{i + 1}</span>
						</Card.Content>
					</Card.Root>
				</div>
			</Carousel.Item>
		{/each}
	</Carousel.Content>
	<Carousel.Previous />
	<Carousel.Next />
</Carousel.Root>
