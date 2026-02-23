{ ... }:

{
  programs.ghostty = {
    enable = true;
    package = null;
    enableZshIntegration = true;
    settings = {
      theme = "Catppuccin Mocha";
      palette = "8=#1e1e2e";
      background = "040a12";
      font-size = 15;
      font-family = "CommitMono";
      macos-option-as-alt = true;
      window-position-x = 0;
      window-position-y = 0;
      keybind = [
        "option+backspace=text:\\x1b[127;3u"
        "shift+backspace=text:\\x7f"
        "shift+space=text: "
        "shift+enter=text:\\n"

        # Pass Rectangle shortcuts through to macOS
        "ctrl+alt+left=ignore"
        "ctrl+alt+right=ignore"
        "ctrl+alt+up=ignore"
        "ctrl+alt+down=ignore"

        # Reorder tabs
        "ctrl+shift+left=move_tab:-1"
        "ctrl+shift+right=move_tab:1"
      ];
    };
  };
}
