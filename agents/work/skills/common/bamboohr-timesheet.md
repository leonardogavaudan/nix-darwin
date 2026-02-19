---
description: Fill out BambooHR timesheets efficiently. Use when adding time entries, backfilling work hours, or doing bulk timesheet entry for pay periods.
---

# BambooHR Timesheet Entry Skill

Use this skill when filling out BambooHR timesheets, adding time entries, or backfilling work hours.

## Overview

BambooHR timesheets require manual entry for each working day. This skill documents the efficient workflow for adding time entries.

## Algolia BambooHR URL

**Timesheet URL**: `https://algolia.bamboohr.com/employees/timesheet/?id={employee_id}`

To find your employee ID, check the URL when on your profile page or timesheet.

## Page Structure

The timesheet page contains:
- A list of days in the pay period
- Each day has a hidden "+ Add Time Entry" link (visible on hover)
- A summary panel on the right showing totals

### Finding Day Entry Links

Read the page with `filter: all` to find the ref_ids for each day's "+ Add Time Entry" link:

```
link "+ Add Time Entry" [ref_XXX]  <- for each day
```

The refs are assigned sequentially. Map them before starting bulk entry by reading the page once.

## Time Entry Modal

When clicking "+ Add Time Entry", a modal opens with:
- **Start Time**: text field (hh:mm format) + AM/PM dropdown
- **End Time**: text field (hh:mm format) + AM/PM dropdown
- **Day Total**: auto-calculated
- **Save/Cancel** buttons

## Efficient Entry Workflow (Viewport-Independent)

**IMPORTANT**: Avoid hardcoded coordinates - they vary by screen size. Use ref_ids and keyboard navigation instead.

### Step 1: Read Page and Map Refs

```python
# Read page once to get all refs
read_page(filter="all", depth=15)

# Extract ref_ids for each day's "+ Add Time Entry" link
# They follow a pattern like: ref_116 (Jan 1), ref_122 (Jan 2), etc.
```

### Step 2: For Each Working Day

```
1. Click the "+ Add Time Entry" link by ref_id (e.g., ref_142 for Jan 6)
   - This works without hover, unlike coordinate clicks

2. Wait for modal to open, then read page to find the Start Time input ref_id

3. Click Start Time field BY REF (or use Tab from modal open to focus it)

4. Type start time: "9:00"

5. Press: Tab Tab
   - First Tab: skips AM dropdown (AM is default, which is correct)
   - Second Tab: lands on End Time field

6. Type end time: "4:42"

7. Press: Tab Down Return
   - Tab: moves to End Time AM/PM dropdown
   - Down: changes AM to PM
   - Return: confirms PM selection

8. Find and click Save button by ref_id (read page to find it)
   - Or press Tab to navigate to Save, then Return

9. Wait 0.5-1 second for save to complete
```

### Alternative: Full Keyboard Navigation

After clicking "+ Add Time Entry" ref:
```
Tab → (focus Start Time) → type "9:00" → Tab Tab → type "4:42" → Tab Down Return → Tab Tab → Return (Save)
```

## Days to Skip

**Always skip:**
- Saturdays and Sundays
- Days already showing time off (RTT, vacation, etc.) - visible in page read
- Company holidays (shown with holiday name like "New Year's Day")

**French Bank Holidays (2026):**
- Jan 1 - New Year's Day
- Apr 6 - Easter Monday
- May 1 - Labour Day
- May 8 - Victory Day
- May 14 - Ascension
- May 25 - Whit Monday
- Jul 14 - Bastille Day
- Aug 15 - Assumption
- Nov 1 - All Saints Day
- Nov 11 - Armistice Day
- Dec 25 - Christmas

## Contracted Work Hours

Based on employment contract (SYNTEC cadre, forfait hebdomadaire):
- **Weekly hours**: 38h30 (38 hours 30 minutes)
- **Daily hours**: 7h42 (7 hours 42 minutes)
- **Lunch**: NOT tracked in BambooHR manual entry

**Standard entry**: 9:00 AM - 4:42 PM = 7h42

## Bulk Entry Strategy

1. **First**: Navigate to `https://algolia.bamboohr.com/employees/timesheet/?id={employee_id}`
2. **Read page** with `filter: all` to map all ref_ids for "+ Add Time Entry" links
3. **Create a list** of working days (exclude weekends, holidays, time off)
4. **Loop through** each day using ref_id clicks + keyboard navigation
5. **Verify** by checking the Pay Period total after completion

## Pre-Entry Checklist

Before starting bulk entry, confirm with user:
- [ ] Start time (default: 9:00 AM)
- [ ] End time (default: 4:42 PM for 7h42 per contract)
- [ ] Which days to fill (current pay period? specific date range?)
- [ ] Any additional days to skip (vacation, sick days, etc.)

## Editing Existing Entries

To fix an incorrect entry (e.g., wrong end time):

1. **Click the existing time display** (e.g., "6:00 PM") - this opens the edit modal for that entry
2. Navigate to the field to change (e.g., Tab to End Time field)
3. **Clear and retype** the value (e.g., select all with Cmd+A, then type "4:42")
4. For End Time PM: Tab → Down → Return (to change AM dropdown to PM)
5. Click Save or Tab to Save and press Return

**Key difference from adding**: You click on the displayed time, not "+ Add Time Entry".

## Verification

After completing entries, verify:
- **Pay Period total**: Should match expected hours (e.g., 154h for 20 working days × 7h42)
- **This Week display**: Should show 38h30 for a full work week (confirms daily entries are correct)

## Troubleshooting

**Modal doesn't open**: Click by ref_id directly - this is more reliable than coordinate clicks and doesn't require hover.

**Wrong AM/PM**: The End Time defaults to AM. Always use Tab → Down → Return after typing end time to select PM.

**Save fails silently**: Wait at least 0.5s between entries. Read page to verify Day Total updated before proceeding.

**Ref_ids changed**: If refs don't work, re-read the page - the page may have updated after a save.

## Example Session

```
User: Fill my BambooHR timesheet for January

Agent:
1. Navigate to https://algolia.bamboohr.com/employees/timesheet/?id=1900
2. Read page (filter: all) to get all "+ Add Time Entry" ref_ids
3. Identify working days from page content:
   - Skip Jan 1 (shows "New Year's Day")
   - Skip Jan 2 (shows "1 days RTT")
   - Skip all Sat/Sun
4. For each working day ref_id:
   - Click ref_id
   - Tab → type "9:00" → Tab Tab → type "4:42" → Tab Down Return
   - Click Save ref or Tab Tab Return
   - Wait 0.5s
5. Verify Pay Period total (~154h for full month with 20 working days × 7h42)
```

## Ref_id Pattern Example (January 2026)

From actual page read:
```
Jan 1 (Thu) - Holiday:     ref_116 (skip)
Jan 2 (Fri) - RTT:         ref_122 (skip)
Jan 3 (Sat):               ref_127 (skip - weekend)
Jan 4 (Sun):               ref_132 (skip - weekend)
Jan 5 (Mon):               ref_137 ✓
Jan 6 (Tue):               ref_142 ✓
Jan 7 (Wed):               ref_147 ✓
...and so on
```

The refs increment by ~5 for each day. Always verify by reading the page.
