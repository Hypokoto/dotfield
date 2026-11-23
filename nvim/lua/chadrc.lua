---@type ChadrcConfig
local M = {}

M.base46 = {
  theme = "dotfield",
  hl_add = {
    MinimalDashboardHeader = { fg = "#04C5F0", bold = true },
    MinimalDashboardFooter = { fg = "#BE3F50" },
    FloatBorder            = { fg = "#251B4A", bg = "#000000" },
    NormalFloat            = { bg = "#140E29" },
    NvimTreeNormal         = { bg = "#000000" },
    NvimTreeNormalNC       = { bg = "#000000" },
    NvimTreeWinSeparator   = { fg = "#251B4A", bg = "#000000" },
    NvimTreeFolderName     = { fg = "#04C5F0", bold = true },
    NvimTreeOpenedFolderName = { fg = "#14B9B5", bold = true },
    LineNr                 = { fg = "#251B4A" },
    CursorLineNr           = { fg = "#04C5F0", bold = true },
    CursorLine             = { bg = "#140E29" },
    DiagnosticError        = { fg = "#BE3F50" },
    DiagnosticWarn         = { fg = "#FF7F41" },
    DiagnosticInfo         = { fg = "#04C5F0" },
    DiagnosticHint         = { fg = "#11AEB3" },
    Visual                 = { bg = "#251B4A" },
    Pmenu                  = { bg = "#140E29", fg = "#14B9B5" },
    PmenuSel               = { bg = "#251B4A", fg = "#04C5F0", bold = true },
    PmenuBorder            = { fg = "#251B4A", bg = "#140E29" },
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
