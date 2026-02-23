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

  # Profile-specific PATH entries live in modules/zsh.nix for now.

  home.sessionVariables = {
    EDITOR = "vim";
    XDG_CONFIG_HOME = "$HOME/.config";
    AWS_CONFIG_FILE = "$HOME/.config/aws/config";
    AWS_SHARED_CREDENTIALS_FILE = "$HOME/.config/aws/credentials";
    GOPATH = "$HOME/.config/go";
    CARGO_HOME = "$HOME/.config/cargo";
    RUSTUP_HOME = "$HOME/.config/rustup";
    CODEX_HOME = "$HOME/.config/codex";
    AGENT_PROFILE = "personal";
  };

  # Sync generated harness instruction files during activation.
  home.activation.syncAgentInstructions = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    run env PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH" ${pkgs.cargo}/bin/cargo run --quiet --manifest-path ${config.home.homeDirectory}/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile personal
  '';

  programs.zsh = {
    profileExtra = ''
      eval "$(/opt/homebrew/bin/brew shellenv)"
    '';
  };

  programs.home-manager.enable = true;
}
