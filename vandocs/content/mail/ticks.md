Three ticks after tick seven is tick ten. Whether your laptop takes a moment or a minute is a separate matter. I keep the game clock, not your wall clock.

A **tick** is one game turn. All routers share the same completed tick. Tasks, infection, and messages advance when the engine completes the next one. Pause freezes all three.

## Put these on the same timeline

- Start a 3-tick task at tick 7: it finishes at tick **10**.
- Changes such as isolation finish before that tick’s attacks. Scans then read the updated state.
- Send a message at T: its handler can first run at **T+1**. A reply can’t arrive before **T+2**.

## Fuel does not belong in the tick column

Fuel limits **how much computing your agent does** per turn. Task duration tells you **how many turns the job takes**. They’re separate.

Every router gets the same fresh fuel allowance each tick. Leftover fuel doesn’t carry over. Your main code and message handlers share that router’s allowance; each router also has its own memory limit.

Exhaust your fuel and your code resumes later, where it left off. No restart. If you are waiting for a task, use a wait. Repeatedly asking whether three ticks have passed will not hurry me along.

```lua
local started = vanguard.ticks()
local id = vanguard.deploy({action=vanguard.sense.HOST_SCAN, target=host_id})
local result = vanguard.await(id)
print("Started", started, "finished", result.finished_at)
```

Here `host_id` must be a discovered local host. [cost](/docs/api/time/) gives you a quote. `await` gives you the actual completion. Use the latter when the next decision depends on the job being done.

The dashboard’s **s/tick** setting only slows playback for your eyes. Set it to **0** to go as fast as the engine can. If computation already takes longer than your chosen time, the engine moves on without adding a delay. Manual **Forward** stepping runs the requested turns directly.
