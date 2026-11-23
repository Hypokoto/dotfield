local M = {}

function M.setup()
  local pid = vim.fn.getpid()
  local sname = vim.v.servername
  local runtime = os.getenv("XDG_RUNTIME_DIR") or "/tmp"
  local file = runtime .. "/vim-niri-nav." .. pid .. ".servername"

  local f = io.open(file, "w")
  if f then
    f:write("nvim " .. sname .. "\n")
    f:close()
  end

  vim.api.nvim_create_autocmd("VimLeavePre", {
    callback = function()
      os.remove(file)
    end,
  })
end

function M.navigate(direction)
  local cur_win = vim.api.nvim_get_current_win()
  local cmd_map = {
    up = "wincmd k",
    down = "wincmd j",
    left = "wincmd h",
    right = "wincmd l",
  }
  local cmd = cmd_map[direction]
  if cmd then
    vim.cmd(cmd)
    local new_win = vim.api.nvim_get_current_win()
    if new_win ~= cur_win then
      return "true"
    end
  end
  return "false"
end

return M
