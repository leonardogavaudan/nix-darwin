---
name: qmd
description: Semantic search for ~/master knowledge vault. Use when searching for Algolia concepts, architecture patterns, or documentation by meaning rather than exact keywords.
---

# QMD - Knowledge Vault Search

QMD is a local semantic search engine for the ~/master Obsidian vault. It combines BM25 keyword search, vector embeddings, and LLM reranking.

## When to Use

- Searching the vault when you're not sure which doc has the answer
- Finding docs by concept rather than exact terms
- Exploring what's documented about an Algolia system or workflow

## Commands

```bash
# Fast keyword search (BM25)
qmd search "dim_application"

# Semantic search with reranking (best quality, slower)
qmd query "how to check if an app is on metis"

# Get a specific document by path or docid
qmd get "suggested-actions/metis-app-detection.md"
qmd get "#9aeeb5"
```

## Output

Results include:
- **Path**: `qmd://master/<file>:<line>`
- **Docid**: Short hash like `#9aeeb5` (use with `qmd get`)
- **Score**: Relevance percentage
- **Snippet**: Context around the match

## Maintaining the Index

After adding new docs to ~/master:
```bash
qmd update   # Re-index files
qmd embed    # Update vector embeddings
```

## Tips

- `qmd query` is slower but finds semantically related content
- `qmd search` is instant but only matches keywords
- Use `qmd status` to see indexed collections and doc counts
