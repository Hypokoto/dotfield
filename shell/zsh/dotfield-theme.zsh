# Dotfield Zsh Theme — consumes central color variables from ~/.config/dotfield/colors.sh
if [[ -f "$HOME/.config/dotfield/colors.sh" ]]; then
  source "$HOME/.config/dotfield/colors.sh"
  export FZF_DEFAULT_OPTS="--color=bg+:${DOTFIELD_OVERLAY},bg:${DOTFIELD_BG},spinner:${DOTFIELD_ACCENT},hl:${DOTFIELD_ACCENT} --color=fg:${DOTFIELD_FG_SECONDARY},header:${DOTFIELD_ACCENT},info:${DOTFIELD_WARNING},pointer:${DOTFIELD_ACCENT} --color=marker:${DOTFIELD_SUCCESS},fg+:${DOTFIELD_FG},prompt:${DOTFIELD_ACCENT},hl+:${DOTFIELD_ACCENT} --color=border:${DOTFIELD_BORDER}"
  export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=${DOTFIELD_C8}"
  if (( ${+ZSH_HIGHLIGHT_STYLES} )); then
    ZSH_HIGHLIGHT_STYLES[default]="fg=${DOTFIELD_FG}"
    ZSH_HIGHLIGHT_STYLES[unknown-token]="fg=${DOTFIELD_DANGER},bold"
    ZSH_HIGHLIGHT_STYLES[reserved-word]="fg=${DOTFIELD_ACCENT},bold"
    ZSH_HIGHLIGHT_STYLES[alias]="fg=${DOTFIELD_C6},bold"
    ZSH_HIGHLIGHT_STYLES[builtin]="fg=${DOTFIELD_ACCENT},bold"
    ZSH_HIGHLIGHT_STYLES[function]="fg=${DOTFIELD_ACCENT},bold"
    ZSH_HIGHLIGHT_STYLES[command]="fg=${DOTFIELD_C6}"
    ZSH_HIGHLIGHT_STYLES[path]="fg=${DOTFIELD_FG},underline"
    ZSH_HIGHLIGHT_STYLES[single-quoted-argument]="fg=${DOTFIELD_SUCCESS}"
    ZSH_HIGHLIGHT_STYLES[double-quoted-argument]="fg=${DOTFIELD_SUCCESS}"
  fi
fi
