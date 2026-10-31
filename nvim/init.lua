-- Neovim initialization managed by Dotfield
-- Clean, modular architecture without external plugin requirements

-- Set leader key to space before loading keymaps
vim.g.mapleader = " "
vim.g.maplocalleader = " "

-- Add current directory lua folder to package path if running locally
local src = debug.getinfo(1, "S").source:gsub("^@", "")
local config_dir = vim.fn.fnamemodify(vim.fn.resolve(src), ":p:h")
package.path = package.path .. ";" .. config_dir .. "/lua/?.lua;" .. config_dir .. "/lua/?/init.lua"

-- Load core options and keybindings
require("core.options")
require("core.keymaps")

-- Load Controlled Chaos theme bridge
require("palette").setup()
