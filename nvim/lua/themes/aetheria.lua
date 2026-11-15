---@type Base46Theme
-- lua/themes/aetheria.lua — Base46 theme for NvChad
-- Single source of truth: palette.md exact tokens.

local M = {}

M.base_30 = {
  white         = "#F2F6FF", -- Text
  darker_black  = "#0A0C12", -- Background
  black         = "#0A0C12", -- Background
  black2        = "#11141D", -- Surface
  one_bg        = "#11141D", -- Surface
  one_bg2       = "#1C2230", -- Overlay
  one_bg3       = "#1C2230", -- Overlay
  grey          = "#8D95B3", -- Muted
  grey_fg       = "#8D95B3", -- Muted
  grey_fg2      = "#8D95B3", -- Muted
  light_grey    = "#B4BAD1", -- Lightened Muted for legibility
  red           = "#FF5470", -- Danger
  baby_pink     = "#FF5470", -- Danger fallback
  pink          = "#A05CFF", -- Highlight
  line          = "#1C2230", -- Overlay
  green         = "#4DFF91", -- Success
  vibrant_green = "#4DFF91", -- Success
  nord_blue     = "#5B8CFF", -- Secondary
  blue          = "#5B8CFF", -- Secondary
  yellow        = "#FFCC66", -- Warning
  sun           = "#FFCC66", -- Warning
  purple        = "#A05CFF", -- Highlight
  dark_purple   = "#A05CFF", -- Highlight
  teal          = "#61E6FF", -- Info
  orange        = "#FFCC66", -- Warning fallback
  cyan          = "#00D9FF", -- Primary
  statusline_bg = "#11141D", -- Surface
  lightbg       = "#1C2230", -- Overlay
  pmenu_bg      = "#00D9FF", -- Primary
  folder_bg     = "#5B8CFF", -- Secondary
}

M.base_16 = {
  base00 = "#0A0C12", -- Background
  base01 = "#11141D", -- Surface
  base02 = "#1C2230", -- Overlay
  base03 = "#8D95B3", -- Muted
  base04 = "#B4BAD1", -- Lightened Muted
  base05 = "#F2F6FF", -- Text
  base06 = "#F2F6FF", -- Text
  base07 = "#FFFFFF", -- Brightest foreground
  base08 = "#FF5470", -- Danger (Variables / XML Tags)
  base09 = "#FFCC66", -- Warning (Integers / Constants)
  base0A = "#FFCC66", -- Warning (Classes / Search Bg)
  base0B = "#4DFF91", -- Success (Strings)
  base0C = "#61E6FF", -- Info (Regex / Escapes)
  base0D = "#5B8CFF", -- Secondary (Functions / Methods)
  base0E = "#A05CFF", -- Highlight (Keywords / Storage)
  base0F = "#FF5470", -- Danger (Deprecated / Delimiters)
}

M.type = "dark"

return M
