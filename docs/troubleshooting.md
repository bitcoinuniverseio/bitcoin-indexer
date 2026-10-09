# Troubleshooting

Organized by what you actually observe.

## The indexer will not start

### `bitcoind.network not supported`

The `[bitcoind] network` value is not one of `mainnet`, `testnet`, `signet`, `devnet`.

This is what you get from a file produced by `bitcoin-indexer config new --regtest`, which writes
`network = "regtest"`. Change it to `devnet`. See [Configuration](configuration.md).

### `Config entry for 'ordinals' not found in config file.`

You ran an `ordinals` subcommand against a configuration file with no `[ordinals.db]` section. The
same applies to `runes` and `[runes]`. Add the section, or run the other protocol.

### `Config file malformatted <parse error>`

TOML syntax error, or a missing required key. Required sections are `[storage]`, `[bitcoind]`, and
`[resources]`. Required keys inside `[bitcoind]` are `network`, `rpc_url`, `rpc_username`,
`rpc_password`, `zmq_url`. A database table requires `database`, `host`, `port`, and `username`.

### `unable to read file <path>`

`--config-path` points at something that is not readable. The flag is required and there is no
default path.

### It logs `Entering maintenance mode` and then does nothing

`ORDHOOK_MAINTENANCE=1` is set in the environment. Unset it and restart. This is intentional
behaviour, not a fault.

### It logs `Unable to open db: ... Retrying in 10s` forever

RocksDB at `<storage.working_dir>/hord.rocksdb` cannot be opened. Usual causes, in order of
likelihood:

1. Another `bitcoin-indexer ordinals` process is already holding it. RocksDB takes an exclusive
   lock. Only one Ordinals process per working directory.
2. The directory is not writable by this user.
3. The file descriptor limit is below `resources.ulimit`, which is used as RocksDB `max_open_files`.

## The indexer starts but never indexes

### `bitcoind has not reached chain tip, trying again...`, once a second

The indexer will not begin until `getblockchaininfo` reports `initial_block_download = false` and
`blocks == headers` on ten consecutive polls. Either your node is still syncing, or it is not
reporting a settled tip. Check `bitcoin-cli getblockchaininfo` directly.

### `bitcoind error checking for chain tip: <error>`

RPC is unreachable or rejecting credentials. Check in this order:

1. `rpc_url` begins with `http://`. The client derives its `Host` header by slicing off the first
   seven characters, so `https://` produces a malformed header.
2. `rpc_username` and `rpc_password` match the node.
3. `rpcallowip` and `rpcbind` on the node permit the indexer host.

### `zmq: Waiting for ZMQ connection acknowledgment from bitcoind` and it stays there

The socket is connected but no `hashblock` message has arrived. That is expected until the next
block is mined. If a block has been mined and nothing arrived, check that the node has
`zmqpubhashblock` set and that `zmq_url` matches it exactly, protocol and port included.

### `zmq: <topic> Topic not supported`

Something is publishing a topic other than `hashblock` on that endpoint. Harmless, but it means the
endpoint is shared. Only `hashblock` is read.

## Indexing is slower than expected

Look at the stage histograms before changing anything:

```console
curl -s localhost:9153/metrics | grep -E 'inscription_(parsing|computation|db_write)_time|block_processing_time'
```

| Dominant stage | Likely cause | First thing to try |
| --- | --- | --- |
| `inscription_db_write_time` | Postgres write throughput | Faster storage, more shared buffers, raise `pool_max_size` |
| `inscription_parsing_time` or `inscription_computation_time` | CPU bound | Raise `resources.cpu_core_available` |
| None of them, and download looks idle | Node RPC is the limit | Raise `resources.bitcoind_rpc_threads` |

`Indexer command channel full, waiting for space` at debug level means the download side is ahead of
indexing. That is backpressure working, not a fault. Raising `indexer_channel_capacity` buys buffer
at the cost of memory; it does not make indexing faster.

See [Performance and sizing](performance.md).

## The indexer restarts from a much lower block

Almost always the RocksDB archive. The effective Ordinals chain tip is the lower of the Postgres
height and the archive height, and a missing or empty archive resets indexing to block 0.

Check that `storage.working_dir` points where you think it does, and that `hord.rocksdb` is present
inside it. A `working_dir` on ephemeral storage produces exactly this symptom after every restart.
The generated configuration file ships `working_dir = "tmp"`, which is relative to the process
working directory, so a service started from a different directory looks at a different archive.

See [Database model](database.md).

## The index height went backwards

A shallow reorg. `last_indexed_block_height` decreasing by a few blocks and then recovering is normal
and is the rollback path doing its job.

If it went back further than six blocks, or keeps oscillating, the in-memory block pool cannot
reconcile it. Roll back manually past the fork point and re-sync:

```console
bitcoin-indexer ordinals index rollback 50 --config-path ./Indexer.toml
bitcoin-indexer ordinals index sync --config-path ./Indexer.toml
```

See [Reorgs and mempool](reorgs-and-mempool.md).

## `Block #<n> was already indexed, skipping`

The indexer received a block at or below its chain tip. Normal after a restart, because the block in
flight at shutdown is reprocessed. Persistent repetition suggests two indexer processes writing to
the same database.

## Postgres errors

### `unable to get pg client` or `unable to build pg connection pool`

Postgres is unreachable, credentials are wrong, or the pool is exhausted. If it appears under load
rather than at startup, raise `pool_max_size` in the indexer configuration or
`PG_CONNECTION_POOL_MAX` in the APIs, and check `max_connections` on the server.

### `unable to commit ordinals pg transaction` or `unable to commit brc20 pg transaction`

The write failed at commit. Look at the Postgres log for the underlying cause: out of disk is the
common one, because inscription content is stored in-row.

### `RunesDb error running pg migrations`

A migration failed. Refinery compares checksums of already applied versions, so this also fires if an
existing migration file was edited after being applied. Never edit a released migration; add a new
one.

### The log says `Resetting ordinals DB` or `Resetting brc20 DB`

That is a destructive reset path being taken. It is not part of the normal `service start` flow.
Stop and find out what invoked it before letting it continue.

## API problems

### The API will not start and complains about a missing environment variable

Both APIs validate their environment at startup and refuse to run without required values. The
Ordinals API requires the `BRC20_PG*` variables even if you do not intend to serve BRC-20 routes,
because they have no defaults. See [Configuration](configuration.md).

### The API returns data but `block_height` never moves

The API is healthy and Postgres is reachable; the indexer is not advancing. Check
`last_indexed_block_height` on the indexer metrics port, and work through the indexing sections
above.

### `ordinals_api_block_height` and `last_indexed_block_height` disagree

The API is pointed at a different database from the one the indexer writes, or at a replica that is
lagging. Compare the `ORDINALS_PGDATABASE` and `ORDINALS_PGHOST` of the API against `[ordinals.db]`
in the indexer configuration.

### `404 {"error": "Not found"}` for something you know exists

Either the block containing it has not been indexed yet, or it was rolled back by a reorg. Check
`block_height` on the status endpoint against the height you expect.

### `400 {"error": "Invalid satoshi ordinal number"}`

The `{ordinal}` path value is outside `[0, 2099999997690000)`.

### Port 3000 is already in use

Both APIs default to `API_PORT` 3000. Running them on the same host requires changing one of them.
The same applies to port 9153, which both APIs use for Prometheus and which is not configurable on
the API side.

### Prometheus or profiler ports are exposed and you did not expect them

Both APIs treat themselves as production unless `NODE_ENV` is exactly `test` or `development`, and an
unset `NODE_ENV` counts as production. In production mode they bind a metrics server on 9153, and the
Ordinals API also binds a profiler on `PROFILER_PORT`. See [Security](security.md).

## Build problems

### The build fails compiling `rocksdb` or `zmq`

Both build native code. You need a C and C++ toolchain plus LLVM and Clang development headers. The
Docker build installs `clang-18`, `libclang-18-dev`, `llvm-18-dev`, and the snappy, gflags, zlib,
bzip2, lz4, and zstd development packages. Match that set on a bare host.

### `cargo bitcoin-indexer-install` is not recognized

The aliases live in `.cargo/config.toml` and only apply when cargo runs from inside the repository.
Run it from the repository root, or use `cargo install --path components/cli --locked --force`.

### Tests fail with database connection errors

The Rust suite and both API suites need PostgreSQL on `localhost:5432` with user `postgres`,
password `postgres`, database `postgres`:

```console
docker compose -f dockerfiles/docker-compose.dev.postgres.yml up -d
```

See [Testing](testing.md).

### Tests interfere with each other

`.cargo/config.toml` sets `RUST_TEST_THREADS = "1"` and the API suites use `jest --runInBand` because
they share a database. If you override either, expect flakes.
