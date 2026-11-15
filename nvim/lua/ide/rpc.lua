-- nvim/lua/ide/rpc.lua
-- JSON-RPC 2.0 client over a stdio job connected to minimal-ide.
-- The server process is spawned once at setup() and reused for all requests.

local M = {}

-- Internal state
local _job_id = nil          -- vim job id for minimal-ide process
local _pending = {}          -- id -> callback map
local _next_id = 1           -- monotonic request id counter
local _buf = ""              -- partial line buffer for stdout
local _notification_handlers = {} -- method -> callback map

--- Register a notification handler.
function M.on_notification(method, cb)
  _notification_handlers[method] = cb
end

--- Send a raw JSON string to the server.
local function send(json_str)
  if not _job_id then
    vim.notify("[NvimIDE/rpc] Server not connected", vim.log.levels.WARN)
    return
  end
  vim.fn.chansend(_job_id, json_str .. "\n")
end

--- Handle a line of JSON output from the server.
local function on_line(line)
  if line == "" then return end
  local ok, msg = pcall(vim.json.decode, line)
  if not ok or type(msg) ~= "table" then
    -- Could be a notification or malformed; ignore silently
    return
  end

  if msg.id ~= nil then
    local id = tostring(msg.id)
    local cb = _pending[id]
    if cb then
      _pending[id] = nil
      if msg.error then
        cb(nil, msg.error)
      else
        cb(msg.result, nil)
      end
    end
  elseif msg.method then
    -- Notifications (no id)
    local handler = _notification_handlers[msg.method]
    if handler then
      handler(msg.params)
    end
  end
end

--- Start the minimal-ide process and connect over stdio.
--- @param config table Plugin config containing .root and .binary fields
function M.start(config)
  if _job_id then return end  -- already running

  local cmd = {
    config.binary or "minimal-ide",
    "--root", config.root or vim.fn.getcwd(),
  }

  _job_id = vim.fn.jobstart(cmd, {
    on_stdout = function(_, data, _)
      for _, chunk in ipairs(data) do
        _buf = _buf .. chunk
        -- Split on newlines (data may arrive in fragments)
        while true do
          local nl = string.find(_buf, "\n")
          if not nl then break end
          local line = string.sub(_buf, 1, nl - 1)
          _buf = string.sub(_buf, nl + 1)
          on_line(line)
        end
      end
    end,
    on_stderr = function(_, data, _)
      for _, line in ipairs(data) do
        if line ~= "" then
          vim.notify("[NvimIDE/rpc] " .. line, vim.log.levels.WARN)
        end
      end
    end,
    on_exit = function(_, code, _)
      _job_id = nil
      if code ~= 0 then
        vim.notify(
          "[NvimIDE/rpc] minimal-ide exited with code " .. code,
          vim.log.levels.ERROR
        )
      end
    end,
    stdin = "pipe",
    stdout = "pipe",
    stderr = "pipe",
  })

  if _job_id == 0 or _job_id == -1 then
    vim.notify(
      "[NvimIDE/rpc] Failed to start minimal-ide — is it on PATH?",
      vim.log.levels.ERROR
    )
    _job_id = nil
  end
end

--- Send a JSON-RPC request and call `cb(result, error)` when the response arrives.
--- @param method string  JSON-RPC method name
--- @param params table   Parameters table (will be JSON-encoded)
--- @param cb     function Callback: cb(result_table, error_table)
function M.request(method, params, cb)
  if not _job_id then
    cb(nil, { code = -32000, message = "Server not connected" })
    return
  end

  local id = _next_id
  _next_id = _next_id + 1
  _pending[tostring(id)] = cb

  local req = vim.json.encode({
    jsonrpc = "2.0",
    id = id,
    method = method,
    params = params or {},
  })
  send(req)
end

--- Stop the minimal-ide process.
function M.stop()
  if _job_id then
    vim.fn.jobstop(_job_id)
    _job_id = nil
    _pending = {}
  end
end

--- Returns true if the server process is running.
function M.is_connected()
  return _job_id ~= nil
end

return M
