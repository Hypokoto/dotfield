-- lua/plugins/init.lua
local ascii_header = {
  "                                    ",
  "                                    ",
  "                                    ",
  "    ⣴⣶⣤⡤⠦⣤⣀⣤⠆      ⣈⣭⣿⣶⣿⣦⣼⣆          ",
  "    ⠉⠻⢿⣿⠿⣿⣿⣶⣦⠤⠄⡠⢾⣿⣿⡿⠋⠉⠉⠻⣿⣿⡛⣦        ",
  "          ⠈⢿⣿⣟⠦ ⣾⣿⣿⣷    ⠻⠿⢿⣿⣧⣄     ",
  "            ⣸⣿⣿⢧ ⢻⠻⣿⣿⣷⣄⣀⠄⠢⣀⡀⠈⠙⠿⠄    ",
  "            ⢠⣿⣿⣿⠈    ⣻⣿⣿⣿⣿⣿⣿⣿⣛⣳⣤⣀⣀   ",
  "    ⢠⣧⣶⣥⡤⢄ ⣸⣿⣿⠘   ⢀⣴⣿⣿⡿⠛⣿⣿⣧⠈⢿⠿⠟⠛⠻⠿⠄  ",
  "  ⣰⣿⣿⠛⠻⣿⣿⡦⢹⣿⣷   ⢊⣿⣿⡏   ⢸⣿⣿⡇ ⢀⣠⣄⣾⠄   ",
  " ⣠⣿⠿⠛ ⢀⣿⣿⣷⠘⢿⣿⣦⡀ ⢸⢿⣿⣿⣄ ⣸⣿⣿⡇⣪⣿⡿⠿⣿⣷⡄  ",
  " ⠙⠃   ⣼⣿⡟  ⠈⠻⣿⣿⣦⣌⡇⠻⣿⣿⣷⣿⣿⣿ ⣿⣿⡇ ⠛⠻⢷⣄ ",
  "       ⢻⣿⣿⣄   ⠈⠻⣿⣿⣿⣷⣿⣿⣿⣿⣿⡟ ⠫⢿⣿⡆     ",
  "        ⠻⣿⣿⣿⣿⣶⣶⣾⣿⣿⣿⣿⣿⣿⣿⣿⡟⢀⣀⣤⣾⡿⠃     ",
  "                                    ",
}

return {
  -- ==========================================================
  -- Dashboard — snacks.nvim
  -- ==========================================================
  {
    "folke/snacks.nvim",
    priority = 1000,
    lazy = false,
    opts = {
      animate = { enabled = false },
      dashboard = {
        preset = {
          header = table.concat(ascii_header, "\n"),
        },
        sections = {
          { section = "header", hl = "MinimalDashboardHeader" },
          { section = "keys", gap = 1, padding = 1 },
          { section = "startup", hl = "MinimalDashboardFooter" },
        },
      },
    },
  },

  -- ==========================================================
  -- Mason — ensure the three toolchains are provisioned.
  -- opts_extend keeps this list additive across other specs that
  -- also touch ensure_installed, matching the merge-behavior fix
  -- already established in this stack.
  -- ==========================================================
  {
    "williamboman/mason.nvim",
    opts = {
      ensure_installed = {
        "clangd",
        "clang-format",
        "jdtls",
        "rust-analyzer",
        "codelldb",
        "pyright",
        "ruff",
      },
    },
    opts_extend = { "ensure_installed" },
  },

  -- ==========================================================
  -- nvim-lspconfig — clangd, jdtls, rust_analyzer.
  -- Actual server setup lives in lua/configs/lspconfig.lua,
  -- loaded from here to keep this file spec-only.
  -- ==========================================================
  {
    "neovim/nvim-lspconfig",
    config = function()
      require "nvchad.configs.lspconfig"
      require "configs.lspconfig"
    end,
  },

  -- ==========================================================
  -- Treesitter — parser anchors for C/C++/Rust/Java.
  -- ==========================================================
  {
    "nvim-treesitter/nvim-treesitter",
    opts = {
      ensure_installed = {
        "c",
        "cpp",
        "rust",
        "java",
        "python",
        "lua",
        "vim",
        "vimdoc",
        "markdown",
        "markdown_inline",
      },
    },
    opts_extend = { "ensure_installed" },
  },

  -- ==========================================================
  -- File tree — forced to the right, <leader>e preserved.
  -- ==========================================================
  {
    "nvim-tree/nvim-tree.lua",
    opts = {
      view = {
        side = "right",
      },
    },
  },

  {
    "nvchad/ui",
    opts = {
      tabufline = {
        lazyload = false,
        -- Force the offset rendering to align with the right sidebar
        modules = {
          -- This overrides NvChad's default left-leaning layout
          offset = { position = "right" } 
        }
      }
    }
  },

  {
    "mfussenegger/nvim-jdtls",
    ft = { "java" },
  },

  -- ==========================================================
  -- Markdown rendering — inline buffer rendering for headings,
  -- code blocks, checkboxes, and tables. Uses treesitter
  -- markdown parser. Toggled with <leader>mp.
  -- ==========================================================
  {
    "MeanderingProgrammer/render-markdown.nvim",
    ft = { "markdown" },
    opts = {
      code = {
        border = "thin",
        width = "block",
        left_pad = 1,
        right_pad = 1,
      },
      checkbox = {
        enabled = true,
      },
      sign = {
        enabled = false,
      },
      warn = true,
    },
    dependencies = { "nvim-treesitter/nvim-treesitter" },
  },

  -- ==========================================================
  -- agentic.nvim — AI chat interface for Neovim (ACP)
  -- ==========================================================
  {
    "carlos-algms/agentic.nvim",
    opts = {
      provider = "antigravity-acp",
      acp_providers = {
        ["antigravity-acp"] = {
          name = "Antigravity",
          command = "antigravity-acp",
          env = {
            AGY_BIN = "/usr/sbin/agy",
          },
        },
      },
    },
    keys = {
      {
        "<C-\\>",
        function() require("agentic").toggle() end,
        mode = { "n", "v", "i" },
        desc = "Toggle Agentic Chat"
      },
      {
        "<C-'>",
        function() require("agentic").add_selection_or_file_to_context() end,
        mode = { "n", "v" },
        desc = "Add file or selection to Agentic to Context"
      },
      {
        "<C-,>",
        function() require("agentic").new_session() end,
        mode = { "n", "v", "i" },
        desc = "New Agentic Session"
      },
      {
        "<A-i>r",
        function()
            require("agentic").restore_session()
        end,
        desc = "Agentic Restore session",
        silent = true,
        mode = { "n", "v", "i" },
      },
      {
        "<leader>ad",
        function()
            require("agentic").add_current_line_diagnostics()
        end,
        desc = "Add current line diagnostic to Agentic",
        mode = { "n" },
      },
      {
        "<leader>aD",
        function()
            require("agentic").add_buffer_diagnostics()
        end,
        desc = "Add all buffer diagnostics to Agentic",
        mode = { "n" },
      },
    },
  },
  
  -- ==========================================================
  -- Right-click Menu (nvzone/menu)
  -- ==========================================================
  {
    "nvzone/volt",
    lazy = true,
  },
  {
    "nvzone/menu",
    lazy = true,
  },

  -- ==========================================================
  -- Dropbar — VS Code style breadcrumbs
  -- ==========================================================
  {
    "Bekaboo/dropbar.nvim",
    event = "BufReadPre",
    dependencies = {
      "nvim-tree/nvim-web-devicons",
    },
  },
}
