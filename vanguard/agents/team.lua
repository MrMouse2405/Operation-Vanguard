-- Multiple files on a router share this VM. Handlers survive main return.
vanguard.rx(function(sender, message, metadata)
    print("direct message", {sender=sender, payload=message, metadata=metadata})
    if message.kind == "hello" then
        vanguard.tx(sender, {kind="ack", router=vanguard.router()})
    end
end)
vanguard.sub("threats", function(sender, topic, message, metadata)
    print("team observation", {sender=sender, topic=topic, report=message, tick=metadata.delivered_at})
end)
for id, router in pairs(vanguard.others()) do
    vanguard.tx(id, {kind="hello", router=vanguard.router()})
end
return true
