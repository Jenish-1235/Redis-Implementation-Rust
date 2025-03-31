-- Simple TCP benchmarking requires custom wrapper
local socket = require("socket")
local host, port = "localhost", 7171

function init(args)
    local conn = socket.tcp()
    conn:settimeout(0)  -- Non-blocking
    conn:connect(host, port)
    return {conn = conn}
end

function request()
    local key = "key_" .. math.random(1000000)
    local req = json.encode({key = key, value = "value"})
    local res, err = wrk.thread:get().conn:send(req .. "\n")
    if not res then
        wrk.thread:get().conn = socket.tcp()
        wrk.thread:get().conn:connect(host, port)
    end
    return wrk.format()
end