# Dotfield Zsh Configuration
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel
# High-Performance, Hardened & Productive Shell Environment

### 1. Security & System Hardening
umask 027

# Prevent accidental file truncation through redirection (use >! or >| for intentional overwrite)
setopt NO_CLOBBER

# Terminal cleanliness
setopt NO_BEEP

# History storage with strict permissions
HISTFILE="${XDG_STATE_HOME:-$HOME/.local/state}/zsh/history"
[[ -d "${HISTFILE:h}" ]] || mkdir -p "${HISTFILE:h}"
chmod 700 "${HISTFILE:h}" 2>/dev/null || true
[[ -f "$HISTFILE" ]] && chmod 600 "$HISTFILE" 2>/dev/null || true

HISTSIZE=50000
SAVEHIST=50000

setopt EXTENDED_HISTORY          # Record timestamps and execution duration
setopt HIST_EXPIRE_DUPS_FIRST    # Trim oldest duplicates first when trimming history
setopt HIST_IGNORE_DUPS          # Do not write consecutive duplicates
setopt HIST_IGNORE_ALL_DUPS      # Remove older duplicates if a new command matches
setopt HIST_FIND_NO_DUPS         # Do not display duplicates in search
setopt HIST_SAVE_NO_DUPS         # Do not write duplicates to the file
setopt HIST_IGNORE_SPACE         # Do not record lines starting with space (vital for secrets)
setopt HIST_REDUCE_BLANKS        # Trim redundant spaces
setopt HIST_VERIFY               # Require edit before running history expansion
setopt SHARE_HISTORY             # Share history across concurrent sessions

# Secret Interception Hook: Prevent secrets, tokens and keys from being logged to disk
zshaddhistory() {
    local line="${1%%$'\n'}"
    # Omit commands starting with whitespace
    [[ "$line" =~ ^[[:space:]] ]] && return 1
    # Omit commands containing sensitive authentication tokens and private keys
    local lower="${(L)line}"
    if [[ "$lower" =~ (bearer|api[_-]?key|access_token|password|passwd|ghp_|glpat-|begin[[:space:]]+private|private_key|aws_secret|id_rsa) ]]; then
        return 1
    fi
    return 0
}

### 2. High-Performance Caching & Completion Initialization
zmodload -i zsh/complist
autoload -Uz compinit
typeset -U path fpath

local ZSH_CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/zsh"
[[ -d "$ZSH_CACHE_DIR" ]] || mkdir -p "$ZSH_CACHE_DIR"
local ZCOMPDUMP="$ZSH_CACHE_DIR/zcompdump"

# Audit and compile completion dump every 24 hours; otherwise skip checks (-C) for instantaneous spawn
if [[ -s "$ZCOMPDUMP" && -n "$ZCOMPDUMP"(N.mh-24) ]]; then
    compinit -C -d "$ZCOMPDUMP"
else
    compinit -i -d "$ZCOMPDUMP"
    { zcompile "$ZCOMPDUMP" 2>/dev/null } &!
fi

zstyle ':completion:*' menu select
zstyle ':completion:*' use-cache on
zstyle ':completion:*' cache-path "$ZSH_CACHE_DIR/zcompcache"
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}' 'r:|[._-]=* r:|=*' 'l:|=* r:|=*'
zstyle ':completion:*:descriptions' format '%F{#FFB000}-- %d --%f'
zstyle ':completion:*:messages' format '%F{#82AAFF}-- %d --%f'
zstyle ':completion:*:warnings' format '%F{#E63946}-- No matches found --%f'

# Use vim motion keys in tab-completion selection menu
bindkey -M menuselect 'h' vi-backward-char
bindkey -M menuselect 'k' vi-up-line-or-history
bindkey -M menuselect 'l' vi-forward-char
bindkey -M menuselect 'j' vi-down-line-or-history

### 3. Keybindings (Vi Mode & Sub-10ms Esc Response)
bindkey -v
export KEYTIMEOUT=1

# Dynamic cursor shape switching for Wayland terminals (Foot, Kitty, Ghostty, Alacritty)
zle-keymap-select() {
    case $KEYMAP in
        vicmd)      echo -ne '\e[2 q' ;; # Solid block (Normal mode)
        viins|main) echo -ne '\e[6 q' ;; # Solid beam (Insert mode)
        visual)     echo -ne '\e[4 q' ;; # Solid underline (Visual mode)
    esac
}
zle -N zle-keymap-select

zle-line-init() {
    zle -K viins
    echo -ne '\e[6 q'
}
zle -N zle-line-init

_fix_cursor() {
    echo -ne '\e[2 q'
}
autoload -Uz add-zsh-hook
add-zsh-hook precmd _fix_cursor

# Edit command line in full Neovim editor (press 'v' in Vi Normal mode)
autoload -Uz edit-command-line
zle -N edit-command-line
bindkey -M vicmd 'v' edit-command-line

# Common Readline muscle memory in Vi Insert Mode
bindkey -M viins '^A' beginning-of-line
bindkey -M viins '^E' end-of-line
bindkey -M viins '^U' kill-whole-line
bindkey -M viins '^K' kill-line
bindkey -M viins '^W' backward-kill-word
bindkey -M viins '^H' backward-delete-char
bindkey -M viins '^?' backward-delete-char

# History prefix searching on Up/Down arrows
autoload -Uz up-line-or-beginning-search down-line-or-beginning-search
zle -N up-line-or-beginning-search
zle -N down-line-or-beginning-search
bindkey '^[[A' up-line-or-beginning-search
bindkey '^[[B' down-line-or-beginning-search
bindkey -M vicmd 'k' up-line-or-beginning-search
bindkey -M vicmd 'j' down-line-or-beginning-search
bindkey '^R' history-incremental-pattern-search-backward

### 4. Fast Directory Navigation & Directory Stack
setopt AUTO_CD              # Type directory path directly to cd
setopt AUTO_PUSHD           # Automatically push visited directories onto stack
setopt PUSHD_IGNORE_DUPS    # Don't push duplicate directories to stack
setopt PUSHD_SILENT         # Suppress directory stack output on cd
setopt PUSHD_MINUS          # Invert the meanings of + and - for stack lookup

alias d='dirs -v | head -10'
alias 1='cd -1'
alias 2='cd -2'
alias 3='cd -3'
alias 4='cd -4'
alias 5='cd -5'
alias ..="cd .."
alias ...="cd ../.."
alias ....="cd ../../.."
alias .....="cd ../../../.."
alias -- -="cd -"

### 5. Environment Variables & Controlled Chaos Theme Pager
export EDITOR="nvim"
export VISUAL="nvim"
export TERMINAL="foot"
export PAGER="less -RF"
export MANPAGER="nvim +Man!"

export LESS_TERMCAP_mb=$'\e[1;31m'     # Crimson bold
export LESS_TERMCAP_md=$'\e[1;33m'     # Amber bold
export LESS_TERMCAP_me=$'\e[0m'
export LESS_TERMCAP_so=$'\e[01;40;33m' # Amber standout
export LESS_TERMCAP_se=$'\e[0m'
export LESS_TERMCAP_us=$'\e[1;32m'     # Green underline
export LESS_TERMCAP_ue=$'\e[0m'

### 6. Modern Productivity Replacements & Safety Aliases
# File Listing (Eza)
if command -v eza >/dev/null 2>&1; then
    alias ls="eza --icons=auto --group-directories-first"
    alias l="eza --icons=auto --group-directories-first"
    alias la="eza -a --icons=auto --group-directories-first"
    alias ll="eza -lh --icons=auto --git --group-directories-first --time-style=relative"
    alias lla="eza -lah --icons=auto --git --group-directories-first --time-style=relative"
    alias lt="eza --tree --level=2 --icons=auto --group-directories-first"
    alias llt="eza --tree --level=3 --icons=auto --group-directories-first"
else
    alias ls="ls --color=auto"
    alias ll="ls -lah"
    alias la="ls -A"
fi

# File Viewer (Bat)
if command -v bat >/dev/null 2>&1; then
    alias cat="bat --style=plain --paging=never"
    alias batp="bat --style=numbers"
fi

# Modern CLI Tools
command -v rg >/dev/null 2>&1 && alias grep="rg --color=auto"
command -v fd >/dev/null 2>&1 && alias find="fd"
command -v duf >/dev/null 2>&1 && alias df="duf"
command -v dust >/dev/null 2>&1 && alias du="dust"
command -v btop >/dev/null 2>&1 && alias top="btop"
command -v doggo >/dev/null 2>&1 && alias dig="doggo"

if command -v xh >/dev/null 2>&1; then
    alias http="xh"
    alias https="xh --https"
fi

# Safe File Operations (Prevent Accidental Deletion & Clashing)
if command -v trash-put >/dev/null 2>&1; then
    alias rm="trash-put"
    alias rmf="/bin/rm -Iv --preserve-root"
    alias tl="trash-list"
    alias tre="trash-restore"
    alias trm="trash-empty"
else
    alias rm="rm -Iv --preserve-root"
fi

alias cp="cp -iv"
alias mv="mv -iv"
alias ln="ln -iv"
alias chmod="chmod --preserve-root"
alias chown="chown --preserve-root"
alias chgrp="chgrp --preserve-root"

# Git Aliases
alias g="git"
alias ga="git add"
alias gaa="git add --all"
alias gc="git commit -v"
alias gcm="git commit -m"
alias gca="git commit --amend"
alias gco="git checkout"
alias gcb="git checkout -b"
alias gst="git status -sb"
alias gd="git diff"
alias gds="git diff --staged"
alias gl="git log --oneline --graph --decorate -n 15"
alias gll="git log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"
alias gp="git push"
alias gpf="git push --force-with-lease"
alias gpull="git pull --rebase"
alias gsta="git stash"
alias gstp="git stash pop"

# Network & System Inspection
alias ports="ss -tulanp"
alias listening="ss -tlnp"
alias estab="ss -tp state established"
alias myip="curl -s https://icanhazip.com || curl -s https://ifconfig.me"
alias v="nvim"

### 7. Helper Utilities
# Mkdir and cd simultaneously
mkcd() {
    [[ -n "$1" ]] || { echo "Usage: mkcd <directory>" >&2; return 1; }
    mkdir -p "$1" && builtin cd "$1"
}

# Universal archive extractor
ex() {
    if [[ -f "$1" ]]; then
        case "$1" in
            *.tar.bz2)   tar xjf "$1"        ;;
            *.tar.gz)    tar xzf "$1"        ;;
            *.tar.xz)    tar xJf "$1"        ;;
            *.tar.zst)   tar --zstd -xf "$1" ;;
            *.bz2)       bunzip2 "$1"        ;;
            *.rar)       unrar x "$1"        ;;
            *.gz)        gunzip "$1"         ;;
            *.tar)       tar xf "$1"         ;;
            *.tbz2)      tar xjf "$1"        ;;
            *.tgz)       tar xzf "$1"        ;;
            *.zip)       unzip "$1"          ;;
            *.Z)         uncompress "$1"     ;;
            *.7z)        7z x "$1"           ;;
            *)           echo "'$1' cannot be extracted via ex()" >&2 ;;
        esac
    else
        echo "'$1' is not a valid file" >&2
    fi
}

# Display $PATH line by line
path() {
    echo "$PATH" | tr ':' '\n'
}

# Inspect SSL certificates of remote servers
sslinfo() {
    local host="${1:?usage: sslinfo <host> [port]}"
    local port="${2:-443}"
    echo | openssl s_client -connect "${host}:${port}" -servername "${host}" 2>/dev/null | openssl x509 -noout -text | grep -E "Subject:|Issuer:|Not Before|Not After|DNS:"
}

# Inspect HTTP headers
headers() {
    local url="${1:?usage: headers <url>}"
    if command -v xh >/dev/null 2>&1; then
        xh HEAD "$url"
    else
        curl -sI "$url"
    fi
}

# Quick Whois & Reverse DNS lookup
ipinfo() {
    local ip="${1:?usage: ipinfo <ip>}"
    echo "=== Reverse DNS ==="
    host "$ip" 2>/dev/null || echo "no PTR record"
    echo ""
    echo "=== Whois ==="
    whois "$ip" 2>/dev/null | grep -E "^(NetName|OrgName|Country|CIDR|inetnum|netname|descr|country):" | head -20
}

### 8. FZF Interactive Fuzzy Finder (Controlled Chaos Theme)
if command -v fzf >/dev/null 2>&1; then
    [[ -r /usr/share/fzf/key-bindings.zsh ]] && source /usr/share/fzf/key-bindings.zsh
    [[ -r /usr/share/fzf/completion.zsh ]] && source /usr/share/fzf/completion.zsh

    export FZF_DEFAULT_OPTS=" \
        --color=bg+:#1C1C23,bg:#0A0A0C,spinner:#FFB000,hl:#E63946 \
        --color=fg:#E8E6E3,header:#8B949E,info:#FFB000,pointer:#FFB000 \
        --color=marker:#7BC47F,fg+:#E8E6E3,prompt:#FFB000,hl+:#E63946 \
        --border=rounded --height=50% --layout=reverse --prompt='❯ ' --pointer='▶' --marker='✔'"

    if command -v fd >/dev/null 2>&1; then
        export FZF_DEFAULT_COMMAND="fd --type f --hidden --exclude .git --exclude node_modules --exclude target"
        export FZF_CTRL_T_COMMAND="$FZF_DEFAULT_COMMAND"
        export FZF_ALT_C_COMMAND="fd --type d --hidden --exclude .git --exclude node_modules --exclude target"
        _fzf_compgen_path() { fd --hidden --exclude .git "$1"; }
        _fzf_compgen_dir() { fd --type d --hidden --exclude .git "$1"; }
    fi
fi

### 9. Cached Subshell Integrations (Zoxide, Direnv, Starship)
# Fast frecency directory jumper
if command -v zoxide >/dev/null 2>&1; then
    local ZOXIDE_CACHE="$ZSH_CACHE_DIR/zoxide_init.zsh"
    local ZOXIDE_BIN="$(whence -p zoxide)"
    if [[ ! -s "$ZOXIDE_CACHE" || "$ZOXIDE_CACHE" -ot "$ZOXIDE_BIN" ]]; then
        zoxide init zsh >| "$ZOXIDE_CACHE"
        zcompile "$ZOXIDE_CACHE" 2>/dev/null || true
    fi
    source "$ZOXIDE_CACHE"
fi

# Isolated per-directory environments
if command -v direnv >/dev/null 2>&1; then
    local DIRENV_CACHE="$ZSH_CACHE_DIR/direnv_init.zsh"
    local DIRENV_BIN="$(whence -p direnv)"
    if [[ ! -s "$DIRENV_CACHE" || "$DIRENV_CACHE" -ot "$DIRENV_BIN" ]]; then
        direnv hook zsh >| "$DIRENV_CACHE"
        zcompile "$DIRENV_CACHE" 2>/dev/null || true
    fi
    source "$DIRENV_CACHE"
fi

# Starship Prompt with cached initialization
if command -v starship >/dev/null 2>&1; then
    export STARSHIP_CONFIG="${STARSHIP_CONFIG:-$HOME/.config/starship.toml}"
    local STARSHIP_CACHE="$ZSH_CACHE_DIR/starship_init.zsh"
    local STARSHIP_BIN="$(whence -p starship)"
    if [[ ! -s "$STARSHIP_CACHE" || "$STARSHIP_CACHE" -ot "$STARSHIP_BIN" ]]; then
        starship init zsh --print-full-init >| "$STARSHIP_CACHE"
        zcompile "$STARSHIP_CACHE" 2>/dev/null || true
    fi
    source "$STARSHIP_CACHE"
fi

### 10. Intelligent Plugins (Autosuggestions & Syntax Highlighting)
# Fish-like autosuggestions
if [[ -r /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh ]]; then
    source /usr/share/zsh/plugins/zsh-autosuggestions/zsh-autosuggestions.zsh
    export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=#5c5c66" # Controlled Chaos Ash
    export ZSH_AUTOSUGGEST_STRATEGY=(history completion)
    export ZSH_AUTOSUGGEST_BUFFER_MAX_SIZE=50
    bindkey '^ ' autosuggest-accept
    bindkey '^F' autosuggest-accept
fi

# Real-time syntax highlighting (Must be loaded after all widgets)
if [[ -r /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh ]]; then
    source /usr/share/zsh/plugins/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
    typeset -A ZSH_HIGHLIGHT_STYLES
    ZSH_HIGHLIGHT_STYLES[command]='fg=#FFB000,bold'       # Amber
    ZSH_HIGHLIGHT_STYLES[builtin]='fg=#FFD166,bold'       # Pale Amber
    ZSH_HIGHLIGHT_STYLES[alias]='fg=#FFD166'              # Pale Amber
    ZSH_HIGHLIGHT_STYLES[function]='fg=#FFB000'           # Amber
    ZSH_HIGHLIGHT_STYLES[path]='fg=#82AAFF,underline'     # Blue/Steel
    ZSH_HIGHLIGHT_STYLES[unknown-token]='fg=#E63946,bold' # Crimson
    ZSH_HIGHLIGHT_STYLES[single-quoted-argument]='fg=#7BC47F' # Green
    ZSH_HIGHLIGHT_STYLES[double-quoted-argument]='fg=#7BC47F' # Green
fi

### 11. Developer Toolchains & Environments
# Bun
if [[ -d "$HOME/.bun" ]]; then
    export BUN_INSTALL="$HOME/.bun"
    export PATH="$BUN_INSTALL/bin:$PATH"
    [[ -s "$HOME/.bun/_bun" ]] && source "$HOME/.bun/_bun"
fi

# Rust Cargo
[[ -d "$HOME/.cargo/bin" ]] && export PATH="$HOME/.cargo/bin:$PATH"

# Go
[[ -d "$HOME/go/bin" ]] && export PATH="$HOME/go/bin:$PATH"

# Local user binaries
[[ -d "$HOME/.local/bin" ]] && export PATH="$HOME/.local/bin:$PATH"

### 12. Self-Compiling Optimization
# Background-compile .zshrc if modified for instant bytecode loading
if [[ ! -s "${ZDOTDIR:-$HOME}/.zshrc.zwc" || "${ZDOTDIR:-$HOME}/.zshrc" -nt "${ZDOTDIR:-$HOME}/.zshrc.zwc" ]]; then
    { zcompile "${ZDOTDIR:-$HOME}/.zshrc" 2>/dev/null } &!
fi
