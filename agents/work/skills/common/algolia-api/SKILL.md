---
name: algolia-api
description: Query Algolia APIs (AB Tests, Analytics, Search, DRR, Suggested Actions, Datamixer). READ-ONLY operations only. Use when working with Algolia internal APIs for the F4T6CUV2AH test app.
---

# Algolia API Skill

**READ-ONLY** - This skill only performs GET requests. No creates, updates, or deletes.

## Configuration

### Credentials

The skill uses these environment variables (or hardcoded defaults for the test app):

| Variable | Description | Default |
|----------|-------------|---------|
| `ALGOLIA_APP_ID` | Application ID | `F4T6CUV2AH` |
| `ALGOLIA_API_KEY` | API Key | _(use prod EU key)_ |

### Endpoints

| Environment | Analytics URL | Search URL | DRR URL | Suggested Actions URL |
|-------------|---------------|------------|---------|----------------------|
| **Prod EU** | `analytics.de.algolia.com` | `F4T6CUV2AH.algolia.net` | `re-ranking.de.algolia.com` | `sact.eu.algolia.com` |
| Prod US | `analytics.us.algolia.com` | `F4T6CUV2AH.algolia.net` | `re-ranking.us.algolia.com` | `sact.us.algolia.com` |
| Staging | `analytics-staging.de.algolia.com` | `F4T6CUV2AH.algolia.net` | _(same)_ | `sact.staging.eu.algolia.com` |

### Default Credentials (Prod EU)

```bash
APP_ID="F4T6CUV2AH"
API_KEY="${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}"
```

---

## AB Tests API

Base URL: `https://analytics.de.algolia.com`

### List AB Tests

```bash
curl -s "https://analytics.de.algolia.com/2/abtests" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

Optional query params:
- `limit` - number of results (default 10)
- `offset` - pagination offset
- `indexPrefix` - filter by index prefix
- `indexSuffix` - filter by index suffix

### Get AB Test by ID

```bash
curl -s "https://analytics.de.algolia.com/2/abtests/{abTestID}" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

### Get AB Test Timeseries (v3)

```bash
curl -s "https://analytics.de.algolia.com/3/abtests/{abTestID}/timeseries" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

Optional query params:
- `startDate` - YYYY-MM-DD
- `endDate` - YYYY-MM-DD
- `metric` - e.g., `user_count`

### Get AB Test Settings (v3)

```bash
curl -s "https://analytics.de.algolia.com/3/abtests/{abTestID}/settings" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

---

## Analytics API

Base URL: `https://analytics.de.algolia.com`

### Get Searches

```bash
curl -s "https://analytics.de.algolia.com/2/searches?index=products&startDate=2024-07-01&endDate=2024-07-07&limit=100" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

Query params:
- `index` - index name (required)
- `startDate` / `endDate` - date range
- `limit` - max results
- `clickAnalytics` - true/false
- `revenueAnalytics` - true/false

### Get Hits

```bash
curl -s "https://analytics.de.algolia.com/2/hits?index=products&startDate=2024-07-01&endDate=2024-07-07" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

### Get No Click Rate

```bash
curl -s "https://analytics.de.algolia.com/2/searches/noClickRate?index=products&startDate=2024-07-01&endDate=2024-07-07" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

### Get Filters by Category

```bash
curl -s "https://analytics.de.algolia.com/2/filters/?index=products&startDate=2024-07-01&endDate=2024-07-07" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

---

## Algolia Search API

Base URL: `https://F4T6CUV2AH.algolia.net`

### List Indices

```bash
curl -s "https://F4T6CUV2AH.algolia.net/1/indexes" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

### Get Index Settings

```bash
curl -s "https://F4T6CUV2AH.algolia.net/1/indexes/{indexName}/settings" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

### List Rules

```bash
curl -s -X POST "https://F4T6CUV2AH.algolia.net/1/indexes/{indexName}/rules/search" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"hitsPerPage": 100}' | jq .
```

### List Synonyms

```bash
curl -s -X POST "https://F4T6CUV2AH.algolia.net/1/indexes/{indexName}/synonyms/search" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{}' | jq .
```

---

## DRR (Dynamic Re-Ranking) API

Base URL: `https://re-ranking.de.algolia.com`

### Get DRR Config

```bash
curl -s "https://re-ranking.de.algolia.com/1/configs/{indexName}" \
  -H "x-algolia-application-id: F4T6CUV2AH" \
  -H "x-algolia-api-key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

---

## Suggested Actions API

Base URL: `https://sact.eu.algolia.com` (Prod EU)

### List Suggested Actions (Public)

```bash
curl -s "https://sact.eu.algolia.com/1/suggested-actions?limit=10" \
  -H "X-Algolia-Application-Id: F4T6CUV2AH" \
  -H "X-Algolia-API-Key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

Query params:
- `target_type` - `APPLICATION` or `INDEX`
- `target_id` - target identifier (e.g., index name)
- `status` - `ACTIVE`, `APPLIED`, `REJECTED`, `EXPIRED`, or `WITHDRAWN`
- `limit` - 1-100 (default 10)
- `offset` - pagination offset (default 0)

### Response Format

```json
{
  "suggestedActions": [
    {
      "id": "uuid",
      "appId": "F4T6CUV2AH",
      "targetType": "INDEX",
      "targetId": "products",
      "intent": "TOGGLE",
      "subject": "FEATURE",
      "payload": {
        "@type": "type.googleapis.com/algolia.suggestedactions.v1.ToggleFeaturePayloadV1",
        "featureName": "dynamic_re_ranking",
        "enable": true,
        "correlationId": "offline-eval-2025-09-30"
      },
      "provenance": {
        "source": "offline_evaluator",
        "modelVersion": "v2.1",
        "evidence": [{"metric": "ndcg@10", "delta": "+4.5%"}]
      },
      "status": "ACTIVE",
      "producerId": "feature-qualifier",
      "createdAt": "2025-11-17T16:29:07.197951Z",
      "updatedAt": "2025-11-17T16:29:07.197951Z"
    }
  ],
  "count": 1
}
```

### Filter by Status (Active only)

```bash
curl -s "https://sact.eu.algolia.com/1/suggested-actions?status=ACTIVE" \
  -H "X-Algolia-Application-Id: F4T6CUV2AH" \
  -H "X-Algolia-API-Key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

### Filter by Target Index

```bash
curl -s "https://sact.eu.algolia.com/1/suggested-actions?target_type=INDEX&target_id=products" \
  -H "X-Algolia-Application-Id: F4T6CUV2AH" \
  -H "X-Algolia-API-Key: ${ALGOLIA_API_KEY:?set ALGOLIA_API_KEY}" | jq .
```

---

## Datamixer API (gRPC)

Datamixer is a unified gRPC service wrapping Dashboard API, Search Admin API, BigQuery, and BigTable. It provides richer application/index metadata than the REST APIs above.

### Prerequisites

1. **grpcurl** installed (via nix: `pkgs.grpcurl`)
2. **GCP auth**: `gcloud auth print-identity-token` (your @algolia.com account)
3. **Proto descriptor**: Build once, reuse until protos change

### Endpoints

| Region | Endpoint |
|--------|----------|
| **EU Prod** | `eu.prod.datamixer.internal.algolia.com:443` |
| **US Prod** | `us.prod.datamixer.internal.algolia.com:443` |
| **Staging** | `staging.datamixer.internal.algolia.com:443` |

Use the endpoint matching your app's `dataRegion`.

### Build Proto Descriptor (one-time)

```bash
cd ~/dev/go && buf build -o /tmp/datamixer.binpb --path proto/internal/algolia/datamixer/v1/datamixer.proto
```

Rebuild when protos are updated (after `git pull` in go repo).

### Get Application

Returns app metadata, event counts, feature flags, region.

```bash
TOKEN=$(gcloud auth print-identity-token) && \
grpcurl -protoset /tmp/datamixer.binpb -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "applications/APP_ID", "view": "APPLICATION_VIEW_FULL"}' \
  eu.prod.datamixer.internal.algolia.com:443 \
  algolia.datamixer.v1.Datamixer/GetApplication
```

Response fields: `name`, `displayName`, `dataRegion`, `eventCount1h`, `eventCount6h`, `eventCount7d`, `eventCount30d`, `avgEventsPerHour7d`, `avgEventsPerHour6h`, `realtimePersonalizationFeatureEnabled`

### List Indexes

Returns all indexes with object counts, replica relationships.

```bash
TOKEN=$(gcloud auth print-identity-token) && \
grpcurl -protoset /tmp/datamixer.binpb -H "Authorization: Bearer $TOKEN" \
  -d '{"parent": "applications/APP_ID", "page_size": 20}' \
  eu.prod.datamixer.internal.algolia.com:443 \
  algolia.datamixer.v1.Datamixer/ListIndexes
```

Response fields per index: `name`, `indexName`, `createTime`, `updateTime`, `objectCount`, `primaryIndexName`, `replicaIndexNames`

### Get Index Config (Settings)

Returns full index settings as structured data.

```bash
TOKEN=$(gcloud auth print-identity-token) && \
grpcurl -protoset /tmp/datamixer.binpb -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "applications/APP_ID/indexes/INDEX_NAME/config", "view": "INDEX_CONFIG_VIEW_FULL"}' \
  eu.prod.datamixer.internal.algolia.com:443 \
  algolia.datamixer.v1.Datamixer/GetIndexConfig
```

### Get Semantic Config (NeuralSearch)

Returns NeuralSearch/semantic settings for an index.

```bash
TOKEN=$(gcloud auth print-identity-token) && \
grpcurl -protoset /tmp/datamixer.binpb -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "applications/APP_ID/indexes/INDEX_NAME/semanticConfig"}' \
  eu.prod.datamixer.internal.algolia.com:443 \
  algolia.datamixer.v1.Datamixer/GetSemanticConfig
```

### List All Methods

```bash
TOKEN=$(gcloud auth print-identity-token) && \
grpcurl -protoset /tmp/datamixer.binpb -H "Authorization: Bearer $TOKEN" \
  eu.prod.datamixer.internal.algolia.com:443 \
  describe algolia.datamixer.v1.Datamixer
```

### Available Methods

| Method | Description |
|--------|-------------|
| `GetApplication` | App metadata, event counts, region, feature flags |
| `ListIndexes` | All indexes with object counts, replicas |
| `GetIndexConfig` | Full index settings (ranking, facets, etc.) |
| `GetSemanticConfig` | NeuralSearch settings |
| `GetObject` / `ListObjects` | Individual records |
| `GetObjectPerformance` | 30-day click/conversion metrics |
| `GetFacetMap` / `ListFacetMaps` | Facet values for objects |
| `ListEventHealthMetrics` | Event health across apps |
| `GetAdvancedPersonalizationEventReadiness` | Perso event readiness |
| `ListNotificationSubscriptions` | User notification prefs |

---

## Quick Reference

| API | Endpoint Pattern | Method | Base URL |
|-----|------------------|--------|----------|
| List AB Tests | `/2/abtests` | GET | analytics.de.algolia.com |
| Get AB Test | `/2/abtests/:id` | GET | analytics.de.algolia.com |
| AB Test Timeseries | `/3/abtests/:id/timeseries` | GET | analytics.de.algolia.com |
| AB Test Settings | `/3/abtests/:id/settings` | GET | analytics.de.algolia.com |
| Searches | `/2/searches` | GET | analytics.de.algolia.com |
| Hits | `/2/hits` | GET | analytics.de.algolia.com |
| No Click Rate | `/2/searches/noClickRate` | GET | analytics.de.algolia.com |
| List Indices | `/1/indexes` | GET | {appId}.algolia.net |
| Index Settings | `/1/indexes/:index/settings` | GET | {appId}.algolia.net |
| List Rules | `/1/indexes/:index/rules/search` | POST | {appId}.algolia.net |
| List Synonyms | `/1/indexes/:index/synonyms/search` | POST | {appId}.algolia.net |
| DRR Config | `/1/configs/:index` | GET | re-ranking.de.algolia.com |
| Suggested Actions | `/1/suggested-actions` | GET | sact.eu.algolia.com |

### Datamixer (gRPC)

| Method | Resource Pattern | Endpoint |
|--------|------------------|----------|
| GetApplication | `applications/{app_id}` | eu.prod.datamixer.internal.algolia.com |
| ListIndexes | `applications/{app_id}/indexes` | eu.prod.datamixer.internal.algolia.com |
| GetIndexConfig | `applications/{app_id}/indexes/{index}/config` | eu.prod.datamixer.internal.algolia.com |
| GetSemanticConfig | `applications/{app_id}/indexes/{index}/semanticConfig` | eu.prod.datamixer.internal.algolia.com |
| GetObjectPerformance | `applications/{app_id}/indexes/{index}/objects/{id}/performance` | eu.prod.datamixer.internal.algolia.com |
