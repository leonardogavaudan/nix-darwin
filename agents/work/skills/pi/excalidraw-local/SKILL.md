---
name: excalidraw-local
description: Build, preview, and screenshot Excalidraw diagrams in Pi with one canonical modal-free URL.
---

# Excalidraw Local (Pi)

Use this skill for all `excalidraw_local_*` work in Pi.

## Golden Rule (single source of truth)

After `excalidraw_local_create_view`, always use the returned canonical preview URL:

- **`http://localhost:8787/latest.html`** (or `file://.../latest.html` fallback)

This URL is generated from checkpoint data, so it avoids Excalidraw import modals and is screenshot-safe.

## What each endpoint is

| Purpose | URL | Notes |
|---|---|---|
| MCP API | `http://127.0.0.1:3001/mcp` | Tool endpoint only, not visual UI |
| Canonical preview | `http://localhost:8787/latest.html` | Modal-free local render from latest checkpoint |
| Per-checkpoint preview | `http://localhost:8787/<checkpointId>.html` | Stable URL for a specific checkpoint |
| Dev mock page | `http://localhost:5173/index-dev.html` | Demo only, not your generated diagram |

## Standard workflow

1. Call `excalidraw_local_create_view` (defaults: `generatePreview=true`, `openPreview=true`).
2. Copy/use `Checkpoint` and `Preview (canonical)` from tool output.
3. For subsequent edits, use `restoreCheckpoint` with the checkpoint ID.
4. If needed later, regenerate/open preview with `excalidraw_local_preview_checkpoint`.

## Screenshot workflow

- Capture the canonical preview page (`latest.html`) or per-checkpoint page.
- Avoid direct Excalidraw share URLs when validating screenshots (they can show "Replace my content").
- Always target explicit tab index before screenshot commands.

## Troubleshooting

### I still see a modal/banner
You are likely on an Excalidraw share URL or old wrapper tab.
Open the canonical preview URL from the latest tool result.

### I only see the 3-box demo
You are on `localhost:5173/index-dev.html` (mock page).

### Preview URL not loading
Run `excalidraw_local_status`:
- MCP tmux session should be running (`excalidraw-mcp`)
- preview tmux session should be running (`excalidraw-preview`)

### Diagram still clipped
Increase final `cameraUpdate` size (e.g., `1200x900`) and regenerate.
