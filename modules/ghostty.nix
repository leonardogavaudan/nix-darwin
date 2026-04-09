{ ... }:

{
  programs.ghostty = {
    enable = true;
    package = null;
    enableZshIntegration = true;
    settings = {
      theme = "Catppuccin Mocha";
      # Keep dim syntax/punctuation readable against the theme background.
      # Pi/tmux often render punctuation and secondary tokens with ANSI color 8.
      palette = "8=#7f849c";
      font-size = 15;
      font-family = "CommitMono";
      macos-option-as-alt = true;
      window-position-x = 0;
      window-position-y = 0;
      keybind = [
        "option+backspace=text:\\x1b[127;3u"
        "shift+backspace=text:\\x7f"
        "shift+space=text: "

        # Pass Rectangle shortcuts through to macOS
        "ctrl+alt+left=ignore"
        "ctrl+alt+right=ignore"
        "ctrl+alt+up=ignore"
        "ctrl+alt+down=ignore"

        # Reorder tabs
        "ctrl+shift+left=move_tab:-1"
        "ctrl+shift+right=move_tab:1"

        # Layout fix: on some non-US/Dvorak setups, Cmd+Shift+= resolves
        # to the physical bracket-right key. Keep that chord for zoom.
        "super+shift+bracket_right=increase_font_size:1"

        # Keep explicit tab navigation bindings (layout-aware symbols).
        "super+shift+]=next_tab"
        "super+shift+[=previous_tab"
      ];
    };
  };
}
