#!/usr/bin/env python3

import argparse
import csv
import json
import os
import re
import sys
from collections import Counter
from datetime import datetime, timezone

DATE_RE = re.compile(r"^\d{1,2}/\d{1,2}/\d{4}$")


def normalize_text(value: str) -> str:
    if not value:
        return ""
    return " ".join(value.split())


def parse_date_key(date_value: str):
    m, d, y = [int(x) for x in date_value.split("/")]
    return (y, m, d)


def detect_header_and_start(rows):
    for i, row in enumerate(rows):
        if len(row) > 6 and normalize_text(row[0]).lower() == "quarters":
            return row, i + 1

    for i, row in enumerate(rows):
        date_count = 0
        for cell in row:
            cell = normalize_text(cell)
            if DATE_RE.match(cell):
                date_count += 1
        if date_count >= 5:
            return row, i + 1

    return rows[0] if rows else [], 1


def parse_csv(csv_path: str):
    with open(csv_path, newline="", encoding="utf-8-sig") as f:
        rows = list(csv.reader(f))

    header, start_index = detect_header_and_start(rows)

    date_columns = []
    for idx, cell in enumerate(header):
        date_value = normalize_text(cell)
        if DATE_RE.match(date_value):
            date_columns.append((idx, date_value))

    date_columns.sort(key=lambda pair: parse_date_key(pair[1]))

    unique_dates = []
    seen_dates = set()
    for _, date_value in date_columns:
        if date_value in seen_dates:
            continue
        seen_dates.add(date_value)
        unique_dates.append(date_value)

    items = []
    for raw_idx, row in enumerate(rows[start_index:], start=start_index + 1):
        quarter = normalize_text(row[0]) if len(row) > 0 else ""
        title = normalize_text(row[1]) if len(row) > 1 else ""
        goal = normalize_text(row[2]) if len(row) > 2 else ""
        dependencies = normalize_text(row[3]) if len(row) > 3 else ""
        lead = normalize_text(row[4]) if len(row) > 4 else ""

        if not title:
            continue

        timeline = []
        for idx, date_value in date_columns:
            if idx >= len(row):
                continue
            cell = normalize_text(row[idx])
            if not cell:
                continue
            timeline.append({"date": date_value, "text": cell})

        quarter_lower = quarter.lower()
        if "goal" in quarter_lower:
            row_kind = "goal"
        elif quarter_lower == "ongoing":
            row_kind = "ongoing"
        elif quarter:
            row_kind = "structured"
        else:
            row_kind = "context"

        item = {
            "source_row": raw_idx,
            "quarter": quarter,
            "title": title,
            "goal": goal,
            "dependencies": dependencies,
            "lead": lead,
            "timeline": timeline,
            "timeline_count": len(timeline),
            "is_goal_row": "goal" in quarter_lower,
            "row_kind": row_kind,
        }
        items.append(item)

    quarter_counts = Counter(item["quarter"] for item in items if item["quarter"])

    return {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "csv_path": os.path.abspath(csv_path),
        "item_count": len(items),
        "date_columns": unique_dates,
        "quarter_counts": dict(quarter_counts),
        "items": items,
    }


def load_json(path):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def score_item(item, keyword_lower: str):
    score = 0
    if keyword_lower in item.get("title", "").lower():
        score += 6
    if keyword_lower in item.get("goal", "").lower():
        score += 3
    if keyword_lower in item.get("dependencies", "").lower():
        score += 2
    if keyword_lower in item.get("lead", "").lower():
        score += 1
    if keyword_lower in item.get("quarter", "").lower():
        score += 1

    for entry in item.get("timeline", []):
        if keyword_lower in entry.get("text", "").lower():
            score += 1

    return score


def cmd_parse(args):
    data = parse_csv(args.csv)
    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
    print(json.dumps({
        "ok": True,
        "action": "parse",
        "csv_path": data["csv_path"],
        "item_count": data["item_count"],
        "date_columns": data["date_columns"],
        "quarter_counts": data["quarter_counts"],
        "out": os.path.abspath(args.out) if args.out else None,
    }, ensure_ascii=False))


def cmd_summary(args):
    data = load_json(args.json)
    items = data.get("items", [])

    if args.quarter:
        quarter_lower = args.quarter.lower()
        items = [it for it in items if quarter_lower in it.get("quarter", "").lower()]

    timeline_items = [
        it
        for it in items
        if it.get("timeline_count", 0) > 0 and (args.include_context or it.get("row_kind") != "context")
    ]
    top_timeline = sorted(timeline_items, key=lambda it: it.get("timeline_count", 0), reverse=True)[: args.limit]

    quarter_counts = Counter(it.get("quarter", "") for it in items if it.get("quarter"))

    print(json.dumps({
        "ok": True,
        "action": "summary",
        "generated_at": data.get("generated_at"),
        "csv_path": data.get("csv_path"),
        "item_count": len(items),
        "date_columns": data.get("date_columns", []),
        "quarter_counts": dict(quarter_counts),
        "top_timeline_items": [
            {
                "quarter": it.get("quarter"),
                "title": it.get("title"),
                "lead": it.get("lead"),
                "timeline_count": it.get("timeline_count", 0),
            }
            for it in top_timeline
        ],
    }, ensure_ascii=False))


def cmd_timeline(args):
    data = load_json(args.json)
    keyword = args.keyword.strip()
    keyword_lower = keyword.lower()

    scored = []
    for item in data.get("items", []):
        if not args.include_context and item.get("row_kind") == "context":
            continue
        score = score_item(item, keyword_lower)
        if score > 0:
            scored.append((score, item))

    scored.sort(key=lambda pair: (-pair[0], pair[1].get("title", "")))
    matches = [
        {
            "score": score,
            "quarter": item.get("quarter"),
            "title": item.get("title"),
            "goal": item.get("goal"),
            "lead": item.get("lead"),
            "timeline": item.get("timeline", []),
        }
        for score, item in scored[: args.limit]
    ]

    print(json.dumps({
        "ok": True,
        "action": "timeline",
        "keyword": keyword,
        "match_count": len(scored),
        "matches": matches,
    }, ensure_ascii=False))


def cmd_search(args):
    data = load_json(args.json)
    keyword = args.keyword.strip()
    keyword_lower = keyword.lower()

    scored = []
    for item in data.get("items", []):
        if not args.include_context and item.get("row_kind") == "context":
            continue
        score = score_item(item, keyword_lower)
        if score > 0:
            scored.append((score, item))

    scored.sort(key=lambda pair: (-pair[0], pair[1].get("title", "")))

    matches = []
    for score, item in scored[: args.limit]:
        preview = []
        for entry in item.get("timeline", []):
            if keyword_lower in entry.get("text", "").lower():
                preview.append(entry)
        if not preview:
            preview = item.get("timeline", [])[:2]

        matches.append({
            "score": score,
            "quarter": item.get("quarter"),
            "title": item.get("title"),
            "goal": item.get("goal"),
            "lead": item.get("lead"),
            "timeline_preview": preview,
        })

    print(json.dumps({
        "ok": True,
        "action": "search",
        "keyword": keyword,
        "match_count": len(scored),
        "matches": matches,
    }, ensure_ascii=False))


def cmd_sprint(args):
    data = load_json(args.json)
    target = normalize_text(args.date)
    if not DATE_RE.match(target):
        print(json.dumps({
            "ok": False,
            "action": "sprint",
            "error": "Invalid date format. Use m/d/yyyy (e.g. 2/17/2026).",
        }, ensure_ascii=False))
        return

    matches = []
    for item in data.get("items", []):
        if not args.include_context and item.get("row_kind") == "context":
            continue
        for entry in item.get("timeline", []):
            if entry.get("date") == target:
                matches.append({
                    "quarter": item.get("quarter"),
                    "title": item.get("title"),
                    "goal": item.get("goal"),
                    "lead": item.get("lead"),
                    "date": target,
                    "text": entry.get("text"),
                })

    matches = matches[: args.limit]

    print(json.dumps({
        "ok": True,
        "action": "sprint",
        "date": target,
        "match_count": len(matches),
        "matches": matches,
        "available_dates": data.get("date_columns", []),
    }, ensure_ascii=False))


def build_parser():
    parser = argparse.ArgumentParser(description="Parse and query roadmap CSV exports")
    sub = parser.add_subparsers(dest="command", required=True)

    parse_p = sub.add_parser("parse", help="Parse roadmap CSV and optionally write JSON")
    parse_p.add_argument("--csv", required=True, help="Path to roadmap CSV file")
    parse_p.add_argument("--out", required=False, help="Output JSON path")
    parse_p.set_defaults(func=cmd_parse)

    summary_p = sub.add_parser("summary", help="Summarize parsed roadmap JSON")
    summary_p.add_argument("--json", required=True, help="Parsed roadmap JSON path")
    summary_p.add_argument("--quarter", required=False, help="Quarter filter (substring)")
    summary_p.add_argument("--limit", type=int, default=10, help="Max top timeline items")
    summary_p.add_argument("--include-context", action="store_true", help="Include non-goal context rows")
    summary_p.set_defaults(func=cmd_summary)

    timeline_p = sub.add_parser("timeline", help="Get timeline for initiative keyword")
    timeline_p.add_argument("--json", required=True, help="Parsed roadmap JSON path")
    timeline_p.add_argument("--keyword", required=True, help="Keyword to match")
    timeline_p.add_argument("--limit", type=int, default=10, help="Max matches")
    timeline_p.add_argument("--include-context", action="store_true", help="Include non-goal context rows")
    timeline_p.set_defaults(func=cmd_timeline)

    search_p = sub.add_parser("search", help="Search roadmap rows by keyword")
    search_p.add_argument("--json", required=True, help="Parsed roadmap JSON path")
    search_p.add_argument("--keyword", required=True, help="Keyword to match")
    search_p.add_argument("--limit", type=int, default=20, help="Max matches")
    search_p.add_argument("--include-context", action="store_true", help="Include non-goal context rows")
    search_p.set_defaults(func=cmd_search)

    sprint_p = sub.add_parser("sprint", help="Get items planned for a sprint date")
    sprint_p.add_argument("--json", required=True, help="Parsed roadmap JSON path")
    sprint_p.add_argument("--date", required=True, help="Sprint date in m/d/yyyy")
    sprint_p.add_argument("--limit", type=int, default=50, help="Max matches")
    sprint_p.add_argument("--include-context", action="store_true", help="Include non-goal context rows")
    sprint_p.set_defaults(func=cmd_sprint)

    return parser


def main():
    parser = build_parser()
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, ensure_ascii=False))
        sys.exit(1)
