Before you change anything, identify what’s on the map. A router and a workstation have rather different problems.

- **Workstations** are ordinary computers. Some may start infected.
- **Servers** provide useful services. They start clean but can get infected.
- **Routers are immune.** Each runs a separate copy of your agent.
- You can change your router and the hosts in its own LAN (its local group of computers). Knowing a remote host’s ID doesn’t let you control it.

## Where the trouble moves

An infected, online host attacks up to **two targets at once**. A target becomes infected after **14 uninterrupted ticks** of exposure. Break the connection before then and that exposure resets.

Turning a host off or isolating it stops traffic. **It does not remove infection.** `RESTORE` takes **20 ticks**, cures the host, and brings it online. It keeps the host’s isolation and firewall rules.

A restored host goes back onto the same network. Leave its attacker a route and you may be restoring it again shortly. Protect first, then restore. These are the baseline timings; check the scenario for published changes.

## The router is not the whole perimeter

**DEFAULT_ALLOW** means “let traffic through, except these peers.” **DEFAULT_DENY** means “block traffic, except these peers.” Changing the policy clears its exception list; repeating the same policy keeps the list.

Isolation blocks all of a host’s data traffic without deleting its saved rules.

Hosts in the same LAN check their own rules, **not the router’s rules**. Traffic between LANs also checks every router on the shortest physical route. A DEFAULT_DENY router needs to allow **both original host IDs**, not the routers in between.

## Your view and the agent’s view

`SCAN` discovers local hosts and neighbouring routers. Other scans reveal symptoms, connections, and peer IDs. Scan results describe the tick when the scan finished, so old results can become out of date.

You can see an infected host in the dashboard’s **True state**. Your agent cannot borrow your eyes. It has its discoveries, completed scans, and messages. Write the defence for that view.

One detail I want you to remember: **same LAN, no router filter**. If the infection keeps spreading after your router block, check who is standing next to whom. [Scoreboard](/docs/scoring/) will explain what the damage costs.
