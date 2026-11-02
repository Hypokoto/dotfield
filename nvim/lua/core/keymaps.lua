-- Core Neovim keymaps managed by Dotfield
-- Harmonized with Zellij and Terminal multiplexer
local map = vim.keymap.set

-- Clear search highlights
map("n", "<Esc>", "<cmd>nohlsearch<CR>", { desc = "Clear search highlight" })

-- Universal save with Ctrl-s across Normal, Insert, and Visual modes (Guaranteed unintercepted by Zellij)
map({ "n", "i", "v" }, "<C-s>", "<cmd>w<CR>", { desc = "Save file" })

-- Window navigation (Ctrl-h/j/k/l unhindered by Zellij)
map("n", "<C-h>", "<C-w>h", { desc = "Move to left split" })
map("n", "<C-j>", "<C-w>j", { desc = "Move to lower split" })
map("n", "<C-k>", "<C-w>k", { desc = "Move to upper split" })
map("n", "<C-l>", "<C-w>l", { desc = "Move to right split" })

-- Window resizing with Ctrl-Arrows
map("n", "<C-Up>", "<cmd>resize +2<CR>", { desc = "Increase window height" })
map("n", "<C-Down>", "<cmd>resize -2<CR>", { desc = "Decrease window height" })
map("n", "<C-Left>", "<cmd>vertical resize -2<CR>", { desc = "Decrease window width" })
map("n", "<C-Right>", "<cmd>vertical resize +2<CR>", { desc = "Increase window width" })

-- Visual mode block moving & persistent indent
map("v", "<", "<gv", { desc = "Indent left and maintain selection" })
map("v", ">", ">gv", { desc = "Indent right and maintain selection" })
map("v", "J", ":m '>+1<CR>gv=gv", { desc = "Move selected lines down" })
map("v", "K", ":m '<-2<CR>gv=gv", { desc = "Move selected lines up" })

-- Buffer navigation
map("n", "[b", "<cmd>bprevious<CR>", { desc = "Previous buffer" })
map("n", "]b", "<cmd>bnext<CR>", { desc = "Next buffer" })

-- File management
map("n", "<leader>w", "<cmd>w<CR>", { desc = "Save file" })
map("n", "<leader>q", "<cmd>q<CR>", { desc = "Quit window" })

-- Reload Dotfield palette
map("n", "<leader>rp", function()
    require("palette").reload()
end, { desc = "Reload Dotfield palette" })
