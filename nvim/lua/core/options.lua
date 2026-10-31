-- Core Neovim editor options
local opt = vim.opt

-- Appearance & Colors
opt.termguicolors = true
opt.number = true
opt.relativenumber = true
opt.signcolumn = "yes"
opt.cursorline = true
opt.showmode = false

-- Indentation & Tabs
opt.tabstop = 4
opt.shiftwidth = 4
opt.expandtab = true
opt.smartindent = true

-- Search behavior
opt.ignorecase = true
opt.smartcase = true
opt.hlsearch = true
opt.incsearch = true

-- Behavior & Splits
opt.splitright = true
opt.splitbelow = true
opt.scrolloff = 8

-- Performance & Low-Latency Tuning
opt.updatetime = 100
opt.timeoutlen = 300
opt.undofile = true
opt.swapfile = false
opt.synmaxcol = 300
opt.clipboard = "unnamedplus"
