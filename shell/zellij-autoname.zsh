# zellij-autoname: shell hooks for instant tab renaming
# Source this file in your .zshrc:
#   source /path/to/zellij-autoname/shell/zellij-autoname.zsh

zellij_tab_name_update() {
  if [[ -n $ZELLIJ ]]; then
    local current_dir=$PWD
    if [[ $current_dir == $HOME ]]; then
      current_dir="~"
    else
      current_dir=${current_dir##*/}
    fi
    command nohup zellij action rename-tab $current_dir >/dev/null 2>&1
  fi
}

zellij_preexec() {
  if [[ -n $ZELLIJ ]]; then
    local cmd="${1%% *}"
    cmd="${cmd##*/}"
    case $cmd in
      nvim|vim|vi)
        local dir=$PWD
        [[ $dir == $HOME ]] && dir="~" || dir="${dir##*/}"
        command nohup zellij action rename-tab $dir >/dev/null 2>&1
        ;;
      *)
        command nohup zellij action rename-tab $cmd >/dev/null 2>&1
        ;;
    esac
  fi
}

zellij_tab_name_update
chpwd_functions+=(zellij_tab_name_update)
preexec_functions+=(zellij_preexec)
precmd_functions+=(zellij_tab_name_update)
