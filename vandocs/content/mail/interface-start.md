The terminal prints the dashboard URL after Vanguard binds its port. Open **that** URL. `--port 0` asks the OS for a free port; it does not mean browsing to port zero.

## The controls across the top

| Control | What it does | When I use it |
| --- | --- | --- |
| Start | Begins continuous ticks from READY | First run |
| Pause / Resume | Stops / continues the game clock | Inspect a decision |
| +1 | Runs exactly one normal tick; stays paused | Follow a task boundary |
| Number + Forward | Runs 1–1000 ticks; stays paused unless finished | Skip a twenty-tick restore |
| s/tick | Minimum seconds per tick during continuous playback; 0 = full speed | Slow the display for humans |
| Restart, circular arrow | Rereads script and network; creates a fresh run | Try a saved edit |
| Information icon | Opens effective scenario settings | Confirm seed and budgets |

Stepping is available from READY or PAUSED. Finished means the horizon or defeat stopped the run; restart for a fresh attempt. Forward can finish early if the match ends. It does not bypass infection, scoring, or fuel rules.

## My first run sequence

1. Start with **s/tick = 0.1** if you want time to see changes.
2. Press **Start**, then **Pause**.
3. Search for a host; inspect it.
4. Press **+1** and watch its task or exposure age.
5. Open **Output** for your print statements.
6. Save a Lua edit, press **Restart**, then **Start** again.

Restart returns to READY. If staging fails, the old run remains paused and the dashboard shows the error. Correct the file and restart again. The network file does not select your script: `--script` does.

## Two common traps

**“My scan takes three seconds.”** It takes three ticks. s/tick changes playback, not that duration. A slow tick does not receive an extra full delay.

**“I edited the code but it still behaves the same.”** Save the file, verify the CLI’s script path, and restart. Existing VMs do not hot-load your editor buffer.

Continue with [the explorer](/docs/interface/explorer/) and [telemetry](/docs/interface/telemetry/). For VanMail itself: folders filter the inbox, search checks every email, stars save favorites locally, the moon/sun changes theme, and every fenced code block has **Copy code**.
