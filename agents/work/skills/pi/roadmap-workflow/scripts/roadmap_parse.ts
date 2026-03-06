#!/usr/bin/env bun

import { Effect, pipe } from "effect";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const DATE_RE = /^\d{1,2}\/\d{1,2}\/\d{4}$/;

type RowKind = "goal" | "ongoing" | "structured" | "context";

type TimelineEntry = {
  date: string;
  text: string;
};

type RoadmapItem = {
  source_row: number;
  quarter: string;
  title: string;
  goal: string;
  dependencies: string;
  lead: string;
  timeline: Array<TimelineEntry>;
  timeline_count: number;
  is_goal_row: boolean;
  row_kind: RowKind;
};

type ParsedRoadmap = {
  generated_at: string;
  csv_path: string;
  item_count: number;
  date_columns: Array<string>;
  quarter_counts: Record<string, number>;
  items: Array<RoadmapItem>;
};

type SummaryResult = {
  ok: true;
  action: "summary";
  generated_at: string;
  csv_path: string;
  item_count: number;
  date_columns: Array<string>;
  quarter_counts: Record<string, number>;
  top_timeline_items: Array<{
    quarter: string;
    title: string;
    lead: string;
    timeline_count: number;
  }>;
};

type TimelineResult = {
  ok: true;
  action: "timeline";
  keyword: string;
  match_count: number;
  matches: Array<{
    score: number;
    quarter: string;
    title: string;
    goal: string;
    lead: string;
    timeline: Array<TimelineEntry>;
  }>;
};

type SearchResult = {
  ok: true;
  action: "search";
  keyword: string;
  match_count: number;
  matches: Array<{
    score: number;
    quarter: string;
    title: string;
    goal: string;
    lead: string;
    timeline_preview: Array<TimelineEntry>;
  }>;
};

type SprintResult =
  | {
      ok: false;
      action: "sprint";
      error: string;
    }
  | {
      ok: true;
      action: "sprint";
      date: string;
      match_count: number;
      matches: Array<{
        quarter: string;
        title: string;
        goal: string;
        lead: string;
        date: string;
        text: string;
      }>;
      available_dates: Array<string>;
    };

class CliError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "CliError";
  }
}

const normalizeText = (value: string | undefined): string => {
  if (!value) return "";
  return value
    .split(/\s+/)
    .filter((part) => part.length > 0)
    .join(" ");
};

const parseDateKey = (dateValue: string): [number, number, number] => {
  const [month, day, year] = dateValue.split("/").map((value) => Number.parseInt(value, 10));
  return [year, month, day];
};

const compareDateStrings = (left: string, right: string): number => {
  const [leftY, leftM, leftD] = parseDateKey(left);
  const [rightY, rightM, rightD] = parseDateKey(right);

  if (leftY !== rightY) return leftY - rightY;
  if (leftM !== rightM) return leftM - rightM;
  return leftD - rightD;
};

const parseCsvRows = (rawContent: string): Array<Array<string>> => {
  const content = rawContent.startsWith("\uFEFF") ? rawContent.slice(1) : rawContent;
  if (content.length === 0) return [];

  const rows: Array<Array<string>> = [];
  let row: Array<string> = [];
  let field = "";
  let inQuotes = false;

  const pushField = () => {
    row.push(field);
    field = "";
  };

  const pushRow = () => {
    rows.push(row);
    row = [];
  };

  for (let index = 0; index < content.length; index++) {
    const char = content[index];

    if (inQuotes) {
      if (char === '"') {
        const next = content[index + 1];
        if (next === '"') {
          field += '"';
          index += 1;
        } else {
          inQuotes = false;
        }
      } else {
        field += char;
      }
      continue;
    }

    if (char === '"') {
      inQuotes = true;
      continue;
    }

    if (char === ",") {
      pushField();
      continue;
    }

    if (char === "\n") {
      pushField();
      pushRow();
      continue;
    }

    if (char === "\r") {
      pushField();
      pushRow();
      if (content[index + 1] === "\n") {
        index += 1;
      }
      continue;
    }

    field += char;
  }

  pushField();

  const endsWithNewline = content.endsWith("\n") || content.endsWith("\r");
  const isTrailingEmptyRow = row.length === 1 && row[0] === "";
  if (!(endsWithNewline && isTrailingEmptyRow)) {
    rows.push(row);
  }

  return rows;
};

const detectHeaderAndStart = (
  rows: Array<Array<string>>,
): { header: Array<string>; startIndex: number } => {
  for (let index = 0; index < rows.length; index++) {
    const row = rows[index] ?? [];
    if (row.length > 6 && normalizeText(row[0]).toLowerCase() === "quarters") {
      return { header: row, startIndex: index + 1 };
    }
  }

  for (let index = 0; index < rows.length; index++) {
    const row = rows[index] ?? [];
    let dateCount = 0;

    for (const cell of row) {
      if (DATE_RE.test(normalizeText(cell))) {
        dateCount += 1;
      }
    }

    if (dateCount >= 5) {
      return { header: row, startIndex: index + 1 };
    }
  }

  return { header: rows[0] ?? [], startIndex: 1 };
};

const countBy = (values: Array<string>): Record<string, number> => {
  const counts = new Map<string, number>();

  for (const value of values) {
    if (!value) continue;
    const current = counts.get(value) ?? 0;
    counts.set(value, current + 1);
  }

  return Object.fromEntries(counts.entries());
};

const parseCsvData = (csvPath: string, content: string): ParsedRoadmap => {
  const rows = parseCsvRows(content);
  const { header, startIndex } = detectHeaderAndStart(rows);

  const dateColumns: Array<{ index: number; date: string }> = [];
  for (let index = 0; index < header.length; index++) {
    const dateValue = normalizeText(header[index]);
    if (DATE_RE.test(dateValue)) {
      dateColumns.push({ index, date: dateValue });
    }
  }

  dateColumns.sort((left, right) => compareDateStrings(left.date, right.date));

  const uniqueDates: Array<string> = [];
  const seenDates = new Set<string>();
  for (const { date } of dateColumns) {
    if (seenDates.has(date)) continue;
    seenDates.add(date);
    uniqueDates.push(date);
  }

  const items: Array<RoadmapItem> = [];

  for (let rowIndex = startIndex; rowIndex < rows.length; rowIndex++) {
    const row = rows[rowIndex] ?? [];
    const sourceRow = rowIndex + 1;

    const quarter = normalizeText(row[0]);
    const title = normalizeText(row[1]);
    const goal = normalizeText(row[2]);
    const dependencies = normalizeText(row[3]);
    const lead = normalizeText(row[4]);

    if (!title) continue;

    const timeline: Array<TimelineEntry> = [];
    for (const { index, date } of dateColumns) {
      if (index >= row.length) continue;
      const text = normalizeText(row[index]);
      if (!text) continue;
      timeline.push({ date, text });
    }

    const quarterLower = quarter.toLowerCase();
    let rowKind: RowKind;
    if (quarterLower.includes("goal")) {
      rowKind = "goal";
    } else if (quarterLower === "ongoing") {
      rowKind = "ongoing";
    } else if (quarter) {
      rowKind = "structured";
    } else {
      rowKind = "context";
    }

    items.push({
      source_row: sourceRow,
      quarter,
      title,
      goal,
      dependencies,
      lead,
      timeline,
      timeline_count: timeline.length,
      is_goal_row: quarterLower.includes("goal"),
      row_kind: rowKind,
    });
  }

  return {
    generated_at: new Date().toISOString(),
    csv_path: path.resolve(csvPath),
    item_count: items.length,
    date_columns: uniqueDates,
    quarter_counts: countBy(items.map((item) => item.quarter)),
    items,
  };
};

const scoreItem = (item: RoadmapItem, keywordLower: string): number => {
  let score = 0;

  if (item.title.toLowerCase().includes(keywordLower)) score += 6;
  if (item.goal.toLowerCase().includes(keywordLower)) score += 3;
  if (item.dependencies.toLowerCase().includes(keywordLower)) score += 2;
  if (item.lead.toLowerCase().includes(keywordLower)) score += 1;
  if (item.quarter.toLowerCase().includes(keywordLower)) score += 1;

  for (const timelineEntry of item.timeline) {
    if (timelineEntry.text.toLowerCase().includes(keywordLower)) {
      score += 1;
    }
  }

  return score;
};

const readText = (filePath: string) =>
  Effect.tryPromise({
    try: () => readFile(filePath, "utf8"),
    catch: (error) =>
      new CliError(
        `Failed to read ${filePath}: ${error instanceof Error ? error.message : String(error)}`,
      ),
  });

const writeJsonFile = (filePath: string, value: unknown) =>
  Effect.tryPromise({
    try: async () => {
      const absolutePath = path.resolve(filePath);
      await mkdir(path.dirname(absolutePath), { recursive: true });
      await writeFile(absolutePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
    },
    catch: (error) =>
      new CliError(
        `Failed to write ${filePath}: ${error instanceof Error ? error.message : String(error)}`,
      ),
  });

const loadJson = <T>(filePath: string): Effect.Effect<T, CliError> =>
  pipe(
    readText(filePath),
    Effect.flatMap((raw) =>
      Effect.try({
        try: () => JSON.parse(raw) as T,
        catch: (error) =>
          new CliError(
            `Failed to parse JSON from ${filePath}: ${error instanceof Error ? error.message : String(error)}`,
          ),
      }),
    ),
  );

const parseInteger = (value: string, flagName: string): number => {
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    throw new CliError(`Invalid ${flagName} value: ${value}`);
  }
  return parsed;
};

const ensureStringFlag = (value: string | boolean | undefined, flagName: string): string => {
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new CliError(`Missing ${flagName}`);
  }
  return value;
};

const parseFlags = (tokens: Array<string>) => {
  const flags: Record<string, string | boolean> = {};
  const positional: Array<string> = [];

  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index];

    if (!token.startsWith("--")) {
      positional.push(token);
      continue;
    }

    const equalIndex = token.indexOf("=");
    if (equalIndex > -1) {
      const key = token.slice(2, equalIndex);
      const value = token.slice(equalIndex + 1);
      flags[key] = value;
      continue;
    }

    const key = token.slice(2);
    const next = tokens[index + 1];
    if (next && !next.startsWith("--")) {
      flags[key] = next;
      index += 1;
    } else {
      flags[key] = true;
    }
  }

  return { flags, positional };
};

const ensureAllowedFlags = (
  flags: Record<string, string | boolean>,
  allowed: ReadonlySet<string>,
): void => {
  for (const key of Object.keys(flags)) {
    if (!allowed.has(key)) {
      throw new CliError(`Unknown option --${key}`);
    }
  }
};

type Command =
  | { tag: "parse"; csv: string; out?: string }
  | { tag: "summary"; json: string; quarter?: string; limit: number; includeContext: boolean }
  | { tag: "timeline"; json: string; keyword: string; limit: number; includeContext: boolean }
  | { tag: "search"; json: string; keyword: string; limit: number; includeContext: boolean }
  | { tag: "sprint"; json: string; date: string; limit: number; includeContext: boolean };

const parseCommand = (argv: Array<string>): Command => {
  if (argv.length === 0) {
    throw new CliError("Missing command. Expected one of: parse, summary, timeline, search, sprint.");
  }

  const [commandName, ...rest] = argv;
  const { flags, positional } = parseFlags(rest);

  if (positional.length > 0) {
    throw new CliError(`Unexpected positional arguments: ${positional.join(" ")}`);
  }

  if (commandName === "parse") {
    ensureAllowedFlags(flags, new Set(["csv", "out"]));
    const csv = ensureStringFlag(flags.csv, "--csv");
    const out = typeof flags.out === "string" ? flags.out : undefined;
    return { tag: "parse", csv, out };
  }

  if (commandName === "summary") {
    ensureAllowedFlags(flags, new Set(["json", "quarter", "limit", "include-context"]));
    const json = ensureStringFlag(flags.json, "--json");
    const quarter = typeof flags.quarter === "string" ? flags.quarter : undefined;
    const limit =
      typeof flags.limit === "string"
        ? parseInteger(flags.limit, "--limit")
        : flags.limit === true
          ? (() => {
              throw new CliError("Missing --limit value");
            })()
          : 10;
    const includeContext = "include-context" in flags;
    return { tag: "summary", json, quarter, limit, includeContext };
  }

  if (commandName === "timeline") {
    ensureAllowedFlags(flags, new Set(["json", "keyword", "limit", "include-context"]));
    const json = ensureStringFlag(flags.json, "--json");
    const keyword = ensureStringFlag(flags.keyword, "--keyword").trim();
    const limit =
      typeof flags.limit === "string"
        ? parseInteger(flags.limit, "--limit")
        : flags.limit === true
          ? (() => {
              throw new CliError("Missing --limit value");
            })()
          : 10;
    const includeContext = "include-context" in flags;
    return { tag: "timeline", json, keyword, limit, includeContext };
  }

  if (commandName === "search") {
    ensureAllowedFlags(flags, new Set(["json", "keyword", "limit", "include-context"]));
    const json = ensureStringFlag(flags.json, "--json");
    const keyword = ensureStringFlag(flags.keyword, "--keyword").trim();
    const limit =
      typeof flags.limit === "string"
        ? parseInteger(flags.limit, "--limit")
        : flags.limit === true
          ? (() => {
              throw new CliError("Missing --limit value");
            })()
          : 20;
    const includeContext = "include-context" in flags;
    return { tag: "search", json, keyword, limit, includeContext };
  }

  if (commandName === "sprint") {
    ensureAllowedFlags(flags, new Set(["json", "date", "limit", "include-context"]));
    const json = ensureStringFlag(flags.json, "--json");
    const date = ensureStringFlag(flags.date, "--date").trim();
    const limit =
      typeof flags.limit === "string"
        ? parseInteger(flags.limit, "--limit")
        : flags.limit === true
          ? (() => {
              throw new CliError("Missing --limit value");
            })()
          : 50;
    const includeContext = "include-context" in flags;
    return { tag: "sprint", json, date, limit, includeContext };
  }

  throw new CliError(`Unknown command: ${commandName}`);
};

const runParse = (command: Extract<Command, { tag: "parse" }>) =>
  pipe(
    readText(command.csv),
    Effect.map((content) => parseCsvData(command.csv, content)),
    Effect.tap((parsed) => (command.out ? writeJsonFile(command.out, parsed) : Effect.void)),
    Effect.map((parsed) => ({
      ok: true,
      action: "parse",
      csv_path: parsed.csv_path,
      item_count: parsed.item_count,
      date_columns: parsed.date_columns,
      quarter_counts: parsed.quarter_counts,
      out: command.out ? path.resolve(command.out) : null,
    })),
  );

const runSummary = (command: Extract<Command, { tag: "summary" }>): Effect.Effect<SummaryResult, CliError> =>
  pipe(
    loadJson<ParsedRoadmap>(command.json),
    Effect.map((data) => {
      let items = data.items;

      if (command.quarter) {
        const quarterLower = command.quarter.toLowerCase();
        items = items.filter((item) => item.quarter.toLowerCase().includes(quarterLower));
      }

      const timelineItems = items.filter(
        (item) => item.timeline_count > 0 && (command.includeContext || item.row_kind !== "context"),
      );

      const topTimelineItems = [...timelineItems]
        .sort((left, right) => right.timeline_count - left.timeline_count)
        .slice(0, command.limit);

      return {
        ok: true,
        action: "summary",
        generated_at: data.generated_at,
        csv_path: data.csv_path,
        item_count: items.length,
        date_columns: data.date_columns,
        quarter_counts: countBy(items.map((item) => item.quarter)),
        top_timeline_items: topTimelineItems.map((item) => ({
          quarter: item.quarter,
          title: item.title,
          lead: item.lead,
          timeline_count: item.timeline_count,
        })),
      } satisfies SummaryResult;
    }),
  );

const runTimeline = (command: Extract<Command, { tag: "timeline" }>): Effect.Effect<TimelineResult, CliError> =>
  pipe(
    loadJson<ParsedRoadmap>(command.json),
    Effect.map((data) => {
      const keywordLower = command.keyword.toLowerCase();

      const scored = data.items
        .filter((item) => command.includeContext || item.row_kind !== "context")
        .map((item) => ({ score: scoreItem(item, keywordLower), item }))
        .filter(({ score }) => score > 0)
        .sort((left, right) => {
          if (left.score !== right.score) return right.score - left.score;
          return left.item.title.localeCompare(right.item.title);
        });

      return {
        ok: true,
        action: "timeline",
        keyword: command.keyword,
        match_count: scored.length,
        matches: scored.slice(0, command.limit).map(({ score, item }) => ({
          score,
          quarter: item.quarter,
          title: item.title,
          goal: item.goal,
          lead: item.lead,
          timeline: item.timeline,
        })),
      } satisfies TimelineResult;
    }),
  );

const runSearch = (command: Extract<Command, { tag: "search" }>): Effect.Effect<SearchResult, CliError> =>
  pipe(
    loadJson<ParsedRoadmap>(command.json),
    Effect.map((data) => {
      const keywordLower = command.keyword.toLowerCase();

      const scored = data.items
        .filter((item) => command.includeContext || item.row_kind !== "context")
        .map((item) => ({ score: scoreItem(item, keywordLower), item }))
        .filter(({ score }) => score > 0)
        .sort((left, right) => {
          if (left.score !== right.score) return right.score - left.score;
          return left.item.title.localeCompare(right.item.title);
        });

      return {
        ok: true,
        action: "search",
        keyword: command.keyword,
        match_count: scored.length,
        matches: scored.slice(0, command.limit).map(({ score, item }) => {
          const timelinePreview = item.timeline.filter((entry) =>
            entry.text.toLowerCase().includes(keywordLower),
          );

          return {
            score,
            quarter: item.quarter,
            title: item.title,
            goal: item.goal,
            lead: item.lead,
            timeline_preview: timelinePreview.length > 0 ? timelinePreview : item.timeline.slice(0, 2),
          };
        }),
      } satisfies SearchResult;
    }),
  );

const runSprint = (command: Extract<Command, { tag: "sprint" }>): Effect.Effect<SprintResult, CliError> =>
  pipe(
    loadJson<ParsedRoadmap>(command.json),
    Effect.map((data) => {
      const target = normalizeText(command.date);
      if (!DATE_RE.test(target)) {
        return {
          ok: false,
          action: "sprint",
          error: "Invalid date format. Use m/d/yyyy (e.g. 2/17/2026).",
        };
      }

      const matches = data.items
        .filter((item) => command.includeContext || item.row_kind !== "context")
        .flatMap((item) =>
          item.timeline
            .filter((entry) => entry.date === target)
            .map((entry) => ({
              quarter: item.quarter,
              title: item.title,
              goal: item.goal,
              lead: item.lead,
              date: target,
              text: entry.text,
            })),
        )
        .slice(0, command.limit);

      return {
        ok: true,
        action: "sprint",
        date: target,
        match_count: matches.length,
        matches,
        available_dates: data.date_columns,
      };
    }),
  );

const runCommand = (command: Command): Effect.Effect<unknown, CliError> => {
  switch (command.tag) {
    case "parse":
      return runParse(command);
    case "summary":
      return runSummary(command);
    case "timeline":
      return runTimeline(command);
    case "search":
      return runSearch(command);
    case "sprint":
      return runSprint(command);
    default: {
      const _exhaustive: never = command;
      return Effect.fail(new CliError(`Unhandled command: ${String(_exhaustive)}`));
    }
  }
};

const program = pipe(
  Effect.sync(() => parseCommand(process.argv.slice(2))),
  Effect.flatMap(runCommand),
  Effect.tap((payload) => Effect.sync(() => console.log(JSON.stringify(payload)))),
);

await Effect.runPromise(program).catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.log(JSON.stringify({ ok: false, error: message }));
  process.exit(1);
});
