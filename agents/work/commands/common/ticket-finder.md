---
description: Find and work on Jira tickets autonomously end-to-end.
---

# Autonomous Ticket Finder

Find tickets that can be completed autonomously end-to-end. The goal is **independence** - can you understand the problem, implement a solution, and verify it works without needing human input? That's more important than raw size.

User preferences or constraints (if provided): $ARGUMENTS

## Backlog Cache

The OPTIM backlog is cached locally for fast querying:

```
/Users/leonardo.gavaudan/dev/.cache/optim_backlog.json
```

### Refresh the cache

Run this when the cache is stale (older than a day or so):

```bash
~/dev/.cache/refresh_optim_backlog.sh
```

### Query examples

```bash
FILE="/Users/leonardo.gavaudan/dev/.cache/optim_backlog.json"

# Count by status
jq 'group_by(.status) | map({status: .[0].status, count: length})' "$FILE"

# Low priority (often smaller scope)
jq '[.[] | select(.priority == "Low" or .priority == "Lowest")]' "$FILE"

# Search by keyword
jq '[.[] | select(.summary | test("dashboard"; "i"))]' "$FILE"

# Dashboard issues (AlgoliaWeb/_client)
jq '[.[] | select(.summary | test("\\[dashboard\\]"; "i"))]' "$FILE"

# Go repo issues (analytics/abtests)
jq '[.[] | select(.summary | test("\\[analytics\\]|\\[abtests\\]"; "i"))]' "$FILE"
```

## Good Candidate Criteria

**These are INITIAL FILTERS only.** The real assessment comes from deep code exploration (step 3).

### Must have (from Jira)
- [ ] Clear description of what needs to change
- [ ] In a repo cloned locally (check ~/dev/)
- [ ] Scope you can reason about (up to ~5 files, <150 lines changed is fine)

### Good signals (from Jira)
- Bug fixes with specific reproduction steps
- UI text changes
- Config/constant changes
- Adding validation or error handling
- Ticket mentions specific file/function names (easier to verify)
- Clear "done when" criteria (even if informal)
- Similar tickets have been completed before (check closed tickets)

### Avoid (from Jira)
- "Investigate", "spike", "RFC" (research tasks - no clear deliverable)
- Missing description entirely
- Requires external access you don't have (production DBs, customer data)

### Proceed with caution (from Jira)
These aren't dealbreakers - just require more exploration:
- Cross-repo changes (might be fine if patterns are similar)
- Only links to Slack threads (context may be incomplete, but explore anyway)
- Vague scope (code exploration might clarify it)
- No explicit acceptance criteria (often inferable from context)

### Red flags discovered during exploration
These are concerns to weigh, not automatic disqualifiers:
- **No existing pattern** - you'd be inventing, not following (but small inventions are OK)
- **Scope explosion** - ticket touches 10+ files unexpectedly (reassess, but might still be tractable)
- **Breaking changes** - URL parameters, CSV headers, API contracts (needs extra care, not impossible)
- **Multiple code paths** - fix works in tests but not in production flow (trace carefully)

### When to push through complexity
Don't bail at the first sign of difficulty. Push through when:
- The complexity is **accidental** (messy code) not **essential** (genuinely hard problem)
- You can write a failing test that captures the bug
- The fix is mechanical even if spread across files (e.g., rename a field everywhere)
- Similar changes exist in git history you can follow
- The ticket is high-value and worth the extra effort

## Workflow

### 0. Setup: Git Worktree

**Always use a git worktree** to avoid conflicts with other agents working on the same repo. Use the `worktrees` skill for full instructions.

**Example for ticket work:**
```bash
cd ~/dev/AlgoliaWeb
git fetch origin
DEFAULT_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")
git worktree add -b feat/optim-XXXX-short-description ~/dev/worktrees/AlgoliaWeb-optim-XXXX "origin/$DEFAULT_BRANCH"
find . -name "AGENTS.local.md" -type f | while read f; do
    mkdir -p ~/dev/worktrees/AlgoliaWeb-optim-XXXX/"$(dirname "$f")"
    cp "$f" ~/dev/worktrees/AlgoliaWeb-optim-XXXX/"$f"
done
cd ~/dev/worktrees/AlgoliaWeb-optim-XXXX
```

**Read `AGENTS.local.md`** in the worktree - it contains repo-specific instructions, gotchas, and multi-agent development tips (like HMR port conflicts).

### 1. Find candidates

**Don't filter by keywords** - that misses good tickets with different wording. Instead, use a multi-stage filter:

```bash
FILE="/Users/leonardo.gavaudan/dev/.cache/optim_backlog.json"

# Stage 1: Get all To Do tickets in repos we have locally
# Tags: [dashboard], [analytics], [abtests], [rankee], [python], [feature-evaluator], [metis]
# Also include untagged tickets (may still be in local repos)
jq '[.[] | select(
  .status == "To Do" and
  (.summary | test("\\[hex\\]"; "i") | not)
)] | length' "$FILE"
# This shows how many candidates we have to work with

# Stage 2: Exclude obvious non-starters
jq '[.[] | select(
  .status == "To Do" and
  (.summary | test("\\[hex\\]"; "i") | not) and
  (.summary | test("investigate|spike|RFC|discuss|\\[To Discuss\\]"; "i") | not)
)] | length' "$FILE"

# Stage 3: Get the filtered list with keys
jq '[.[] | select(
  .status == "To Do" and
  (.summary | test("\\[hex\\]"; "i") | not) and
  (.summary | test("investigate|spike|RFC|discuss|\\[To Discuss\\]"; "i") | not)
) | {key, summary, priority}]' "$FILE"
```

**Then fetch full details from Jira** for 15-20 candidates using your harness Jira issue tool in parallel (for example: `atlassian_jira_get_issue`, `jira_get_issue`, or `mcp__atlassian__jira_get_issue`). Score each ticket:

✅ **Good signals:**
- Clear acceptance criteria ("Done if...")
- Specific file/function mentions
- Example request/response or reproduction steps
- Unassigned

⚠️ **Caution signals (explore anyway, but note the risk):**
- Only links to Slack (context may be incomplete)
- "nice-to-have" language (might be deprioritized, but often still valid work)
- Vague description (code exploration might clarify)

❌ **Skip these:**
- No description at all
- "determine if we should..." (decision needed first)
- "Solution: TBD" or "investigate before working"
- Already assigned to someone else

After scoring, pick 3-5 best candidates for deep exploration.

### 2. Get full details
Fetch from Jira API to see description and acceptance criteria:
```bash
curl -s -H "Authorization: Basic $(echo -n "leonardo.gavaudan@algolia.com:${ATLASSIAN_API_TOKEN}" | base64)" \
  "https://algolia.atlassian.net/rest/api/3/issue/OPTIM-XXXX?fields=summary,description" | jq '.fields'
```

### 3. Deep exploration of candidates (CRITICAL)

**Jira descriptions often misrepresent complexity.** Before recommending a ticket:

1. **Use Explore agents IN PARALLEL** to investigate 3-5 promising candidates simultaneously
2. For each candidate, the Explore agent should:
   - Find the actual files/functions mentioned in the ticket
   - Understand the existing code patterns
   - Identify what specifically needs to change
   - Look for existing tests
   - Estimate lines of code and files affected
   - Surface any complications or unknowns

**Example prompt for Explore agent:**
```
Explore the codebase to understand [TICKET]: "[summary]"

The ticket says: [paste description]

Tasks:
1. Find the specific files/functions mentioned
2. Understand the existing patterns - is there a template to follow?
3. What exactly needs to change? How many lines/files?
4. Are there existing tests? What test coverage is needed?
5. Any complications? (breaking changes, multiple code paths, missing context)

Report back with:
- Current state of the code
- What the fix would involve (be specific)
- Estimated scope (files, lines of code)
- Complications or concerns
- Verdict: Can this be completed autonomously? What's the confidence level?
```

**Why this matters:**
- "Change wording" can mean 19 files with breaking URL parameters
- "Add validation" can mean 5 lines if the helper already exists
- "Fix bug" can be trivial or require deep architectural understanding

**Concerns to weigh (not automatic disqualifiers):**
- Pattern doesn't exist (but small inventions are OK if the scope is clear)
- Multiple code paths discovered (trace them, understand them)
- Scope is larger than expected (reassess, but 5-7 files can still be tractable)
- Breaking changes possible (needs extra care with tests and verification)

**Hard stops (actually skip these):**
- Requires access you don't have (prod data, external systems)
- Needs product/design decisions that haven't been made
- Depends on other unfinished work

**For cross-stack changes (frontend calling backend):**
- Always verify the backend supports what the ticket assumes
- Don't trust ticket descriptions - check the actual backend code
- Confirm parameter names, allowed values, and validation rules

### 4. Compare and recommend

After exploration, create a comparison table:

| Ticket | Scope | Pattern exists? | Concerns | Confidence | Verdict |
|--------|-------|-----------------|----------|------------|---------|
| OPTIM-1234 | 2 files, ~20 lines | YES | None | High | ✅ GO |
| OPTIM-5678 | 5 files, ~80 lines | YES | Multiple code paths | Medium | ✅ GO |
| OPTIM-9999 | 8 files | NO | Breaking API changes, no tests | Low | ⚠️ RISKY |
| OPTIM-0000 | Unknown | N/A | Needs prod access | N/A | ❌ SKIP |

Present the top 5 candidates with:
- Why they're good fits
- Specific files that will change
- Confidence level and any caveats
- What could go wrong (be honest)

### 5. Confirm before starting work

**STOP and ask for user confirmation before starting work on a ticket.**

Present:
- Ticket key and summary
- Brief description of what needs to be done
- Which repo/files will likely be affected
- Any concerns or uncertainties

Wait for explicit approval (e.g., "yes", "go ahead", "looks good") before proceeding to implementation.

### 6. Locate the code
- Dashboard UI → `AlgoliaWeb/_client/src/`
- Analytics/ABTest API → `go/` repo
- Python services → `python/` repo
- Search for keywords from the ticket in the codebase

### 7. Verify the gap exists

Before writing any code, confirm the ticket is valid:

**For bugs:**
- Write a failing test that reproduces the issue
- This proves the bug exists and prevents regressions

**For features/enhancements:**
- Search the codebase for the functionality described
- Confirm it's not already implemented (grep for key terms, field names, etc.)
- Example: ticket says "display X in tooltip" → grep for "X" usage, check if it's already passed to the component

This step catches:
- Duplicate work (feature already exists)
- Stale tickets (bug already fixed)
- Misunderstanding of scope

### 8. Implement and verify
1. Make the minimal change
2. Run the test → should pass
3. Run all tests for the file
4. Lint modified files
5. Typecheck (frontend) or build (backend)
6. **For frontend changes:** visually verify in browser (see "Frontend-Specific: Visual Verification" below)

**⚠️ Unit tests passing ≠ feature working**

Unit tests may pass while the actual feature is broken. This happens when:
- The component test uses correct props, but the real page constructs props differently
- An adapter/transformer works correctly, but some components bypass it and manually construct their own objects
- Multiple code paths exist and tests only cover one

**Always trace the real data flow.** Before assuming your fix works:
1. Find where the data originates (API response, store, etc.)
2. Follow it through every transformation until it reaches the UI
3. Check if there are multiple code paths - some components may use shared utilities while others manually construct objects
4. Verify your change affects the actual path used by the page, not just the path covered by tests

A common gotcha: adapters/transformers that work perfectly in isolation, but the component you're fixing doesn't use them - it manually constructs the same object and your new field isn't included.

Before committing, verify the fix works in the actual app, not just in tests.

### 9. Visually verify frontend changes

**⚠️ FOR FRONTEND CHANGES: DO NOT SKIP THIS STEP ⚠️**

Unit tests passing does NOT mean the feature works. You MUST visually verify in the browser before proceeding.

1. Start the dev server: `yarn dev:beta` (use `--port 8182` if 8181 is taken)
2. Navigate to the affected page
3. Test the actual user flow - click buttons, change tabs, submit forms
4. Verify the fix works as expected
5. Test edge cases if applicable
6. Take a screenshot if useful for the PR

**Only proceed to the next step after visual verification is complete.**

If visual verification isn't practical (see "When visual verification isn't practical" section), document why in the PR.

### 10. Confirm before committing

**STOP and ask for user confirmation before committing.**

Present a summary:
- Files changed and a brief description of changes
- Tests run and their status
- Proposed commit message

Wait for explicit approval (e.g., "yes", "go ahead", "looks good") before proceeding.

### 11. Commit
Once confirmed and tests pass, commit with the Jira ticket reference:
```
fix(component): brief description [OPTIM-XXXX]
```

### 12. Open PR and update Jira
After creating the PR:
1. **Move ticket to sprint** - Add the ticket to the current sprint
2. **Change status** - Transition the ticket to "In Review"
3. **Assign ticket** - Assign the ticket to leonardo.gavaudan@algolia.com
4. **Link PR** - Add the PR URL to the ticket

Use the Jira tools available in your harness:
```bash
# Transition to "In Review" (get transition ID first)
jira_get_transitions for OPTIM-XXXX
jira_transition_issue OPTIM-XXXX to "In Review"

# Assign ticket
jira_update_issue OPTIM-XXXX with assignee
```

### 13. After PR is merged: Cleanup

Once the PR is merged:

1. **Update Jira** - Transition to "Done"
```bash
jira_transition_issue OPTIM-XXXX to "Done"
```

2. **Remove the worktree**
```bash
cd ~/dev/AlgoliaWeb
git worktree remove ~/dev/worktrees/AlgoliaWeb-optim-XXXX --force
```

3. **Delete the local branch**
```bash
git branch -D feat/optim-XXXX-short-description
```

---

## Frontend-Specific: Visual Verification

*The sections below apply only to dashboard/frontend tickets in AlgoliaWeb/_client.*

### Starting the dev server

```bash
cd ~/dev/AlgoliaWeb-optim-XXXX && yarn dev:beta
```

**If another agent is using port 8181**, use a different port:
```bash
yarn dev:beta --port 8182
```

**Note:** See `AGENTS.local.md` for the HMR fix if you get infinite reload loops on non-default ports.

- No backend needed - proxies to beta-dashboard.algolia.com (staging)
- Use the browser skill to navigate and verify the fix
- Take a screenshot if useful for the PR

### Testing edge cases (fetch override technique)

For bugs that require specific data conditions (e.g., control value = 0), unit tests are often sufficient. But if visual verification is needed, use the **fetch override technique**.

#### CRITICAL: SPA Navigation Required

**⚠️ The fetch override is cleared by full page reloads.**

- ❌ **DON'T** use the `navigate` MCP tool - it does a full page reload
- ❌ **DON'T** use `location.reload()` after installing the override
- ✅ **DO** use click-based navigation (clicking links within the app)

React Router handles link clicks as client-side navigation, which preserves JavaScript context.

#### Step-by-step workflow

**1. Understand the data flow first**

Before modifying data, trace how it flows from API → component. See the **"Understanding Component Architecture"** section at the end of this document for detailed guidance.

Key questions to answer:
- What API endpoint returns the data?
- How is the data transformed before rendering?
- What field names does the component actually use?
- Are there multiple code paths (e.g., different calculations for different metric types)?

Read the component code to understand what values trigger the bug.

**2. Navigate to the page first**

Use the browser skill to get to the page where you'll test.

**Load the browser tools available in your harness first:**
- list/select tabs
- run JavaScript in the page
- read page content and console logs
- click/type automation

The override must be installed while you're already in the app.

**3. Install the fetch override**

Run this via your browser JavaScript execution tool:

```javascript
// Store original fetch and mark as installed (for verification)
window._originalFetch = window._originalFetch || window.fetch;
window._fetchOverrideInstalled = true;

window.fetch = async (url, options) => {
  const response = await window._originalFetch(url, options);

  // Match the specific API endpoint
  if (url.includes('/your-api-endpoint/')) {
    try {
      const data = await response.clone().json();

      // Log to verify override is working
      console.log('[OVERRIDE] Modifying response for:', url);

      // Modify the specific field that triggers the bug
      // BE SPECIFIC - know exactly what field the component reads
      if (data.someField) {
        data.someField.value = 0; // Create the edge case
      }

      return new Response(JSON.stringify(data), {
        status: response.status,
        statusText: response.statusText,
        headers: response.headers
      });
    } catch (e) {
      return response;
    }
  }
  return response;
};

'Override installed';
```

**4. Verify the override is installed**

```javascript
window._fetchOverrideInstalled === true ? 'Override active ✓' : 'Override lost ✗'
```

**5. Trigger a fresh data fetch via SPA navigation**

Click a link to navigate away, then click back:
- Click "Back to list" link
- Click on the item again

This triggers a new API call that your override will intercept.

**6. Check console logs**

Use your browser console-reading tool with a pattern filter:
```
tabId: <your-tab-id>
pattern: "OVERRIDE"
```

This shows if your override was triggered during the fetch.

**7. Take screenshot to verify**

The UI should now show the edge case behavior (e.g., "-" instead of "+Infinity%").

#### Debugging tips

- **Override not triggering?** Check URL matching - log all URLs to see what's being fetched
- **Data not changing?** The component might calculate from different fields than you're modifying
- **Override lost after navigation?** You used `navigate` tool instead of clicking links
- **Multiple code paths?** Some metrics may use different calculation formulas - read the component code

#### Example: Testing division-by-zero in A/B test metrics

```javascript
window._originalFetch = window._originalFetch || window.fetch;
window._fetchOverrideInstalled = true;

window.fetch = async (url, options) => {
  const response = await window._originalFetch(url, options);

  // Only intercept main A/B test endpoint (not settings/timeseries)
  if (url.includes('/abtests/') && !url.includes('timeseries') && !url.includes('settings')) {
    try {
      const data = await response.clone().json();

      if (data.variants && data.variants.length >= 2) {
        // Set control's metric to 0 to trigger division by zero
        data.variants[0].metrics.forEach(metric => {
          if (metric.name === 'revenue' || metric.name === 'no_result_count') {
            console.log(`[OVERRIDE] Setting control ${metric.name} to 0`);
            metric.value = 0;
          }
        });
      }

      return new Response(JSON.stringify(data), {
        status: response.status,
        statusText: response.statusText,
        headers: response.headers
      });
    } catch (e) {
      return response;
    }
  }
  return response;
};
```

#### When visual verification isn't practical

Sometimes it's not worth the effort:
- Data structure is too complex to mock correctly
- Multiple interdependent fields need modification
- The edge case requires state that can't be faked via API alone
- **Data is cached at app startup** (see below)

In these cases, a solid unit test is sufficient. Document why visual verification was skipped in the PR.

#### The caching gotcha

**Fetch mocking only works if the component actually calls `fetch` when you need it to.**

Many React apps have context providers that pre-fetch and cache data at app startup. Your component might call a function like `searchIndices()` which *looks* like an API call but actually just searches an in-memory cache.

```
What you think happens:

  Component → searchIndices() → fetch('/indices') → your mock
                                       ↑
                                   intercept here

What actually happens:

  App startup → IndicesProvider → fetch('/indices') → cache
                                         ↑
                                    too late to mock

  Component → searchIndices() → reads cache (no fetch)
```

**Before mocking, trace the data to the actual `fetch` call:**
1. Find where the data is used in the component
2. Trace back through hooks/contexts to find the real API call
3. Check if it's cached - look for context providers, `useMemo`, global stores
4. Determine *when* the cache is populated (app startup? route change? component mount?)

If the data is cached at app startup, fetch mocking won't work. Use unit tests instead.

## Repos by Tag

| Tag in summary | Repo | Path |
|----------------|------|------|
| `[dashboard]` | AlgoliaWeb | `_client/src/` |
| `[analytics]`, `[abtests]` | go | `analytics/`, `abtests/` |
| `[rankee]` | rankee | `.` |
| `[python]`, `[feature-evaluator]` | python | various |
| `[hex]` | N/A (external) | Skip these |

## Tips

- Start with the test, not the fix
- Fetch ticket details before committing to work on it
- If you can't find the code after thorough exploration, move to another ticket
- Don't touch tickets already assigned to someone
- Check the PR template before creating PRs

## Calibration Examples

These examples help calibrate expectations - some tickets look scary but are fine, others look simple but aren't.

### Looked hard, was actually fine ✅

| Ticket pattern | Why it seemed hard | Why it was fine |
|----------------|-------------------|-----------------|
| "Fix display of X in 5 different views" | Multiple files | Same pattern repeated, mechanical changes |
| "Add new field to API response" | Touches backend + frontend | Clear data flow, existing field to copy |
| "Handle edge case when value is 0" | Vague description | One conditional check, existing test pattern |
| "Update error message wording" | No specific files mentioned | grep found it immediately, 2-line change |

### Looked simple, was actually hard ❌

| Ticket pattern | Why it seemed simple | Why it was hard |
|----------------|---------------------|-----------------|
| "Change button text" | Just text, right? | Text was dynamically generated from 3 sources |
| "Fix off-by-one error" | One line fix | Required understanding complex pagination state |
| "Add tooltip to field" | UI only | Field was in a shared component used 20 places with different contexts |
| "Update CSV export column" | Just add a field | Column order was a breaking change for downstream consumers |

### Key insight

**Jira summaries are unreliable predictors of complexity.** The only way to know is to explore the code. Don't skip tickets because they sound hard, and don't assume tickets are easy because they sound simple.

## Frontend: Understanding Component Architecture

*This section helps with visual verification when using the fetch override technique.*

When modifying API data for visual testing, you MUST understand how data flows through the component tree. Blindly modifying fields won't work if you're changing the wrong thing.

### Trace the data flow

For a React frontend, trace the path:

```
API Response
    ↓
Page Component (fetches data)
    ↓
Container Component (transforms data)
    ↓
Presentational Component (renders UI)
    ↓
Column/Cell Definition (calculates display value)
```

### Example: A/B Test Metrics in AlgoliaWeb

```
API: /abtests/{id}
  returns: { variants: [{ metrics: [{name, value}, ...] }] }
    ↓
ABTestDetails.tsx
  fetches abTest, passes to ABTestMetricBreakdown
    ↓
ABTestMetricBreakdown.tsx
  calls processMetrics(abTest) to pivot data
  metrics["ctr"].metricRows = [{value, variantIndex}, ...]
    ↓
ABTestMetricBreakdownCard.tsx
  sets control = filteredRows[0]
  passes {metric, control} to column.render()
    ↓
metricSettings.tsx - createDefaultColumns()
  'difference' column calculates:
    PATH A: relativeRatio(metric.value, control.value) - handles zero safely
    PATH B: (metric/control) with adjustForTrafficSplit - CAN produce Infinity
```

### Key insight

The same API field might go through different code paths depending on metric type. In our case:
- `click_through_rate` → Path A (safe)
- `revenue`, `no_result_count` → Path B (buggy)

We had to modify the right metrics (`revenue`, `no_result_count`) to trigger the bug, not just any metric.
