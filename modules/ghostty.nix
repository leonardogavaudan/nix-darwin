{ config, pkgs, ... }:

{
  xdg.configFile."ghostty/config".text = ''
    background = 040a12
    font-family = CommitMono Nerd Font
    font-size = 17
    macos-option-as-alt = true
    window-position-x = 0
    window-position-y = 0

    # Pi key compatibility
    keybind = alt+backspace=text:\x1b\x7f
    keybind = shift+enter=text:\n
    # Pass Rectangle shortcuts through to macOS
    keybind = ctrl+alt+left=ignore
    keybind = ctrl+alt+right=ignore
    keybind = ctrl+alt+up=ignore
    keybind = ctrl+alt+down=ignore

    # Reorder tabs
    keybind = ctrl+shift+left=move_tab:-1
    keybind = ctrl+shift+right=move_tab:1
  '';
}
