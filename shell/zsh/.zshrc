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

### Keybindings (Vi Mode)
bindkey -v
export KEYTIMEOUT=1

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
alias dm="dotfield morph"
alias da="dotfield apply"
alias dp="dotfield preview"
alias du="dotfield undo"
alias ds="dotfield status"
alias dd="dotfield diff"
alias dc="dotfield check"
alias dl="dotfield list"

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
