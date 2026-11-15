-- Ly Display Manager Configuration managed by Dotfield
-- BLACKSITE Industrial Noir — Tactical Interface (65-20-8-5-2 palette)
-- Target installation: /etc/ly/config.lua

return {
    -- Terminal & Colors (Hex mapping: Void #08080A, Amber #FFB000, Crimson #E63946, Bone #E6E6EB, Iron #282A36)
    bg = 0x08080A,
    fg = 0xE6E6EB,
    border_fg = 0xFFB000,
    border_bg = 0x08080A,
    text_fg = 0xE6E6EB,
    error_fg = 0xE63946,
    error_bg = 0x08080A,

    -- Visual Styling & Frame
    box_title = "BLACKSITE // DOTFIELD",
    hide_borders = false,
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
