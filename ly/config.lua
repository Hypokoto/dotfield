-- Ly Display Manager Configuration managed by Dotfield
-- BLACKSITE Industrial Noir — Tactical Interface (65-20-8-5-2 palette)
-- Target installation: /etc/ly/config.lua

return {
    -- Terminal & Colors (Dotfield Fluent: Void #121216, Fluent Blue #60CDFF, Crimson #E63946, Bone #F0F0F5)
    bg = 0x121216,
    fg = 0xF0F0F5,
    border_fg = 0x60CDFF,
    border_bg = 0x121216,
    text_fg = 0xF0F0F5,
    error_fg = 0xE63946,
    error_bg = 0x121216,

    -- Visual Styling & Frame
    box_title = "ARCH LINUX // DOTFIELD",
    hide_borders = false,
    hide_fkeys = false,
    blank_password = false,
    asterisk = "•",
    margin_box_h = 2,
    margin_box_v = 1,

    -- Clock & Status
    clock = "%a %d %b %H:%M:%S",
    bigclock = true,

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
