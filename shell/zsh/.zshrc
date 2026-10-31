# Dotfield Zsh Configuration
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

### History Configuration
HISTFILE="${XDG_STATE_HOME:-$HOME/.local/state}/zsh/history"
[[ -d "${HISTFILE:h}" ]] || mkdir -p "${HISTFILE:h}"
HISTSIZE=50000
SAVEHIST=50000
setopt EXTENDED_HISTORY
setopt HIST_EXPIRE_DUPS_FIRST
setopt HIST_IGNORE_DUPS
setopt HIST_IGNORE_SPACE
setopt HIST_VERIFY
setopt SHARE_HISTORY

### Keybindings (Vi Mode & Sub-10ms Esc Response)
bindkey -v
export KEYTIMEOUT=1

# Dynamic Cursor Shape Switching for Modern Wayland Terminals
# 2: Solid block (Normal/Command), 6: Solid beam (Insert), 4: Solid underline (Visual/Replace)
zle-keymap-select() {
    if [[ ${KEYMAP} == vicmd ]] || [[ $1 = 'block' ]]; then
        echo -ne '\e[2 q'
    elif [[ ${KEYMAP} == main ]] || [[ ${KEYMAP} == viins ]] || [[ ${KEYMAP} = '' ]] || [[ $1 = 'beam' ]]; then
        echo -ne '\e[6 q'
    fi
}
zle -N zle-keymap-select

zle-line-init() {
    echo -ne '\e[6 q'
}
zle -N zle-line-init

# Reset cursor on exit
_fix_cursor() {
    echo -ne '\e[2 q'
}
precmd_functions+=(_fix_cursor)

# Edit command line in full Neovim editor (press 'v' in normal mode)
autoload -Uz edit-command-line
zle -N edit-command-line
bindkey -M vicmd 'v' edit-command-line

# History search using Up/Down and Ctrl+R
bindkey '^R' history-incremental-pattern-search-backward
bindkey '^[[A' up-line-or-search
bindkey '^[[B' down-line-or-search
bindkey -M vicmd 'k' up-line-or-search
bindkey -M vicmd 'j' down-line-or-search

### Fast Completion Initialization & Caching (<15ms shell startup)
zmodload -i zsh/complist
autoload -Uz compinit
typeset -U fpath

# Re-audit fpath security once every 24 hours; otherwise skip checks (-C) for instantaneous spawn
local -a ZCOMPCACHE=($HOME/.zcompdump(N.mh+24))
if [[ -n $ZCOMPCACHE ]]; then
    compinit
else
    compinit -C
fi

zstyle ':completion:*' menu select
zstyle ':completion:*' use-cache on
zstyle ':completion:*' cache-path "${XDG_CACHE_HOME:-$HOME/.cache}/zsh/zcompcache"
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}' 'r:|[._-]=* r:|=*' 'l:|=* r:|=*'

# Fast directory navigation options
setopt AUTO_CD
setopt AUTO_PUSHD
setopt PUSHD_IGNORE_DUPS
setopt PUSHD_SILENT

# Use vim keys in tab complete menu
bindkey -M menuselect 'h' vi-backward-char
bindkey -M menuselect 'k' vi-up-line-or-history
bindkey -M menuselect 'l' vi-forward-char
bindkey -M menuselect 'j' vi-down-line-or-history

### Environment Variables
export EDITOR="nvim"
export VISUAL="nvim"
export TERMINAL="foot"
export PAGER="less -R"
export MANPAGER="nvim +Man!"

### Controlled Chaos Colored Man Pages
export LESS_TERMCAP_mb=$'\e[1;31m'     # Crimson bold
export LESS_TERMCAP_md=$'\e[1;33m'     # Amber bold
export LESS_TERMCAP_me=$'\e[0m'
export LESS_TERMCAP_so=$'\e[01;40;33m' # Amber standout
export LESS_TERMCAP_se=$'\e[0m'
export LESS_TERMCAP_us=$'\e[1;32m'     # Green underline
export LESS_TERMCAP_ue=$'\e[0m'

### Dotfield Aliases & Shortcuts
alias df="dotfield"
alias dfs="dotfield status"
alias dfa="dotfield apply"
alias dfd="dotfield diff"
alias dfc="dotfield check"
alias dfl="dotfield list"
alias dfr="dotfield rollback"
alias df-hud="dotfield-gui --hud"
alias df-switch="dotfield-gui --switcher"

### Navigation & General Productivity Aliases
alias ..="cd .."
alias ...="cd ../.."
alias ....="cd ../../.."
alias ls="ls --color=auto"
alias ll="ls -lah"
alias la="ls -A"
alias grep="grep --color=auto"
alias v="nvim"
alias g="git"
alias gst="git status -sb"
alias gco="git checkout"
alias gd="git diff"
alias gds="git diff --staged"
alias gl="git log --oneline --graph --decorate -n 15"

### Starship Prompt Initialization
if command -v starship >/dev/null 2>&1; then
    export STARSHIP_CONFIG="${HOME}/.config/starship.toml"
    eval "$(starship init zsh)"
fi
