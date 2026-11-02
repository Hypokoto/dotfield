-- Ly Display Manager Configuration managed by Dotfield
-- Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel
-- Target installation: /etc/ly/config.lua

return {
    -- Terminal & Colors (Hex mapping)
    -- Void #0A0A0C · Amber #FFB000 · Crimson #E63946 · Bone #E8E6E3 · Iron #34343D
    bg = 0x0A0A0C,
    fg = 0xE8E6E3,
    border_fg = 0xFFB000,
    border_bg = 0x0A0A0C,
    text_fg = 0xE8E6E3,
    error_fg = 0xE63946,
    error_bg = 0x0A0A0C,

    -- Visual Styling & Frame
    hide_borders = false,
    border_radius = true,
    hide_fkeys = false,
    blank_password = false,
    asterisk = "*",
    margin_box_h = 2,
    margin_box_v = 1,

    -- Clock & Status
    clock = "%a %d %b %H:%M:%S",
    bigclock = false,

    -- Performance & Latency (Zero animation overhead)
    animate = false,
    animation = 0,

    -- Session & User State Persistence
    save = true,
    load = true,
    save_file = "/etc/ly/save",
    default_input = "login",

    -- Session Detection Paths
    waylandsessions = "/usr/share/wayland-sessions",
    xsessions = "/usr/share/xsessions",

    -- Audio / Input
    numlock = false,
    clear_password = true,
}
