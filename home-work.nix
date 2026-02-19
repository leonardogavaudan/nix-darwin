{ pkgs, lib, config, ... }:

{
  home.stateVersion = "24.11";
  home.username = "leonardo.gavaudan";
  home.homeDirectory = lib.mkForce "/Users/leonardo.gavaudan";

  home.sessionPath = [
    "$HOME/.local/bin"
    "/usr/local/bin"
    "$HOME/go/bin"
    "$HOME/.bun/bin"
  ];

  home.sessionVariables = {
    NVM_DIR = "$HOME/.nvm";
    VAULT_ADDR = "https://vault.algolia.net";
    GOPRIVATE = "github.com/algolia/*";
    GONOSUMDB = "github.com/algolia/*";
    GOPATH = "$HOME/go";
    COREPACK_ENABLE_AUTO_PIN = "0";
    BUN_INSTALL = "$HOME/.bun";
    TMPDIR = "/tmp";
    CODEX_HOME = "$HOME/.config/codex";
    EDITOR = "nvim";
    VISUAL = "nvim";
    AGENT_PROFILE = "work";
  };

  # Sync generated harness instruction files (shared + work profile).
  home.activation.syncAgentInstructions = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    run env PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH" ${pkgs.cargo}/bin/cargo run --quiet --manifest-path ${config.home.homeDirectory}/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile work
  '';

  # Auto-update flake (only to cached versions).
  launchd.agents.nix-flake-update = {
    enable = true;
    config = {
      Label = "com.user.nix-flake-update";
      ProgramArguments = [
        "/bin/sh"
        "-c"
        "PATH=/nix/var/nix/profiles/default/bin:/run/current-system/sw/bin:/usr/bin:/bin:/usr/sbin:/sbin ${pkgs.cargo}/bin/cargo run --quiet --manifest-path ${config.home.homeDirectory}/.config/nix-darwin/scripts/nix-update-cached-rs/Cargo.toml -- --apply --flake-dir ${config.home.homeDirectory}/.config/nix-darwin"
      ];
      RunAtLoad = true;
      StartCalendarInterval = [ { Hour = 9; Minute = 0; } ];
      StandardOutPath = "/tmp/nix-flake-update.log";
      StandardErrorPath = "/tmp/nix-flake-update.log";
    };
  };

  home.shellAliases = {
    sync-agent-instructions = "cargo run --quiet --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile work";
    sync-agents = "sync-agent-instructions";
    ".." = "cd ..";
    python = "python3";
    ns = "sudo darwin-rebuild switch --flake ~/.config/nix-darwin";
    tc = "tmux new-session claude";
    tn = "tmux new-session";
    tp = "tmux new-session pi";
    tx = "tmux new-session codex";
    vl = "vault_auto_login";
  };

  programs.fzf = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.zsh = {
    enable = true;
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
    plugins = [
      {
        name = "fzf-tab";
        src = pkgs.zsh-fzf-tab + "/share/fzf-tab";
      }
    ];
    initContent = ''
      # NVM (lazy-loaded)
      _nvm_lazy_load() {
        unset -f nvm node npm npx corepack 2>/dev/null
        [ -s "/opt/homebrew/opt/nvm/nvm.sh" ] && \. "/opt/homebrew/opt/nvm/nvm.sh"
        [ -s "/opt/homebrew/opt/nvm/etc/bash_completion.d/nvm" ] && \. "/opt/homebrew/opt/nvm/etc/bash_completion.d/nvm"
        nvm use 22 --silent 2>/dev/null
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

      vault_auto_login() {
        if [[ -f ~/.vault-token ]]; then
          export VAULT_TOKEN=$(cat ~/.vault-token)
        fi
        if ! vault token lookup > /dev/null 2>&1; then
          echo "Vault token expired or invalid, logging in..."
          vault login -method=oidc > /dev/null 2>&1
          export VAULT_TOKEN=$(cat ~/.vault-token)
          echo "Vault token refreshed!"
        else
          echo "Vault token is valid."
        fi
      }

      # rbenv (lazy-loaded)
      _rbenv_lazy_load() {
        unset -f rbenv ruby gem bundle rake 2>/dev/null
        eval "$(command rbenv init -)"
      }
      for cmd in rbenv ruby gem bundle rake; do
        eval "$cmd() { _rbenv_lazy_load; $cmd \"\$@\" }"
      done
      [ -d "$HOME/.rbenv/shims" ] && export PATH="$HOME/.rbenv/shims:$PATH"

      [ -s "$HOME/.bun/_bun" ] && source "$HOME/.bun/_bun"

      if [ -f '/opt/homebrew/share/google-cloud-sdk/path.zsh.inc' ]; then . '/opt/homebrew/share/google-cloud-sdk/path.zsh.inc'; fi
      if [ -f '/opt/homebrew/share/google-cloud-sdk/completion.zsh.inc' ]; then . '/opt/homebrew/share/google-cloud-sdk/completion.zsh.inc'; fi

      bindkey '\e[3;5~' backward-kill-word
      bindkey '^[^?' backward-kill-word
      bindkey '\e\x7f' backward-kill-word
      bindkey '\e[127;3u' backward-kill-word
      bindkey '\e[Z' autosuggest-accept

      [ -f "$HOME/.secrets" ] && source "$HOME/.secrets"
    '';
  };

  programs.zoxide = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.eza = {
    enable = true;
    enableZshIntegration = true;
    icons = "auto";
    git = true;
    extraOptions = [ "--group-directories-first" ];
  };

  programs.bat.enable = true;

  programs.starship = {
    enable = true;
    enableZshIntegration = true;
    settings = {
      buf.disabled = true;
      gcloud.disabled = true;
      docker_context.disabled = true;
      package.disabled = true;
      cmd_duration.min_time = 3000;
      directory.truncation_length = 3;
    };
  };

  programs.gh = {
    enable = true;
    settings = {
      git_protocol = "https";
    };
  };

  programs.ghostty = {
    enable = true;
    package = null;
    enableZshIntegration = true;
    settings = {
      theme = "Catppuccin Mocha";
      palette = "8=#1e1e2e";
      font-size = 15;
      font-family = "CommitMono";
      keybind = [
        "option+backspace=text:\\x1b[127;3u"
        "shift+backspace=text:\\x7f"
        "shift+space=text: "
        "shift+enter=text:\\n"
        "ctrl+shift+left=move_tab:-1"
        "ctrl+shift+right=move_tab:1"
      ];
    };
  };

  programs.tmux = {
    enable = true;
    prefix = "C-t";
    keyMode = "vi";
    mouse = true;
    escapeTime = 0;
    extraConfig = ''
      set -g default-terminal "tmux-256color"
      set -g history-limit 200000
      set -g renumber-windows on

      bind [ split-window -h -c "#{pane_current_path}"
      bind ] split-window -v -c "#{pane_current_path}"

      bind h select-pane -L
      bind j select-pane -D
      bind k select-pane -U
      bind l select-pane -R

      bind t copy-mode
      bind-key -T copy-mode-vi y send-keys -X copy-pipe-and-cancel "pbcopy"
      bind-key -T copy-mode-vi , send-keys -X scroll-up
      bind-key -T copy-mode-vi . send-keys -X scroll-down
      bind r source-file ~/.config/tmux/tmux.conf \; display-message "Config reloaded!"

      set -g extended-keys always
      set -as terminal-features 'xterm-ghostty:extkeys'
      set -as terminal-features 'xterm-256color:extkeys'
      set -ga terminal-overrides ',*:kDC5=\e[3;5~'
      set -ga terminal-overrides ',*:kDC6=\e[3;6~'
      set -ga terminal-overrides ',*:kDC7=\e[3;7~'
      set -gw xterm-keys on
    '';
  };

  programs.git = {
    enable = true;
    ignores = [
      "**/.claude/settings.local.json"
      "CLAUDE.local.md"
      ".local"
    ];
    settings = {
      user.name = "Leonardo Gavaudan";
      user.email = "leonardogavaudan@gmail.com";
      init.defaultBranch = "main";
      pull.rebase = true;
      url."https://github.com/".insteadOf = "ssh://git@github.com/";
      url."git@github.com:".insteadOf = "https://github.com/";
    };
  };
}
