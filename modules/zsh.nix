{ config, pkgs, ... }:

{
  programs.zsh = {
    enable = true;

    # ── Environment variables (written to .zshenv) ──────────────
    sessionVariables = {
      EDITOR = "nvim";
      XDG_CONFIG_HOME = "$HOME/.config";
      AWS_CONFIG_FILE = "$HOME/.config/aws/config";
      AWS_SHARED_CREDENTIALS_FILE = "$HOME/.config/aws/credentials";
      GOPATH = "$HOME/.config/go";
      CARGO_HOME = "$HOME/.config/cargo";
      RUSTUP_HOME = "$HOME/.config/rustup";
    };

    # ── Aliases ─────────────────────────────────────────────────
    shellAliases = {
      ".." = "cd ..";
      "..." = "cd ../..";
      ll = "ls -la | sort -k 1";
      vim = "nvim";
      python = "python3";
      pip = "pip3";
      cc = "tmux new-session claude";
      tn = "tmux new-session";
      pt = "tmux new-session pi";
    };

    # ── .zprofile (login shell) ─────────────────────────────────
    profileExtra = ''
      eval "$(/opt/homebrew/bin/brew shellenv)"
    '';

    # ── .zshenv additions (after sessionVariables) ──────────────
    envExtra = ''
      # Cargo/Rust environment
      if [ -f "$HOME/.config/cargo/env" ]; then
        . "$HOME/.config/cargo/env"
      fi
    '';

    # ── .zshrc (interactive shell) ──────────────────────────────
    initExtra = ''
      # PATH
      export PATH="$HOME/.local/bin:$HOME/.bun/bin:$GOPATH/bin:$PATH"

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

      # Brave debug function
      brave-debug() {
        "/Applications/Nix Apps/Brave Browser.app/Contents/MacOS/Brave Browser" \
          --remote-debugging-port=9222 \
          --profile-directory="Default" \
          --enable-features=WebUIDarkMode &
        echo "Waiting for Brave CDP..."
        while ! curl -s http://localhost:9222/json/version >/dev/null 2>&1; do sleep 0.5; done
        sleep 1
        agent-browser --cdp 9222 set media dark >/dev/null 2>&1
        echo "Brave ready with dark mode"
      }
    '';
  };
}
