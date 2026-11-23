---@type Base46Theme
-- lua/themes/dotfield.lua — Base46 theme for NvChad
-- Cyber Synthwave Theme — Void Black · Deep Violet · Purple · Cyan-Teal · Neon Blue

local M = {}

M.base_30 = {
  white         = "#ffffff", -- Pure White
  darker_black  = "#000000", -- Void Black
  black         = "#000000", -- Void Black
  black2        = "#140e29", -- Surface
  one_bg        = "#140e29", -- Surface
  one_bg2       = "#251b4a", -- Overlay
  one_bg3       = "#251b4a", -- Overlay
  grey          = "#251b4a", -- Overlay
  grey_fg       = "#11AEB3", -- Statusline Text
  grey_fg2      = "#ffbe74", -- Muted / Comments
  light_grey    = "#ffbe74", -- Comments
  red           = "#BE3F50", -- Red Accent
  baby_pink     = "#BE3F50", -- Coral Red
  pink          = "#CE4F48", -- Dark Red
  line          = "#251b4a", -- Borders & Dividers
  green         = "#7cd699", -- Strings / Green
  vibrant_green = "#7cd699", -- Emerald
  nord_blue     = "#11AEB3", -- Teal
  blue          = "#04C5F0", -- Functions / Blue
  yellow        = "#FF7F41", -- Constants / Orange
  sun           = "#ffbe74", -- Peach / Yellow-Orange
  purple        = "#9147a8", -- Keywords / Purple
  dark_purple   = "#9147a8", -- Purple
  teal          = "#14B9B5", -- Cyan-Teal Foreground
  orange        = "#FF7F41", -- Constants / Orange
  cyan          = "#04C5F0", -- Neon Blue
  statusline_bg = "#140e29", -- Surface
  lightbg       = "#251b4a", -- Overlay
  pmenu_bg      = "#140e29", -- Surface
  folder_bg     = "#04C5F0", -- Neon Blue Accent
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
