vim.g.base46_cache = vim.fn.stdpath("data") .. "/base46/"
vim.g.mapleader = " "

-- bootstrap lazy and all plugins
local lazypath = vim.fn.stdpath("data") .. "/lazy/lazy.nvim"

if not vim.uv.fs_stat(lazypath) then
  local repo = "https://github.com/folke/lazy.nvim.git"
  vim.fn.system({ "git", "clone", "--filter=blob:none", repo, "--branch=stable", lazypath })
end

vim.opt.rtp:prepend(lazypath)

local lazy_config = require("configs.lazy")

-- load plugins
require("lazy").setup({
  {
    "NvChad/NvChad",
    lazy = false,
    branch = "v2.5",
    import = "nvchad.plugins",
  },

  { import = "plugins" },
}, lazy_config)

-- auto-compile base46 cache if missing (guards against fresh clone or wiped cache)
if not vim.uv.fs_stat(vim.g.base46_cache .. "defaults") then
  pcall(function()
    require("base46").load_all_highlights()
  end)
end

local function apply_dotfield_colors()
  local ok, c = pcall(dofile, vim.fn.expand("~/.config/dotfield/colors.lua"))
  if not ok or type(c) ~= "table" then
    return
  end
  vim.o.background = c.mode or "dark"
  local hl = vim.api.nvim_set_hl
  hl(0, "Normal", { bg = "NONE", fg = c.fg })
  hl(0, "NormalFloat", { bg = c.surface, fg = c.fg })
  hl(0, "FloatBorder", { bg = c.surface, fg = c.accent })
  hl(0, "CursorLine", { bg = c.surface })
  hl(0, "Visual", { bg = c.overlay })
  hl(0, "Comment", { fg = c.fg_muted, italic = true })
  hl(0, "LineNr", { fg = c.fg_muted })
  hl(0, "CursorLineNr", { fg = c.accent, bold = true })
  hl(0, "StatusLine", { bg = c.surface, fg = c.fg })
  hl(0, "Pmenu", { bg = c.surface, fg = c.fg })
  hl(0, "PmenuSel", { bg = c.accent, fg = c.on_accent, bold = true })
  hl(0, "Search", { bg = c.overlay, fg = c.accent, bold = true })
end

-- load theme safely
pcall(dofile, vim.g.base46_cache .. "defaults")
pcall(dofile, vim.g.base46_cache .. "statusline")
apply_dotfield_colors()

require("options")
require("autocmds")

vim.api.nvim_create_autocmd("Signal", {
  pattern = "SIGUSR1",
  callback = apply_dotfield_colors,
})

vim.schedule(function()
  require("mappings")
end)
