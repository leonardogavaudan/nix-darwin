---
name: gcp-querying
description: Query Google Cloud Platform resources using gcloud, bq, and gsutil. Use when working with BigQuery, GCS, or GCP projects at Algolia. Includes project context and account info.
---

# GCP Querying

## Projects

| Project | Purpose |
|---------|---------|
| `alg-analytics` | Default project |
| `alg-ai-platform` | AI platform production |
| `alg-ai-platform-staging` | AI platform staging |
| `alg-ai-platform-sandbox` | AI platform sandbox |

## Accounts

- `leonardo.gavaudan@algolia.com` - personal (admin on alg-analytics)
- `readonly-sa@alg-analytics.iam.gserviceaccount.com` - service account (read-only)

## Common Queries

### BigQuery

List tables in dataset:
```bash
bq ls alg-analytics:dataset_name
```

Query with output:
```bash
bq query --use_legacy_sql=false 'SELECT * FROM `alg-analytics.dataset.table` LIMIT 10'
```

Get table schema:
```bash
bq show --schema --format=prettyjson alg-analytics:dataset.table
```

### GCS

List buckets:
```bash
gsutil ls gs://bucket-name/
```

### Cloud Logging

**Shell quoting gotcha**: Filters with double quotes inside fail with single-quote wrapping. Use a variable:
```bash
FILTER='resource.labels.container_name="my-container"'
gcloud logging read "$FILTER" --project=alg-analytics --limit=20 --freshness=5m --format=json
```

Query by pod name (most reliable for K8s):
```bash
gcloud logging read 'resource.labels.pod_name:"my-pod"' --project=alg-analytics --limit=20 --format=json
```

Query by log content:
```bash
gcloud logging read 'jsonPayload.message:"job started"' --project=alg-analytics --limit=20 --format=json
```

Extract just the payload with jq:
```bash
gcloud logging read "..." --format=json | jq -r '.[] | "\(.timestamp) | \(.jsonPayload)"'
```

**Useful flags:**
- `--freshness=5m` - only logs from last 5 minutes
- `--limit=20` - max entries
- `--format=json` - for jq processing

### gcloud

List projects:
```bash
gcloud projects list
```

Switch project:
```bash
gcloud config set project alg-analytics
```
