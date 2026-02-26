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
    vl = "vault_auto_login";
  };

  programs.zsh.initContent = lib.mkAfter ''
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
  '';

  programs.git.settings = {
    init.defaultBranch = "main";
    url."https://github.com/".insteadOf = "ssh://git@github.com/";
    url."git@github.com:".insteadOf = "https://github.com/";
  };

}
