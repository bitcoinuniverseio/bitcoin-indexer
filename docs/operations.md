# Operations

## What listens where

| Process | Listener | Port | Configured by |
| --- | --- | --- | --- |
| Indexer | Prometheus `GET /metrics` | `metrics.prometheus_port`, generated value 9153 | `Indexer.toml`, binds `0.0.0.0` |
| Ordinals API | REST | `API_PORT`, default 3000 | environment |
| Ordinals API | Prometheus `GET /metrics` | 9153, not configurable | production mode only |
| Ordinals API | Profiler | `PROFILER_PORT`, default 9119 | production mode only |
| Runes API | REST | `API_PORT`, default 3000 | environment |
| Runes API | Prometheus `GET /metrics` | 9153, not configurable | production mode only |

Production mode is the default. Both APIs treat themselves as production unless `NODE_ENV` is
exactly `test` or `development`. Only the REST port belongs on a public interface. See
[Security](security.md).

The indexer metrics server answers `GET /metrics` and nothing else. Any other method or path is
logged and rejected.

## Health checks

There is no dedicated health endpoint on either service. Use these instead.

| Check | Request | Healthy signal |
| --- | --- | --- |
| API liveness | `GET /ordinals/v1/` or `GET /runes/v1/` | `200` with `"status": "ready"` |
| API to database | the same request | It queries `chain_tip`, so a `200` proves Postgres is reachable |
| Index freshness | `block_height` from the same response | Compare against your Bitcoin node height |
| Indexer liveness | `GET /metrics` on the indexer port | `200` with a text body |
| Indexer progress | `last_indexed_block_height` in that body | Increasing over time |

A minimal freshness probe:

```console
curl -s localhost:3000/ordinals/v1/ | python -c "import json,sys; print(json.load(sys.stdin)['block_height'])"
```

## Metrics

### Indexer, Ordinals

| Metric | Type | Meaning |
| --- | --- | --- |
| `last_indexed_block_height` | gauge | Latest Bitcoin block indexed |
| `last_indexed_inscription_number` | gauge | Latest indexed inscription number |
| `last_classic_indexed_blessed_inscription_number` | gauge | Latest blessed inscription number |
| `last_classic_indexed_cursed_inscription_number` | gauge | Latest cursed inscription number |
| `block_processing_time` | histogram | Milliseconds to process a block |
| `inscription_parsing_time` | histogram | Milliseconds to parse a block's inscriptions |
| `inscription_computation_time` | histogram | Milliseconds to compute inscription state |
| `inscription_db_write_time` | histogram | Milliseconds writing to Postgres |
| `inscriptions_per_block` | histogram | Inscriptions found per block |
| `brc20_operations_per_block` | histogram | BRC-20 operations per block |
| `brc20_deploy_operations_per_block`, `brc20_mint_operations_per_block`, `brc20_transfer_operations_per_block`, `brc20_transfer_send_operations_per_block` | gauges | Per-block counts by operation |
| `brc20_deploy_operations_total`, `brc20_mint_operations_total`, `brc20_transfer_operations_total`, `brc20_transfer_send_operations_total` | gauges | Running totals by operation |

`block_processing_time` buckets are 10s, 20s, 30s, 60s, 120s, 300s expressed in milliseconds, which
tells you what the authors considered a normal range for a heavy block.

### Indexer, Runes

| Metric | Type | Meaning |
| --- | --- | --- |
| `last_indexed_block_height` | gauge | Latest Bitcoin block indexed |
| `last_indexed_rune_number` | gauge | Number of the last indexed rune |
| `runes_block_processing_time` | histogram | Milliseconds to process a block |
| `rune_parsing_time`, `rune_computation_time`, `rune_db_write_time` | histograms | Stage timings in milliseconds |
| `runes_per_block` | histogram | Runes seen per block |
| `runes_etching_operations_per_block`, `runes_edict_operations_per_block`, `runes_mint_operations_per_block` | gauges | Per-block operation counts |
| `runes_cenotaph_operations_per_block`, `runes_cenotaph_etching_operations_per_block`, `runes_cenotaph_mint_operations_per_block` | gauges | Per-block cenotaph counts |
| `runes_etching_inputs_checked_per_block` | gauge | Inputs checked for a rune commitment |

### APIs

| Metric | Service |
| --- | --- |
| `ordinals_api_block_height`, `ordinals_api_max_inscription_number`, `ordinals_api_max_cursed_inscription_number` | Ordinals API |
| `runes_api_block_height` | Runes API |

Plus the standard `fastify-metrics` HTTP request counters and duration histograms, and default
Node.js process metrics.

## What to alert on

| Alert | Condition | Why it matters |
| --- | --- | --- |
| Index stalled | `last_indexed_block_height` unchanged for longer than three expected block intervals | The most common real failure. Usually the node, ZeroMQ, or Postgres. |
| Index falling behind | Bitcoin node height minus `last_indexed_block_height` above your tolerance | Sustained lag means the indexer cannot keep up with the chain |
| Index went backwards | `last_indexed_block_height` decreased | Normal for a shallow reorg, worth investigating if deep or repeated |
| API and indexer disagree | `ordinals_api_block_height` diverging from `last_indexed_block_height` | The API is reading a different or stale database |
| Block processing time climbing | `block_processing_time` p95 rising | Usually Postgres write pressure or a cache that is too small |
| Metrics endpoint down | scrape failure | The process is gone or wedged |

Alert on `last_indexed_block_height` before anything else. Everything else is diagnosis.

## Backup and recovery

The Ordinals index has two pieces of state that must be backed up and restored **together**:

1. the `ordinals` Postgres database (and `brc20` when enabled),
2. the RocksDB archive at `<storage.working_dir>/hord.rocksdb`.

At startup the effective chain tip is the lower of the Postgres height and the archive height, and a
missing archive resets indexing to block 0. Restoring one without the other therefore either
reprocesses a large range or restarts the whole sync. See [Database model](database.md).

The Runes index has only its Postgres database.

Recommended procedure:

```console
# 1. stop the indexer cleanly
kill -INT <pid>

# 2. back up Postgres
pg_dump -Fc ordinals > ordinals.dump
pg_dump -Fc brc20    > brc20.dump
pg_dump -Fc runes    > runes.dump

# 3. back up the Ordinals block archive from the same stopped state
tar -C "$WORKING_DIR" -czf hord-rocksdb.tar.gz hord.rocksdb
```

Taking the Postgres dump and the archive copy from the same stopped process is the point of the
exercise. A hot Postgres dump paired with a later archive copy will not line up.

To restore, put both back, start the indexer, and let it catch up from whatever tip it computes.

## Upgrades

1. Read the migration files that are new in the target version. `migrations/` is the whole story;
   there are 18 Ordinals migrations, 7 BRC-20, and 5 Runes today.
2. Take a backup as above.
3. Stop the indexer with `SIGINT`.
4. Deploy the new binary.
5. Run `bitcoin-indexer <protocol> database migrate --config-path ...` and watch it finish.
6. Start the indexer.

Migrations are embedded into the binary at compile time, so a binary can only apply the migrations it
was built with. There is no down migration and no rollback path: to go back to an older binary you
restore the backup.

Two migrations do data work as well as schema work and are the ones to time on a copy of production
first: `V15__inscription_parents.sql` and `V18__chain_tip_block_hash.sql`.

## Rolling back the index

To drop recent blocks without touching backups:

```console
bitcoin-indexer ordinals index rollback 50 --config-path ./Indexer.toml
bitcoin-indexer ordinals index sync --config-path ./Indexer.toml
```

The rollback command prompts on stdin for confirmation, so run it interactively.

## Maintenance mode

Setting `ORDHOOK_MAINTENANCE=1` makes `service start` log and sleep instead of indexing. It is useful
when a supervisor restarts the process automatically and you need it to stay down while you work on
the database. Unset the variable and restart to resume.

## Zero-downtime reads during indexer work

The read APIs and the indexer are separate processes over the same databases. Stopping the indexer
does not stop the APIs; they keep serving whatever is committed, with a `block_height` that stops
advancing. For read availability during a long migration, run the APIs against a replica and cut
them over after the migration completes on the primary.
