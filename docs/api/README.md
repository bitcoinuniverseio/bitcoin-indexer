# API reference

Two independent read-only HTTP services ship in this repository. Each one reads the Postgres
databases the Rust indexer fills. Neither talks to Bitcoin Core, neither talks to the indexer
process, and neither can write anything.

| Service | Source | Databases read | Machine-readable contract |
| --- | --- | --- | --- |
| Ordinals API | `api/ordinals` | `ordinals`, `brc20` | [`ordinals-openapi.json`](ordinals-openapi.json) |
| Runes API | `api/runes` | `runes` | [`runes-openapi.json`](runes-openapi.json) |

Both contracts are OpenAPI 3.1 and cover every public read endpoint the services expose. There are
no other endpoints: every route is a `GET`, there is no authentication, and there is no mutation
surface to document.

## Route shape

The Bitcoin Universe route standard for indexers is `/<indexer-name>/<endpoint-name>`, where the
prefix is the indexer repository name with a leading `index-` removed. This repository predates that
naming (it is a fork carrying upstream route prefixes) and hosts two protocol indexes in one
repository, so it uses the protocol name as the prefix rather than the repository name:

| Service | Prefixes actually served |
| --- | --- |
| Ordinals API | `/ordinals/v1/...` and `/ordinals/...` |
| Runes API | `/runes/v1/...` and `/runes/...` |

Each service registers the identical route tree twice, once under the versioned prefix and once
under the bare prefix. Both sets are the same handlers, so `/ordinals/v1/inscriptions` and
`/ordinals/inscriptions` return the same thing. **Use the versioned prefix.** The published OpenAPI
contracts document only the versioned prefix.

Both services default to `API_PORT` 3000, so they cannot share a host without changing one of them.

## Regenerating the contracts

The contracts are generated from the live route schemas, not hand-written:

```console
(cd api/ordinals && npm ci)
(cd api/runes && npm ci)
node scripts/generate-openapi-contracts.mjs
```

The script runs each service's own `generate:openapi` target, converts the OpenAPI 3.0.3 output to
3.1, and replaces the upstream branding and upstream public server URL with values that are true for
this repository. Regenerate and commit whenever you change a route schema.

## Ordinals API endpoints

Base prefix `/ordinals/v1`.

### Status

| Method and path | Returns |
| --- | --- |
| `GET /` | `server_version`, `status`, `block_height`, `max_inscription_number`, `max_cursed_inscription_number` |

`block_height` is the indexer chain tip, taken from the `chain_tip` table. This is the endpoint to
poll for index freshness and for reorg detection.

### Inscriptions

| Method and path | Notes |
| --- | --- |
| `GET /inscriptions` | The main search endpoint. 24 filter parameters, listed below. |
| `GET /inscriptions/transfers` | Transfers within one block, selected by `block` |
| `GET /inscriptions/{id}` | One inscription, by inscription id or by inscription number |
| `GET /inscriptions/{id}/content` | The raw inscription bytes, served under the content type recorded for that inscription |
| `GET /inscriptions/{id}/transfers` | Movement history of one inscription |

`{id}` accepts either form:

- an inscription id matching `^[a-fA-F0-9]{64}i[0-9]+$`, for example
  `38c46a8bf7ec90bc7f6b797e7dc84baa97f4e5fd4286b92fe1b50176d03b18dci0`
- an inscription number, for example `145000`

`GET /inscriptions` filters, all optional and all combinable:

| Group | Parameters |
| --- | --- |
| Identity | `id`, `number` |
| Genesis block | `genesis_block`, `from_genesis_block_height`, `to_genesis_block_height`, `from_genesis_timestamp`, `to_genesis_timestamp` |
| Satoshi | `from_sat_ordinal`, `to_sat_ordinal`, `from_sat_coinbase_height`, `to_sat_coinbase_height`, `rarity` |
| Number range | `from_number`, `to_number` |
| Ownership | `address`, `genesis_address`, `output` |
| Content | `mime_type`, `recursive`, `cursed` |
| Paging and order | `offset`, `limit`, `order_by`, `order` |

`rarity` accepts `common`, `uncommon`, `rare`, `epic`, `legendary`, `mythic`. `order_by` accepts
`number`, `genesis_block_height`, `ordinal`, `rarity`. `order` accepts `asc` or `desc`.

### Satoshis

| Method and path | Notes |
| --- | --- |
| `GET /sats/{ordinal}` | Ordinal theory attributes of one satoshi: rarity, degree, name, cycle, epoch, period |
| `GET /sats/{ordinal}/inscriptions` | Inscriptions carried by that satoshi |

`{ordinal}` is an integer in `[0, 2099999997690000)`. Out-of-range values return `400` with
`{"error": "Invalid satoshi ordinal number"}`.

### Statistics

| Method and path | Notes |
| --- | --- |
| `GET /stats/inscriptions` | Inscription count per block, optionally bounded by `from_block_height` and `to_block_height`. Served from the maintained `counts_by_block` table, not from a scan. |

### BRC-20

| Method and path | Notes |
| --- | --- |
| `GET /brc-20/tokens` | Token list, filterable by `ticker`, sortable with `order_by` |
| `GET /brc-20/tokens/{ticker}` | Deploy parameters and supply for one token |
| `GET /brc-20/tokens/{ticker}/holders` | Holders of one token |
| `GET /brc-20/balances/{address}` | Balances held by one address, optionally at a historical `block_height` |
| `GET /brc-20/balances/{address}/transferable` | Inscriptions currently in a transferable state for that address |
| `GET /brc-20/activity` | Operation feed, filterable by `ticker`, `block_height`, `operation`, `address` |

These routes read the BRC-20 database. They return empty results, not errors, if the indexer ran
with BRC-20 disabled.

## Runes API endpoints

Base prefix `/runes/v1`.

| Method and path | Notes |
| --- | --- |
| `GET /` | `server_version`, `status`, `block_height` |
| `GET /etchings` | All etched runes, newest first |
| `GET /etchings/{etching}` | One rune, by rune id, name, or spaced name |
| `GET /etchings/{etching}/activity` | Ledger events for one rune |
| `GET /etchings/{etching}/activity/{address}` | Ledger events for one rune and one address |
| `GET /etchings/{etching}/holders` | Holders of one rune, ordered by balance |
| `GET /etchings/{etching}/holders/{address}` | One holder balance in one rune |
| `GET /addresses/{address}/balances` | All rune balances for one address |
| `GET /addresses/{address}/activity` | All rune activity for one address |
| `GET /transactions/{tx_id}/activity` | Rune activity in one transaction |
| `GET /blocks/{block}/activity` | Rune activity in one block, filterable by `operation_type` |

`operation_type` accepts `etching`, `mint`, `burn`, `send`, `receive`, matching the
`ledger_operation` enum in the database.

## Conventions shared by both APIs

### Pagination

Every list endpoint takes `offset` (minimum 0) and `limit`. Responses are
`{ "limit": n, "offset": n, "total": n, "results": [...] }`.

The `limit` maxima differ:

- **Ordinals API:** fixed at 60 in the route schemas.
- **Runes API:** taken from the `API_RESULTS_MAX_LIMIT` environment variable, default 60. That
  variable is read when the schema is built, so changing it changes the published maximum.
  Regenerate the contract if you change it in a deployment whose contract you publish.

### Caching

The Ordinals API sets `ETag` and `Cache-Control: must-revalidate` on inscription, inscription
transfer, and per-block responses. Send the value back as `If-None-Match` and you get
`304 Not Modified` when nothing changed. The ETag is derived from the underlying location timestamp,
so it changes when a reorg rewrites that range. Use it: it is the cheapest correct way to poll.

### Errors

| Status | Body | When |
| --- | --- | --- |
| `400` | `{"error": "Invalid satoshi ordinal number"}` | Satoshi routes with an out-of-range ordinal |
| `404` | `{"error": "Not found"}` | Unknown inscription, token, or rune |
| `304` | empty | `If-None-Match` matched the current ETag |

Malformed parameters are rejected by schema validation before reaching a handler.

### CORS

Both services register `@fastify/cors` with default settings, which permits cross-origin requests
from any origin. If that is not what you want in a deployment, terminate CORS at your proxy.

### Metrics

Both services expose Prometheus metrics on port 9153 when running in production mode, which is the
default. See [Operations](../operations.md).
