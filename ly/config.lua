-- Ly Display Manager Configuration managed by Dotfield
-- Cyber Synthwave — Void Black · Cyan-Teal · Neon Blue · Red Accent
-- Target installation: /etc/ly/config.lua

return {
    -- Terminal & Colors (Dotfield Cyber Synthwave: Void #000000, Cyan-Teal #14B9B5, Neon Blue #04C5F0, Danger #BE3F50)
    bg = 0x000000,
    fg = 0x14B9B5,
    border_fg = 0x04C5F0,
    border_bg = 0x000000,
    text_fg = 0x14B9B5,
    error_fg = 0xBE3F50,
    error_bg = 0x000000,

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
