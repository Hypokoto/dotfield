---@type Base46Theme
-- lua/themes/dotfield.lua — Base46 theme for NvChad
-- Obsidian Mint Theme — Single Source of Truth

local M = {}

M.base_30 = {
  white         = "#ECFDF5", -- Minted White
  darker_black  = "#080B0B", -- Void
  black         = "#080B0B", -- Void
  black2        = "#0E1413", -- Obsidian
  one_bg        = "#0E1413", -- Obsidian
  one_bg2       = "#16201E", -- Graphite
  one_bg3       = "#23322F", -- Iron
  grey          = "#23322F", -- Iron
  grey_fg       = "#71827E", -- Steel
  grey_fg2      = "#94A3A0", -- Ash
  light_grey    = "#94A3A0", -- Ash
  red           = "#F43F5E", -- Rose Critical
  baby_pink     = "#F43F5E", -- Rose
  pink          = "#9F1239", -- Deep Rose
  line          = "#23322F", -- Iron
  green         = "#10B981", -- Emerald
  vibrant_green = "#10B981", -- Emerald
  nord_blue     = "#71827E", -- Steel
  blue          = "#38BDF8", -- Sky Blue
  yellow        = "#FBBF24", -- Amber Gold
  sun           = "#FBBF24", -- Amber Gold
  purple        = "#818CF8", -- Violet
  dark_purple   = "#6366F1", -- Deep Violet
  teal          = "#2DD4BF", -- Mint
  orange        = "#F59E0B", -- Warm Amber
  cyan          = "#2DD4BF", -- Mint
  statusline_bg = "#0E1413", -- Obsidian
  lightbg       = "#16201E", -- Graphite
  pmenu_bg      = "#0E1413", -- Obsidian
  folder_bg     = "#2DD4BF", -- Mint
}

M.base_16 = {
  base00 = "#080B0B", -- Void (Default Background)
  base01 = "#0E1413", -- Obsidian (Lighter Background)
  base02 = "#16201E", -- Graphite (Selection Background)
  base03 = "#23322F", -- Iron (Comments, Invisibles, LineHighlight)
  base04 = "#71827E", -- Steel (Dark Foreground / Statusline)
  base05 = "#ECFDF5", -- Minted White (Default Foreground / Caret)
  base06 = "#ECFDF5", -- Minted White (Light Foreground)
  base07 = "#FFFFFF", -- Pure White (Light Background)
  base08 = "#F43F5E", -- Rose (Variables, XML Tags, Markup Bold)
  base09 = "#FBBF24", -- Amber Gold (Integers, Boolean, Constants)
  base0A = "#2DD4BF", -- Mint (Classes, Markup Bold, Search Text)
  base0B = "#10B981", -- Emerald (Strings, Inherited Class)
  base0C = "#2DD4BF", -- Mint (Support, Regex, Escape Characters)
  base0D = "#38BDF8", -- Sky Blue (Functions, Methods)
  base0E = "#2DD4BF", -- Mint (Keywords, Storage, Selector)
  base0F = "#9F1239", -- Deep Rose (Deprecated, Opening/Closing Embedded Tags)
}

M.type = "dark"

return M
