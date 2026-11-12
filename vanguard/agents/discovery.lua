-- Standalone discovery/logging example. Can also be required by another entry.
local job = vanguard.deploy({action=vanguard.sense.SCAN})
local result = vanguard.await(job)
print("local topology", result.data)
return result.data
