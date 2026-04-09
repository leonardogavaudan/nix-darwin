---
name: excalidraw-local
description: Build, preview, and screenshot Excalidraw diagrams in Pi with one canonical modal-free URL.
---

# Excalidraw Local (Pi)

Use this skill for all `excalidraw_local_*` work in Pi.

## Golden Rule (single source of truth)

After `excalidraw_local_create_view` **or** `excalidraw_local_create_from_mermaid`, always use the returned canonical preview URL:

- **`http://localhost:8787/latest.html`** (or `file://.../latest.html` fallback)

This URL is generated from checkpoint data, so it avoids Excalidraw import modals and is screenshot-safe.

## What each endpoint is

| Purpose | URL | Notes |
|---|---|---|
| MCP API | `http://127.0.0.1:3001/mcp` | Tool endpoint only, not visual UI |
| Canonical preview | `http://localhost:8787/latest.html` | Modal-free local render from latest checkpoint |
| Per-checkpoint preview | `http://localhost:8787/<checkpointId>.html` | Stable URL for a specific checkpoint |
| Dev mock page | `http://localhost:5173/index-dev.html` | Demo only, not your generated diagram |

## Default creation policy

**Prefer Mermaid first for diagram creation.**

When the user asks for a diagram, the default order is:

1. **First try `excalidraw_local_create_from_mermaid`**
   - even if the user did not provide Mermaid explicitly
   - draft Mermaid yourself when the diagram can be expressed clearly as a flowchart, sequence diagram, architecture diagram, or similar structured diagram
2. **Fallback to `excalidraw_local_create_view`** only when Mermaid is a poor fit
   - the layout needs very explicit manual placement
   - the styling needs custom Excalidraw-specific control
   - the diagram is more illustration-like than Mermaid-like
   - you are iterating from an existing checkpoint via `restoreCheckpoint`

## Choose the right creation tool

- Use **`excalidraw_local_create_from_mermaid`** when:
  - the user already provides Mermaid
  - you want a fast flowchart / sequence / architecture diagram from Mermaid text
  - you want Excalidraw rendering without hand-authoring JSON elements
  - the user asks for a normal diagram and Mermaid can express it adequately
- Use **`excalidraw_local_create_view`** when:
  - you are composing raw Excalidraw element JSON yourself
  - you need very explicit element placement or custom styling
  - the diagram is better as a hand-built canvas than as Mermaid
  - you are iterating from an existing checkpoint via `restoreCheckpoint`

## Standard workflow

1. Start Mermaid-first:
   - if the user already gave Mermaid → use `excalidraw_local_create_from_mermaid`
   - if the user asked for a diagram in plain English → first draft a Mermaid version and use `excalidraw_local_create_from_mermaid`
   - only switch to `excalidraw_local_create_view` if Mermaid is clearly the wrong tool
2. Copy/use `Checkpoint` and `Preview (canonical)` from tool output.
3. For subsequent edits, use `restoreCheckpoint` with the checkpoint ID.
4. If needed later, regenerate/open preview with `excalidraw_local_preview_checkpoint`.

## Mermaid multiline labels

For the current Mermaid → Excalidraw conversion path, multiline labels should use **`#10;`** inside the Mermaid label text.

- ✅ Use: `A["Hello#10;World"]`
- ❌ Do not rely on: `A["Hello\nWorld"]`
- ❌ Do not rely on: `A["Hello<br/>World"]`
- ❌ Do not rely on: `A["Hello&#10;World"]`

If you are sending Mermaid through a JSON/tool string, keep using `#10;` literally:

```json
{
  "mermaid": "flowchart TD\nA[\"Hello#10;World\"] --> B[\"Second#10;Line\"]"
}
```

Treat this as a tool-specific gotcha for `excalidraw_local_create_from_mermaid`.

## Preview controls

The generated preview page includes built-in viewing controls:

- `Fit` — scale the whole diagram to the viewport
- `+` / `-` — zoom in or out
- `100%` — reset zoom
- `Ctrl/Cmd + wheel` — zoom with mouse/trackpad
- `Drag` while zoomed in — pan around the diagram
- `Shift + wheel` while zoomed in — horizontal pan fallback on mice/trackpads that do not emit horizontal scroll deltas cleanly
- `f` — fit to viewport
- `Ctrl/Cmd + 0` — reset zoom

If a diagram looks too large, prefer `Fit` first before regenerating it.

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

### Diagram still clipped or oversized
1. Try `Fit` in the preview first.
2. If using raw elements, increase final `cameraUpdate` size (e.g., `1200x900`) and regenerate.
3. If using Mermaid, rerender with a smaller `fontSize`.
