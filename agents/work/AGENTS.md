# Context

## Algolia
- I work at Algolia on the Optimization team
- Team Jira: project key `OPTIM`, board ID `284` (https://algolia.atlassian.net/jira/software/c/projects/OPTIM/boards/284)
- Dev directory: `~/dev` - see `~/master/Repos.md` for local repo catalog

## Nix Setup
System is managed declaratively with **nix-darwin** + **home-manager**.

- **Config**: `~/.config/nix-darwin/flake.nix`
- **Rebuild**: `sudo darwin-rebuild switch --flake ~/.config/nix-darwin` (passwordless sudo configured)
- **Host**: `PAR-M4P-LGavaudan` (Apple Silicon)

**Package sources:**
- `environment.systemPackages` — CLI tools via nixpkgs (go, ripgrep, gh, neovim, terraform, etc.)
- `homebrew.brews` — packages not in nixpkgs or needing specific versions (nvm, postgresql@14, rbenv)
- `homebrew.casks` — GUI apps (ghostty, obsidian, cursor, docker-desktop, etc.)

**Shell (zsh):** Managed by Home Manager (`programs.zsh`). `.zshrc` is a read-only symlink to the Nix store.
- Aliases, env vars, PATH → declarative HM options (`home.shellAliases`, `home.sessionVariables`, `home.sessionPath`)
- Functions, completions, keybindings → `programs.zsh.initContent`
- API keys/tokens → `~/.secrets` (sourced by initContent, not in flake)

**Homebrew cleanup**: `zap` mode removes anything not declared in flake.

**Flake lock updates (cache-safe):**
- Always update `flake.lock` through the Rust checker at `~/.config/nix-darwin/scripts/nix-update-cached-rs/Cargo.toml`
- This is the source of truth for picking a nixpkgs revision that is already cached for declared Nix packages (`environment.systemPackages` + Home Manager packages)

## Global Instruction Sync
- **Layered sources:**
  - Shared: `~/.config/nix-darwin/agents/shared/AGENTS.md`
  - Profile: `~/.config/nix-darwin/agents/work/AGENTS.md`
- **Generated (global) per harness:**
  - **Pi:** `~/.pi/agent/AGENTS.md`
  - **Claude Code:** `~/.claude/CLAUDE.md`
  - **Codex CLI:** `${CODEX_HOME:-~/.config/codex}/AGENTS.md`
- **Sync binary source:** `~/.config/nix-darwin/scripts/agent-config-sync/Cargo.toml`
- **How to update:** edit layered files, then either:
  - `darwin-rebuild switch` (syncs automatically via home.activation hook)
  - `sync-agent-instructions` (manual alias)
- **Repo-level files** (e.g., `repo/AGENTS.md`, `repo/CLAUDE.md`) are separate and not touched by this sync.
- Do **not** edit generated files directly.

## Global Skills, Hooks & Commands Sync
- **Layered sources (manifest-free):**
  - `~/.config/nix-darwin/agents/shared/{skills,hooks,commands}/{common,pi,claude,codex}`
  - `~/.config/nix-darwin/agents/work/{skills,hooks,commands}/{common,pi,claude,codex}`
- **Generated directories:**
  - `~/.pi/agent/{skills,hooks,prompts}` (commands sync to `prompts`)
  - `~/.claude/{skills,hooks,commands}`
  - `${CODEX_HOME:-~/.config/codex}/{skills,hooks,prompts}` (commands sync to `prompts`)
- **Merge order (per target):**
  1. `shared/<kind>/common`
  2. `shared/<kind>/<target>`
  3. `work/<kind>/common`
  4. `work/<kind>/<target>`
- **How to update:** edit/add skills, hooks, or commands in layered source dirs, then run `sync-agent-instructions` (or `darwin-rebuild switch`).
- Do **not** edit generated harness directories directly.

## Assistant Response Preferences
- Prefer verbose explanations by default (unless I explicitly ask for short answers).
- Include concrete code snippets when explaining implementation details or tradeoffs.
- For PR/comment discussions, explain rationale, alternatives, and practical impact with code examples.
- **Always use `YYYY/MM/DD` for calendar dates in responses** (e.g., `2026/03/03`), unless I explicitly ask for a different format.
- **When referencing quarters at Algolia, always use the "tE FY'XX" format** (e.g., "Q4 tE FY'26" instead of just "Q4"). "tE" = "the End" of the fiscal year.

---

# Rules

## Knowledge Discovery (Check Before Exploring)
Before grepping codebases or reading source files to understand something, ALWAYS:
1. Check `~/master/INDEX.md` for relevant vault docs
2. Search Confluence using Atlassian tools (`atlassian_confluence_search`)

**Triggers - do this when:**
- User asks about an Algolia system, feature, or concept
- Need to understand business logic or architecture
- Starting work in an unfamiliar area
- About to explore code to figure out "how does X work"

## GitHub
- NEVER comment on, review, or approve PRs without explicit user confirmation
- When reviewing PRs, only provide feedback in terminal - do not submit to GitHub
- No "Generated with …" or "Co-Authored-By: Claude" attribution in commits or PR descriptions
- Before creating a PR, ALWAYS check for `.github/pull_request_template.md` or `.github/PULL_REQUEST_TEMPLATE.md` and follow that format exactly

### PR Stack Tracking
When a PR is part of a stack, **always** add a `## PR Stack` section in the PR description with a numbered list of all PRs in the stack (in order, base first). Mark the current PR. Example:
```
## PR Stack
1. #123
2. **#124** ⬅
3. #125
```
Just use the `#number` reference — GitHub auto-renders the PR title and status. When creating or updating any PR in a stack, **sync this section across all open PRs in the stack**. Bold the current PR and add ⬅ to mark it.

## Version Control (jj / Jujutsu)
**Prefer `jj` over `git`** for all version control operations. Repos should be colocated (`jj git init --colocate`) so both `jj` and `git` work, but default to `jj` commands.

- Use `jj` for: commits, rebases, amends, log, status, bookmarks, push/fetch
- Use `git` only for: operations jj doesn't support yet (e.g., some worktree edge cases)
- Bookmarks = branches: `jj bookmark create <name>` to create what GitHub sees as a branch

**Displaying stacks:** When showing a jj stack (e.g., in a table or summary):
- List changes **newest at top, oldest at bottom** (top-down reading order)
- Include columns for: change ID, bookmark (if any), sync status (synced / diverged / no bookmark), and description

**Key commands:**
- `jj new` — start a new change
- `jj commit -m "msg"` — commit current change
- `jj edit <change>` — go back and edit a previous change (descendants auto-rebase)
- `jj log` — visualize history
- `jj bookmark create/set <name>` — manage branches
- `jj git push --all` — push to GitHub

## Incremental Changes (Stacked PRs)
**Philosophy**: Think in PR stacks, not monolithic changes. Smaller is almost always better.

- **Atomic PRs**: Each PR should do one thing well. If you can split a change into multiple independent PRs, do it.
- **Ship minimal first**: Start with the smallest viable change that moves things forward. Clean up dead code, add tests, refactor - those can be follow-up PRs.
- **When in doubt, split**: A 50-line PR that changes behavior + a 200-line PR that removes dead code is better than one 250-line PR.
- **Even cleanup can be split**: Removing 3 unused functions? That could be 3 PRs. The cost of creating a PR is low; the cost of reviewing/reverting a big one is high.
- **Explicit "what's left"**: If deferring work to follow-ups, note it in the PR description so nothing gets lost.

**Why this matters**:
- Easier to review → faster merge
- Easier to revert if something breaks
- Reduces risk of conflicts with other work
- Each PR is a checkpoint - if priorities change, partial progress is still shipped

## Google Cloud
- READ ONLY - no writes (no INSERT, UPDATE, DELETE, DROP, CREATE, ALTER, gcloud create/delete/update, etc.)

## Code Style
- Use `golangci-lint fmt` instead of `gofmt`
- Bugs: add regression test when it fits.

## Package Installation
When a tool or package needs to be installed:
1. **First choice**: Add to `~/.config/nix-darwin/flake.nix` and rebuild
   - CLI tools → `environment.systemPackages`
   - Homebrew-only packages → `homebrew.brews`
   - GUI apps → `homebrew.casks`
2. **Only use ad-hoc install** (`brew install`, `go install`, etc.) if explicitly requested or for one-off testing
3. After editing the flake, run: `sudo darwin-rebuild switch --flake ~/.config/nix-darwin`

## Git Worktrees
- **Use worktrees by default** for all development work that involves code changes
- All worktrees go in `~/dev/worktrees/`
- This keeps the main repo directory clean and avoids conflicts with other agents
- **Always read the `worktrees` skill before any worktree action** (create, switch, update, cleanup)
- Use the `worktrees` skill for setup and cleanup instructions
- Naming convention: `<repo>-<feature>` (e.g., `AlgoliaWeb-fix-tooltip`, `go-add-metric`)

## Skills
Load relevant skills before taking action (GCP, GitHub, browser, worktrees, Jira, React, etc.).
Skills are synced from layered directories under `~/.config/nix-darwin/agents/{shared,work}/skills/{common,pi,claude,codex}` into harness skill folders (`~/.pi/agent/skills`, `~/.claude/skills`, `$CODEX_HOME/skills`).
Check available skill descriptions for the current harness to find the right one.

## Verifying Work
- When appropriate, use the browser to verify your work visually rather than assuming it's correct
- Don't just trust code changes - check if they actually work when possible

## Self-Improvement
- If you notice repeated friction (same context provided multiple times, manual steps that could be automated, missing skills), proactively suggest adding it to AGENTS.md or creating a skill
- When a workflow feels clunky, suggest hooks or skills to fix it
- If you learn something new about this user's preferences mid-session, ask if it should be persisted

## CLAUDE.local.md Files
- Many repos contain `CLAUDE.md` / `CLAUDE.local.md` or `AGENTS.md` / `AGENTS.local.md` files (usually in `.claude/` or `.agents/` directories) with shared and personal repo context
- Context-loading behavior differs by harness:
  - **Pi:** `repo-context-loader` auto-loads matching CLAUDE/AGENTS files recursively from the repo root when files are read/edited/written (once per repo per session)
  - **Claude Code:** native `CLAUDE.md` / `CLAUDE.local.md` loading + `~/.claude/hooks/agents-md` loads `AGENTS.md` at session start and lazily from ancestor directories on `Read|Grep|Glob`
  - **Codex/OpenCode:** no equivalent global auto-loader is configured; read relevant context files explicitly when starting in unfamiliar repos

---

# Knowledge Vault

**Location:** `~/master/` (Obsidian vault)

The vault is a persistent knowledge base about Algolia systems, architecture, and workflows. It serves as shared context across sessions - treat it as Claude's long-term memory for this user.

## Philosophy

1. **Filesystem as memory**: Important context learned in conversations should be persisted, not lost when sessions end
2. **Iterative improvement**: Each session is an opportunity to improve the knowledge base - don't just consume, contribute
3. **Discovery before deep exploration**: Check `~/master/INDEX.md` before exploring a topic to see if docs already exist
4. **Architecture before organizing**: Before suggesting where new information should go, read `~/master/Knowledge Architecture.md`

## When to Write to Vault

**DO write** when you learn something valuable for future sessions:
- Architecture patterns or system behaviors that weren't obvious
- Gotchas, caveats, non-obvious constraints
- Business logic explanations (e.g., how dim_application filters work)
- Mappings between systems
- Query patterns that work well

**DON'T write**:
- Trivial or temporary information
- Things already well-documented in official docs
- Session-specific debugging that won't generalize
- Every small detail - be selective

## How to Contribute

1. **Be proactive** - Don't ask permission, just improve the vault. Create files, directories, restructure as needed.
2. **Check before creating** - Make sure a file/dir doesn't already exist to avoid duplicates
3. **Keep INDEX.md updated** - Always update when adding/moving/renaming content
4. **Reorganize freely** - If the structure could be better, change it. Create subdirs, move files, consolidate docs.
5. **Inform, don't ask** - Tell the user what you did ("Added X to vault", "Created Y directory"), don't ask if you should

## Discovery Pattern

See **Knowledge Discovery** rule above. When checking the vault:

1. Check the "When to Read" column in INDEX.md for relevant docs — then actually read them
2. If a doc might exist but isn't obvious in INDEX, use the `qmd` skill to search semantically
3. Only explore codebases/external sources if vault doesn't have what you need

---

# CLI Tools

Most CLI tools are managed via nix-darwin (see Nix Setup above).

| Tool | Purpose | Notes |
|------|---------|-------|
| `gh` | GitHub | Authenticated as `leonardogavaudan` |
| `gcloud` / `bq` / `gsutil` | Google Cloud | READ ONLY |
| `rg` (ripgrep) | Text search | **Always use instead of `grep`**. Faster, respects .gitignore, better defaults |
| `fd` | File search | **Always use instead of `find`**. Faster, respects .gitignore, simpler syntax (e.g., `fd -e go` instead of `find . -name "*.go"`) |
| `jq` | JSON processing | |
| `yq` | YAML/TOML/XML processing | Like `jq` but for YAML; use for K8s manifests, CI configs, gcloud YAML output |
| `gron` | Make JSON greppable | Flattens JSON so you can `grep` for fields; use to explore unfamiliar API responses |
| `mlr` (miller) | CSV/TSV/JSON tabular processing | Like `jq` for tabular data; use for BQ CSV output, data exploration. Sort: `mlr --csv sort -nr field` |
| `sg` (ast-grep) | Structural code search | AST-aware search; use instead of regex grep for code patterns (e.g., `sg -p 'func $NAME($$$) error' -l go`) |
| `hyperfine` | Benchmarking | Proper statistical benchmarking for comparing commands (e.g., `hyperfine 'go test ./...' --warmup 3`) |
| `pup` | HTML processing | Like `jq` for HTML; use to extract content from web pages (e.g., `curl -s url \| pup 'div.content text{}'`) |
| `tree` | Directory structure | Quick structural overview of directories; **use instead of recursive `ls`** |
| `jj` | Version control (Jujutsu) | Preferred over `git`; see Version Control section |

## Tool Usage Guidelines

When running Bash commands, prefer modern tools over legacy defaults:

- **Search text**: `rg` over `grep` — always
- **Search files**: `fd` over `find` — always
- **Search code patterns**: `sg` over `rg` when looking for structural patterns (function signatures, struct definitions, import statements)
- **Explore JSON structure**: `gron` first to find the right path, then `jq` to extract
- **Directory overview**: `tree -L 2` over `ls -R` or multiple `ls` calls
- **Parse YAML**: `yq` over manual grep on YAML files
- **Tabular data**: `mlr` over `awk`/`cut`/`sort` pipelines on CSV/TSV
