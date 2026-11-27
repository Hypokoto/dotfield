# Dotfield Zsh Theme — consumes central color variables from ~/.config/dotfield/colors.sh
if [[ -f "$HOME/.config/dotfield/colors.sh" ]]; then
  source "$HOME/.config/dotfield/colors.sh"
  export FZF_DEFAULT_OPTS="--color=bg+:${DOTFIELD_OVERLAY},bg:${DOTFIELD_BG},spinner:${DOTFIELD_ACCENT},hl:${DOTFIELD_ACCENT} --color=fg:${DOTFIELD_FG_SECONDARY},header:${DOTFIELD_ACCENT},info:${DOTFIELD_WARNING},pointer:${DOTFIELD_ACCENT} --color=marker:${DOTFIELD_SUCCESS},fg+:${DOTFIELD_FG},prompt:${DOTFIELD_ACCENT},hl+:${DOTFIELD_ACCENT} --color=border:${DOTFIELD_BORDER}"
  export ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE="fg=${DOTFIELD_C8}"
fi
