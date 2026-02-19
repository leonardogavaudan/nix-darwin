{ config, pkgs, lib, ... }:

{
  imports = [
    ./modules/zsh.nix
    ./modules/tmux.nix
    ./modules/ghostty.nix
    ./modules/resource-monitor.nix
  ];

  home.username = "leonardogavaudan";
  home.homeDirectory = "/Users/leonardogavaudan";
  home.stateVersion = "24.11";

  # Required for xdg.configFile on macOS
  xdg.enable = true;

  # Single source of truth for user-level PATH entries.
  home.sessionPath = [
    "$HOME/.local/bin"
    "$HOME/.cache/.bun/bin"
    "$HOME/.bun/bin"
    "$HOME/.config/go/bin"
    "$HOME/.config/cargo/bin"
  ];

  # Sync generated harness instruction files during activation.
  home.activation.syncAgentInstructions = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    run env PATH="/usr/bin:/bin:/usr/sbin:/sbin:$PATH" ${pkgs.cargo}/bin/cargo run --quiet --manifest-path ${config.home.homeDirectory}/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile personal
  '';

  programs.home-manager.enable = true;
}
