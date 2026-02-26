{ config, lib, pkgs, ... }:

{
  # Shared PATH prefixes for all profiles.
  home.sessionPath = lib.mkBefore [
    "/opt/homebrew/bin"
    "/opt/homebrew/sbin"
    "/usr/local/bin"
    "$HOME/.local/bin"

    # Shared runtime/toolchain bins.
    "\${BUN_INSTALL:-$HOME/.bun}/bin"
    "$HOME/.cache/.bun/bin"
    "\${GOPATH:-$HOME/go}/bin"
    "\${CARGO_HOME:-$HOME/.config/cargo}/bin"
  ];

  # Non-interactive-compatible wrapper (aliases are interactive-shell only)
  home.file.".local/bin/fdu" = {
    executable = true;
    text = ''
      #!/usr/bin/env bash
      exec fd -u "$@"
    '';
  };

  home.shellAliases = {
    ".." = "cd ..";
    "..." = "cd ../..";

    # Shared ls/eza setup across profiles.
    eza = "eza --icons auto --git --group-directories-first";
    ls = "eza";
    la = "eza -a";
    lla = "eza -la";
    lt = "eza --tree";
    ll = "eza -la";

    fdu = "fd -u";
    python = "python3";
    pip = "pip3";
    vim = "nvim";
    rl = "exec $SHELL -l";

    ns = "sudo darwin-rebuild switch --flake ~/.config/nix-darwin";

    tc = "tmux new-session claude";
    tn = "tmux new-session";
    ta = "tmux attach";
    tp = "tmux new-session pi";
    tx = "tmux new-session codex";

    sync-agent-instructions = "cargo run --quiet --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile $AGENT_PROFILE";
    sync-agents = "sync-agent-instructions";
  };

  programs.fzf = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.zsh = {
    enable = true;
    dotDir = "${config.xdg.configHome}/zsh";

    syntaxHighlighting.enable = true;
    autosuggestion.enable = true;
    historySubstringSearch.enable = true;

    plugins = [
      {
        name = "fzf-tab";
        src = pkgs.zsh-fzf-tab + "/share/fzf-tab";
      }
    ];

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

      # NVM (lazy-loaded)
      [ -n "$NVM_DIR" ] || export NVM_DIR="$HOME/.nvm"
      _nvm_lazy_load() {
        unset -f nvm node npm npx corepack 2>/dev/null
        if [ -s "/opt/homebrew/opt/nvm/nvm.sh" ]; then
          . "/opt/homebrew/opt/nvm/nvm.sh"
          [ -s "/opt/homebrew/opt/nvm/etc/bash_completion.d/nvm" ] && . "/opt/homebrew/opt/nvm/etc/bash_completion.d/nvm"
          type nvm >/dev/null 2>&1 && nvm use 22 --silent 2>/dev/null || true
        fi
      }
      for cmd in nvm node npm npx corepack; do
        eval "$cmd() { _nvm_lazy_load; $cmd \"\$@\" }"
      done

      if [ -d "$NVM_DIR/versions/node" ] && [ -f "$NVM_DIR/alias/default" ]; then
        _nvm_alias="$(cat "$NVM_DIR/alias/default")"
        _nvm_default="$(ls -1d "$NVM_DIR/versions/node/v$_nvm_alias"* 2>/dev/null | sort -V | tail -1)"
        [ -n "$_nvm_default" ] && export PATH="$_nvm_default/bin:$PATH"
        unset _nvm_alias _nvm_default
      fi

      if [ -f '/opt/homebrew/share/google-cloud-sdk/path.zsh.inc' ]; then . '/opt/homebrew/share/google-cloud-sdk/path.zsh.inc'; fi
      if [ -f '/opt/homebrew/share/google-cloud-sdk/completion.zsh.inc' ]; then . '/opt/homebrew/share/google-cloud-sdk/completion.zsh.inc'; fi

      bindkey '\e[3;5~' backward-kill-word
      bindkey '^[^?' backward-kill-word
      bindkey '\e\x7f' backward-kill-word
      bindkey '\e[127;3u' backward-kill-word
      bindkey '\e[Z' autosuggest-accept
    '';
  };
}
