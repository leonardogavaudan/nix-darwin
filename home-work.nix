{ pkgs, lib, config, ... }:

{
  imports = [
    ./modules/zsh.nix
    ./modules/common-programs.nix
    ./modules/tmux.nix
    ./modules/ghostty.nix
    ./modules/resource-monitor.nix
  ];

  home.stateVersion = "24.11";
  home.username = "leonardo.gavaudan";
  home.homeDirectory = lib.mkForce "/Users/leonardo.gavaudan";

  # Profile-specific PATH entries are currently empty; shared PATH lives in modules/zsh.nix.

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

  # Update top-level non-worktree git repos in ~/dev every hour.
  launchd.agents.dev-repos-update = {
    enable = true;
    config = {
      Label = "com.user.dev-repos-update";
      ProgramArguments = [
        "/bin/sh"
        "-c"
        "PATH=/run/current-system/sw/bin:/usr/bin:/bin:/usr/sbin:/sbin ${config.home.homeDirectory}/dev/update.sh"
      ];
      RunAtLoad = true;
      StartInterval = 3600;
      StandardOutPath = "/tmp/dev-repos-update.log";
      StandardErrorPath = "/tmp/dev-repos-update.log";
    };
  };

  home.shellAliases = {
    ns = "sudo darwin-rebuild switch --flake ~/.config/nix-darwin";
    vl = "vault_auto_login";
  };

  programs.fzf = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.zsh = {
    plugins = [
      {
        name = "fzf-tab";
        src = pkgs.zsh-fzf-tab + "/share/fzf-tab";
      }
    ];
    initContent = lib.mkAfter ''
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

      if [ -f '/opt/homebrew/share/google-cloud-sdk/path.zsh.inc' ]; then . '/opt/homebrew/share/google-cloud-sdk/path.zsh.inc'; fi
      if [ -f '/opt/homebrew/share/google-cloud-sdk/completion.zsh.inc' ]; then . '/opt/homebrew/share/google-cloud-sdk/completion.zsh.inc'; fi

      bindkey '\e[3;5~' backward-kill-word
      bindkey '^[^?' backward-kill-word
      bindkey '\e\x7f' backward-kill-word
      bindkey '\e[127;3u' backward-kill-word
      bindkey '\e[Z' autosuggest-accept

    '';
  };

  programs.git.settings = {
    init.defaultBranch = "main";
    url."https://github.com/".insteadOf = "ssh://git@github.com/";
    url."git@github.com:".insteadOf = "https://github.com/";
  };

}

