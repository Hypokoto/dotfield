local on_attach = require("nvchad.configs.lspconfig").on_attach
local capabilities = require("nvchad.configs.lspconfig").capabilities
local jdtls = require("jdtls")

local root_markers = { "pom.xml", "build.gradle", "build.gradle.kts", ".git" }
local root_dir = require("jdtls.setup").find_root(root_markers)
if root_dir == "" then
  root_dir = vim.fn.getcwd()
end

local project_name = vim.fn.fnamemodify(root_dir, ":p:h:t")
local workspace_dir = vim.fn.stdpath("data") .. "/jdtls-workspace/" .. project_name

-- Robust JDK discovery prioritized for java-21-openjdk without breaking alternative JDKs in /usr/lib/jvm/
local function find_jdk()
  -- 1. Explicit check for java-21-openjdk
  local jdk21_openjdk = vim.fn.glob("/usr/lib/jvm/java-21-openjdk*", false, true)
  if #jdk21_openjdk > 0 and vim.fn.isdirectory(jdk21_openjdk[1]) == 1 then
    return jdk21_openjdk[1]
  end

  -- 2. General search for java-21*
  local jdk21_gen = vim.fn.glob("/usr/lib/jvm/java-21*", false, true)
  if #jdk21_gen > 0 and vim.fn.isdirectory(jdk21_gen[1]) == 1 then
    return jdk21_gen[1]
  end

  -- 3. Fallback: Search for any java-* directory excluding default symlinks
  local candidates = vim.fn.glob("/usr/lib/jvm/java-*", false, true)
  for _, path in ipairs(candidates) do
    if vim.fn.isdirectory(path) == 1 and not path:match("default") then
      return path
    end
  end

  -- 4. Environment variable JAVA_HOME fallback
  local java_home = os.getenv("JAVA_HOME")
  if java_home and vim.fn.isdirectory(java_home) == 1 then
    return java_home
  end

  return nil
end

local jdk_home = find_jdk()
local java_cmd = jdk_home and (jdk_home .. "/bin/java") or "java"

-- Find jdtls jar installed via Mason
local mason_packages = vim.fn.stdpath("data") .. "/mason/packages"
local launcher_glob = mason_packages .. "/jdtls/plugins/org.eclipse.equinox.launcher_*.jar"
local launcher_jar = vim.fn.glob(launcher_glob)
-- config_linux is required — omitting it causes OSGi "Unable to acquire application service" (exit 13)
local config_dir = mason_packages .. "/jdtls/config_linux"

local config = {
  cmd = {
    java_cmd,
    "-Declipse.application=org.eclipse.jdt.ls.core.id1",
    "-Dosgi.bundles.defaultStartLevel=4",
    "-Declipse.product=org.eclipse.jdt.ls.core.product",
    "-Dlog.protocol=true",
    "-Dlog.level=ALL",
    "-Xmx1g",
    "--add-modules=ALL-SYSTEM",
    "--add-opens", "java.base/java.util=ALL-UNNAMED",
    "--add-opens", "java.base/java.lang=ALL-UNNAMED",
    "-jar", launcher_jar,
    "-configuration", config_dir,
    "-data", workspace_dir,
  },
  root_dir = root_dir,
  on_attach = on_attach,
  capabilities = capabilities,
}

-- Only start if launcher jar is found
if launcher_jar ~= "" then
  jdtls.start_or_attach(config)
else
  vim.notify("jdtls launcher jar not found. Make sure to install jdtls via Mason (:MasonInstall jdtls)", vim.log.levels.WARN)
end
