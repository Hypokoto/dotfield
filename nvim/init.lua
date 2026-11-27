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

-- load theme safely
pcall(dofile, vim.g.base46_cache .. "defaults")
pcall(dofile, vim.g.base46_cache .. "statusline")
pcall(dofile, vim.fn.expand("~/.config/dotfield/themes/nvim.lua"))

require("options")
require("autocmds")

vim.api.nvim_create_autocmd("Signal", {
  pattern = "SIGUSR1",
  callback = function()
    pcall(dofile, vim.fn.expand("~/.config/dotfield/themes/nvim.lua"))
  end,
})

vim.schedule(function()
  require("mappings")
end)
