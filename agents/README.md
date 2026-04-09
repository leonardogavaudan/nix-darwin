# Agent Layout (`shared` / `personal` / `work`)

`agent-config-sync` supports layered agent config by profile.

## Directory structure

- `agents/shared/AGENTS.md` (optional)
- `agents/shared/overlays.yaml` (optional)
- `agents/<profile>/AGENTS.md` (required)
- `agents/<profile>/overlays.yaml` (optional)
- `agents/<scope>/skills/<target>/...` (optional; create target dir when needed)
- `agents/<scope>/hooks/<target>/...` (optional; create target dir when needed)
- `agents/<scope>/commands/<target>/...` (optional; create target dir when needed)

Where:
- `<scope>` is `shared`, `personal`, or `work`
- `<target>` is `common`, `pi`, `claude`, or `codex`

## OMP / oh-my-pi bridge

`agent-config-sync` still treats Pi as the source-of-truth harness for the Pi/OMP setup.

OMP is mirrored separately by `scripts/sync-omp-from-pi.py`, which runs after the normal layered sync and copies/links the generated Pi resources into `~/.omp/agent`:

- `AGENTS.md` -> mirrored from Pi
- `skills`, `hooks`, `commands`, `models.json`, `mcp` -> mirrored from Pi
- `sessions`, `terminal-sessions`, `history.db`, `blobs` -> shared with Pi via symlinks
- `SYSTEM.md`, `config.yml`, `agent.db` -> generated for OMP
- `extensions` -> mirrored from the Pi extension directory, with a local compatibility package root for Pi-style imports

So OMP is **not** a first-class layered target here; it is a Pi-backed bridge.

## Profile selection

By default, profile comes from `$AGENT_PROFILE` (fallback: `personal`).

## `AGENTS.md` section markers

Each `AGENTS.md` can define any subset of:

- `<!-- BEGIN SHARED --> ... <!-- END SHARED -->`
- `<!-- BEGIN CLAUDE --> ... <!-- END CLAUDE -->`
- `<!-- BEGIN CODEX --> ... <!-- END CODEX -->`
- `<!-- BEGIN PI --> ... <!-- END PI -->`

If a file has no marker blocks, the whole file is treated as `SHARED` content.

Merge order:
1. `shared`
2. `<profile>`

## Overlay behavior

Overlay files support:

- `targets.<name>.prepend`
- `targets.<name>.append`
- `targets.<name>.remove_sections` (H1 names)
- `targets.<name>.section_replacements` (H2 name -> replacement body)

Target names: `pi`, `claude` (or `claude-code`), `codex`.

Overlay merge order:
1. `shared/overlays.yaml`
2. `<profile>/overlays.yaml`
3. optional CLI `--overlays PATH`

## Skills/hooks/commands sync behavior (manifest-free)

When running in layered mode (without `--source`), the sync links skills, hooks, and commands:

- merge order per target:
  1. `shared/<kind>/common`
  2. `shared/<kind>/<target>`
  3. `<profile>/<kind>/common`
  4. `<profile>/<kind>/<target>`
- `<kind>` is `skills`, `hooks`, or `commands`
- profile entries override shared entries when names collide
- destination roots:
  - Pi: `~/.pi/agent/{skills,hooks,prompts}` (commands sync to `prompts`)
  - Claude: `~/.claude/{skills,hooks,commands}`
  - Codex: `$CODEX_HOME/{skills,hooks,prompts}` (commands sync to `prompts`)
- command entries should be top-level Markdown files (Pi and Codex prompt discovery is non-recursive)

The sync updates entries that exist in your layered sources and only removes stale symlinks that point back into those managed source folders.

## Commands

Layered mode (default):

```bash
cargo run --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --profile personal
```

Legacy single-file mode:

```bash
cargo run --manifest-path ~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml -- --source /path/to/AGENTS.md
```
