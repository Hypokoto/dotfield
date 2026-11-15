require "nvchad.autocmds"

vim.api.nvim_create_autocmd("BufDelete", {
  group = vim.api.nvim_create_augroup("DashboardAutoOpen", { clear = true }),
  callback = function(ev)
    if vim.bo[ev.buf].filetype == "snacks_dashboard" then
      return
    end

    vim.schedule(function()
      for _, buf in ipairs(vim.api.nvim_list_bufs()) do
        if vim.api.nvim_buf_is_valid(buf) and vim.bo[buf].buflisted then
          local ft = vim.bo[buf].filetype
          if ft == "snacks_dashboard" then
            return
          end
          local name = vim.api.nvim_buf_get_name(buf)
          local bt = vim.bo[buf].buftype
          local modified = vim.bo[buf].modified
          if name ~= "" or ft ~= "" or bt ~= "" or modified then
            return
          end
          local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
          if #lines > 1 or (#lines == 1 and lines[1] ~= "") then
            return
          end
        end
      end
      require("snacks").dashboard.open({ win = 0 })
    end)
  end,
})


