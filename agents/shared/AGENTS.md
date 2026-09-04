# AGENTS.md (shared)

Shared instruction layer for all profiles.

## Workflow

- Edit this file for shared guidance.
- Add profile-specific guidance in `agents/<profile>/AGENTS.md`.
- Run:
  - `cargo run --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml --`

## Section Contract

Each file can define any subset of these sections:
- `SHARED`
- `CLAUDE`
- `CODEX`
- `PI`

In layered mode, `shared` content is merged before profile content.

<!-- BEGIN SHARED -->
## Project Overview

Personal development environment and dotfiles for Leonardo Gavaudan.

## Key Directories

- `~/master` - Obsidian vault (symlink to `~/Google Drive/My Drive/master`). Primary knowledge base for notes, research, and general content.
- `~/.config/nix-darwin/` - Nix Darwin + Home Manager configuration
  - `flake.nix` - System packages, Homebrew casks/brews, Home Manager wiring
  - `home-personal.nix` / `home-work.nix` - Top-level Home Manager configs by profile
  - `modules/zsh.nix` - Shell config (aliases, env vars, PATH, secrets)
  - `modules/tmux.nix` - Tmux config (prefix, keybindings, options)
  - `modules/ghostty.nix` - Ghostty terminal config

Managed dotfiles are read-only symlinks created by Home Manager: zsh files live in `~/.config/zsh/` (`.zshrc`, `.zshenv`, `.zprofile`) with a bootstrap `~/.zshenv`, plus `~/.config/tmux/tmux.conf` and `~/.config/ghostty/config`. Edit `modules/*.nix` and rebuild.

## User Info

- Email: leonardogavaudan@gmail.com
- Work email: leonardo.gavaudan@algolia.com
- Phone: +33 6 74 39 79 75
- Address: 78 Rue du Cherche-Midi, 75006, Paris, France
- Date of birth: 03/08/1998
- Birthplace: London
- Nationalities: French, Italian
- Languages: Fluent French and English; conversational Italian
- Weight: ~72-74 kg

## Rules

- **Nix packages must never build from source.** Before adding a package to the nix-darwin config, verify it has a pre-built binary in the Nix binary cache for `aarch64-darwin`. If no binary is available, install via Homebrew (`brews`/`casks`) or `bun add -g` instead.

## Available Tools

### MCP Servers (Cloud Integrations)

- **Beeper** - Search & send messages across all chat networks (iMessage, WhatsApp, etc.)
- **Gmail** - Read, send, draft, search, label, filter emails
- **Google Calendar** - List, create, update, delete events; check free/busy
- **Google Maps** - Geocode, directions, place search, distance matrix
- **Notion** - Search, read, create, update pages & databases
- **Exa** - AI-powered web search, company research, code context
- **Context7** - Up-to-date library/framework documentation lookup from source repos
- **Xpoz** - Social media data — Twitter, Instagram, Reddit (users, posts, comments)
- **Claude in Chrome** - Browser automation — navigate, click, fill forms, screenshots, record GIFs

### Local CLI Tools

- `git`, `gh` - Version control & GitHub CLI
- `bun`, `node`, `npm` - JavaScript runtimes & package managers
- `rustc`, `cargo` - Rust toolchain
- `python3` - Python scripting
- `nix`, `darwin-rebuild` - System config (Nix Darwin + Home Manager)
- `rg` (ripgrep), `fd`, `eza`, `tree` - Fast search & file utilities
- `jq` - JSON processing
- `yq` - YAML/TOML processing
- `mlr` (miller) - CSV/tabular data processing
- `pup` - HTML parsing (like jq for HTML)
- `difftastic` - Syntax-aware structural diffs
- `ast-grep` (`sg`) - Structural code search/replace using AST patterns
- `duckdb` - SQL queries directly on CSV/JSON/Parquet files
- `pandoc` - Universal document format conversion
- `imagemagick` (`convert`, `magick`) - Image processing & manipulation
- `shellcheck` - Shell script static analysis
- `tmux` - Terminal multiplexer
- `sqlite3` - Local database queries
- `curl`, `wget` - HTTP requests
- `podman` - Container runtime
- `cloudflare-wrangler` - Cloudflare Workers deployment

## Common Tasks

- Rebuild nix-darwin: `sudo darwin-rebuild switch --flake ~/.config/nix-darwin`
- `sudo darwin-rebuild switch --flake ~/.config/nix-darwin` is passwordless on this machine; agents may run it directly.
- Codex config location is `~/.config/codex` only; `CODEX_HOME` must be set to `~/.config/codex`.
- Sync agent configs: `cargo run --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml --`
<!-- END SHARED -->
