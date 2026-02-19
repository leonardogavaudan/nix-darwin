---
name: roadmap-workflow
description: Fetch and analyze the Optimization roadmap from Google Sheets exports. Use when the user asks about roadmap status, sprint-by-sprint plans, initiative/epoch timelines, quarterly priorities, or "what is planned when".
---

# Roadmap Workflow

Use this skill whenever roadmap questions come up.

## Primary Workflow (extension-first)

1. Check cache freshness with `roadmap_status` (optional).
2. Refresh the latest roadmap snapshot with `roadmap_refresh`.
3. Query the parsed roadmap with `roadmap_query`:
   - `mode: "summary"` for high-level overview
   - `mode: "timeline"` + `keyword` for initiative/epoch timeline
   - `mode: "sprint"` + `sprintDate` for a specific sprint date
   - `mode: "search"` + `keyword` for broad matching
4. Present concise results, then propose follow-up drill-downs.

## Query Mapping

- "What is planned this sprint?" → `mode: "sprint"`
- "Show timeline for rich reporting" → `mode: "timeline"`, `keyword: "rich reporting"`
- "What’s in Q1 vs Q2?" → `mode: "summary"` (optionally filter by quarter)
- "Find everything about offline eval" → `mode: "search"`, `keyword: "offline eval"`

## Terminology

- User may say "epoch" but roadmap data is often initiative/epic rows by title.
- Sprint dates are in `m/d/yyyy` format in the roadmap.

## Date Semantics (must be explicit)

When reporting roadmap dates, always disambiguate whether it is an exact date or a sprint marker.

- Roadmap date columns are usually **sprint start markers**.
- If using a roadmap date as a sprint marker, always provide the full sprint window:
  - **Start** = the roadmap date column
  - **End** = next roadmap date minus 1 day (fallback: start + 13 days if no next date exists)
- If you mean a literal day milestone, label it explicitly as **Exact date**.
- Never use ambiguous phrasing like `~3/3` without clarifying the interpretation.

Example:
- `3/3/2026` as sprint marker → **Sprint window: 3/3/2026–3/16/2026** (next marker is `3/17/2026`)
- `3/3/2026` as literal milestone → **Exact date: 3/3/2026**

## Quarter Semantics (must be explicit)

When reporting quarters (Q1/Q2/Q3/Q4), always include the fiscal year label.

- Use format: **FYxx Qy** (example: `FY27 Q1`), not just `Q1`.
- Infer fiscal year from roadmap context (sheet title/date range/milestones) and state assumptions when needed.
- If fiscal year cannot be inferred confidently, explicitly say it is ambiguous and provide the likely options.

Example:
- `Q1 Goal` with milestones around Jan–Mar 2026 in a `FY26-27` roadmap should be reported as **FY27 Q1**.

## Output Style

Default output should include:
- Snapshot freshness (when it was refreshed)
- Source sheet URL or ID
- Requested view (sprint/timeline/search/summary)
- **Date semantics** for every referenced date (Exact date vs Sprint start marker)
- If sprint marker: explicit **start and end dates**
- **Quarter semantics** for every referenced quarter (include fiscal year, or state ambiguity)
- Bullet list of key items

## Fallback (if extension is unavailable)

Use browser export + parser script manually:

1. Export CSV from Google Sheets (active roadmap tab)
2. Parse with:

```bash
python3 ~/.pi/agent/skills/roadmap-workflow/scripts/roadmap_parse.py parse --csv /path/to/roadmap.csv --out ~/.cache/roadmap-workflow/roadmap-latest.json
```

3. Query parsed JSON with:

```bash
python3 ~/.pi/agent/skills/roadmap-workflow/scripts/roadmap_parse.py summary --json ~/.cache/roadmap-workflow/roadmap-latest.json
python3 ~/.pi/agent/skills/roadmap-workflow/scripts/roadmap_parse.py timeline --json ~/.cache/roadmap-workflow/roadmap-latest.json --keyword "rich reporting"
python3 ~/.pi/agent/skills/roadmap-workflow/scripts/roadmap_parse.py sprint --json ~/.cache/roadmap-workflow/roadmap-latest.json --date "2/17/2026"
```