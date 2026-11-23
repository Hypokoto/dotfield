-- Ly Display Manager Configuration managed by Dotfield
-- Obsidian Mint — Cyber-Botanical Interface (60-30-10 palette)
-- Target installation: /etc/ly/config.lua

return {
    -- Terminal & Colors (Dotfield Obsidian Mint: Void #080B0B, Mint #2DD4BF, Rose #F43F5E, Minted White #ECFDF5)
    bg = 0x080B0B,
    fg = 0xECFDF5,
    border_fg = 0x2DD4BF,
    border_bg = 0x080B0B,
    text_fg = 0xECFDF5,
    error_fg = 0xF43F5E,
    error_bg = 0x080B0B,

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
