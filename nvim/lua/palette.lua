-- Controlled Chaos Highlight Bridge for Neovim
-- Translates Dotfield palette tokens into standard Neovim highlight groups
local M = {}

function M.setup()
    -- Reset existing highlights
    vim.cmd("highlight clear")
    if vim.fn.exists("syntax_on") == 1 then
        vim.cmd("syntax reset")
    end
    vim.o.termguicolors = true
    vim.g.colors_name = "dotfield"

    local c = require("palette_generated")
    local hl = function(group, opts)
        vim.api.nvim_set_hl(0, group, opts)
    end

    -- Base editor UI
    hl("Normal", { fg = c.bone, bg = c.void })
    hl("NormalFloat", { fg = c.bone, bg = c.obsidian })
    hl("FloatBorder", { fg = c.iron, bg = c.obsidian })
    hl("ColorColumn", { bg = c.obsidian })
    hl("Cursor", { fg = c.void, bg = c.bone })
    hl("CursorLine", { bg = c.obsidian })
    hl("CursorLineNr", { fg = c.amber, bold = true })
    hl("LineNr", { fg = c.iron })
    hl("SignColumn", { bg = c.void })
    hl("VertSplit", { fg = c.iron })
    hl("WinSeparator", { fg = c.iron })
    hl("StatusLine", { fg = c.bone, bg = c.obsidian })
    hl("StatusLineNC", { fg = c.ash, bg = c.obsidian })
    hl("TabLine", { fg = c.ash, bg = c.obsidian })
    hl("TabLineSel", { fg = c.bone, bg = c.graphite, bold = true })
    hl("TabLineFill", { bg = c.obsidian })
    hl("Visual", { bg = c.graphite })
    hl("Search", { fg = c.void, bg = c.amber })
    hl("IncSearch", { fg = c.void, bg = c.amber, bold = true })
    hl("Pmenu", { fg = c.ash, bg = c.obsidian })
    hl("PmenuSel", { fg = c.bone, bg = c.graphite, bold = true })
    hl("PmenuSbar", { bg = c.obsidian })
    hl("PmenuThumb", { bg = c.iron })

    -- Syntax highlights
    hl("Comment", { fg = c.ash, italic = true })
    hl("Constant", { fg = c.amber })
    hl("String", { fg = c.steel })
    hl("Character", { fg = c.steel })
    hl("Number", { fg = c.amber })
    hl("Boolean", { fg = c.amber })
    hl("Identifier", { fg = c.bone })
    hl("Function", { fg = c.amber })
    hl("Statement", { fg = c.crimson })
    hl("Conditional", { fg = c.crimson })
    hl("Repeat", { fg = c.crimson })
    hl("Operator", { fg = c.ash })
    hl("Keyword", { fg = c.crimson })
    hl("PreProc", { fg = c.steel })
    hl("Type", { fg = c.amber })
    hl("Special", { fg = c.amber })
    hl("Underlined", { underline = true })
    hl("Error", { fg = c.crimson, bold = true })
    hl("Todo", { fg = c.void, bg = c.amber, bold = true })

    -- Diagnostics
    hl("DiagnosticError", { fg = c.crimson })
    hl("DiagnosticWarn", { fg = c.amber })
    hl("DiagnosticInfo", { fg = c.steel })
    hl("DiagnosticHint", { fg = c.ash })
end

function M.reload()
    package.loaded["palette_generated"] = nil
    package.loaded["palette"] = nil
    require("palette").setup()
    vim.notify("Dotfield palette reloaded", vim.log.levels.INFO)
end

return M
