---
name: jira-querying
description: Query Jira issues and boards using MCP tools. Use when searching Jira, fetching issues, or working with sprints. Includes workaround for broken jira_search.
---

# Jira Querying

## Team Context

- **Project**: OPTIM
- **Board ID**: 284
- **Board URL**: https://algolia.atlassian.net/jira/software/c/projects/OPTIM/boards/284

## Search Workaround

`jira_search` is broken. Use `jira_get_board_issues` instead:

```
jira_get_board_issues(board_id="284", jql="your JQL query", limit=50)
```

## JQL Syntax Tips

- Use `NOT IN` instead of `!=` (avoids URL encoding issues)
  - ✅ `statusCategory NOT IN (Done)`
  - ❌ `statusCategory != Done`
- Backlog items: `sprint is EMPTY`
- Open items: `statusCategory NOT IN (Done)`

## Response Size Management

MCP responses can exceed context limits. To reduce size:

1. Use `fields` parameter to limit returned fields:
   ```
   fields="key,summary,status,priority,assignee"
   ```

2. Use smaller `limit` values (20-50) for inline display

3. For bulk fetches, consider direct API calls with pagination and save to file

## Common Operations

### Get single issue
```
jira_get_issue(issue_key="OPTIM-123")
```

### Get board issues with JQL
```
jira_get_board_issues(
  board_id="284",
  jql="assignee = currentUser() AND statusCategory NOT IN (Done)",
  limit=50
)
```

### Get current sprint issues
```
jira_get_sprints_from_board(board_id="284", state="active")
jira_get_sprint_issues(sprint_id="SPRINT_ID", limit=50)
```

### Get backlog (not in any sprint)
```
jira_get_board_issues(
  board_id="284",
  jql="sprint is EMPTY AND statusCategory NOT IN (Done) ORDER BY rank",
  limit=50
)
```

### Get available transitions
```
jira_get_transitions(issue_key="OPTIM-123")
```

### Add issue to sprint
```
jira_update_issue(
  issue_key="OPTIM-123",
  fields={"customfield_10010": 13030}  # sprint ID as integer
)
```

**Important:** Use the `fields` parameter, not `additional_fields`. The sprint field (`customfield_10010`) fails with "not on appropriate screen" error when passed via `additional_fields`.

To get the active sprint ID:
```
jira_get_sprints_from_board(board_id="284", state="active")
```

## OPTIM Ticket Fields Reference

When querying issues, use the `fields` parameter to request specific fields. Here's what's actually used by OPTIM:

### Standard Fields
| Field | Access Path | Description |
|-------|-------------|-------------|
| `key` | `.key` | Issue key (OPTIM-1634) |
| `summary` | `.summary` | Title |
| `description` | `.description` | Full description |
| `status` | `.status.name` | In Progress, Done, In Review |
| `status.category` | `.status.category` | Done, In Progress, To Do |
| `priority` | `.priority.name` | Low, Medium, High |
| `assignee` | `.assignee.display_name` | Who's working on it |
| `reporter` | `.reporter.display_name` | Who created it |
| `parent` | `.parent.key` | Parent epic key |
| `issue_type` | `.issue_type.name` | Task, Bug, Story, Epic |
| `created` | `.created` | Creation timestamp |
| `updated` | `.updated` | Last update timestamp |

### Custom Fields Used by OPTIM
| Field ID | Access Path | Name | Example Value |
|----------|-------------|------|---------------|
| `customfield_10033` | `.customfield_10033.value` | Story Points | `2.0`, `3.0`, `5.0` |
| `customfield_10010` | `.customfield_10010.value[0]` | Sprint | `"Happy New Year 2"` |
| `customfield_10008` | `.customfield_10008.value` | Epic Link | `"OPTIM-2147"` |
| `customfield_10011` | `.customfield_10011.value` | Rank | `"1|hyp90b:"` |
| `customfield_10000` | `.customfield_10000.value` | Development/PR | JSON with PR state |

### Recommended Fields Parameter

For typical queries, use:
```
fields="key,summary,status,priority,assignee,parent,customfield_10033,customfield_10010"
```

This gives you: key, title, status, priority, assignee, epic, story points, and sprint.

### Field Value Structures

**Status object:**
```json
{
  "name": "In Review",
  "category": "In Progress",
  "color": "yellow"
}
```

**Assignee object:**
```json
{
  "display_name": "Leonardo Gavaudan",
  "email": "leonardo.gavaudan@algolia.com"
}
```

**Parent object:**
```json
{
  "key": "OPTIM-2147",
  "fields": {
    "summary": "Analytics Continuous Improvement",
    "status": { "name": "Perpetual" },
    "issuetype": { "name": "Epic" }
  }
}
```

**Custom field wrapper:**
Most custom fields wrap their value: `{"value": <actual_value>}`
