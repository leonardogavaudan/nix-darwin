{ config, pkgs, ... }:

{
  programs.tmux = {
    enable = true;
    prefix = "C-t";
    keyMode = "vi";
    mouse = true;
    escapeTime = 0;
    terminal = "tmux-256color";

    extraConfig = ''
      set -g default-terminal "tmux-256color"
      set -g history-limit 200000
      set -g renumber-windows on

      bind [ split-window -h -c "#{pane_current_path}"
      bind ] split-window -v -c "#{pane_current_path}"

      bind h select-pane -L
      bind j select-pane -D
      bind k select-pane -U
      bind l select-pane -R

      bind-key t copy-mode
      bind-key -T copy-mode-vi y send-keys -X copy-pipe-and-cancel "pbcopy"
      bind-key -T copy-mode-vi , send-keys -X scroll-up
      bind-key -T copy-mode-vi . send-keys -X scroll-down
      bind-key r source-file ~/.config/tmux/tmux.conf \; display-message "Config reloaded!"

      # Enable extended keys. Force tmux to emit CSI-u instead of xterm
      # modifyOtherKeys sequences so TUIs like Pi can recognize Ctrl bindings.
      set -s extended-keys always
      set -s extended-keys-format csi-u
      set -as terminal-features 'tmux-256color:extkeys'
      set -as terminal-features 'xterm-ghostty:extkeys'
      set -as terminal-features 'xterm-256color:extkeys'
      set -ga terminal-overrides ',*:kDC5=\e[3;5~'
      set -ga terminal-overrides ',*:kDC6=\e[3;6~'
      set -ga terminal-overrides ',*:kDC7=\e[3;7~'
      set -gw xterm-keys on

      # Allow escape sequences for clipboard image pasting (e.g., Claude Code)
      set-option -g allow-passthrough on
    '';
  };
}
