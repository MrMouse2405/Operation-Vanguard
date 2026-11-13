<script lang="ts">
	import { onMount } from 'svelte';
	import { Pause, Play, RotateCcw, ArrowRight, Globe2 } from '@lucide/svelte';
	import type { Snapshot } from './types';
	import { networkFlows,flowColors } from './flows';
	import { coastlines } from './globe-earth';
	let {snapshot,scope,selected,matchingIds,onselect,onscope}:{snapshot:Snapshot|null;scope:number|null;selected:number|null;matchingIds:Set<number>;onselect:(id:number)=>void;onscope:(id:number|null)=>void}=$props();
	let canvas:HTMLCanvasElement;let container:HTMLDivElement;
	let rotate=$state(true);let kind=$state('all');let page=$state(0);let hovered=$state('');
	let width=720,height=520,yaw=-.35,pitch=.15,zoom=1,drag:{x:number;y:number;moved:boolean}|null=null;
	let invalidate=()=>{};
	let byId=$derived(new Map((snapshot?.nodes??[]).map(n=>[n.id,n])));
	let routers=$derived(snapshot?.nodes.filter(n=>n.type==='ROUTER')??[]);
	let flows=$derived(networkFlows(snapshot));
	let filtered=$derived(flows.filter(f=>{
		const a=byId.get(f.source),b=byId.get(f.target);
		return (kind==='all'||kind===f.kind)&&(scope===null||a?.router_id===scope||b?.router_id===scope)&&(matchingIds.has(f.source)||matchingIds.has(f.target)||a?.type==='ROUTER'&&b?.type==='ROUTER');
	}));
	let drawn=$derived.by(()=>{
		const priority=filtered.toSorted((a,b)=>Number(b.source===selected||b.target===selected)-Number(a.source===selected||a.target===selected));
		const tasks=priority.filter(f=>f.kind==='task'),connections=priority.filter(f=>f.kind!=='task');
		const taskLimit=Math.min(tasks.length,Math.max(80,160-connections.length));
		return [...tasks.slice(0,taskLimit),...connections.slice(0,160-taskLimit)];
	});
	let pages=$derived(Math.max(1,Math.ceil(filtered.length/8)));
	let runId=$derived(snapshot?.run_id);
	$effect(()=>{kind;scope;runId;page=0;});
	$effect(()=>{if(page>=pages)page=pages-1;});
	type Vec=[number,number,number];
	const sphere=(lon:number,lat:number):Vec=>{const a=lon*Math.PI/180,b=lat*Math.PI/180;return [Math.cos(b)*Math.sin(a),Math.sin(b),Math.cos(b)*Math.cos(a)];};
	// Stable illustrative land-based anchors, unrelated to a network's real location.
	const anchors=[[-74,41],[2,49],[104,1],[-122,38],[139,36],[18,-34],[-47,-23],[151,-34],[77,28],[-99,19],[31,30],[37,56],[121,31],[-3,40],[28,-26],[-70,-33],[55,25],[-79,44],[127,38],[13,52],[-58,-35],[116,40],[-9,39],[101,14],[-123,49],[29,41],[174,-37],[73,19],[-77,-12],[106,21],[7,46],[36,-1],[67,25],[-118,34],[10,60],[106,-6],[-84,10],[46,24],[15,-4],[121,14]];
	let locations=$derived.by(()=>new Map(routers.map((r,i)=>[r.id,anchors[i%anchors.length].map((n,k)=>n+Math.floor(i/anchors.length)*(k?1.7:2.7))])));
	function position(id:number):Vec{
		const n=byId.get(id),anchor=locations.get(n?.router_id??id)??[0,0];
		if(n?.type==='ROUTER')return sphere(anchor[0],anchor[1]);
		const angle=(id*137.508)%360*Math.PI/180,spread=2.8+(id%7)*.55;
		return sphere(anchor[0]+Math.cos(angle)*spread/Math.max(.4,Math.cos(anchor[1]*Math.PI/180)),anchor[1]+Math.sin(angle)*spread);
	}
	function project(v:Vec,alt=1){
		const x=v[0]*Math.cos(yaw)+v[2]*Math.sin(yaw),z=v[2]*Math.cos(yaw)-v[0]*Math.sin(yaw);
		const y=v[1]*Math.cos(pitch)-z*Math.sin(pitch),depth=z*Math.cos(pitch)+v[1]*Math.sin(pitch);
		const r=Math.min(width*.43,height*.39)*zoom;
		return {x:width/2+x*r*alt,y:height/2-y*r*alt,z:depth,visible:depth>=0||Math.hypot(x*alt,y*alt)>1};
	}
	function interpolate(a:Vec,b:Vec,t:number):Vec{
		const dot=Math.max(-.99999,Math.min(.99999,a.reduce((sum,n,i)=>sum+n*b[i],0))),angle=Math.acos(dot),sin=Math.sin(angle);
		return a.map((n,i)=>(n*Math.sin((1-t)*angle)+b[i]*Math.sin(t*angle))/sin) as Vec;
	}
	let geometry=$derived(drawn.map(flow=>({flow,points:Array.from({length:41},(_,i)=>{const t=i/40;return {v:interpolate(position(flow.source),position(flow.target),t),alt:1+Math.sin(t*Math.PI)*(flow.kind==='task'?.08:.22)};})})));
	let hits:{x:number;y:number;id:number;label:string}[]=[];
	function focusLAN(id:number|null){onscope(id);if(id!==null){const location=locations.get(id);if(location){yaw=-location[0]*Math.PI/180;pitch=location[1]*Math.PI/180;}}invalidate();}
	function down(event:PointerEvent){drag={x:event.clientX,y:event.clientY,moved:false};canvas.setPointerCapture(event.pointerId);}
	function move(event:PointerEvent){
		if(drag){const dx=event.clientX-drag.x,dy=event.clientY-drag.y;if(Math.abs(dx)+Math.abs(dy)>2)drag.moved=true;yaw+=dx*.006;pitch=Math.max(-1.2,Math.min(1.2,pitch+dy*.005));drag.x=event.clientX;drag.y=event.clientY;invalidate();}
		else{const rect=canvas.getBoundingClientRect(),hit=hits.find(h=>Math.hypot(h.x-(event.clientX-rect.left),h.y-(event.clientY-rect.top))<13);hovered=hit?.label??'';canvas.style.cursor=hit?'pointer':'grab';}
	}
	function up(event:PointerEvent){if(drag&&!drag.moved){const rect=canvas.getBoundingClientRect();const hit=hits.find(h=>Math.hypot(h.x-event.clientX+rect.left,h.y-event.clientY+rect.top)<14);if(hit)onselect(hit.id);}drag=null;if(canvas.hasPointerCapture(event.pointerId))canvas.releasePointerCapture(event.pointerId);}
	function keys(event:KeyboardEvent){if(['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','+','-'].includes(event.key)){event.preventDefault();if(event.key==='ArrowLeft')yaw-=.15;if(event.key==='ArrowRight')yaw+=.15;if(event.key==='ArrowUp')pitch=Math.min(1.2,pitch+.1);if(event.key==='ArrowDown')pitch=Math.max(-1.2,pitch-.1);if(event.key==='+')zoom=Math.min(1.6,zoom+.1);if(event.key==='-')zoom=Math.max(.65,zoom-.1);invalidate();}}
	onMount(()=>{
		const context=canvas.getContext('2d');if(!context)return;
		let frame=0,last=0,dirty=true,visible=true;
		const reduced=matchMedia('(prefers-reduced-motion: reduce)');
		function inside(lon:number,lat:number,polygon:number[][]){let yes=false;for(let i=0,j=polygon.length-1;i<polygon.length;j=i++){const [x,y]=polygon[i],[u,v]=polygon[j];if((y>lat)!==(v>lat)&&lon<(u-x)*(lat-y)/(v-y)+x)yes=!yes;}return yes;}
		const dots:Vec[]=[];for(let lat=-80;lat<82;lat+=2.6)for(let lon=-180;lon<180;lon+=2.6/Math.max(.25,Math.cos(lat*Math.PI/180)))if(coastlines.some(p=>inside(lon,lat,p)))dots.push(sphere(lon,lat));
		const grid:Vec[][]=[];for(let lat=-60;lat<=60;lat+=30)grid.push(Array.from({length:121},(_,i)=>sphere(i*3-180,lat)));for(let lon=-180;lon<180;lon+=30)grid.push(Array.from({length:61},(_,i)=>sphere(lon,i*3-90)));
		invalidate=()=>{dirty=true;};
		const resize=new ResizeObserver(([entry])=>{width=entry.contentRect.width;height=entry.contentRect.height;const dpr=Math.min(devicePixelRatio||1,2);canvas.width=Math.round(width*dpr);canvas.height=Math.round(height*dpr);context.setTransform(dpr,0,0,dpr,0,0);dirty=true;});resize.observe(container);
		const observer=new IntersectionObserver(([entry])=>{visible=entry.isIntersecting;dirty=true;});observer.observe(container);
		function line(points:{x:number;y:number;visible:boolean}[],color:string,lineWidth:number){context!.strokeStyle=color;context!.lineWidth=lineWidth;context!.beginPath();let pen=false;for(const p of points){if(!p.visible){pen=false;continue;}if(pen)context!.lineTo(p.x,p.y);else context!.moveTo(p.x,p.y);pen=true;}context!.stroke();}
		function arrow(a:{x:number;y:number},b:{x:number;y:number},color:string){const angle=Math.atan2(b.y-a.y,b.x-a.x);context!.fillStyle=color;context!.beginPath();context!.moveTo(b.x,b.y);context!.lineTo(b.x-7*Math.cos(angle-.45),b.y-7*Math.sin(angle-.45));context!.lineTo(b.x-7*Math.cos(angle+.45),b.y-7*Math.sin(angle+.45));context!.fill();}
		function draw(time:number){
			frame=requestAnimationFrame(draw);if(time-last<33||!visible||document.hidden)return;
			const moving=rotate&&snapshot?.control==='RUNNING'&&!reduced.matches&&!drag;
			const animating=snapshot?.control==='RUNNING'&&!reduced.matches;
			if(!dirty&&!moving&&!animating)return;const dt=Math.min(50,time-last);last=time;dirty=false;if(moving)yaw+=dt*.000025;
			context!.clearRect(0,0,width,height);const cx=width/2,cy=height/2,r=Math.min(width*.43,height*.39)*zoom;
			const aura=context!.createRadialGradient(cx,cy,r*.9,cx,cy,r*1.16);aura.addColorStop(0,'#18384b00');aura.addColorStop(.55,'#397bb72b');aura.addColorStop(1,'#18384b00');context!.fillStyle=aura;context!.fillRect(0,0,width,height);
			const sea=context!.createRadialGradient(cx-r*.35,cy-r*.4,0,cx,cy,r);sea.addColorStop(0,'#152d42');sea.addColorStop(.7,'#0c1b2b');sea.addColorStop(1,'#060f1c');context!.fillStyle=sea;context!.beginPath();context!.arc(cx,cy,r,0,Math.PI*2);context!.fill();
			for(const points of grid)line(points.map(p=>project(p)),'#5f9ec31c',.65);
			for(const point of dots){const p=project(point);if(p.z<0)continue;context!.fillStyle=`rgba(108,166,193,${.2+p.z*.55})`;context!.beginPath();context!.arc(p.x,p.y,Math.max(.65,r/210),0,Math.PI*2);context!.fill();}
			context!.strokeStyle='#508fb15c';context!.lineWidth=1;context!.beginPath();context!.arc(cx,cy,r,0,Math.PI*2);context!.stroke();hits=[];
			for(const {flow,points} of geometry){const path=points.map(p=>project(p.v,p.alt)),color=flowColors[flow.kind];const focus=selected===flow.source||selected===flow.target;context!.globalAlpha=selected!==null&&!focus ? .32 : .8;line(path,color,focus?2.4:1.2);const tip=path[34],tail=path[32];if(tip.visible&&tail.visible)arrow(tail,tip,color);
				const index=animating?Math.floor((time/28+flow.source)%39)+1:22,pulse=path[index];if(pulse.visible){context!.fillStyle=color;context!.beginPath();context!.arc(pulse.x,pulse.y,focus?3:2,0,Math.PI*2);context!.fill();}
			}context!.globalAlpha=1;
			const endpoints=new Set(drawn.flatMap(f=>[f.source,f.target]));if(selected!==null)endpoints.add(selected);
			for(const id of endpoints){const n=byId.get(id),p=project(position(id));if(p.z<0||!n)continue;const color=n.type==='ROUTER'?flowColors.task:n.compromised?flowColors.compromised:n.exposure?flowColors.exposure:flowColors.connection;context!.fillStyle=color;context!.beginPath();context!.arc(p.x,p.y,selected===id?5:2.4,0,Math.PI*2);context!.fill();hits.unshift({x:p.x,y:p.y,id,label:`${n.type==='ROUTER'?'Router':'Host'} ${id} · LAN ${n.router_id}`});}
			for(const router of routers.slice(0,128)){const p=project(position(router.id));if(p.z<0)continue;const active=scope===router.id;context!.strokeStyle=active?'#ffffff':'#90b8fc';context!.lineWidth=active?2:1;context!.beginPath();context!.arc(p.x,p.y,active?7:4,0,Math.PI*2);context!.stroke();hits.push({x:p.x,y:p.y,id:router.id,label:`LAN ${router.lan_id??router.id} · Router ${router.id}`});if(active||routers.length<=12){context!.fillStyle='#c1d4e5';context!.font='10px monospace';context!.fillText(`R${router.id}`,p.x+9,p.y-7);}}
		}
		frame=requestAnimationFrame(draw);return()=>{cancelAnimationFrame(frame);resize.disconnect();observer.disconnect();invalidate=()=>{};};
	});
	$effect(()=>{geometry;scope;selected;rotate;snapshot?.control;invalidate();});
</script>

<section class="globe-view" aria-label="Global network communications">
	<div class="globe-toolbar"><div><Globe2 size={16}/><strong>Global communications</strong><span>{filtered.length} live flows</span></div><div class="globe-actions"><button aria-label={rotate?'Pause globe rotation':'Enable globe rotation'} aria-pressed={rotate} onclick={()=>{rotate=!rotate;invalidate();}}>{#if rotate}<Pause size={13}/>{:else}<Play size={13}/>{/if} Rotation</button><button aria-label="Reset globe camera" onclick={()=>{yaw=-.35;pitch=.15;zoom=1;invalidate();}}><RotateCcw size={13}/></button></div></div>
	<div class="globe-filters"><label>Show <select aria-label="Globe flow type" bind:value={kind}><option value="all">All live activity</option><option value="task">Vanguard tasks</option><option value="exposure">Active exposure</option><option value="compromised">Compromised endpoints</option><option value="connection">Other connections</option></select></label><label>LAN <select aria-label="Globe LAN" value={scope??''} onchange={e=>focusLAN(e.currentTarget.value?Number(e.currentTarget.value):null)}><option value="">All LANs</option>{#each routers as r}<option value={r.id}>LAN {r.lan_id??r.id} / R{r.id}</option>{/each}</select></label></div>
	<div class="globe-stage" bind:this={container}><canvas bind:this={canvas} aria-label="Interactive Earth globe. Directional arcs show actual tasks and live host connections. Drag or use arrow keys to rotate; plus and minus to zoom. Select endpoints below to inspect." tabindex="0" onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={()=>drag=null} onkeydown={keys}></canvas><div class="globe-caption"><span>{hovered||'Drag to orbit · Arrow keys to rotate · + / − to zoom'}</span><small>Illustrative LAN locations · {drawn.length} / {filtered.length} flows drawn{routers.length>128?' · first 128 LAN markers':''}</small></div>{#if !filtered.length}<div class="globe-empty"><strong>No live flows in this scope</strong><span>Start or step the match, or choose another LAN/filter.</span></div>{/if}</div>
	<div class="flow-legend" aria-label="Connection colours">{#each Object.entries(flowColors) as [key,color]}<span><i style:background={color}></i>{key==='task'?'Vanguard → host':key==='exposure'?'Active exposure':key==='compromised'?'Compromised endpoint':'Other connection'}</span>{/each}</div>
	<div class="globe-flow-list"><header><h3>Live flow directory</h3><span>Arrows indicate direction · select an endpoint to inspect</span></header>{#each filtered.slice(page*8,page*8+8) as flow (flow.id)}<div class="globe-flow"><i style:background={flowColors[flow.kind]}></i><button onclick={()=>onselect(flow.source)}>{byId.get(flow.source)?.type==='ROUTER'?'Router':'Host'} {flow.source}</button><ArrowRight size={13}/><button onclick={()=>onselect(flow.target)}>{byId.get(flow.target)?.type==='ROUTER'?'Router':'Host'} {flow.target}</button><span>{flow.label}</span><small>T{flow.opened}</small></div>{:else}<p class="empty">No active tasks or host connections match these filters. Required-service dependencies are not live sessions.</p>{/each}{#if pages>1}<div class="table-pagination"><button disabled={!page} onclick={()=>page--}>Previous flows</button><span>{page+1} / {pages}</span><button disabled={page+1>=pages} onclick={()=>page++}>Next flows</button></div>{/if}</div>
</section>

<style>
	.globe-view{min-height:100%;background:#080f1b;color:var(--text)}.globe-toolbar,.globe-toolbar>div,.globe-actions,.globe-filters{display:flex;align-items:center;gap:10px}.globe-toolbar{justify-content:space-between;padding:16px 18px 10px}.globe-toolbar strong{font-size:13px;font-weight:550}.globe-toolbar span{color:var(--muted);font:10px monospace}.globe-actions button{display:flex;align-items:center;gap:5px;font-size:10px}.globe-filters{padding:0 18px 10px;flex-wrap:wrap}.globe-filters label{font:10px monospace;color:var(--muted);display:flex;align-items:center;gap:8px}.globe-filters select{font:11px system-ui;max-width:190px}.globe-stage{position:relative;height:clamp(350px,49vw,540px);overflow:hidden;background:radial-gradient(ellipse at 50% 45%,#12203570,transparent 67%)}canvas{display:block;width:100%;height:100%;touch-action:none;cursor:grab}canvas:active{cursor:grabbing}.globe-caption{position:absolute;bottom:12px;left:18px;right:18px;display:flex;flex-direction:column;gap:6px;pointer-events:none;color:#a2b9cf;font:10px monospace}.globe-caption small{font-size:9px;color:#819bb3}.globe-empty{position:absolute;top:14px;left:18px;display:flex;flex-direction:column;gap:6px;pointer-events:none}.globe-empty strong{font-size:12px}.globe-empty span{font-size:11px;color:var(--muted)}.flow-legend{display:flex;flex-wrap:wrap;gap:9px 18px;padding:12px 18px;border-block:1px solid var(--line);font:10px monospace}.flow-legend span{display:flex;align-items:center;gap:6px}.flow-legend i,.globe-flow>i{width:6px;height:6px;border-radius:50%;flex-shrink:0}.globe-flow-list{padding:16px 18px}.globe-flow-list header{display:flex;flex-direction:column;gap:6px;margin-bottom:10px}.globe-flow-list h3{font-size:12px;font-weight:550;margin:0}.globe-flow-list header span{font:10px monospace;color:var(--muted)}.globe-flow{display:flex;align-items:center;gap:8px;padding:8px 0;border-bottom:1px solid #25344080;flex-wrap:wrap}.globe-flow button{font:11px monospace;padding:5px 7px}.globe-flow>span{font:10px monospace;color:var(--muted);flex:1}.globe-flow small{font:10px monospace;color:var(--muted)}@media(max-width:650px){.globe-toolbar{flex-wrap:wrap}.globe-toolbar>div{flex-wrap:wrap}.globe-stage{height:370px}.globe-flow>span{flex-basis:70%}}
</style>
