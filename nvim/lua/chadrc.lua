---@type ChadrcConfig
local M = {}

M.base46 = {
  theme = "dotfield",
  hl_add = {
    MinimalDashboardHeader = { fg = "#FFB000", bold = true },
    MinimalDashboardFooter = { fg = "#E63946" },
    FloatBorder            = { fg = "#34343D", bg = "#0A0A0C" },
    NormalFloat            = { bg = "#0A0A0C" },
    NvimTreeNormal         = { bg = "#0A0A0C" },
    NvimTreeNormalNC       = { bg = "#0A0A0C" },
    NvimTreeWinSeparator   = { fg = "#34343D", bg = "#0A0A0C" },
    NvimTreeFolderName     = { fg = "#FFB000", bold = true },
    NvimTreeOpenedFolderName = { fg = "#F4A261", bold = true },
    LineNr                 = { fg = "#34343D" },
    CursorLineNr           = { fg = "#FFB000", bold = true },
    CursorLine             = { bg = "#121216" },
    DiagnosticError        = { fg = "#E63946" },
    DiagnosticWarn         = { fg = "#FFB000" },
    DiagnosticInfo         = { fg = "#82AAFF" },
    DiagnosticHint         = { fg = "#8B949E" },
    Visual                 = { bg = "#1C1C23" },
    Pmenu                  = { bg = "#121216", fg = "#E8E6E3" },
    PmenuSel               = { bg = "#1C1C23", fg = "#FFB000", bold = true },
    PmenuBorder            = { fg = "#34343D", bg = "#121216" },
  },
}

M.ui = {
  statusline = {
    theme = "default",
    style = "default",
  },
  tabufline = {
    enabled = true,
    order = { "buffers", "tabs", "btns", "treeOffset" },
  },
}

M.nvdash = {
  load_on_startup = false,
}

return M
