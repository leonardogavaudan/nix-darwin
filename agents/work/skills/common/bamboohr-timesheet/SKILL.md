---
name: bamboohr-timesheet
description: Fill BambooHR timesheets quickly using Browser Pi DOM automation. Use when adding, editing, or backfilling BambooHR time entries at Algolia, including bulk weekday filling and lunch-split schedules.
---

# BambooHR Timesheet (Browser Pi)

Use Browser Pi + DOM selectors for all BambooHR timesheet work.

## Core Rules

1. **Use Browser Pi tab targeting** (`--tab` or `--url-match`) for every action.
2. **Do not use coordinate clicks**.
3. **Do not use ref_id/read_page workflows** (deprecated for this task).
4. **Verify after each save** (BambooHR UI updates can be delayed by ~2-3s).

## URL

`https://algolia.bamboohr.com/employees/timesheet/?id={employee_id}`

## Required Tab Isolation Workflow

1. List tabs with `browser-tabs.js`.
2. Pick the BambooHR tab index.
3. Use that index for every `browser-eval.js` / `browser-nav.js` call.

## DOM Selectors That Matter

- Day row: `.TimesheetSlat`
- Day of week: `.TimesheetSlat__dayOfWeek`
- Day date: `.TimesheetSlat__dayDate`
- Day total: `.TimesheetSlat__dayTotal`
- Add link: `.TimesheetSlat__addEntryLink`
- Existing entry row: `.TimeEntry`
- Expand row summary: `.TimesheetSlat__firstAndLast`
- Disabled/non-editable day: `.TimesheetSlat--disabled`
- Time inputs in modal: `input.ClockField__formInput`
- AM/PM toggles in modal: `button.fab-SelectToggle`

## Critical UI Behavior (Important)

### 1) Hidden `<select>` is not reliable
BambooHR shows hidden `select.chzn-ignore`, but it is effectively read-only in this flow.  
**Do not set AM/PM by writing `<select>.value`.**

### 2) Set AM/PM using the visible toggle button
For each meridian toggle (`button.fab-SelectToggle`):
- focus toggle
- `Space`
- `ArrowDown`
- `Enter`

This reliably changes `AM -> PM`.

### 3) `Add Entry` vs `Save`
- **Add Entry**: append another interval inside the same day modal
- **Save**: persist all intervals for that day

For split-lunch days, use both (`Add Entry`, then `Save`).

## Work Schedule Defaults

Contract baseline:
- Weekly: **38h30**
- Daily: **7h42**

### Single-block day
- `9:00 AM -> 4:42 PM` (7h42)

### Lunch-split day (recommended)
- `9:00 AM -> 12:00 PM`
- `1:00 PM -> 5:42 PM`
- Total = 7h42

## Identify Fillable Weekdays (Fast)

Use DOM filtering instead of visual scanning:

- include rows with `+ Add Time Entry`
- exclude `.TimesheetSlat--disabled`
- exclude rows that already contain `.TimeEntry`
- exclude Sat/Sun
- exclude rows containing RTT / Vacation / Holiday / Sick / Wellness markers

## Add One Day (Lunch-Split)

1. Click day `.TimesheetSlat__addEntryLink`.
2. In modal first row:
   - start: `9:00` + `AM`
   - end: `12:00` + `PM`
3. Click **Add Entry**.
4. In newly added row:
   - start: `1:00` + `PM`
   - end: `5:42` + `PM`
5. Click **Save**.
6. Wait ~2-3s.
7. Verify day row now has:
   - total `7h 42m`
   - two `.TimeEntry` rows (`9:00 AM-12:00 PM`, `1:00 PM-5:42 PM`)

## Edit Existing Day

Use when fixing wrong AM/PM or converting an existing single entry.

1. Expand day via `.TimesheetSlat__firstAndLast` if needed.
2. Click target `.TimeEntry` to open **Edit Timesheet Entry** modal.
3. Update input values.
4. Set meridians with toggle-keyboard sequence (`Space`, `ArrowDown`, `Enter`).
5. Click **Save**.
6. Wait ~2-3s and re-verify row values.

## Bulk Fill Strategy

1. Compute all fillable weekdays first.
2. Loop day-by-day (deterministic order).
3. For each day, perform lunch-split add flow above.
4. After each save, verify that day before moving on.
5. Final verification:
   - Pay period total
   - No remaining fillable weekdays

## Days to Skip

Always skip:
- Saturdays/Sundays
- Existing time-off rows (RTT, vacation, sick leave, wellness day)
- Company holidays shown in UI
- Any row already containing time entries (unless user asked to edit)

## Pre-Run Checklist

Before bulk changes, confirm with user:
- [ ] date range/pay period
- [ ] schedule pattern (single-block or lunch-split)
- [ ] known days to skip
- [ ] whether to edit existing entries or only empty days

## Troubleshooting

### Save clicked but row still unchanged
- Wait 2-3 seconds and re-read row.
- BambooHR can apply changes asynchronously.

### PM not applying
- Do not write hidden `<select>` values.
- Use toggle focus + `Space` → `ArrowDown` → `Enter`.

### Unexpected extra row in modal
- `Add Entry` was clicked one extra time.
- Either fill/delete extra row before Save, or Cancel and retry.

### Dialog state is dirty from prior attempt
- Close with Cancel, reopen cleanly from the row.

### Delete Time Entry appears to do nothing
- Deleting an entry opens a confirmation dialog.
- Click **Yes, Delete Entry**, then click **Save** on the edit dialog.
- Without the confirmation step, no deletion is persisted.

## Example Verification Target

For a correct lunch-split day, expect:
- Summary: `9:00 AM - 5:42 PM (2 Entries)`
- Entries:
  - `9:00 AM` to `12:00 PM` = `3h 00m`
  - `1:00 PM` to `5:42 PM` = `4h 42m`
- Day total: `7h 42m`
