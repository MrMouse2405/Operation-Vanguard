<script lang="ts">
  let { label, value, min, max, axis = 'x', reverse = false, onresize, onreset }:
    { label:string; value:number; min:number; max:number; axis?:'x'|'y'; reverse?:boolean; onresize:(value:number)=>void; onreset?:()=>void } = $props();
  let drag: { id:number; start:number; value:number } | null = null;
  function update(next:number) { onresize(Math.round(Math.max(min, Math.min(max, next)))); }
  function down(event:PointerEvent) {
    if(event.button!==0)return;
    event.preventDefault();
    drag={id:event.pointerId,start:axis==='x'?event.clientX:event.clientY,value};
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }
  function move(event:PointerEvent) {
    if(drag?.id!==event.pointerId)return;
    update(drag.value+((axis==='x'?event.clientX:event.clientY)-drag.start)*(reverse?-1:1));
  }
  function keys(event:KeyboardEvent) {
    const delta=event.shiftKey?50:10;
    if(event.key==='Home'){event.preventDefault();update(min);}
    else if(event.key==='End'){event.preventDefault();update(max);}
    else if((axis==='x'?['ArrowLeft','ArrowRight']:['ArrowUp','ArrowDown']).includes(event.key)){
      event.preventDefault();update(value+(event.key==='ArrowLeft'||event.key==='ArrowUp'?-delta:delta)*(reverse?-1:1));
    }
  }
</script>

<!-- A focusable separator is an interactive WAI-ARIA window splitter. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div class="pane-resize" class:vertical={axis==='y'} role="separator" tabindex="0"
  aria-label={label} aria-orientation={axis==='x'?'vertical':'horizontal'}
  aria-valuenow={Math.round(value)} aria-valuemin={Math.round(min)} aria-valuemax={Math.round(max)}
  title={`${label} · drag or use arrow keys · double-click to reset`}
  onpointerdown={down} onpointermove={move} onpointerup={()=>drag=null} onpointercancel={()=>drag=null}
  onlostpointercapture={()=>drag=null} onkeydown={keys} ondblclick={onreset}></div>

<style>
  .pane-resize { flex-shrink:0; width:7px; background:var(--bg); border-inline:1px solid var(--line); cursor:col-resize; touch-action:none; position:relative; z-index:2; }
  .pane-resize::after { content:''; position:absolute; width:3px; height:30px; background:var(--muted); opacity:.5; inset:0; margin:auto; border-radius:2px; }
  .pane-resize:hover,.pane-resize:focus-visible { background:var(--selected,#20333e); outline:1px solid var(--green); outline-offset:-1px; }
  .pane-resize.vertical { width:100%; height:7px; border-inline:0; border-block:1px solid var(--line); cursor:row-resize; }
  .pane-resize.vertical::after { width:30px; height:3px; }
</style>
