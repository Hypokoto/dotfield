local M = {}

M.base_30 = {
  white         = "#14B9B5", -- Using your bright foreground cyan for text
  darker_black  = "#000000", -- Slightly darkened background for contrast
  black         = "#000000", -- Your primary background color
  black2        = "#140e29", -- Surface/elevated background
  one_bg        = "#1b1336", -- Lighter surface
  one_bg2       = "#251b4a", -- Selection/overlay
  one_bg3       = "#2f225e", -- Brighter overlay
  grey          = "#ffbe74", -- Using color14 for subtle text/muted elements
  grey_fg       = "#ffbe74",
  grey_fg2      = "#ffbe74",
  light_grey    = "#ffbe74",
  red           = "#BE3F50", -- Your main accent/red (color4)
  baby_pink     = "#FD3E6A", -- Color11
  pink          = "#9147a8", -- Color5
  line          = "#251b4a", -- Grid/Line background
  green         = "#7cd699", -- Color3
  vibrant_green = "#c8e967", -- Color1
  nord_blue     = "#04C5F0", -- Color12
  blue          = "#04C5F0",
  yellow        = "#ffbe74", -- Color14
  sun           = "#ffbe74",
  purple        = "#9147a8", -- Color5
  dark_purple   = "#6C032C", -- Color13
  teal          = "#11AEB3", -- Color15
  orange        = "#FF7F41", -- Your cursor/orange color
  cyan          = "#14B9B5", -- Main foreground cyan
  statusline_bg = "#140e29",
  lightbg       = "#251b4a",
  pmenu_bg      = "#14B9B5",
  folder_bg     = "#04C5F0",
}

M.base_16 = {
  base00 = "#000000", -- background
  base01 = "#140e29", -- surface
  base02 = "#251b4a", -- overlay
  base03 = "#ffbe74", -- muted/comments
  base04 = "#11AEB3", -- statusline text
  base05 = "#14B9B5", -- foreground text
  base06 = "#ffffff", -- pure white fallback
  base07 = "#ffbe74", -- extra light accent
  base08 = "#BE3F50", -- variables / red accent
  base09 = "#FF7F41", -- constants / orange
  base0A = "#ffbe74", -- classes / yellow-orange
  base0B = "#7cd699", -- strings / green
  base0C = "#11AEB3", -- regex / teal
  base0D = "#04C5F0", -- functions / blue
  base0E = "#9147a8", -- keywords / purple
  base0F = "#CE4F48", -- deprecated / dark red
}

M.type = "dark"

return M
