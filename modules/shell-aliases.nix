{ ... }:

{
  home.shellAliases = {
    ".." = "cd ..";
    python = "python3";
    vim = "nvim";

    tc = "tmux new-session claude";
    tn = "tmux new-session";
    tp = "tmux new-session pi";
    tx = "tmux new-session codex";

    sync-agent-instructions = "cargo run --quiet --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile $AGENT_PROFILE";
    sync-agents = "sync-agent-instructions";
  };
}
