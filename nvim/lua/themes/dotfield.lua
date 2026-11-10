---@type Base46Theme
-- lua/themes/dotfield.lua — Base46 theme for NvChad
-- BLACKSITE Tactical Workstation Theme — Single Source of Truth

local M = {}

M.base_30 = {
  white         = "#E8E6E3", -- Bone
  darker_black  = "#0A0A0C", -- Void
  black         = "#0A0A0C", -- Void
  black2        = "#121216", -- Obsidian
  one_bg        = "#121216", -- Obsidian
  one_bg2       = "#1C1C23", -- Graphite
  one_bg3       = "#34343D", -- Iron
  grey          = "#34343D", -- Iron
  grey_fg       = "#8B949E", -- Steel
  grey_fg2      = "#A1A1AA", -- Ash
  light_grey    = "#A1A1AA", -- Ash
  red           = "#E63946", -- Crimson
  baby_pink     = "#E63946", -- Crimson
  pink          = "#9D0208", -- Blood Red
  line          = "#34343D", -- Iron
  green         = "#7BC47F", -- Success Green
  vibrant_green = "#7BC47F", -- Success Green
  nord_blue     = "#8B949E", -- Steel
  blue          = "#8B949E", -- Steel
  yellow        = "#FFB000", -- Industrial Amber
  sun           = "#FFD166", -- Pale Amber
  purple        = "#FFD166", -- Pale Amber
  dark_purple   = "#FFB000", -- Industrial Amber
  teal          = "#7BC47F", -- Success Green
  orange        = "#FFB000", -- Industrial Amber
  cyan          = "#7BC47F", -- Success Green
  statusline_bg = "#121216", -- Obsidian
  lightbg       = "#1C1C23", -- Graphite
  pmenu_bg      = "#121216", -- Obsidian
  folder_bg     = "#FFB000", -- Industrial Amber
}

M.base_16 = {
  base00 = "#0A0A0C", -- Void (Default Background)
  base01 = "#121216", -- Obsidian (Lighter Background)
  base02 = "#1C1C23", -- Graphite (Selection Background)
  base03 = "#34343D", -- Iron (Comments, Invisibles, LineHighlight)
  base04 = "#8B949E", -- Steel (Dark Foreground / Statusline)
  base05 = "#E8E6E3", -- Bone (Default Foreground / Caret)
  base06 = "#E8E6E3", -- Bone (Light Foreground)
  base07 = "#E8E6E3", -- Bone (Light Background)
  base08 = "#E63946", -- Crimson (Variables, XML Tags, Markup Bold)
  base09 = "#FFB000", -- Industrial Amber (Integers, Boolean, Constants)
  base0A = "#FFD166", -- Pale Amber (Classes, Markup Bold, Search Text)
  base0B = "#7BC47F", -- Success Green (Strings, Inherited Class)
  base0C = "#8B949E", -- Steel (Support, Regex, Escape Characters)
  base0D = "#FFD166", -- Pale Amber (Functions, Methods)
  base0E = "#FFB000", -- Industrial Amber (Keywords, Storage, Selector)
  base0F = "#9D0208", -- Blood Red (Deprecated, Opening/Closing Embedded Tags)
}

M.type = "dark"

return M
