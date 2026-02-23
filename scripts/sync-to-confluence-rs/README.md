# sync-to-confluence-rs

Current Rust implementation for vault-to-Confluence sync.

## Usage

```bash
cd ~/.config/nix-darwin/scripts/sync-to-confluence-rs
cargo run --
cargo run -- --pull
cargo run -- suggested-actions/Datamixer.md
cargo run -- --pull data/dim_application.md
```

## Required environment variables

- `CONFLUENCE_EMAIL`
- `ATLASSIAN_API_TOKEN`
- Optional: `MASTER_DIR` (defaults to `~/master`)
- Optional: `SYNC_TO_CONFLUENCE_MAPPING_FILE` (defaults to `~/.config/nix-darwin/scripts/sync-to-confluence-rs/link_mapping.yaml`)
