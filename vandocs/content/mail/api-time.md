Keep two columns: **ticks elapsed** and **fuel spent**. A twenty-tick restore pays its admission charge once. A busy loop can burn fuel without getting you a result. Below are the instruments; read the right one.

## vanguard.fuel

`vanguard.fuel(agent_id?) → integer`

Cumulative fuel **used so far**. If you label this “fuel remaining,” your display will become increasingly optimistic as the match goes on. It includes VM work, native calls, communication, and accepted-task fuel.

Leave out the ID (or use your own) for the current value, including this slice of execution. Teammate values come from the latest synchronization. Unknown IDs raise `UNKNOWN_AGENT`; malformed IDs raise `INVALID_ARGUMENT`. Asking the question also uses a little fuel.

## vanguard.ticks

`vanguard.ticks() → integer`

The last completed tick T. It stays fixed during your current VM slice. Ten thousand calls in a loop will read the same tick; they will not read the same cumulative fuel. Use this clock for agent decisions.

## vanguard.wait

`vanguard.wait(duration_ticks) → true`

Pauses only the calling coroutine until T+duration_ticks. Use an integer of at least 1, or get `INVALID_ARGUMENT`. No job, slot, or world change is created. Other handlers and tasks continue.

```lua
local before = vanguard.ticks()
vanguard.wait(3)
print("Resumed at", vanguard.ticks(), "started at", before)
```

Fuel limits can delay when the call itself executes. Calculate deadlines from completed ticks. I cannot do arithmetic with “the browser seemed to pause for a bit.”

## vanguard.cost

`vanguard.cost(descriptor) → {fuel, ticks, exact, quoted_at}`

Checks the same descriptor fields, discovery, scope, and target conditions as `deploy`, but needs no free slot and takes no lock. Bad requests raise their reason label.

It’s a quote, not a reservation, and doesn’t reveal hidden truth. `fuel` excludes the cost of asking; `ticks` is task duration. `quoted_at` is the tick of the quote. `exact=false` means the actual cost may differ later; cost is fixed when admitted. The baseline gives exact configured admission fuel.

```lua
local quote = vanguard.cost({action=vanguard.sense.SCAN})
print(quote.fuel, quote.ticks, quote.exact, quote.quoted_at)
print("Tick", vanguard.ticks(), "fuel used", vanguard.fuel())
for teammate in pairs(vanguard.others()) do
    print("Teammate", teammate, "fuel", vanguard.fuel(teammate))
end
vanguard.wait(1)
```
