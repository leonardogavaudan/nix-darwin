{ config, lib, ... }:

{
  home.shellAliases = {
    ".." = "cd ..";
    python = "python3";
    vim = "nvim";

    tc = "tmux new-session claude";
    tn = "tmux new-session";
    tp = "tmux new-session pi";
    tx = "tmux new-session codex";

    sync-agent-instructions = "cargo run --quiet --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile $AGENT_PROFILE";
    sync-agents = "sync-agent-instructions";
  };

  programs.zsh = {
    enable = true;
    dotDir = "${config.xdg.configHome}/zsh";

    syntaxHighlighting.enable = true;
    autosuggestion.enable = true;
    historySubstringSearch.enable = true;

    completionInit = ''
      autoload -Uz compinit
      if [[ -f ~/.zcompdump && $(date +'%j') == $(stat -f '%Sm' -t '%j' ~/.zcompdump 2>/dev/null) ]]; then
        compinit -C
      else
        compinit
      fi
    '';

    history = {
      size = 50000;
      save = 50000;
      ignoreDups = true;
      ignoreAllDups = true;
      ignoreSpace = true;
      extended = true;
      share = true;
    };

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
