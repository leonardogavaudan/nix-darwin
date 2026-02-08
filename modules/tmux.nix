{ config, pkgs, ... }:

{
  programs.tmux = {
    enable = true;
    keyMode = "vi";
    prefix = "C-t";
    terminal = "tmux-256color";

    extraConfig = ''
      # Enable extended keys (CSI u / kitty protocol) so Shift+Enter etc. are forwarded
      set -g extended-keys on
      set -as terminal-features 'tmux-256color:extkeys'

      # Enter copy mode with prefix + t
      bind-key t copy-mode

      # Scroll up/down by one line with , and . in copy mode
      bind-key -T copy-mode-vi , send-keys -X scroll-down
      bind-key -T copy-mode-vi . send-keys -X scroll-up

      # Reload config
      bind-key r source-file ~/.tmux.conf \; display-message "Config reloaded!"

      # Allow escape sequences for clipboard image pasting (e.g., Claude Code)
      set-option -g allow-passthrough on
    '';
  };
}
