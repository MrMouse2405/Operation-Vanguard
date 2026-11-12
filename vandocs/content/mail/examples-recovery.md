We are going to isolate one workstation, shut it down, start it, and restore it. This is a controlled demonstration of power states, not a plan to reboot every computer until the scoreboard approves.

```sh
vanguard --script vanguard/agents/examples/recovery.lua --network vanguard/networks/tutorial-1.toml
```

<!-- attachment:agents/examples/recovery.lua -->

## What to watch

Open Jobs and the selected host’s True state. SHUTDOWN takes 1 tick. STARTUP takes 3. RESTORE takes 20. The host is unable to communicate while STARTING or RESTORING.

After startup, established infection is still present. After a successful restore, infection is gone. Isolation and shield settings survive both. The example leaves the workstation isolated deliberately; service to that client stays unavailable until a policy safely releases it.

Try [the solo patrol](/docs/examples/solo/) for a full scan-to-release loop. The [recovery reference](/docs/actions/recovery/) explains failed jobs and boundary timing.
