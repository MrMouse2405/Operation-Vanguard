<script lang="ts">
  import JsonTree from './JsonTree.svelte';
  import PaneResize from './PaneResize.svelte';
  import type { Snapshot } from './types';
  let { snapshot }: { snapshot: Snapshot | null } = $props();
  let left = $state('');
  let right = $state('');
  let pageLeft = $state(0);
  let pageRight = $state(0);
  let width = $state(1200);
  let columns = $derived(width<800?4:8);
  let pairWidth = $state(1200);
  let split = $state(0.5);
  let workspaceHeight = $state(550);
  let leftSize = $derived(Math.max(240, Math.min(pairWidth-247, (pairWidth-7)*split)));
  let options = $derived(Object.entries(snapshot?.debug?.agents ?? {}).flatMap(([agent, views]) => Object.entries(views as Record<string, any>).map(([name, view]) => ({ key:`${agent}/${name}`, label:`A${agent} · ${name}`, view }))));
  let selections = $derived([options.find(v=>v.key===left)??options[0], options.find(v=>v.key===right)??options[1]??options[0]]);
</script>
<svelte:window bind:innerWidth={width}/>
<section class="debug-workspace" aria-label="Debug network comparison" style:height={`${workspaceHeight}px`}>
  <header><h2>Agent drawings</h2><p>Agent-authored claims, not engine truth. Compare two reconstructions at tick {snapshot?.tick ?? 0}. Up to 80 nodes per page; edges appear when both endpoints are on the page.</p></header>
  {#if !options.length}<p class="empty">No drawings yet. Start an agent using dbgguard.view, node and edge. Each router owns its own views.</p>{:else}
  <div class="debug-pair" bind:clientWidth={pairWidth} style={`--debug-left:${leftSize}px`}>
    {#each selections as selection, index}
      {#if index===1&&width>800}<PaneResize label="Resize debug perspectives" value={leftSize} min={240} max={Math.max(240,pairWidth-247)} onresize={value=>split=value/(pairWidth-7)} onreset={()=>split=0.5}/>{/if}
      <section class="debug-pane" aria-label={index===0?'Left debug view':'Right debug view'}>
        <label>Perspective <select aria-label={index===0?'Left debug view':'Right debug view'} value={selection?.key} onchange={e=>{if(index===0){left=e.currentTarget.value;pageLeft=0;}else{right=e.currentTarget.value;pageRight=0;}}}>{#each options as option}<option value={option.key}>{option.label}</option>{/each}</select></label>
        {#if selection}
          {@const view=selection.view}
          {@const all=Object.values(view.nodes) as any[]}
          {@const pages=Math.max(1,Math.ceil(all.length/80))}
          {@const current=Math.min(index===0?pageLeft:pageRight,pages-1)}
          {@const nodes=all.slice(current*80,current*80+80)}
          {@const ids=new Set(nodes.map(n=>n.id))}
          {@const edges=(Object.values(view.edges) as any[]).filter(e=>ids.has(e.from)&&ids.has(e.to))}
          <p class="debug-meta">Updated T{view.updated_at} · {all.length} nodes · {Object.keys(view.edges).length} edges</p>
          <div class="debug-pagination"><button disabled={current===0} onclick={()=>index===0?pageLeft=current-1:pageRight=current-1}>Previous</button><span>Page {current+1} / {pages}</span><button disabled={current+1>=pages} onclick={()=>index===0?pageLeft=current+1:pageRight=current+1}>Next</button></div>
          <svg class="debug-graph" viewBox={`0 0 ${columns*75} ${Math.max(160,Math.ceil(nodes.length/columns)*65+40)}`} role="img" aria-label={`Network drawing: ${selection.label}`}>
            <title>{selection.label} — agent-authored network</title>
            {#each edges as edge}
              {@const a=nodes.findIndex(n=>n.id===edge.from)}{@const b=nodes.findIndex(n=>n.id===edge.to)}
              <line x1={35+(a%columns)*75} y1={30+Math.floor(a/columns)*65} x2={35+(b%columns)*75} y2={30+Math.floor(b/columns)*65}><title>{edge.id}: {JSON.stringify(edge.info)}</title></line>
            {/each}
            {#each nodes as node, i}<g><circle cx={35+(i%columns)*75} cy={30+Math.floor(i/columns)*65} r="12"/><text x={35+(i%columns)*75} y={56+Math.floor(i/columns)*65} text-anchor="middle">{node.id.length>10?`${node.id.slice(0,9)}…`:node.id}</text><title>{node.id}: {JSON.stringify(node.info)}</title></g>{/each}
          </svg>
          <details open><summary>View notes</summary><JsonTree value={view.info}/></details>
          <details><summary>Node information ({nodes.length} on this page)</summary>{#each nodes as node}<details><summary>{node.id}</summary><JsonTree value={node.info}/></details>{/each}</details>
          <details><summary>Edge information ({edges.length} on this page)</summary>{#each edges as edge}<details><summary>{edge.id}: {edge.from} → {edge.to}</summary><JsonTree value={edge.info}/></details>{/each}</details>
        {/if}
      </section>
    {/each}
  </div>{/if}
</section>
<PaneResize label="Resize agent drawings" axis="y" value={workspaceHeight} min={180} max={1000} onresize={value=>workspaceHeight=value} onreset={()=>workspaceHeight=550}/>
<style>
  .debug-workspace { padding:20px; border-bottom:1px solid var(--line); background:var(--panel); min-height:180px; overflow:auto; flex-shrink:0; }
  h2 { font-size:18px; margin:0 0 8px; } p { color:var(--muted); font-size:12px; line-height:1.6; }
  .debug-pair { display:grid; grid-template-columns:var(--debug-left) 7px minmax(0,1fr); }
  .debug-pane { min-width:0; padding:14px; border:1px solid var(--line); }
  label { display:flex; flex-wrap:wrap; align-items:center; gap:10px; } select { max-width:100%; }
  .debug-meta { margin:10px 0; } .debug-pagination { display:flex; justify-content:space-between; gap:8px; align-items:center; font-size:12px; }
  .debug-graph { width:100%; background:var(--bg); margin:14px 0; }
  line { stroke:var(--muted); stroke-width:1.5; } circle { fill:var(--panel); stroke:var(--blue); stroke-width:2; } text { fill:var(--text); font:10px monospace; }
  details { padding:8px 0; overflow-wrap:anywhere; font-size:12px; } summary { cursor:pointer; }
  @media(max-width:800px) { .debug-pair { grid-template-columns:1fr; } .debug-workspace { padding:12px; } }
</style>
