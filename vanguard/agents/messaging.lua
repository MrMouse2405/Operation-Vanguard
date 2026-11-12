-- Valid sends are accepted even when the recipient has not registered a handler.
vanguard.rx(function(sender, message, metadata)
    print("received", {sender=sender, message=message, metadata=metadata})
    if message.kind == "ping" then vanguard.tx(sender, {kind="pong"}) end
end)
for id in pairs(vanguard.others()) do vanguard.tx(id, {kind="ping"}) end
-- No keepalive loop: the host continues to schedule registered handlers.
