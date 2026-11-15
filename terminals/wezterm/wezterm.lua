-- WezTerm Configuration — Managed by Dotfield
-- BLACKSITE Industrial Noir — Tactical Interface (65-20-8-5-2 palette)
-- Architecture: Pure GPU Acceleration · Alt-based Navigation · Exception Telemetry

local wezterm = require("wezterm")
local config = wezterm.config_builder and wezterm.config_builder() or {}

-- Ensure modular sub-configs resolve across all invocation paths
local config_dir = wezterm.config_dir or (debug.getinfo(1, "S").source:match("@?(.*/)") or ".")
if config_dir then
    package.path = config_dir .. "/?.lua;" .. config_dir .. "/?/init.lua;" .. package.path
end

local colors = require("colors")
local keys = require("keys")
local status = require("status")

-- --- Low-Latency Rendering & Direct GPU Push ---
config.front_end = "WebGpu"
config.webgpu_power_preference = "HighPerformance"
config.max_fps = 120
config.animation_fps = 1
config.cursor_blink_rate = 0
config.default_cursor_style = "SteadyBlock"

-- --- Native Wayland Integration ---
config.enable_wayland = true
config.prefer_egl = true

-- --- Typography & Font Rendering ---
config.font = wezterm.font_with_fallback({
    { family = "JetBrainsMono Nerd Font", weight = "Regular" },
    { family = "Noto Color Emoji" },
})
config.font_size = 11.0
config.harfbuzz_features = { "calt=1", "clig=1", "liga=1" }
config.underline_position = -2
config.underline_thickness = 1

-- --- Window Geometry & Decorations (Zero Frame Overhead) ---
config.window_decorations = "NONE"
config.window_padding = {
    left = 12,
    right = 12,
    top = 12,
    bottom = 12,
}
config.window_background_opacity = 1.0
config.window_close_confirmation = "NeverPrompt"
config.unzoom_on_switch_pane = false

-- --- Tab Bar Presentation (BLACKSITE Tactical Interface) ---
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = false
config.show_new_tab_button_in_tab_bar = false
config.tab_max_width = 36
config.hide_tab_bar_if_only_one_tab = false

-- --- Scrollback & Pure Native Search ---
config.scrollback_lines = 10000
config.mouse_wheel_scrolls_tabs = false

-- --- Hyperlinks & URL Highlighting ---
config.hyperlink_rules = wezterm.default_hyperlink_rules()

-- --- Silence & Bell Suppression ---
config.audible_bell = "Disabled"
config.visual_bell = {
    fade_in_function = "EaseIn",
    fade_in_duration_ms = 0,
    fade_out_function = "EaseOut",
    fade_out_duration_ms = 0,
}

-- --- Hygiene & Telemetry Disabling ---
config.check_for_updates = false
config.check_for_updates_interval_seconds = 31536000

-- --- Apply Modular Components ---
colors.apply_to_config(config)
keys.apply_to_config(config)
status.apply_to_config(config)

return config
