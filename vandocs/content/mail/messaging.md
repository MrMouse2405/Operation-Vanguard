I can carry “host 1007, SUSPICIOUS, observed at tick 12.” I can also carry “help.” One of those is a considerably better message.

Routers have private memory. **Direct messages** go to one agent; **topics** go to subscribed teammates. Payloads are copied. Changing your original table after sending will not revise the note already in transit.

## What can you send?

Strings, booleans, finite numbers, dense arrays (no missing indexes), and objects with string keys. Empty tables mean objects. Tables are copied: nobody shares the original reference.

Don’t send nil, mixed or sparse tables, functions, threads, userdata, metatables, cycles, or infinite/NaN numbers. Trees can be at most 16 levels deep and must fit the configured byte budget.

Topic names are exact, case-sensitive strings of 1–128 bytes. No wildcards, self-delivery, or old-message replay for new subscribers.

## Allow time for the trip

A message sent in T can first run its handler in **T+1**. A reply can first arrive in **T+2**. Messages are delivered by sender agent ID, then sender sequence—not by when handlers finish.

Topic publications use the subscriber list from the **start of the tick**. Subscribe in T and your teammates see that membership at T+1. You won’t receive publications from T.

```lua
vanguard.rx(function(sender, message, metadata)
    print("direct", sender, message, metadata.sent_at, metadata.delivered_at)
end)
vanguard.sub("threats", function(sender, topic, message, metadata)
    print(topic, sender, message, metadata.message_id)
end)
vanguard.wait(1) -- let teammates see the subscription
vanguard.pub("threats", {kind="ready", router=vanguard.router()})
```

The engine starts handlers in separate coroutines. Register them and let the engine do the dispatch; you don’t need to poll an inbox or resume them yourself. Your main code can return without taking the receiver off the desk.

## About that receipt

A valid send is accepted even if the teammate is inactive or has no handler. It may be dropped later. My receipt confirms acceptance, not that somebody read it. If your plan needs an acknowledgement, have the recipient send one. There is no can-send call or queue-full rejection. Nonexistent IDs, self-send, and bad payloads are errors.

At the active-handler limit, accepted messages wait. Replacing a handler doesn’t cancel ones already running. Unsubscribing discards your undelivered topic messages. A handler error stops that handler only; its tasks keep running and the error logs message context.

Messages use fuel. If the engine’s explicit pending-delivery limit is exhausted, the run stops with a resource error instead of silently losing accepted work.

[The call sheet](/docs/api/communication/) has the signatures. Register before you send. Include a tick. Keep it brief enough that the recipient can afford to read it.
