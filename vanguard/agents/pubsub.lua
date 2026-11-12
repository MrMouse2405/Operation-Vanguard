vanguard.sub("threats", function(sender, topic, message, metadata)
    print("publication", {sender=sender, topic=topic, payload=message, metadata=metadata})
end)
-- Subscriptions become committed at the next barrier.
vanguard.wait(1)
vanguard.pub("threats", {router=vanguard.router(), kind="ready"})
