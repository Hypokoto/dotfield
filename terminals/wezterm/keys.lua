-- BLACKSITE Industrial Noir Keybindings & Navigation for WezTerm
-- Managed by Dotfield — Alt-based Splits · Vim-Aware Navigation · QuickSelect

local wezterm = require("wezterm")
local act = wezterm.action

local M = {}

--- Detect if current pane is running Neovim / Vim
local function is_vim(pane)
    local process_info = pane:get_foreground_process_info()
    if not process_info or not process_info.name then
        return false
    end
    local name = process_info.name:lower()
    return name == "nvim" or name == "vim" or name == "view" or name:find("nvim") ~= nil
end

--- Ghostty-style pane cycling that preserves zoom state across splits
local function cycle_pane_ghostty(win, pane)
    local tab = pane:tab()
    local panes = tab:panes_with_info()
    if #panes <= 1 then
        return
    end

    local is_zoomed = false
    local current_idx = 1
    for i, p in ipairs(panes) do
        if p.is_zoomed then
            is_zoomed = true
        end
        if p.is_active then
            current_idx = i
        end
    end

    local next_idx = (current_idx % #panes) + 1
    local next_pane = panes[next_idx].pane

    if is_zoomed then
        win:perform_action(act.SetPaneZoomState(false), pane)
        next_pane:activate()
        win:perform_action(act.SetPaneZoomState(true), next_pane)
    else
        next_pane:activate()
    end
end

--- Directional navigation with Neovim edge awareness and Ghostty zoom preservation
local function conditional_navigate(direction, vim_key)
    return wezterm.action_callback(function(win, pane)
        if is_vim(pane) then
            win:perform_action(act.SendKey({ key = vim_key, mods = "ALT" }), pane)
            return
        end

        local tab = pane:tab()
        local is_zoomed = false
        for _, p in ipairs(tab:panes_with_info()) do
            if p.is_zoomed and p.is_active then
                is_zoomed = true
                break
            end
        end

        if is_zoomed then
            -- Ghostty-parity: preserve zoom state when navigating across splits
            win:perform_action(act.SetPaneZoomState(false), pane)
            win:perform_action(act.ActivatePaneDirection(direction), pane)
            local active_pane = win:active_pane()
            win:perform_action(act.SetPaneZoomState(true), active_pane)
        else
            win:perform_action(act.ActivatePaneDirection(direction), pane)
        end
    end)
end

function M.apply_to_config(config)
    -- Multiplexer Leader: Ctrl+A (timeout 1000ms), harmonized with Tmux
    config.leader = { key = "a", mods = "CTRL", timeout_milliseconds = 1000 }

    config.keys = {
        -- --- Clipboard & Font Size ---
        { key = "c", mods = "CTRL|SHIFT", action = act.CopyTo("Clipboard") },
        { key = "v", mods = "CTRL|SHIFT", action = act.PasteFrom("Clipboard") },
        { key = "=", mods = "CTRL", action = act.IncreaseFontSize },
        { key = "+", mods = "CTRL", action = act.IncreaseFontSize },
        { key = "-", mods = "CTRL", action = act.DecreaseFontSize },
        { key = "0", mods = "CTRL", action = act.ResetFontSize },

        -- --- Native Tabs (Alt-based, matching Dotfield Kitty & Ghostty) ---
        { key = "t", mods = "ALT", action = act.SpawnTab("CurrentPaneDomain") },
        { key = "w", mods = "ALT", action = act.CloseCurrentPane({ confirm = false }) },
        { key = "]", mods = "ALT", action = act.ActivateTabRelative(1) },
        { key = "[", mods = "ALT", action = act.ActivateTabRelative(-1) },
        { key = "1", mods = "ALT", action = act.ActivateTab(0) },
        { key = "2", mods = "ALT", action = act.ActivateTab(1) },
        { key = "3", mods = "ALT", action = act.ActivateTab(2) },
        { key = "4", mods = "ALT", action = act.ActivateTab(3) },
        { key = "5", mods = "ALT", action = act.ActivateTab(4) },

        -- --- Native Splits (Alt-based) ---
        -- Alt+Enter: Split right (vertical line)
        { key = "Enter", mods = "ALT", action = act.SplitHorizontal({ domain = "CurrentPaneDomain" }) },
        -- Alt+Shift+Enter: Split down (horizontal line)
        { key = "Enter", mods = "ALT|SHIFT", action = act.SplitVertical({ domain = "CurrentPaneDomain" }) },

        -- Directional Pane Navigation (Vi keys)
        { key = "h", mods = "ALT", action = conditional_navigate("Left", "h") },
        { key = "j", mods = "ALT", action = conditional_navigate("Down", "j") },
        { key = "k", mods = "ALT", action = conditional_navigate("Up", "k") },
        { key = "l", mods = "ALT", action = conditional_navigate("Right", "l") },

        -- Pane Zoom & Navigation (Ghostty-parity independent zoom in/out & cycling)
        { key = "z", mods = "ALT", action = act.TogglePaneZoomState },
        { key = "Enter", mods = "CTRL|SHIFT", action = act.TogglePaneZoomState },
        { key = "o", mods = "ALT", action = wezterm.action_callback(cycle_pane_ghostty) },

        -- --- Search, Copy Mode & QuickSelect ---
        { key = "s", mods = "ALT", action = act.Search({ CaseSensitiveString = "" }) },
        { key = "f", mods = "ALT", action = act.Search({ CaseSensitiveString = "" }) },
        { key = "x", mods = "ALT", action = act.ActivateCopyMode },
        { key = "u", mods = "ALT", action = act.QuickSelect },

        -- --- Leader Multiplexer Shortcuts (Ctrl+A prefix) ---
        { key = "a", mods = "LEADER|CTRL", action = act.SendKey({ key = "a", mods = "CTRL" }) },
        { key = "h", mods = "LEADER", action = act.AdjustPaneSize({ "Left", 5 }) },
        { key = "j", mods = "LEADER", action = act.AdjustPaneSize({ "Down", 5 }) },
        { key = "k", mods = "LEADER", action = act.AdjustPaneSize({ "Up", 5 }) },
        { key = "l", mods = "LEADER", action = act.AdjustPaneSize({ "Right", 5 }) },
        { key = "r", mods = "LEADER", action = act.ReloadConfiguration },
        { key = "Space", mods = "LEADER", action = act.QuickSelect },
    }

    -- QuickSelect Regex Patterns (Git SHAs, file paths with lines, URLs, IP addresses, hashes)
    config.quick_select_patterns = {
        -- File paths with line numbers (e.g., src/main.rs:42:15 or tests/unit.rs:10)
        "[.\\w\\-+/]+:\\d+(?::\\d+)?",
        -- Git commit SHAs (7 to 40 hex characters)
        "[0-9a-f]{7,40}",
        -- IPv4 & IPv6 addresses
        "\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b",
        -- Hex colors
        "#[0-9a-fA-F]{6}\\b",
    }
end

return M
