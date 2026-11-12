Identity asked me to put this first: **messages take agent IDs, not router IDs**. I would prefer to deliver the note on the first attempt. [The earlier email](/docs/messaging/) covers what fits in a message and when it can arrive.

Handlers get metadata shaped like `{message_id, sent_at, delivered_at}`. Registering a handler doesn’t use a task slot.

## vanguard.tx

`vanguard.tx(agent_id, message) → {message_id, sent_at}`

Sends a copied note to a known teammate. The ID must be a positive agent ID, and you can’t send to yourself. A valid send succeeds even if that teammate is inactive or has no direct-message handler; the note may be dropped later.

Unknown IDs raise `UNKNOWN_AGENT`. Invalid IDs or payloads raise `INVALID_ARGUMENT`; oversized trees raise `PAYLOAD_TOO_LARGE`. Earliest delivery is T+1.

## vanguard.rx

`vanguard.rx(handler_or_nil) → true`

Sets or replaces your direct-message handler. Use nil to unregister it. Other nonfunction values raise `INVALID_ARGUMENT`. Handlers already running continue after replacement or removal.

```lua
vanguard.rx(function(sender_agent_id, message, metadata)
    print(sender_agent_id, message, metadata)
end)
```

## vanguard.pub

`vanguard.pub(topic, message) → {message_id, sent_at, recipient_count}`

Sends to the topic’s committed start-of-tick subscribers, excluding you. Zero recipients is accepted; it is also zero people hearing your announcement. `recipient_count` counts intended recipients, not handlers that ran. No replay for late arrivals. Topic and payload errors raise normally. Earliest execution is T+1.

## vanguard.sub

`vanguard.sub(topic, handler) → true`

Sets or replaces one handler for an exact, case-sensitive topic of 1–128 bytes. It doesn’t add duplicate membership. The handler must be a function; bad arguments raise `INVALID_ARGUMENT`.

Your handler is registered locally right away, but teammates see the subscription at T+1.

```lua
vanguard.sub("threats", function(sender_agent_id, topic, message, metadata)
    print(sender_agent_id, topic, message, metadata)
end)
```

## vanguard.unsub

`vanguard.unsub(topic) → true`

Removes your membership and handler. Repeating it is safe. Bad topic names raise `INVALID_ARGUMENT`. Undelivered local topic messages are discarded; handlers already running continue. Other VMs see the change at T+1.

## Leave the receiver on, then try this

```lua
vanguard.rx(function(sender, message)
    if message.kind == "ping" then
        vanguard.tx(sender, {kind="pong"})
    end
end)
for id in pairs(vanguard.others()) do
    local receipt = vanguard.tx(id, {kind="ping"})
    print(receipt.message_id)
end
-- Your handler stays registered when main code returns.
```

```lua
vanguard.sub("alerts", function(sender, topic, message)
    print(sender, topic, message)
end)
vanguard.wait(1)
print(vanguard.pub("alerts", {kind="ready"}))
vanguard.wait(2)
vanguard.unsub("alerts")
-- vanguard.rx(nil) removes the direct handler.
```
