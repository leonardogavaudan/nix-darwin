---
name: confluence-querying
description: Query Confluence spaces, folders, pages, comments, labels, and attachments using Atlassian MCP tools. Use when users ask to explore a Confluence space (especially Optimization/optmz), map page trees, find docs by keyword/CQL, or retrieve page content.
---

# Confluence Querying

## Quick Workflow

1. Discover the target space key.
2. Map folders and top-level structure.
3. Pull recent/high-signal pages.
4. Open specific pages and summarize findings.
5. Use comments/labels/attachments when body output is incomplete.

## Core Queries

### Discover space key from human name
Use CQL first:

```text
atlassian_confluence_search(query='space.title ~ "Optimization"', mode='cql')
```

### Map folder structure in a space

```text
atlassian_confluence_search(query='space = optmz and type = folder order by title', mode='cql', limit=100)
```

Then expand each folder:

```text
atlassian_confluence_get_page_children(pageId='<folder-id>', limit=100)
```

### Find recent pages

```text
atlassian_confluence_search(query='space = optmz and type = page order by lastmodified desc', mode='cql', limit=25)
```

### Read a page

```text
atlassian_confluence_get_page(pageId='<page-id>', includeBody=true, bodyFormat='markdown')
```

## Handling Common Limitations

1. **No direct "list spaces" tool**
   - Use `space.title ~ "..."` CQL to discover keys.

2. **Page body missing macro-rendered content in markdown**
   - Retry with HTML:
   ```text
   atlassian_confluence_get_page(pageId='<id>', includeBody=true, bodyFormat='html')
   ```
   - Also inspect:
     - `atlassian_confluence_get_comments`
     - `atlassian_confluence_get_labels`
     - `atlassian_confluence_list_attachments`

3. **New Confluence objects (folders/databases) hide structure**
   - Query `type = folder` and traverse folder IDs via `get_page_children`.

4. **Large responses/context pressure**
   - Keep `limit` small while exploring.
   - Fetch full body only for selected pages.

## Optimization Space Defaults

- Space key: `optmz`
- Useful starter queries:

```text
space = optmz and type = folder order by title
space = optmz and type = page order by lastmodified desc
space = optmz and title ~ "RFC"
space = optmz and title ~ "Offline Evaluation"
```

## Output Expectations

When reporting back:
- Include page title + URL.
- Separate **structure** (folders/pages) from **content summary**.
- Call out blind spots explicitly (macro-heavy pages, inaccessible attachments, truncated output).
