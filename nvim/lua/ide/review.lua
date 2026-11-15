-- nvim/lua/ide/review.lua
-- Floating window UI for diff review: changed file list + per-file diff viewer.
-- Uses extmarks to animate additions (green) and deletions (red) in the diff.

local M = {}
local anim = require("ide.anim")

-- Highlight groups (set up once on first use)
local HL_INITIALIZED = false
local function ensure_highlights()
  if HL_INITIALIZED then return end
  HL_INITIALIZED = true
  vim.api.nvim_set_hl(0, "IDEDiffAdd",     { fg = "#14B9B5", bg = "#0e091d", bold = true })
  vim.api.nvim_set_hl(0, "IDEDiffDel",     { fg = "#BE3F50", bg = "#0e091d", bold = true })
  vim.api.nvim_set_hl(0, "IDEDiffHdr",     { fg = "#ff7f41", bg = "#0e091d", bold = true })
  vim.api.nvim_set_hl(0, "IDEBadge",       { fg = "#0e091d", bg = "#BE3F50", bold = true })
  vim.api.nvim_set_hl(0, "IDEBadgeGood",   { fg = "#0e091d", bg = "#14B9B5", bold = true })
  vim.api.nvim_set_hl(0, "IDEBadgeWarn",   { fg = "#0e091d", bg = "#ff7f41", bold = true })
  vim.api.nvim_set_hl(0, "IDETitle",       { fg = "#14B9B5", bold = true })
end

--- Create a centered floating window.
--- @param width  number  Width in columns
--- @param height number  Height in rows
--- @param title  string  Window title
--- @return buf, win
local function floating_win(width, height, title)
  ensure_highlights()
  local ui = vim.api.nvim_list_uis()[1]
  local col = math.floor((ui.width - width) / 2)
  local row = math.floor((ui.height - height) / 2)

  local buf = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_set_option_value("bufhidden", "wipe", { buf = buf })
  vim.api.nvim_set_option_value("filetype", "markdown", { buf = buf })

  local win = vim.api.nvim_open_win(buf, true, {
    relative = "editor",
    width = width,
    height = height,
    col = col,
    row = row,
    style = "minimal",
    border = "rounded",
    title = " " .. title .. " ",
    title_pos = "center",
  })

  -- Key mappings for the floating window
  local function map(key, action)
    vim.keymap.set("n", key, action, { buffer = buf, nowait = true, silent = true })
  end
  map("q",     function() vim.api.nvim_win_close(win, true) end)
  map("<Esc>", function() vim.api.nvim_win_close(win, true) end)

  return buf, win
end

--- Show the list of changed files in a floating window.
--- @param changes table  Result from ide/get_changes RPC call
function M.show_changes(changes)
  local files = changes.files or {}
  if #files == 0 then
    vim.notify("[NvimIDE] No changes tracked", vim.log.levels.INFO)
    return
  end

  local width = math.min(80, vim.o.columns - 4)
  local height = math.min(#files + 6, vim.o.lines - 4)
  local buf, win = floating_win(width, height, "Agent Changes")

  local lines = {
    string.format(
      "  +%d  -%d  files=%d  (agent=%d user=%d overlap=%d)",
      changes.total_additions or 0,
      changes.total_deletions or 0,
      #files,
      changes.total_agent_files or 0,
      changes.total_user_files or 0,
      changes.total_overlap_files or 0
    ),
    string.rep("─", width - 2),
    "",
  }

  for _, f in ipairs(files) do
    local attr_badge = ({
      AgentOnly            = "● Agent",
      UserOnly             = "● User ",
      UserAndAgentOverlap  = "⚠ Both ",
      Clean                = "  ─    ",
    })[f.attribution] or "?      "

    local status_char = ({
      Added    = "+",
      Deleted  = "-",
      Modified = "~",
      Renamed  = "r",
    })[f.status] or "?"

    table.insert(lines, string.format(
      " %s %s  +%-4d -%-4d  %s",
      status_char,
      attr_badge,
      f.additions or 0,
      f.deletions or 0,
      f.path
    ))
  end

  table.insert(lines, "")
  table.insert(lines, "  <Enter> review  |  a accept  |  r reject  |  A accept-all  |  R reject-all  |  q close")

  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_set_option_value("modifiable", false, { buf = buf })

  -- Highlight the summary line
  vim.api.nvim_buf_add_highlight(buf, -1, "IDETitle", 0, 0, -1)
  vim.api.nvim_buf_add_highlight(buf, -1, "IDEDiffHdr", 1, 0, -1)

  -- Highlight per-file lines starting from line 3 (0-indexed: 3)
  for i, f in ipairs(files) do
    local row = i + 2  -- 0-indexed
    local hl = ({
      AgentOnly           = "IDEBadge",
      UserOnly            = "IDEBadgeGood",
      UserAndAgentOverlap = "IDEBadgeWarn",
    })[f.attribution] or "Normal"
    vim.api.nvim_buf_add_highlight(buf, -1, hl, row, 3, 10)
  end

  -- Animate lines in with staggered delay
  if true then
    anim.stagger_lines(buf, 3, #files, 30)
  end

  -- File-level key mappings
  local rpc = require("ide.rpc")

  local function current_file()
    local line = vim.api.nvim_win_get_cursor(win)[1]
    local idx = line - 3  -- header is 3 lines
    if idx >= 1 and idx <= #files then
      return files[idx]
    end
    return nil
  end

  vim.keymap.set("n", "<Enter>", function()
    local f = current_file()
    local path = f and f.path
    if path then
      vim.api.nvim_win_close(win, true)
      rpc.request("ide/get_file_diff", { path = path }, function(result, err)
        if err then
          vim.notify("[NvimIDE] " .. (err.message or "error"), vim.log.levels.ERROR)
          return
        end
        M.show_diff(result, path, {})
      end)
    end
  end, { buffer = buf, silent = true })

  vim.keymap.set("n", "a", function()
    local f = current_file()
    if f and f.path then
      rpc.request("ide/accept_file", { path = f.path, hash = f.current_hash or 0 }, function(_, err)
        if err then
          vim.notify("[NvimIDE] " .. (err.message or "error"), vim.log.levels.ERROR)
        else
          vim.notify("[NvimIDE] Accepted " .. f.path, vim.log.levels.INFO)
          vim.api.nvim_win_close(win, true)
        end
      end)
    end
  end, { buffer = buf, silent = true })

  vim.keymap.set("n", "r", function()
    local f = current_file()
    if f and f.path then
      rpc.request("ide/reject_file", { path = f.path, hash = f.current_hash or 0 }, function(_, err)
        if err then
          vim.notify("[NvimIDE] " .. (err.message or "error"), vim.log.levels.ERROR)
        else
          vim.notify("[NvimIDE] Rejected " .. f.path, vim.log.levels.INFO)
          vim.api.nvim_win_close(win, true)
        end
      end)
    end
  end, { buffer = buf, silent = true })

  vim.keymap.set("n", "A", function()
    rpc.request("ide/accept_all", {}, function(result, _)
      vim.notify("[NvimIDE] Accepted " .. (result and result.accepted_count or 0) .. " files", vim.log.levels.INFO)
      vim.api.nvim_win_close(win, true)
    end)
  end, { buffer = buf, silent = true })

  vim.keymap.set("n", "R", function()
    rpc.request("ide/reject_all", {}, function(result, _)
      vim.notify("[NvimIDE] Rejected " .. (result and result.reverted_count or 0) .. " files", vim.log.levels.INFO)
      vim.api.nvim_win_close(win, true)
    end)
  end, { buffer = buf, silent = true })
end

--- Show a diff for a single file in a floating window.
--- Lines prefixed with "+" are highlighted green, "-" red.
--- @param diff   table  FileDiff result from ide/get_file_diff
--- @param path   string Relative file path
--- @param config table  Plugin config
function M.show_diff(diff, path, config)
  ensure_highlights()

  local hunks = diff.hunks or {}
  if #hunks == 0 then
    vim.notify("[NvimIDE] No diff for " .. path, vim.log.levels.INFO)
    return
  end

  -- Build display lines
  local lines = {
    "  " .. path,
    string.format("  +%d  -%d  %s",
      diff.additions or 0,
      diff.deletions or 0,
      diff.attribution or ""),
    string.rep("─", 76),
    "",
  }
  local line_hl = {}  -- { row, hl_group }

  for _, hunk in ipairs(hunks) do
    local hdr = string.format(
      "@@ -%d,%d +%d,%d @@",
      hunk.old_start or 0, hunk.old_lines or 0,
      hunk.new_start or 0, hunk.new_lines or 0
    )
    table.insert(lines, hdr)
    table.insert(line_hl, { #lines - 1, "IDEDiffHdr" })

    for _, dline in ipairs(hunk.lines or {}) do
      local tag = dline.tag or " "
      local text = dline.content or ""
      table.insert(lines, tag .. " " .. text)
      if tag == "+" then
        table.insert(line_hl, { #lines - 1, "IDEDiffAdd" })
      elseif tag == "-" then
        table.insert(line_hl, { #lines - 1, "IDEDiffDel" })
      end
    end
    table.insert(lines, "")
  end

  table.insert(lines, "  a accept  |  r reject  |  q close")

  local width  = math.min(90, vim.o.columns - 4)
  local height = math.min(#lines + 2, vim.o.lines - 4)
  local buf, win = floating_win(width, height, "Diff: " .. vim.fn.fnamemodify(path, ":t"))

  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.api.nvim_set_option_value("modifiable", false, { buf = buf })

  -- Apply highlights
  for _, hl in ipairs(line_hl) do
    vim.api.nvim_buf_add_highlight(buf, -1, hl[2], hl[1], 0, -1)
  end
  vim.api.nvim_buf_add_highlight(buf, -1, "IDETitle", 0, 0, -1)

  -- Animate diff lines
  anim.stagger_lines(buf, 4, #lines - 4, 20)

  local rpc = require("ide.rpc")

  vim.keymap.set("n", "a", function()
    rpc.request("ide/accept_file", { path = path, hash = diff.current_hash or 0 }, function(_, err)
      if err then
        vim.notify("[NvimIDE] Accept failed: " .. (err.message or "?"), vim.log.levels.ERROR)
      else
        vim.notify("[NvimIDE] Accepted " .. path, vim.log.levels.INFO)
        vim.api.nvim_win_close(win, true)
        vim.cmd("checktime")
      end
    end)
  end, { buffer = buf, silent = true })

  vim.keymap.set("n", "r", function()
    rpc.request("ide/reject_file", { path = path, hash = diff.current_hash or 0 }, function(_, err)
      if err then
        vim.notify("[NvimIDE] Reject failed: " .. (err.message or "?"), vim.log.levels.ERROR)
      else
        vim.notify("[NvimIDE] Rejected " .. path, vim.log.levels.INFO)
        vim.api.nvim_win_close(win, true)
        vim.cmd("checktime")
      end
    end)
  end, { buffer = buf, silent = true })
end

return M
