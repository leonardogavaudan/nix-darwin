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

**Performance rule:** start with selective indexed predicates. Broad filters can be very slow.

Use this filter shape (in order):
1. `logName`
2. `resource.type`
3. `resource.labels.cluster_name` / `namespace_name` / `container_name` / `pod_name`
4. Time bound (`--freshness` and/or `timestamp>=...`)
5. Payload filters (`jsonPayload.*`, `httpRequest.*`)

**Shell quoting gotcha**: filters with double quotes inside fail with single-quote wrapping. Use a variable:
```bash
FILTER='resource.labels.container_name="my-container"'
gcloud logging read "$FILTER" --project=alg-analytics --limit=20 --freshness=5m --format=json
```

Fast, targeted query (example for Analytics API 401s):
```bash
FILTER='logName="projects/alg-analytics/logs/api.analytics" AND resource.type="k8s_container" AND resource.labels.cluster_name="analytics-us-east1" AND jsonPayload.app_id="E7PHE9BB38" AND httpRequest.status=401'
gcloud logging read "$FILTER" --project=alg-analytics --limit=20 --freshness=168h --format='table(timestamp,httpRequest.status,jsonPayload.api_key,httpRequest.requestUrl,httpRequest.userAgent,httpRequest.remoteIp)'
```

When debugging a specific key/user/session, add an explicit timestamp bound:
```bash
FILTER='logName="projects/alg-analytics/logs/api.analytics" AND resource.type="k8s_container" AND resource.labels.cluster_name="analytics-us-east1" AND jsonPayload.app_id="E7PHE9BB38" AND timestamp>="2026-02-17T00:00:00Z" AND jsonPayload.api_key:"468a"'
gcloud logging read "$FILTER" --project=alg-analytics --limit=50 --format='table(timestamp,httpRequest.status,jsonPayload.api_key,httpRequest.requestUrl)'
```

If `gcloud logging read` is slow, query the exported BigQuery logs table directly:
```bash
bq query --use_legacy_sql=false --project_id=alg-analytics '
SELECT timestamp, httpRequest.status AS status, jsonPayload.api_key AS api_key, httpRequest.requestUrl AS request_url
FROM `alg-analytics.analytics_prod_api_requests_us_east1.api_analytics`
WHERE timestamp >= TIMESTAMP_SUB(CURRENT_TIMESTAMP(), INTERVAL 7 DAY)
  AND jsonPayload.app_id = "E7PHE9BB38"
  AND httpRequest.status = 401
ORDER BY timestamp DESC
LIMIT 50
'
```

Extract payload with jq:
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
