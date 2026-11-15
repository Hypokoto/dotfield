-- nvim/lua/ide/anim.lua
-- Extmark-based animation utilities for the IDE plugin.
-- Provides fade-in / stagger effects for diff lines in floating windows.

local M = {}

--- Stagger-reveal lines in a buffer by making them briefly transparent
--- (via a transparent highlight) then fading to normal.
--- This is a lightweight CSS-style entrance effect using vim timers.
---
--- @param buf        number  Buffer handle
--- @param start_line number  0-indexed first line to animate
--- @param count      number  Number of lines to animate
--- @param delay_ms   number  Delay between each line reveal (ms)
function M.stagger_lines(buf, start_line, count, delay_ms)
  if count <= 0 then return end

  -- We can't truly fade in terminal, so we "reveal" lines by briefly
  -- showing them with a dim concealed highlight then removing it.
  -- The effect: lines appear to materialize in sequence.

  local ns = vim.api.nvim_create_namespace("ide_anim")

  -- Initially conceal all target lines with a virtual text overlay
  local marks = {}
  for i = 0, count - 1 do
    local row = start_line + i
    local ok, mark_id = pcall(
      vim.api.nvim_buf_set_extmark,
      buf, ns, row, 0,
      {
        hl_group = "Conceal",
        end_row = row,
        end_col = 0,
        hl_eol = true,
        priority = 200,
      }
    )
    if ok then
      marks[i] = mark_id
    end
  end

  -- Reveal each line after its staggered delay
  for i = 0, count - 1 do
    local row = start_line + i
    local mark_id = marks[i]
    if mark_id then
      vim.defer_fn(function()
        -- Safety: buffer may have been wiped by the time the timer fires
        if not vim.api.nvim_buf_is_valid(buf) then return end
        pcall(vim.api.nvim_buf_del_extmark, buf, ns, mark_id)
      end, delay_ms * (i + 1))
    end
  end
end

--- Flash a single line briefly with a highlight to draw attention.
--- @param buf     number  Buffer handle
--- @param row     number  0-indexed row
--- @param hl      string  Highlight group name
--- @param duration_ms number  How long to show the flash (ms)
function M.flash_line(buf, row, hl, duration_ms)
  local ns = vim.api.nvim_create_namespace("ide_flash")
  local mark_id = vim.api.nvim_buf_set_extmark(buf, ns, row, 0, {
    hl_group = hl,
    end_row = row,
    end_col = 0,
    hl_eol = true,
    priority = 300,
  })
  vim.defer_fn(function()
    if not vim.api.nvim_buf_is_valid(buf) then return end
    pcall(vim.api.nvim_buf_del_extmark, buf, ns, mark_id)
  end, duration_ms)
end

return M
