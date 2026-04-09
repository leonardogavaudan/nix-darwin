#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import sqlite3
import subprocess
from pathlib import Path

PI_AGENT = Path.home() / ".pi" / "agent"
OMP_AGENT = Path.home() / ".omp" / "agent"
NIX_DARWIN = Path.home() / ".config" / "nix-darwin"
PI_SETTINGS = PI_AGENT / "settings.json"
PI_AUTH = PI_AGENT / "auth.json"
PI_EXTENSIONS = PI_AGENT / "extensions"
OMP_EXTENSIONS = OMP_AGENT / "extensions"
OMP_LOCAL_EXTENSIONS = NIX_DARWIN / "scripts" / "omp-extensions"
OMP_BUN_VERSION = "bun-v1.3.10"
OMP_RUNTIME_BUN = Path.home() / ".local" / "share" / "omp-runtime" / OMP_BUN_VERSION / "bun"
OMP_RUNTIME_BUN_CMD = str(OMP_RUNTIME_BUN)

CORE_DEPENDENCIES = {
    "@mariozechner/pi-coding-agent": "^0.57.1",
    "@mariozechner/pi-ai": "^0.57.1",
    "@mariozechner/pi-tui": "^0.57.1",
    "@oh-my-pi/pi-coding-agent": "^13.12.8",
    "@oh-my-pi/pi-utils": "^13.12.8",
    "@sinclair/typebox": "^0.34.48",
    "mermaid-isomorphic": "^3.1.0",
    "playwright": "^1.55.0",
}

SKIP_EXTENSION_NAMES = {
    "node_modules",
    "package-lock.json",
    "package.json",
    "bun.lock",
}


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def write_text_if_changed(path: Path, content: str) -> None:
    ensure_parent(path)
    if path.exists() and not path.is_symlink() and path.read_text() == content:
        return
    path.write_text(content)


def replace_with_symlink(dst: Path, src: Path) -> None:
    ensure_parent(dst)
    if dst.is_symlink() and dst.resolve() == src.resolve():
        return
    if dst.exists() or dst.is_symlink():
        if dst.is_dir() and not dst.is_symlink():
            shutil.rmtree(dst)
        else:
            dst.unlink()
    dst.symlink_to(src)


def remove_path(path: Path) -> None:
    if not path.exists() and not path.is_symlink():
        return
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink()


def sync_basic_symlinks() -> None:
    mappings = {
        OMP_AGENT / "AGENTS.md": PI_AGENT / "AGENTS.md",
        OMP_AGENT / "skills": PI_AGENT / "skills",
        OMP_AGENT / "commands": PI_AGENT / "prompts",
        OMP_AGENT / "models.json": PI_AGENT / "models.json",
        OMP_AGENT / "mcp": PI_AGENT / "mcp",
        OMP_AGENT / "hooks": PI_AGENT / "hooks",
    }
    for dst, src in mappings.items():
        if src.exists() or src.is_symlink():
            replace_with_symlink(dst, src)
        else:
            remove_path(dst)

    shared_storage_dirs = [
        PI_AGENT / "sessions",
        PI_AGENT / "terminal-sessions",
        PI_AGENT / "blobs",
    ]
    for directory in shared_storage_dirs:
        directory.mkdir(parents=True, exist_ok=True)

    shared_storage_mappings = {
        OMP_AGENT / "sessions": PI_AGENT / "sessions",
        OMP_AGENT / "terminal-sessions": PI_AGENT / "terminal-sessions",
        OMP_AGENT / "history.db": PI_AGENT / "history.db",
        OMP_AGENT / "blobs": PI_AGENT / "blobs",
    }
    for dst, src in shared_storage_mappings.items():
        ensure_parent(src)
        replace_with_symlink(dst, src)


def write_system_prompt() -> None:
    system_text = """# OMP System Prompt

This OMP install is intentionally configured to mirror the existing `~/.pi/agent` setup as closely as possible.

## Resource locations

- Global reference instructions: `~/.omp/agent/AGENTS.md` (mirrored from Pi)
- Global skills: `~/.omp/agent/skills`
- Global commands: `~/.omp/agent/commands`
- Global extensions: `~/.omp/agent/extensions`

## Global working rules

- Follow repo `AGENTS.md` / `AGENTS.local.md` files when present.
- Prefer verbose explanations by default unless the user asks for brevity.
- Always use `YYYY/MM/DD` for dates unless the user asks otherwise.
- Prefer `jj` over `git` for version control unless repo instructions say otherwise.
- Use worktrees by default for code changes when that fits the repo workflow.
- Never submit GitHub reviews, comments, approvals, or other remote write actions without explicit user confirmation.
- For unfamiliar Algolia systems or business logic, check `~/master/INDEX.md` and Confluence before deep code exploration.
- For nix package additions, do not choose options that build from source on `aarch64-darwin`; prefer cached binaries or Homebrew / official installers.
- Prefer verifying important changes rather than assuming they work.
- When sharing Mermaid diagrams in OMP, prefer the `render_mermaid` tool over raw fenced Mermaid blocks unless the user explicitly asks for source text.

## Mirror note

If a path differs only because this is OMP instead of Pi, translate `~/.pi/agent/...` to `~/.omp/agent/...` when appropriate.
"""
    system_path = OMP_AGENT / "SYSTEM.md"
    if system_path.is_symlink():
        system_path.unlink()
    write_text_if_changed(system_path, system_text)


def adapt_model_ref(model_ref: str) -> str:
    if model_ref == "opencode/kimi-k2.5":
        return "opencode-zen/kimi-k2.5"
    return model_ref


def adapt_service_tier(raw_value: object) -> str | None:
    if not isinstance(raw_value, str):
        return None

    normalized = raw_value.strip().lower()
    if not normalized:
        return None

    if normalized == "fast":
        return "priority"

    if normalized in {"auto", "default", "flex", "scale", "priority"}:
        return normalized

    return None


def write_config() -> None:
    settings = {}
    if PI_SETTINGS.exists():
        settings = json.loads(PI_SETTINGS.read_text())

    enabled_models = [adapt_model_ref(model) for model in (settings.get("enabledModels") or [])]
    default_provider = settings.get("defaultProvider", "openai-codex")
    default_model = settings.get("defaultModel", "gpt-5.4")
    default_model_ref = adapt_model_ref(f"{default_provider}/{default_model}")
    smol_model = enabled_models[1] if len(enabled_models) > 1 else default_model_ref
    plan_model = enabled_models[-1] if enabled_models else default_model_ref
    service_tier = adapt_service_tier(settings.get("openaiCodexServiceTier"))

    lines = [
        "# Generated by ~/.config/nix-darwin/scripts/sync-omp-from-pi.py",
        "# Mirrors the current ~/.pi/agent setup as closely as possible.",
        "",
        "modelRoles:",
        f"  default: {default_model_ref}",
        f"  smol: {smol_model}",
        f"  plan: {plan_model}",
        f"  commit: {default_model_ref}",
        f"defaultThinkingLevel: {settings.get('defaultThinkingLevel', 'high')}",
        f"hideThinkingBlock: {'true' if settings.get('hideThinkingBlock', False) else 'false'}",
        "steeringMode: one-at-a-time",
        "followUpMode: one-at-a-time",
        "skills:",
        "  enabled: true",
        "extensions:",
        "  - ~/.omp/agent/extensions/render-mermaid-image.ts",
        "  - ~/.omp/agent/extensions/enable-render-mermaid.ts",
        "disabledExtensions: []",
    ]

    if service_tier:
        lines.append(f"serviceTier: {service_tier}")

    if enabled_models:
        lines.append("enabledModels:")
        for model in enabled_models:
            lines.append(f"  - {model}")

    config_path = OMP_AGENT / "config.yml"
    write_text_if_changed(config_path, "\n".join(lines) + "\n")


def write_extensions_package_json() -> None:
    OMP_EXTENSIONS.mkdir(parents=True, exist_ok=True)
    package_json = {
        "name": "omp-pi-extension-bridge",
        "private": True,
        "type": "module",
        "scripts": {
            "postinstall": f"{OMP_RUNTIME_BUN_CMD} ./apply-omp-dependency-patches.mjs",
        },
        "dependencies": CORE_DEPENDENCIES,
    }
    write_text_if_changed(OMP_EXTENSIONS / "package.json", json.dumps(package_json, indent=2) + "\n")


def sync_extensions() -> None:
    OMP_EXTENSIONS.mkdir(parents=True, exist_ok=True)
    desired_names: set[str] = set()

    if PI_EXTENSIONS.exists():
        for entry in PI_EXTENSIONS.iterdir():
            if entry.name in SKIP_EXTENSION_NAMES:
                continue
            desired_names.add(entry.name)
            dst = OMP_EXTENSIONS / entry.name

            if entry.is_dir() and ((entry / "package.json").exists() or (entry / "node_modules").exists()):
                replace_with_symlink(dst, entry)
                continue

            if dst.is_symlink():
                dst.unlink()
            if entry.is_dir():
                if dst.exists():
                    shutil.rmtree(dst)
                shutil.copytree(entry, dst)
            else:
                ensure_parent(dst)
                shutil.copy2(entry, dst)

    if OMP_LOCAL_EXTENSIONS.exists():
        for entry in OMP_LOCAL_EXTENSIONS.iterdir():
            if entry.name in SKIP_EXTENSION_NAMES:
                continue
            desired_names.add(entry.name)
            dst = OMP_EXTENSIONS / entry.name

            if dst.is_symlink():
                dst.unlink()
            if entry.is_dir():
                if dst.exists():
                    shutil.rmtree(dst)
                shutil.copytree(entry, dst)
            else:
                ensure_parent(dst)
                shutil.copy2(entry, dst)

    for existing in OMP_EXTENSIONS.iterdir():
        if existing.name in {"package.json", "package-lock.json", "bun.lock", "node_modules"}:
            continue
        if existing.name not in desired_names:
            remove_path(existing)


def ensure_extension_dependencies() -> None:
    if not OMP_RUNTIME_BUN.exists() or not (OMP_EXTENSIONS / "package.json").exists():
        return
    subprocess.run([str(OMP_RUNTIME_BUN), "install"], cwd=str(OMP_EXTENSIONS), check=True)


def apply_dependency_patches() -> None:
    patch_script = OMP_EXTENSIONS / "apply-omp-dependency-patches.mjs"
    if not patch_script.exists():
        return
    subprocess.run([OMP_RUNTIME_BUN_CMD, str(patch_script)], check=True)


def init_agent_db() -> sqlite3.Connection:
    db_path = OMP_AGENT / "agent.db"
    ensure_parent(db_path)
    conn = sqlite3.connect(db_path)
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)")
    conn.execute(
        """
        CREATE TABLE IF NOT EXISTS auth_credentials (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            credential_type TEXT NOT NULL,
            data TEXT NOT NULL,
            disabled_cause TEXT DEFAULT NULL,
            identity_key TEXT DEFAULT NULL,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        )
        """
    )
    conn.execute("CREATE INDEX IF NOT EXISTS idx_auth_provider ON auth_credentials(provider)")
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_auth_provider_identity ON auth_credentials(provider, identity_key) WHERE identity_key IS NOT NULL"
    )
    conn.execute("INSERT OR REPLACE INTO schema_version(version) VALUES (4)")
    return conn


def identity_key(provider: str, data: dict[str, object]) -> str | None:
    email = data.get("email")
    account_id = data.get("accountId")
    project_id = data.get("projectId")
    if isinstance(email, str) and email.strip():
        return f"email:{email.strip().lower()}"
    if isinstance(account_id, str) and account_id.strip():
        return f"account:{account_id.strip()}"
    if isinstance(project_id, str) and project_id.strip():
        return f"project:{project_id.strip()}"
    return None


def sync_auth() -> None:
    if not PI_AUTH.exists():
        return

    auth = json.loads(PI_AUTH.read_text())
    if not isinstance(auth, dict):
        return

    conn = init_agent_db()
    try:
        for provider, credential in auth.items():
            if not isinstance(provider, str):
                continue
            entries = credential if isinstance(credential, list) else [credential]
            conn.execute(
                "DELETE FROM auth_credentials WHERE provider = ? AND disabled_cause IS NULL",
                (provider,),
            )
            for entry in entries:
                if not isinstance(entry, dict):
                    continue
                cred_type = entry.get("type")
                if cred_type == "oauth":
                    payload = {k: v for k, v in entry.items() if k != "type"}
                    conn.execute(
                        "INSERT INTO auth_credentials (provider, credential_type, data, identity_key) VALUES (?, 'oauth', ?, ?)",
                        (provider, json.dumps(payload), identity_key(provider, payload)),
                    )
                elif cred_type == "api_key" and isinstance(entry.get("key"), str):
                    payload = {"key": entry["key"]}
                    conn.execute(
                        "INSERT INTO auth_credentials (provider, credential_type, data, identity_key) VALUES (?, 'api_key', ?, NULL)",
                        (provider, json.dumps(payload)),
                    )
        conn.commit()
    finally:
        conn.close()


def main() -> None:
    OMP_AGENT.mkdir(parents=True, exist_ok=True)
    sync_basic_symlinks()
    write_system_prompt()
    write_config()
    write_extensions_package_json()
    sync_extensions()
    ensure_extension_dependencies()
    apply_dependency_patches()
    sync_auth()


if __name__ == "__main__":
    main()
