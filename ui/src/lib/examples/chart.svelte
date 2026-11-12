<script lang="ts">
	import { scaleBand } from 'd3-scale';
	import { BarChart } from 'layerchart';
	import * as Chart from '$lib/components/ui/chart/index.js';
	const data = [
		{ run: 'A', fuel: 186 }, { run: 'B', fuel: 305 }, { run: 'C', fuel: 237 },
		{ run: 'D', fuel: 73 }, { run: 'E', fuel: 209 }, { run: 'F', fuel: 214 }
	];
	const config = { fuel: { label: 'Fuel', color: 'var(--foreground)' } } satisfies Chart.ChartConfig;
</script>

<div class="w-full space-y-4">
	<p class="text-sm font-medium">Fuel by run <span class="text-muted-foreground font-normal">· demo data</span></p>
	<Chart.Container {config} class="h-52 w-full">
		<BarChart {data} x="run" xScale={scaleBand().padding(0.3)} axis="x"
			series={[{ key: 'fuel', label: 'Fuel', color: config.fuel.color }]}
			props={{ bars: { stroke: 'none', rounded: 'all', radius: 4 } }}>
			{#snippet tooltip()}<Chart.Tooltip />{/snippet}
		</BarChart>
	</Chart.Container>
</div>
