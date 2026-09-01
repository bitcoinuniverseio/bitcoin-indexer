# Configuration

The Rust indexer is configured by a single TOML file. The two read APIs are configured entirely by
environment variables. They share nothing but the Postgres databases.

## Generating a starting file

```console
bitcoin-indexer config new --mainnet
bitcoin-indexer config new --testnet
bitcoin-indexer config new --regtest
```

Exactly one network flag is required. The file is always written to `./Indexer.toml`, and an
existing file at that path is overwritten.

> **Known defect.** `--regtest` writes `network = "regtest"`, but the parser accepts only `devnet`,
> `testnet`, `mainnet`, and `signet`. Starting the indexer with the generated regtest file fails with
> `bitcoind.network not supported`. Change the value to `devnet` by hand.

## A complete, working configuration

Every section shown here except `[ordinals]`, `[runes]`, and `[metrics]` is required. Values are the
ones the generator writes, with comments describing what they do.

```toml
[storage]
# Directory for the Ordinals RocksDB block archive. Unused by the Runes index.
# Optional. Defaults to "data" relative to the working directory.
working_dir = "tmp"

[metrics]
# Optional section. Omit it, or set enabled = false, to run without a metrics server.
enabled = true
prometheus_port = 9153

# --- Ordinals index -----------------------------------------------------------
# Omit the whole [ordinals] tree to disable the Ordinals index. `ordinals` CLI
# commands then fail with "Config entry for `ordinals` not found in config file."
[ordinals.db]
database = "ordinals"
host = "localhost"
port = 5432
username = "postgres"
password = "postgres"        # optional
# search_path = "public"     # optional, defaults to "public"
# pool_max_size = 10         # optional, defaults to the deadpool default

# BRC-20 is a meta-protocol indexed inside the Ordinals run loop.
# Omit this tree, or set enabled = false, to index inscriptions without BRC-20.
[ordinals.meta_protocols.brc20]
enabled = true
lru_cache_size = 10000       # optional, defaults to 50000

[ordinals.meta_protocols.brc20.db]
database = "brc20"
host = "localhost"
port = 5432
username = "postgres"
password = "postgres"

# --- Runes index --------------------------------------------------------------
# Omit the whole [runes] tree to disable the Runes index.
[runes]
lru_cache_size = 10000       # optional, defaults to 50000

[runes.db]
database = "runes"
host = "localhost"
port = 5432
username = "postgres"
password = "postgres"

# --- Bitcoin node -------------------------------------------------------------
[bitcoind]
network = "mainnet"          # mainnet | testnet | signet | devnet
rpc_url = "http://localhost:8332"
rpc_username = "devnet"
rpc_password = "devnet"
zmq_url = "tcp://0.0.0.0:18543"

# --- Resources ----------------------------------------------------------------
[resources]
ulimit = 2048
cpu_core_available = 6
memory_available = 16
bitcoind_rpc_threads = 2
bitcoind_rpc_timeout = 15
indexer_channel_capacity = 10
```

## Key reference

### `[storage]`

| Key | Type | Default | Effect |
| --- | --- | --- | --- |
| `working_dir` | string | `"data"` | Parent directory of the Ordinals RocksDB archive, which is created at `<working_dir>/hord.rocksdb`. The Runes index never writes here. |

### `[metrics]`

| Key | Type | Default | Effect |
| --- | --- | --- | --- |
| `enabled` | bool | section absent means no metrics server | Starts an HTTP server exposing `GET /metrics` |
| `prometheus_port` | integer | none, required when the section is present | Port to bind. The server binds `0.0.0.0`. |

### `[bitcoind]`

All five keys are required.

| Key | Type | Notes |
| --- | --- | --- |
| `network` | string | One of `mainnet`, `testnet`, `signet`, `devnet`. `devnet` maps to Bitcoin regtest. Any other value is a startup error. |
| `rpc_url` | string | Must begin with `http://`. The client derives the `Host` header by slicing the first seven characters off this string, so an `https://` URL produces a malformed header. |
| `rpc_username` | string | Bitcoin Core RPC user |
| `rpc_password` | string | Bitcoin Core RPC password |
| `zmq_url` | string | ZeroMQ endpoint, for example `tcp://127.0.0.1:18543`. Only the `hashblock` topic is subscribed. |

### `[ordinals.db]`, `[ordinals.meta_protocols.brc20.db]`, `[runes.db]`

Identical shape for all three.

| Key | Type | Default | Notes |
| --- | --- | --- | --- |
| `database` | string | required | Database name |
| `host` | string | required | Host |
| `port` | integer | required | Port |
| `username` | string | required | Role |
| `password` | string | optional | Omit for peer or trust authentication |
| `search_path` | string | `"public"` | Passed as `-csearch_path=<value>` |
| `pool_max_size` | integer | deadpool default | Maximum pooled connections |

Connections are made without TLS. See [Security](security.md).

### `[ordinals.meta_protocols.brc20]` and `[runes]`

| Key | Type | Default | Effect |
| --- | --- | --- | --- |
| `enabled` | bool | required for BRC-20 | When false, inscriptions are still indexed but no BRC-20 database is touched |
| `lru_cache_size` | integer | `50000` | Entries held in the protocol's in-memory cache. Larger values trade memory for fewer Postgres round trips. |

### `[resources]`

The section is required. Every key inside it is optional.

| Key | Type | Default | Effect |
| --- | --- | --- | --- |
| `ulimit` | integer | `2048` | Sets RocksDB `max_open_files` for the Ordinals block archive. Keep it at or below the OS file descriptor limit. No effect on the Runes index. |
| `cpu_core_available` | integer | detected core count | Sizes the block compression and inscription sequencing thread pools as `cpu_core_available - 2`, with a floor of 1 |
| `memory_available` | integer | `8` | Currently has no effect. It is read and passed into the RocksDB options builder, which ignores it. |
| `bitcoind_rpc_threads` | integer | `4` | Concurrent block download workers during historical catch-up |
| `bitcoind_rpc_timeout` | integer | `15` | Currently has no effect. It is parsed but never read by the request path. |
| `indexer_channel_capacity` | integer | `10` | Depth of the two bounded channels between download, standardization, and indexing. This is the main lever on peak memory during catch-up. |

## Indexer environment variables

| Variable | Effect |
| --- | --- |
| `ORDHOOK_MAINTENANCE` | Set to `1` to make `service start` log `Entering maintenance mode` and sleep instead of indexing. Unset it and restart to resume. Only `service start` checks it; `index sync`, `index rollback`, and `database migrate` ignore it. |
| `ORDHOOK_TEST_PG_DATABASE`, `ORDHOOK_TEST_PG_HOST`, `ORDHOOK_TEST_PG_PORT`, `ORDHOOK_TEST_PG_USER`, `ORDHOOK_TEST_PG_PASSWORD` | Point the Rust test suite at a Postgres instance. Defaults are `postgres`, `localhost`, `5432`, `postgres`, `postgres`. Not read outside tests. |

## Ordinals API environment variables

Defined by the schema in `api/ordinals/src/env.ts`. A `.env` file in the API directory is loaded
automatically. Variables without a default are required, and the process refuses to start without
them.

| Variable | Default | Notes |
| --- | --- | --- |
| `API_HOST` | `0.0.0.0` | Bind address |
| `API_PORT` | `3000` | API port |
| `PROFILER_PORT` | `9119` | Profiler server port. Started in production mode only, which is the default. See the note below. |
| `ORDINALS_PGHOST` | required | Ordinals database host |
| `ORDINALS_PGPORT` | `5432` | Ordinals database port |
| `ORDINALS_PGUSER` | required | Ordinals database role |
| `ORDINALS_PGPASSWORD` | required | Ordinals database password |
| `ORDINALS_PGDATABASE` | required | Ordinals database name |
| `ORDINALS_SCHEMA` | unset | Schema override |
| `BRC20_PGHOST` | required | BRC-20 database host |
| `BRC20_PGPORT` | `5432` | BRC-20 database port |
| `BRC20_PGUSER` | required | BRC-20 database role |
| `BRC20_PGPASSWORD` | required | BRC-20 database password |
| `BRC20_PGDATABASE` | required | BRC-20 database name |
| `BRC20_SCHEMA` | unset | Schema override |
| `PG_CONNECTION_POOL_MAX` | `10` | Maximum concurrent connections |
| `PG_IDLE_TIMEOUT` | `30` | Seconds |
| `PG_MAX_LIFETIME` | `60` | Seconds |
| `PG_STATEMENT_TIMEOUT` | `60000` | Milliseconds |

The BRC-20 connection settings have no defaults, so the Ordinals API cannot currently start without
a reachable BRC-20 database even if you do not intend to serve BRC-20 routes.

> The checked-in `api/ordinals/README.md` describes a `RUN_MODE` variable with `default`, `readonly`,
> and `writeonly` values. That variable is not in the current environment schema and setting it does
> nothing.

## Runes API environment variables

Defined by the schema in `api/runes/src/env.ts`.

| Variable | Default | Notes |
| --- | --- | --- |
| `API_HOST` | `0.0.0.0` | Bind address |
| `API_PORT` | `3000` | API port |
| `ADMIN_RPC_PORT` | `3001` | Declared in the schema. No server is bound to it in the current code. |
| `RUNES_PGHOST` | required | Runes database host |
| `RUNES_PGPORT` | `5432` | Runes database port |
| `RUNES_PGUSER` | required | Runes database role |
| `RUNES_PGPASSWORD` | required | Runes database password |
| `RUNES_PGDATABASE` | required | Runes database name |
| `PG_CONNECTION_POOL_MAX` | `10` | Maximum concurrent connections |
| `PG_IDLE_TIMEOUT` | `30` | Seconds |
| `PG_MAX_LIFETIME` | `60` | Seconds |
| `PG_STATEMENT_TIMEOUT` | `60000` | Milliseconds |
| `API_RESULTS_MAX_LIMIT` | `60` | Largest `limit` a Runes request may ask for |

## Production mode and the extra listeners

Both APIs treat themselves as running in production unless `NODE_ENV` is exactly `test` or
`development`. An unset `NODE_ENV` counts as production. In production mode each API binds two extra
listeners on `API_HOST`:

| Listener | Port | Contents |
| --- | --- | --- |
| Prometheus | `9153`, not configurable | `GET /metrics` |
| Profiler (Ordinals API only) | `PROFILER_PORT`, default `9119` | Node.js profiling endpoints |

Bind these to a private interface, or set `API_HOST` to a private address and put a reverse proxy in
front of `API_PORT`. See [Security](security.md).
