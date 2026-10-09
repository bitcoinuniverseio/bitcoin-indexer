# CLI reference

The binary is `bitcoin-indexer`. Its top level splits by protocol.

```
bitcoin-indexer
├── ordinals
│   ├── service start   --config-path <PATH>
│   ├── index sync      --config-path <PATH>
│   ├── index rollback  <BLOCKS> --config-path <PATH>
│   └── database migrate --config-path <PATH>
├── runes
│   ├── service start   --config-path <PATH>
│   ├── index sync      --config-path <PATH>
│   ├── index rollback  <BLOCKS> --config-path <PATH>
│   └── database migrate --config-path <PATH>
└── config
    └── new  (--mainnet | --testnet | --regtest)
```

`--config-path` is required on every command that takes it. There is no default path and no
environment variable fallback.

## `ordinals service start` and `runes service start`

Runs migrations, catches the index up to the node's chain tip over JSON-RPC, then stays running and
follows new blocks over ZeroMQ. This is the long-lived production command.

```console
bitcoin-indexer ordinals service start --config-path ./Indexer.toml
```

Checks `ORDHOOK_MAINTENANCE` before doing anything. When it is `1`, the process logs
`Entering maintenance mode. Unset ORDHOOK_MAINTENANCE and reboot to resume operations` and sleeps
indefinitely without touching the database.

Requires the matching configuration section. `ordinals ...` without an `[ordinals]` section fails
with `Config entry for 'ordinals' not found in config file.`, and the same applies to `[runes]`.

## `ordinals index sync` and `runes index sync`

The same catch-up work as `service start`, but the process exits once the index reaches the node's
chain tip instead of switching to the ZeroMQ stream. Use it for scripted backfills and for
one-shot catch-up in a maintenance window.

```console
bitcoin-indexer runes index sync --config-path ./Indexer.toml
```

## `ordinals index rollback` and `runes index rollback`

Drops the most recent N blocks from the index.

```console
bitcoin-indexer ordinals index rollback 10 --config-path ./Indexer.toml
```

It reads the index chain tip, prints it, prints the resulting new tip, and waits on stdin for
confirmation:

```
Index chain tip is at #870000
10 blocks will be dropped. New index chain tip will be at #869990. Confirm? [Y/n]
```

Any answer starting with `n` aborts with `Deletion aborted`. Anything else proceeds. Because it
blocks on stdin, it is not usable in a non-interactive pipeline as written.

This is the manual counterpart to the automatic rollback the indexer performs on a reorg. It removes
rows from Postgres; it does not re-index. Run `index sync` or `service start` afterwards to rebuild
the dropped range.

## `ordinals database migrate` and `runes database migrate`

Applies pending Refinery migrations and exits. `service start` and `index sync` already do this, so
this command exists for the case where you want migrations to run in their own controlled step.

For Ordinals this migrates both the Ordinals database and, when BRC-20 is enabled, the BRC-20
database.

## `config new`

```console
bitcoin-indexer config new --mainnet
```

Writes `./Indexer.toml`. Exactly one of `--mainnet`, `--testnet`, `--regtest` is required; the flags
are mutually exclusive, and passing none fails with `Invalid network`. An existing `Indexer.toml` in
the working directory is overwritten without warning.

See the defect note about `--regtest` in [Configuration](configuration.md).

## Signals and shutdown

`SIGINT` (Ctrl-C) is handled. The process logs
`bitcoin-indexer received interrupt signal, shutting down...`, sets an abort flag, lets the current
block finish, and then joins its threads. Prefer this over `SIGKILL` so the Postgres transaction in
flight commits or rolls back cleanly and RocksDB is flushed.

## Exit behaviour

| Situation | Result |
| --- | --- |
| Argument parsing failure | Usage text on stdout, exit code 1 |
| Any command error | Error logged, 500 ms grace for log flushing, exit code 1 |
| Successful completion | Exit code 0 |

## Logging

Logging goes through `hiro-system-kit` and `slog`. There is no `RUST_LOG` handling and no runtime
log level switch: verbosity and format are chosen at build time by Cargo feature.

| Build | Feature | Output | Level |
| --- | --- | --- | --- |
| `cargo build` | `cli` (default) | Human-readable terminal format on stderr | slog default |
| `cargo build --features release --release` | `release` | JSON on stderr, one object per line | `info` and above; debug statements are compiled out |
| `cargo build --features debug` | `debug` | Human-readable terminal format on stderr | up to `trace` |

The container image is built with `--features release --release`, so it emits JSON at `info`. Fork
selection detail and the `Indexer command channel full` backpressure message are logged at `debug`,
which means you need a `debug` build to see them.
