{ config, pkgs, ... }:

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

  programs.home-manager.enable = true;
}
