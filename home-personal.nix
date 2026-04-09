{ config, pkgs, lib, ... }:

{
  imports = [
    ./modules/zsh.nix
    ./modules/common-programs.nix
    ./modules/tmux.nix
    ./modules/ghostty.nix
    ./modules/resource-monitor.nix
  ];

  home.username = "leonardogavaudan";
  home.homeDirectory = "/Users/leonardogavaudan";
  home.stateVersion = "24.11";

  # Keep XDG-based paths enabled on macOS.
  xdg.enable = true;
  xdg.configHome = "${config.home.homeDirectory}/.config";

  # Profile-specific PATH entries live in modules/zsh.nix for now.

  home.sessionVariables = {
    EDITOR = "vim";
    AWS_CONFIG_FILE = "$HOME/.config/aws/config";
    AWS_SHARED_CREDENTIALS_FILE = "$HOME/.config/aws/credentials";
    GOPATH = "$HOME/.config/go";
    CARGO_HOME = "$HOME/.config/cargo";
    RUSTUP_HOME = "$HOME/.config/rustup";
    CODEX_HOME = "$HOME/.config/codex";
    AGENT_PROFILE = "personal";
  };

  # Sync generated harness instruction files during activation, then mirror Pi into OMP.
  home.activation.syncAgentInstructions = lib.hm.dag.entryAfter [ "ensureOmpInstalled" ] ''
    run ${config.home.homeDirectory}/.local/bin/sync-agent-harnesses personal
  '';

  programs.zsh = {
    profileExtra = ''
      eval "$(/opt/homebrew/bin/brew shellenv)"
    '';
  };

  programs.home-manager.enable = true;
}
