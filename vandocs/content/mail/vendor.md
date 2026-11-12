Thank you for your interest in **Vanguard**, an add-on module for network routers. Vanguard gives your cyber teams **sensing, acting, shielding and command capabilities**, with custom Lua policies tailored to your operational needs.

Our brochure calls this “adaptive distributed cyber resilience.” Your team may prefer “each router runs the program we gave it.” Both descriptions fit on an invoice; only one explains the implementation.

## Four capabilities, one policy

| Capability | What your code does | Example |
| --- | --- | --- |
| Sensing | Ask for evidence | Scan local hosts, inspect symptoms or connections |
| Acting | Change a host’s operating state | Shut down, start up, restore |
| Shielding | Control permitted traffic | Block a peer, isolate a host, maintain an allowlist |
| Command | Coordinate router agents | Send a direct message or publish a warning topic |

You supply **one Lua entrypoint**. Each router receives the same code in a private VM. Customization lives in your policy, not a separate script per router. Your code can discover its identity, its local machines and its teammates, then decide which job to do.

## A small illustration

```lua
-- Start a local discovery task, wait for it, then print the findings.
local ticket = vanguard.deploy({action=vanguard.sense.SCAN})
local result = vanguard.await(ticket)
print("My local network", result.data)
```

This senses. It does not cure anything. The competition’s [sensing](/docs/examples/sensing/), [shielding](/docs/examples/shields/), [acting](/docs/examples/recovery/) and [command](/docs/examples/warning/) attachments demonstrate the rest.

## A note from Legal, who read the footnotes

Vanguard does not give your program omniscience. Knowing a remote host ID does not let you control that host. A completed restore does not protect against the next attack. A message receipt does not mean the recipient acted on it.

The supplied **Operation Vanguard** simulator is the training environment for this evaluation. It is a deterministic game, not a claim that real infection always takes fourteen turns. [The simulator briefing](/docs/operation/) describes those rules.

We look forward to increasing both your operational capability and our shareholder value. Preferably during the same procurement cycle.
