---
name: hex-notebook
description: Interact with Hex notebooks (app.hex.tech) via Chrome browser automation. Use when navigating, reading, editing, or analyzing Hex notebook cells, SQL queries, or results. Triggers on tasks involving Hex URLs, Hex notebooks, Databricks SQL in Hex, or when the user asks to open/read/modify a Hex notebook.
---

# Hex Notebook Automation

Hex notebooks use a React app with a Redux store and Monaco editors. The DOM uses virtual scrolling (only cells in the viewport are rendered), so direct DOM scraping is unreliable. **Always prefer the Redux store approach.**

## Prerequisites

Load Chrome tools before use:
```
ToolSearch: "chrome tabs context navigate"
```

Then get tab context, create a new tab, and navigate:
```
mcp__claude-in-chrome__tabs_context_mcp (createIfEmpty: true)
mcp__claude-in-chrome__tabs_create_mcp
mcp__claude-in-chrome__navigate (url: "https://app.hex.tech/...", tabId: <id>)
```

## Reading All Cell Metadata (Recommended)

Hex's Redux store contains all cell data regardless of scroll position. Access it via `javascript_tool`:

### Step 1: Get the version ID

```javascript
const state = window.__HEX_AO_CONTROLLER__.dataStore.getState();
const versionId = Object.keys(state.hexVersionMP)[0];
versionId;
```

### Step 2: List all cells with types and output variable names

```javascript
const state = window.__HEX_AO_CONTROLLER__.dataStore.getState();
const versionId = Object.keys(state.hexVersionMP)[0];
const hv = state.hexVersionMP[versionId];
const cells = hv.cells.entities;
const cellIds = hv.cells.ids;
const contents = hv.cellContents.entities;
const contentsIds = hv.cellContents.ids;
const cellContentMap = {};
contentsIds.forEach(cid => { cellContentMap[contents[cid].cellId] = contents[cid]; });

cellIds.map((id, i) => {
  const c = cells[id];
  const content = cellContentMap[id];
  return {
    idx: i,
    type: c.cellType,
    resultVar: content?.resultVariable || '',
    source: (content?.source || '').substring(0, 120)
  };
});
```

Cell types: `SQL`, `PYTHON`, `TEXT`, `EXPLORE`, `CHART`, `PIVOT`, `TABLE`, `INPUT`

### Step 3: Read full SQL/Python source for a specific cell

```javascript
const state = window.__HEX_AO_CONTROLLER__.dataStore.getState();
const versionId = Object.keys(state.hexVersionMP)[0];
const hv = state.hexVersionMP[versionId];
const cellIds = hv.cells.ids;
const contents = hv.cellContents.entities;
const contentsIds = hv.cellContents.ids;
const cellContentMap = {};
contentsIds.forEach(cid => { cellContentMap[contents[cid].cellId] = contents[cid]; });
// Change index to target cell
cellContentMap[cellIds[0]].source;
```

For multiple cells (watch for truncation — read 3-4 at a time):

```javascript
[0, 1, 2, 3].map(i => {
  const content = cellContentMap[cellIds[i]];
  return `=== Cell ${i}: ${content.resultVariable} ===\n${content.source}`;
}).join('\n\n');
```

## Important: Output Truncation

The `javascript_tool` truncates responses. When reading cell source code:

- Read cells **individually** if SQL is long (>40 lines)
- Or read in small batches (3-4 cells) with truncated previews
- For very long cells, read in slices: `content.source.substring(0, 2000)` then `.substring(2000, 4000)`

## Reading Rendered Editors (Fallback)

If Redux is unavailable, extract from rendered Monaco editors:

```javascript
const editors = document.querySelectorAll('.monaco-editor');
editors.forEach((ed, i) => {
  const lines = ed.querySelectorAll('.view-line');
  const text = Array.from(lines).map(l => l.textContent).join('\n');
  // text contains the visible code
});
```

**Caveat**: Only editors scrolled into view are rendered. The scrollable container is:
```javascript
document.querySelector('[class*="LogicViewContents__OLContainer"]')
```
Scroll it to force rendering: `container.scrollTo(0, container.scrollHeight)`

## Redux Store Reference

Key paths in `window.__HEX_AO_CONTROLLER__.dataStore.getState()`:

| Path | Contains |
|------|----------|
| `hexVersionMP[versionId].cells` | Cell metadata (id, type, label) — normalized `{ids, entities}` |
| `hexVersionMP[versionId].cellContents` | Cell source code, connection, output variable — normalized `{ids, entities}` |
| `hexVersionMP[versionId].hexVersion` | Notebook metadata |
| `hexVersionMP[versionId].dataConnectionHexVersionLinks` | Database connections |
| `outputContent` | Cached cell output/results |
| `logicView` | UI state (selected cell, scroll position) |

### Cell entity fields

```
cellId, cellType, cellLabel
```

### CellContent entity fields (SQL cells)

```
cellId, source, resultVariable, connectionId,
sqlCellOutputType, castDecimals, useNativeDates,
loadIntoDataFrame, dataFrameCell, cellReferencesV2
```

## Navigating the Notebook UI

### Accessibility tree approach

Use `read_page` with specific `ref_id` to inspect cell structure:
```
mcp__claude-in-chrome__read_page (tabId, depth: 5, max_chars: 10000)
```

Cell titles appear as `generic "Cell Title Name"` nodes in the list structure.

### Hex URL patterns

- Logic view: `app.hex.tech/{org}/hex/{name}-{id}/draft/logic`
- App view: `app.hex.tech/{org}/hex/{name}-{id}/draft/app`
- Published: `app.hex.tech/{org}/app/{name}-{id}/latest`

## Running Cells

The notebook must be in a running state. Check status in the bottom bar (look for "Stopped" vs "Running"). To run:
1. Find the "Run all" button via `read_page` with `filter: "interactive"`
2. Click it, or click individual cell run buttons

## Tips

- **Always start with Redux** — it has all data, no scrolling needed
- **`window.monaco`** is NOT accessible (bundled privately) — don't try `monaco.editor.getModels()`
- **Virtual scrolling** means only ~4-6 cells are in the DOM at once
- Cell labels are often empty — use `resultVariable` to identify cells
- Notification IDs in SQL (e.g., `YTSGD05V84MHWEHD19AKDC02F346`) map to Courier templates
