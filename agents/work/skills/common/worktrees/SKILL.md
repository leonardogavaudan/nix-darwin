---
name: worktrees
description: Git worktree management for isolated development. Use when working on multiple features, running parallel agents, or needing isolation from the main working directory.
---

# Git Worktrees

Worktrees let you check out multiple branches simultaneously in separate directories. Each worktree has its own working directory but shares the same git history.

## When to Use Worktrees

- **Multi-agent development** - Each agent needs its own directory to avoid git conflicts
- **Parallel feature work** - Work on multiple features without stashing
- **Testing in isolation** - Keep your main worktree clean while experimenting
- **Long-running tasks** - Let one agent work while you continue in main directory

## Creating a Worktree

All worktrees live in `~/dev/worktrees/`.

```bash
cd ~/dev/<repo>

# Fetch latest and determine default branch
git fetch origin
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")

# Create worktree with new branch from origin's default
WORKTREE_PATH=~/dev/worktrees/<repo>-<suffix>
BRANCH="<branch-name>"
git worktree add -b "$BRANCH" "$WORKTREE_PATH" "origin/$DEFAULT_BRANCH"

# Copy all AGENTS.local.md files to their corresponding paths
fd -H -I -t f -g "AGENTS.local.md" . | while read -r f; do
    mkdir -p "$WORKTREE_PATH/$(dirname "$f")"
    cp "$f" "$WORKTREE_PATH/$f"
    echo "✓ Copied $f"
done

# Copy globally-ignored files (from core.excludesfile) into the new worktree
cargo run --quiet --manifest-path ~/.config/nix-darwin/scripts/worktree-copy-global-ignored/Cargo.toml -- \
    --source "$(git rev-parse --show-toplevel)" \
    --worktree "$WORKTREE_PATH"

cd "$WORKTREE_PATH"

# If repo has submodules: nuke copied files and re-init properly
if [ -f .gitmodules ]; then
    git submodule status | awk '{print $2}' | while read sub; do
        rm -rf "$sub"
    done
    git submodule update --init
fi
```

**Example:**
```bash
cd ~/dev/AlgoliaWeb
git fetch origin
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")
git worktree add -b feat/add-feature-x ~/dev/worktrees/AlgoliaWeb-feature-x "origin/$DEFAULT_BRANCH"
fd -H -I -t f -g "AGENTS.local.md" . | while read -r f; do
    mkdir -p ~/dev/worktrees/AlgoliaWeb-feature-x/"$(dirname "$f")"
    cp "$f" ~/dev/worktrees/AlgoliaWeb-feature-x/"$f"
done
cargo run --quiet --manifest-path ~/.config/nix-darwin/scripts/worktree-copy-global-ignored/Cargo.toml -- \
  --source "$(git rev-parse --show-toplevel)" \
  --worktree ~/dev/worktrees/AlgoliaWeb-feature-x
cd ~/dev/worktrees/AlgoliaWeb-feature-x
```

### Using an Existing Branch

If the branch already exists locally:

```bash
git worktree add ~/dev/worktrees/<repo>-<suffix> <existing-branch>
```

## After Creating a Worktree

1. **Read `AGENTS.local.md`** - Contains repo-specific gotchas, commands, and multi-agent tips
2. **Re-initialize git submodules** - Worktrees copy submodule files but not git metadata, so `git submodule update` will fail. Nuke and re-clone:
   ```bash
   # List submodules to find their paths
   git submodule status

   # For each submodule: remove the directory and re-initialize
   rm -rf <submodule-path>
   git submodule update --init <submodule-path>
   ```
3. **Install dependencies** if needed - worktrees share git history but not installed packages
4. **Check for port conflicts** - If running dev servers in parallel, see `AGENTS.local.md` for port management

## Listing Worktrees

```bash
git worktree list
```

Output shows all worktrees and their branches:
```
/Users/leonardo.gavaudan/dev/AlgoliaWeb                       abc1234 [develop]
/Users/leonardo.gavaudan/dev/worktrees/AlgoliaWeb-feature-x   def5678 [feat/add-feature-x]
```

## Cleanup

After your work is merged, remove the worktree and branch:

```bash
# From the main repo directory
cd ~/dev/<repo>

# Remove the worktree
git worktree remove ~/dev/worktrees/<repo>-<suffix> --force

# Delete the local branch
git branch -D <branch-name>
```

**Example:**
```bash
cd ~/dev/AlgoliaWeb
git worktree remove ~/dev/worktrees/AlgoliaWeb-feature-x --force
git branch -D feat/add-feature-x
```

## Troubleshooting

### "fatal: '<path>' is already checked out"
The branch is in use by another worktree. Either:
- Use a different branch name
- Remove the existing worktree first

### Worktree shows as "prunable"
The worktree directory was deleted without `git worktree remove`. Clean up:
```bash
git worktree prune
```

### Git submodule fails with "already exists and is not an empty directory"
Worktrees copy submodule files but not the `.git` metadata that submodule commands need. Running `git submodule update --init` will fail because the directory exists but isn't a proper git repo. Fix by nuking the submodule directory first:
```bash
rm -rf <submodule-path>
git submodule update --init <submodule-path>
```

### Dependencies missing in new worktree
Worktrees share git history but not installed dependencies:
```bash
yarn install   # For JS projects
bundle install # For Ruby projects
pip install -e .  # For Python projects
```
