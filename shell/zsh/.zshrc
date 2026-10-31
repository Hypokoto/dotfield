# Dotfield Zsh Configuration
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

### History Configuration
HISTFILE="$HOME/.zsh_history"
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

### Dotfield Aliases & Shortcuts
alias df-apply="dotfield apply"
alias df-status="dotfield status"
alias df-diff="dotfield diff"
alias df-check="dotfield check"
alias df-list="dotfield list"
alias df-rollback="dotfield rollback"

### General Productivity Aliases
alias ls="ls --color=auto"
alias ll="ls -lah"
alias la="ls -A"
alias grep="grep --color=auto"
alias v="nvim"
alias g="git"

### Starship Prompt Initialization
if command -v starship >/dev/null 2>&1; then
    eval "$(starship init zsh)"
fi
