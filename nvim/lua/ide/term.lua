-- nvim/lua/ide/term.lua
-- Agent terminal launcher.
-- Opens a floating or split terminal and runs the chosen agent CLI.

local M = {}

-- Map of agent names to their launch commands
local AGENT_COMMANDS = {
  claude  = { "claude" },
  codex   = { "codex" },
  gemini  = { "gemini" },
  aider   = { "aider", "--no-auto-commits" },
  default = { vim.o.shell or "zsh" },
}

--- Open a terminal window running the given agent.
--- Prefers a large vertical split; falls back to a full-height split.
---
--- @param agent_name string|nil  Agent key (e.g. "claude", "codex"). Defaults to a bare shell.
--- @param config     table       Plugin config (reads config.terminal.shell as fallback)
function M.open_agent(agent_name, config)
  local cmd

  if agent_name and agent_name ~= "" then
    cmd = AGENT_COMMANDS[agent_name]
    if not cmd then
      -- Unknown agent: try running it by name directly
      cmd = { agent_name }
    end
  else
    -- No agent specified: open configured shell
    local shell = (config and config.terminal and config.terminal.shell)
      or vim.o.shell
      or "zsh"
    cmd = { shell }
  end

  -- Create a new vertical split terminal
  vim.cmd("vsplit")
  local win = vim.api.nvim_get_current_win()
  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_win_set_buf(win, buf)

  -- Reasonable width for a terminal alongside an editor
  local target_width = math.floor(vim.o.columns * 0.40)
  vim.api.nvim_win_set_width(win, target_width)

  -- Start the terminal
  local job_id = vim.fn.termopen(cmd, {
    on_exit = function(_, code, _)
      if code ~= 0 then
        vim.notify(
          string.format("[NvimIDE] Agent '%s' exited with code %d", cmd[1], code),
          vim.log.levels.WARN
        )
      end
    end,
  })

  if job_id == -1 then
    vim.notify(
      "[NvimIDE] Failed to start agent: " .. cmd[1],
      vim.log.levels.ERROR
    )
    return
  end

  -- Enter insert mode so the user can type immediately
  vim.cmd("startinsert")

  -- Title the window buffer
  vim.api.nvim_set_option_value(
    "statusline",
    " Agent: " .. cmd[1] .. " ",
    { win = win }
  )
end

return M
