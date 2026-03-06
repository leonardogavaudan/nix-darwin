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

  # Share Terraform provider binaries across directories (faster `terraform init`).
  home.sessionVariables = {
    TF_PLUGIN_CACHE_DIR = "$HOME/.cache/terraform/plugin-cache";
  };

  # Ensure plugin cache directory exists after each Home Manager activation.
  home.activation.ensureTerraformPluginCacheDir = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    $DRY_RUN_CMD mkdir -p "$HOME/.cache/terraform/plugin-cache"
  '';

  # Non-interactive-compatible wrapper (aliases are interactive-shell only)
  home.file.".local/bin/fdu" = {
    executable = true;
    text = ''
      #!/usr/bin/env bash
      exec fd -u "$@"
    '';
  };

  # Helper for attaching agent-browser to Brave over CDP.
  home.file.".local/bin/brave-cdp" = {
    executable = true;
    text = ''
      #!/usr/bin/env bash
      set -euo pipefail

      BRAVE_APP="''${BRAVE_APP:-/Applications/Brave Browser.app}"
      BRAVE_BIN="$BRAVE_APP/Contents/MacOS/Brave Browser"
      ISOLATED_DIR="''${BRAVE_CDP_ISOLATED_DIR:-$HOME/.cache/browser-pi}"
      DEFAULT_DIR="''${BRAVE_CDP_DEFAULT_DIR:-$HOME/Library/Application Support/BraveSoftware/Brave-Browser}"

      usage() {
        cat <<'EOF'
      Usage:
        brave-cdp status [port]
        brave-cdp ws-url [port]
        brave-cdp connect [port]
        brave-cdp launch-isolated [port]
        brave-cdp launch-default [port] [--force-quit]

      Notes:
        - launch-isolated starts a separate Brave instance with its own automation profile.
        - launch-default uses your real Brave Default profile and therefore requires Brave to be closed first.
        - connect attaches agent-browser to the given CDP endpoint.
      EOF
      }

      require_cmd() {
        if ! command -v "$1" >/dev/null 2>&1; then
          echo "Missing required command: $1" >&2
          exit 1
        fi
      }

      fetch_version_json() {
        local port="$1"
        curl -fsS "http://127.0.0.1:$port/json/version"
      }

      wait_for_cdp() {
        local port="$1"
        local attempts="''${2:-40}"
        local i

        for ((i = 0; i < attempts; i++)); do
          if fetch_version_json "$port" >/dev/null 2>&1; then
            return 0
          fi
          sleep 0.5
        done

        echo "Timed out waiting for Brave CDP on port $port" >&2
        exit 1
      }

      main_process_lines() {
        ps -Ao command | grep -F "$BRAVE_BIN" | grep -Fv "Helper" | grep -Fv "chrome_crashpad_handler" || true
      }

      command_name="''${1:-}"
      port="''${2:-}"

      case "$command_name" in
        status)
          require_cmd curl
          require_cmd jq
          port="''${port:-9222}"
          if json="$(fetch_version_json "$port" 2>/dev/null)"; then
            echo "CDP is up on port $port"
            echo "Browser: $(printf '%s' "$json" | jq -r '.Browser')"
            echo "WebSocket: $(printf '%s' "$json" | jq -r '.webSocketDebuggerUrl')"
            main_process_lines | grep -F -- "--remote-debugging-port=$port" || true
          else
            echo "No Brave CDP endpoint on port $port" >&2
            exit 1
          fi
          ;;
        ws-url)
          require_cmd curl
          require_cmd jq
          port="''${port:-9222}"
          fetch_version_json "$port" | jq -r '.webSocketDebuggerUrl'
          ;;
        connect)
          require_cmd agent-browser
          port="''${port:-9222}"
          agent-browser connect "$port"
          ;;
        launch-isolated)
          port="''${port:-9222}"
          mkdir -p "$ISOLATED_DIR"
          open -na "$BRAVE_APP" --args \
            --remote-debugging-port="$port" \
            --user-data-dir="$ISOLATED_DIR" \
            --no-first-run \
            --no-default-browser-check >/dev/null
          wait_for_cdp "$port"
          echo "Brave isolated CDP instance is ready on port $port"
          ;;
        launch-default)
          port="''${port:-9223}"
          force_flag="''${3:-}"

          if main_process_lines | grep -q .; then
            if [[ "$force_flag" != "--force-quit" ]]; then
              echo "Brave is currently running. Close it first, or rerun with:" >&2
              echo "  brave-cdp launch-default $port --force-quit" >&2
              exit 1
            fi

            osascript -e 'tell application "Brave Browser" to quit' >/dev/null 2>&1 || true

            for _ in {1..40}; do
              if ! main_process_lines | grep -q .; then
                break
              fi
              sleep 0.5
            done

            if main_process_lines | grep -q .; then
              echo "Brave did not exit cleanly; not relaunching Default profile." >&2
              exit 1
            fi
          fi

          open -na "$BRAVE_APP" --args \
            --remote-debugging-port="$port" \
            --user-data-dir="$DEFAULT_DIR" \
            --profile-directory=Default \
            --no-first-run \
            --no-default-browser-check >/dev/null
          wait_for_cdp "$port"
          echo "Brave Default profile CDP instance is ready on port $port"
          ;;
        ""|-h|--help|help)
          usage
          ;;
        *)
          usage >&2
          exit 1
          ;;
      esac
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
