-- BLACKSITE Industrial Noir Tab Bar & Right Status Telemetry for WezTerm
-- Managed by Dotfield — Tactical Telemetry · Zoom Indicators · Void Black Canvas

local wezterm = require("wezterm")

local M = {}

--- Strips path directory to get base process or tab name
local function basename(path)
    if not path or path == "" then
        return "term"
    end
    return string.gsub(path, "(.*[/\\])(.*)", "%2")
end

function M.apply_to_config(config)
    -- Formatting individual tabs
    wezterm.on("format-tab-title", function(tab, _tabs, _panes, _config, hover, max_width)
        local is_active = tab.is_active
        local tab_idx = tab.tab_index + 1

        -- Detect if any pane in this tab is currently zoomed
        local has_zoomed = false
        for _, pane in ipairs(tab.panes) do
            if pane.is_zoomed then
                has_zoomed = true
                break
            end
        end

        local title = tab.active_pane and tab.active_pane.title or ""
        if title == "" or title:find("^wezterm") then
            title = basename(tab.active_pane and tab.active_pane.foreground_process_name or "zsh")
        end

        -- Truncate title if exceeding tab width budget
        local max_title_len = math.max(10, max_width - 12)
        if #title > max_title_len then
            title = string.sub(title, 1, max_title_len - 1) .. "…"
        end

        local bg = is_active and "#181920" or (hover and "#111216" or "#08080A")
        local fg = is_active and "#FFB000" or (hover and "#8F94A6" or "#606474")

        local elements = {
            { Background = { Color = bg } },
            { Foreground = { Color = "#282A36" } },
            { Text = is_active and " ▍" or " │" },
            { Foreground = { Color = fg } },
            { Attribute = { Intensity = is_active and "Bold" or "Normal" } },
            { Text = string.format(" %d // %s ", tab_idx, title) },
        }

        if has_zoomed then
            table.insert(elements, { Foreground = { Color = "#E63946" } })
            table.insert(elements, { Attribute = { Intensity = "Bold" } })
            table.insert(elements, { Text = "[ZOOM] " })
        end

        table.insert(elements, { Foreground = { Color = "#282A36" } })
        table.insert(elements, { Text = is_active and "▐" or "│" })

        return elements
    end)

    -- Right Status Telemetry HUD
    wezterm.on("update-right-status", function(window, pane)
        local status_elements = {}

        -- 1. Active Leader or Key Table Badge
        if window:leader_is_active() then
            table.insert(status_elements, { Background = { Color = "#E63946" } })
            table.insert(status_elements, { Foreground = { Color = "#08080A" } })
            table.insert(status_elements, { Attribute = { Intensity = "Bold" } })
            table.insert(status_elements, { Text = " [SYS:LEADER] " })
            table.insert(status_elements, { Background = { Color = "#08080A" } })
            table.insert(status_elements, { Text = " " })
        end

        local key_table = window:active_key_table()
        if key_table then
            table.insert(status_elements, { Background = { Color = "#FFB000" } })
            table.insert(status_elements, { Foreground = { Color = "#08080A" } })
            table.insert(status_elements, { Attribute = { Intensity = "Bold" } })
            table.insert(status_elements, { Text = string.format(" [%s] ", key_table:upper()) })
            table.insert(status_elements, { Background = { Color = "#08080A" } })
            table.insert(status_elements, { Text = " " })
        end

        -- 2. Current Directory or Workspace
        local workspace = window:active_workspace()
        if workspace and workspace ~= "default" then
            table.insert(status_elements, { Background = { Color = "#181920" } })
            table.insert(status_elements, { Foreground = { Color = "#FFB000" } })
            table.insert(status_elements, { Text = string.format(" %s ", workspace) })
            table.insert(status_elements, { Background = { Color = "#08080A" } })
            table.insert(status_elements, { Foreground = { Color = "#282A36" } })
            table.insert(status_elements, { Text = "│" })
        end

        -- 3. Local Time & Date Pill
        local time_str = wezterm.strftime("%H:%M")
        local date_str = wezterm.strftime("%d-%b")

        table.insert(status_elements, { Background = { Color = "#111216" } })
        table.insert(status_elements, { Foreground = { Color = "#606474" } })
        table.insert(status_elements, { Text = "  " })
        table.insert(status_elements, { Foreground = { Color = "#FFB000" } })
        table.insert(status_elements, { Attribute = { Intensity = "Bold" } })
        table.insert(status_elements, { Text = time_str .. " " })

        table.insert(status_elements, { Foreground = { Color = "#282A36" } })
        table.insert(status_elements, { Text = "│" })

        table.insert(status_elements, { Foreground = { Color = "#8F94A6" } })
        table.insert(status_elements, { Text = "  " .. date_str .. " " })

        window:set_right_status(wezterm.format(status_elements))
    end)
end

return M
