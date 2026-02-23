{ config, lib, ... }:

{
  programs.zsh = {
    enable = true;
    dotDir = "${config.xdg.configHome}/zsh";

    # ── .zshenv additions (after sessionVariables) ──────────────
    envExtra = ''
      # Ensure Codex always uses XDG location even when HM session vars are pre-sourced.
      export CODEX_HOME="$HOME/.config/codex"

      # Cargo/Rust environment
      if [ -f "$HOME/.config/cargo/env" ]; then
        . "$HOME/.config/cargo/env"
      fi
    '';

    # ── .zshrc (interactive shell) ──────────────────────────────
    initContent = lib.mkBefore ''
      # Load secrets and sync to tmux environment
      if [ -f ~/.secrets ]; then
        source ~/.secrets
        if command -v tmux >/dev/null 2>&1 && tmux ls >/dev/null 2>&1; then
          grep "^export " ~/.secrets | cut -d' ' -f2 | cut -d'=' -f1 | while read -r var; do
            eval "val=\$$var"
            tmux set-environment -g "$var" "$val" 2>/dev/null
          done
        fi
      fi

      # Bun completions
      [ -s "$HOME/.bun/_bun" ] && source "$HOME/.bun/_bun"
    '';
  };
}
