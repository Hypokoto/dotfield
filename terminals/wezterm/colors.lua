-- BLACKSITE Industrial Noir (65-20-8-5-2 palette) for WezTerm
-- Managed by Dotfield — Void Black · Obsidian · Graphite · Iron · Bone · Tactical Amber · Crimson

local M = {}

M.colors = {
    -- Base canvas & primary foreground
    foreground = "#E6E6EB",
    background = "#08080A",

    -- Cursor styling
    cursor_bg = "#FFB000",
    cursor_fg = "#08080A",
    cursor_border = "#FFB000",

    -- Text selection
    selection_fg = "#E6E6EB",
    selection_bg = "#181920",

    -- Scrollbar & split lines
    scrollbar_thumb = "#282A36",
    split = "#282A36",

    -- Normal ANSI colors (0-7)
    ansi = {
        "#181920", -- black (Graphite)
        "#E63946", -- red (Crimson)
        "#4AF626", -- green (Terminal Green)
        "#FFB000", -- yellow (Tactical Amber)
        "#4C78A8", -- blue (Tactical Blue)
        "#9D7CD8", -- magenta
        "#56B6C2", -- cyan (Ice Cyan)
        "#E6E6EB", -- white (Bone)
    },

    -- Bright ANSI colors (8-15)
    brights = {
        "#282A36", -- bright black (Iron)
        "#FF4D5A", -- bright red
        "#69FF47", -- bright green
        "#FFD166", -- bright yellow
        "#6BA0DB", -- bright blue
        "#B392F0", -- bright magenta
        "#70D4E0", -- bright cyan
        "#FFFFFF", -- bright white
    },

    -- QuickSelect highlighting
    quick_select_label_bg = { Color = "#FFB000" },
    quick_select_label_fg = { Color = "#08080A" },
    quick_select_match_bg = { Color = "#181920" },
    quick_select_match_fg = { Color = "#E63946" },

    -- Copy mode highlighting
    copy_mode_active_highlight_bg = { Color = "#181920" },
    copy_mode_active_highlight_fg = { Color = "#FFB000" },
    copy_mode_inactive_highlight_bg = { Color = "#111216" },
    copy_mode_inactive_highlight_fg = { Color = "#8F94A6" },

    -- Tab Bar Color Tokens
    tab_bar = {
        background = "#08080A",
        active_tab = {
            bg_color = "#181920",
            fg_color = "#FFB000",
            intensity = "Bold",
            underline = "None",
            italic = false,
            strikethrough = false,
        },
        inactive_tab = {
            bg_color = "#08080A",
            fg_color = "#606474",
        },
        inactive_tab_hover = {
            bg_color = "#111216",
            fg_color = "#8F94A6",
            italic = false,
        },
        new_tab = {
            bg_color = "#08080A",
            fg_color = "#606474",
        },
        new_tab_hover = {
            bg_color = "#111216",
            fg_color = "#FFB000",
        },
    },
}

function M.apply_to_config(config)
    config.colors = M.colors
end

return M
