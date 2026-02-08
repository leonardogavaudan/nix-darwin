{ config, pkgs, ... }:

{
  xdg.configFile."ghostty/config".text = ''
    background = 040a12
    font-family = CommitMono Nerd Font
    font-size = 17
    macos-option-as-alt = true
    window-position-x = 0
    window-position-y = 0

    # Pass Rectangle shortcuts through to macOS
    keybind = ctrl+alt+left=ignore
    keybind = ctrl+alt+right=ignore
    keybind = ctrl+alt+up=ignore
    keybind = ctrl+alt+down=ignore
  '';
}
