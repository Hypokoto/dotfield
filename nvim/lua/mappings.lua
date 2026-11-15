-- lua/custom/mappings.lua

require "nvchad.mappings"

local map = vim.keymap.set

map("n", "<leader>e", function()
  local api = require("nvim-tree.api")
  if not api.tree.is_visible() then
    local buf = vim.api.nvim_get_current_buf()
    local name = vim.api.nvim_buf_get_name(buf)
    local ft = vim.bo[buf].filetype
    local bt = vim.bo[buf].buftype
    if name ~= "" and ft ~= "snacks_dashboard" and bt == "" then
      api.tree.toggle { find_file = true, focus = true }
    else
      api.tree.toggle { focus = true }
    end
  elseif vim.bo.filetype == "NvimTree" then
    api.tree.close()
  else
    api.tree.focus()
  end
end, { desc = "Toggle/focus file tree (right)" })

map("n", "<leader>x", function()
  if vim.bo.filetype == "snacks_dashboard" then
    return
  end
  require("nvchad.tabufline").close_buffer()
end, { desc = "Close buffer" })

map("n", "<leader>mp", "<cmd>RenderMarkdown toggle<CR>", { desc = "Toggle markdown render" })

-- VS Code style terminal toggle (maps to NvChad horizontal terminal)
map({"n", "t"}, "<C-`>", function()
  require("nvchad.term").toggle({ pos = "float", id = "floatTerm" })
end, { desc = "Toggle Terminal (VS Code style)" })

-- Right-Click Menu
map("n", "<RightMouse>", function()
  vim.cmd.exec '"normal! \\<RightMouse>"'
  local options = vim.bo.ft == "NvimTree" and "nvimtree" or "default"
  require("menu").open(options, { mouse = true })
end, { desc = "Open Context Menu" })

-- Load Dotfield core keybindings
pcall(require, "core.keymaps")


