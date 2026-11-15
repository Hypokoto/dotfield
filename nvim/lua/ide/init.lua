-- nvim/lua/ide/init.lua
-- Minimal IDE Neovim Plugin
-- Provides agent diff visualization, review windows, and session management.
--
-- Usage: require("ide").setup({ root = vim.fn.getcwd() })
-- Commands:
--   :NvimIDEStatus          — show server status
--   :NvimIDEStart [agent]   — start an agent session
--   :NvimIDEStop            — stop the current agent session
--   :NvimIDEChanges         — list changed files in a floating window
--   :NvimIDEReview          — open the diff review UI for current file
--   :NvimIDEAccept          — accept agent changes to current file
--   :NvimIDEReject          — reject agent changes to current file
--   :NvimIDEAcceptAll       — accept all agent changes
--   :NvimIDERejectAll       — reject all agent changes
--   :NvimIDEAgent           — open a new agent terminal split

local M = {}

local rpc = require("ide.rpc")
local review = require("ide.review")
local term = require("ide.term")

-- Default configuration
M.config = {
  root = vim.fn.getcwd(),
  binary = "minimal-ide",
  stdio = true,
  auto_review = true,
  animation = true,
  animation_duration_ms = 200,
}

--- Setup the plugin with user config.
--- @param opts table Optional overrides for M.config
function M.setup(opts)
  M.config = vim.tbl_deep_extend("force", M.config, opts or {})

  -- Start the RPC server connection
  rpc.start(M.config)

  -- Initialize zero-copy native IPC for instant buffer updates
  require("ide.zero_copy").setup()

  -- Register user commands
  vim.api.nvim_create_user_command("NvimIDEStatus", function()
    rpc.request("ide/status", {}, function(result, err)
      if err then
        vim.notify("[NvimIDE] Error: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
        return
      end
      local lines = {
        "── Minimal IDE Status ───────────────────────────────",
        "Workspace : " .. (result.workspace_root or "unknown"),
        "Branch    : " .. (result.git_branch or "unknown"),
        "Changed   : " .. (result.uncommitted_files_count or 0) .. " files",
      }
      if result.session then
        local s = result.session
        table.insert(lines, "Session   : " .. s.session_id)
        table.insert(lines, "Agent     : " .. s.agent)
        table.insert(lines, "Status    : " .. s.status)
        table.insert(lines, "Modified  : " .. s.changed_files_count .. " files")
      else
        table.insert(lines, "Session   : (none active)")
      end
      vim.notify(table.concat(lines, "\n"), vim.log.levels.INFO)
    end)
  end, { desc = "Show Minimal IDE server status" })

  vim.api.nvim_create_user_command("NvimIDEStart", function(args)
    local agent = args.args ~= "" and args.args or nil
    rpc.request("ide/start_session", { agent = agent }, function(result, err)
      if err then
        vim.notify("[NvimIDE] Start failed: " .. (err.message or "unknown"), vim.log.levels.ERROR)
        return
      end
      vim.notify(
        "[NvimIDE] Session started: " .. result.session_id
          .. " | agent=" .. result.agent
          .. " | baseline=" .. result.baseline_files_recorded .. " files",
        vim.log.levels.INFO
      )
    end)
  end, { nargs = "?", desc = "Start an agent session (optionally specify agent name)" })

  vim.api.nvim_create_user_command("NvimIDEStop", function()
    rpc.request("ide/stop_session", {}, function(result, err)
      if err then
        vim.notify("[NvimIDE] Stop failed: " .. (err.message or "unknown"), vim.log.levels.ERROR)
        return
      end
      vim.notify("[NvimIDE] " .. (result.summary or "Session stopped"), vim.log.levels.INFO)
    end)
  end, { desc = "Stop the current agent session" })

  vim.api.nvim_create_user_command("NvimIDEChanges", function()
    rpc.request("ide/get_changes", {}, function(result, err)
      if err then
        vim.notify("[NvimIDE] Error: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
        return
      end
      review.show_changes(result)
    end)
  end, { desc = "Show changed files in a floating window" })

  vim.api.nvim_create_user_command("NvimIDEReview", function()
    local path = vim.fn.expand("%:.")  -- relative path
    rpc.request("ide/get_file_diff", { path = path }, function(result, err)
      if err then
        vim.notify("[NvimIDE] Diff error: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
        return
      end
      review.show_diff(result, path, M.config)
    end)
  end, { desc = "Open diff review UI for current file" })

  vim.api.nvim_create_user_command("NvimIDEAccept", function()
    local path = vim.fn.expand("%:.")
    rpc.request("ide/get_file_diff", { path = path }, function(diff_result, diff_err)
      if diff_err then
        vim.notify("[NvimIDE] Accept failed: could not fetch diff", vim.log.levels.ERROR)
        return
      end
      local hash = diff_result.current_hash or 0
      rpc.request("ide/accept_file", { path = path, hash = hash }, function(result, err)
        if err then
          vim.notify("[NvimIDE] Accept failed: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
          return
        end
        vim.notify("[NvimIDE] Accepted changes to " .. path, vim.log.levels.INFO)
        vim.cmd("checktime")  -- reload buffer if file changed on disk
      end)
    end)
  end, { desc = "Accept agent changes to the current file" })

  vim.api.nvim_create_user_command("NvimIDEReject", function()
    local path = vim.fn.expand("%:.")
    rpc.request("ide/get_file_diff", { path = path }, function(diff_result, diff_err)
      if diff_err then
        vim.notify("[NvimIDE] Reject failed: could not fetch diff", vim.log.levels.ERROR)
        return
      end
      local hash = diff_result.current_hash or 0
      rpc.request("ide/reject_file", { path = path, hash = hash }, function(result, err)
        if err then
          vim.notify("[NvimIDE] Reject failed: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
          return
        end
        vim.notify("[NvimIDE] Reverted " .. path .. " to baseline", vim.log.levels.INFO)
        vim.cmd("checktime")
      end)
    end)
  end, { desc = "Reject agent changes to the current file (restore baseline)" })

  vim.api.nvim_create_user_command("NvimIDEAcceptAll", function()
    rpc.request("ide/accept_all", {}, function(result, err)
      if err then
        vim.notify("[NvimIDE] AcceptAll failed: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
        return
      end
      vim.notify("[NvimIDE] Accepted " .. (result.accepted_count or 0) .. " files", vim.log.levels.INFO)
    end)
  end, { desc = "Accept all agent changes" })

  vim.api.nvim_create_user_command("NvimIDERejectAll", function()
    rpc.request("ide/reject_all", {}, function(result, err)
      if err then
        vim.notify("[NvimIDE] RejectAll failed: " .. (err.message or tostring(err)), vim.log.levels.ERROR)
        return
      end
      vim.notify("[NvimIDE] Reverted " .. (result.reverted_count or 0) .. " files", vim.log.levels.INFO)
    end)
  end, { desc = "Reject all agent changes (restore all baselines)" })

  vim.api.nvim_create_user_command("NvimIDEAgent", function(args)
    local agent_name = args.args ~= "" and args.args or nil
    term.open_agent(agent_name, M.config)
  end, { nargs = "?", desc = "Open an agent terminal (optionally specify agent name)" })

  vim.notify("[NvimIDE] Minimal IDE plugin loaded", vim.log.levels.DEBUG)
end

return M
