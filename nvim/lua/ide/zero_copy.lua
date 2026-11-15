local ffi = require("ffi")

ffi.cdef[[
  int minimal_nvim_init(void (*cb)(void));
  void minimal_nvim_shutdown(void);
  uint32_t minimal_nvim_get_num_files(void);
  const char* minimal_nvim_get_file_path(uint32_t file_idx, size_t* len_out);
  uint32_t minimal_nvim_get_num_hunks(uint32_t file_idx);
  uint32_t minimal_nvim_get_hunk_new_start(uint32_t file_idx, uint32_t hunk_idx);
  uint32_t minimal_nvim_get_num_lines(uint32_t file_idx, uint32_t hunk_idx);
  uint8_t minimal_nvim_get_line_tag(uint32_t file_idx, uint32_t hunk_idx, uint32_t line_idx);
]]

local M = {}

function M.setup()
  local script_path = debug.getinfo(1, "S").source:sub(2)
  local plugin_root = vim.fn.fnamemodify(script_path, ":h:h:h:h")
  
  local lib_name = "libminimal_nvim.so"
  if jit.os == "OSX" then
    lib_name = "libminimal_nvim.dylib"
  elseif jit.os == "Windows" then
    lib_name = "minimal_nvim.dll"
  end
  
  local lib_path
  local candidates = {
    vim.fn.expand("~") .. "/.local/lib/minimal/" .. lib_name,
    plugin_root .. "/target/release/" .. lib_name,
    plugin_root .. "/target/debug/" .. lib_name,
    vim.fn.expand("~") .. "/minimal/target/release/" .. lib_name,
    vim.fn.expand("~") .. "/minimal/target/debug/" .. lib_name,
  }
  for _, c in ipairs(candidates) do
    if vim.fn.filereadable(c) == 1 then
      lib_path = c
      break
    end
  end
  if not lib_path then
    return
  end
  
  local ok, lib = pcall(ffi.load, lib_path)
  if not ok then
    print("Failed to load native IPC library from: " .. lib_path)
    return
  end
  
  local len_out = ffi.new("size_t[1]")
  
  local cb = ffi.cast("void (*)(void)", function()
    -- Run protected call because Lua errors crossing C boundary crash Nvim
    pcall(function()
      local anim = require("ide.anim")
      local num_files = lib.minimal_nvim_get_num_files()
      
      for f_idx = 0, num_files - 1 do
        local path_ptr = lib.minimal_nvim_get_file_path(f_idx, len_out)
        if path_ptr ~= nil then
          local path = ffi.string(path_ptr, len_out[0])
          local bufnr = vim.fn.bufnr(path)
          
          if bufnr ~= -1 and vim.api.nvim_buf_is_loaded(bufnr) then
            -- Avoid checktime in headless tests to prevent aborts, but normally we'd want it
            if #vim.api.nvim_list_uis() > 0 then
              vim.cmd("checktime " .. bufnr)
            end
            
            local num_hunks = lib.minimal_nvim_get_num_hunks(f_idx)
            for h_idx = 0, num_hunks - 1 do
              local row = lib.minimal_nvim_get_hunk_new_start(f_idx, h_idx) - 1
              if row < 0 then row = 0 end
              
              local num_lines = lib.minimal_nvim_get_num_lines(f_idx, h_idx)
              for l_idx = 0, num_lines - 1 do
                local tag_byte = lib.minimal_nvim_get_line_tag(f_idx, h_idx, l_idx)
                local tag = string.char(tag_byte)
                
                if tag == "+" then
                  anim.flash_line(bufnr, row, "DiffAdd", 500)
                  row = row + 1
                elseif tag == " " then
                  row = row + 1
                end
              end
            end
          end
        end
      end
    end)
  end)
  
  M._cb = cb
  M._lib = lib
  
  local res = lib.minimal_nvim_init(cb)
  if res ~= 0 then
    print("Failed to init native IPC: " .. tostring(res))
  end
end

function M.teardown()
  if M._lib then
    M._lib.minimal_nvim_shutdown()
  end
end

return M
