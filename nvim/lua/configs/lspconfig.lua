require("nvchad.configs.lspconfig").defaults()

-- jdtls is intentionally NOT configured here.
-- nvim-jdtls (ftplugin/java.lua) owns Java — doing both causes a dual-start conflict.

vim.lsp.config("clangd", {
  cmd = {
    "clangd",
    "--background-index",
    "--clang-tidy",
    "--header-insertion=iwyu",
    "--completion-style=detailed",
    "--function-arg-placeholders",
    "--fallback-style=llvm",
  },
  init_options = {
    usePlaceholders = true,
    completeUnimported = true,
    clangdFileStatus = true,
  },
})

vim.lsp.config("rust_analyzer", {
  settings = {
    ["rust-analyzer"] = {
      cargo = { allFeatures = true, buildScripts = { enable = true } },
      checkOnSave = { command = "clippy", extraArgs = { "--no-deps" } },
      procMacro = { enable = true },
      diagnostics = { enable = true, experimental = { enable = true } },
      workspace = { symbol = { search = { scope = "workspace_and_dependencies" } } },
    },
  },
})

vim.lsp.config("pyright", {
  settings = {
    python = {
      analysis = {
        typeCheckingMode = "basic",
        autoSearchPaths = true,
        useLibraryCodeForTypes = true,
      },
    },
  },
})

local servers = { "html", "cssls", "clangd", "rust_analyzer", "pyright" }
vim.lsp.enable(servers)

-- read :h vim.lsp.config for changing options of lsp servers
