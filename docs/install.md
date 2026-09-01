# Installation

## Dependencies you must provide

### Bitcoin Core

A fully synced node with both interfaces enabled.

- **JSON-RPC.** The pipeline calls `getblockchaininfo`, `getblockhash`, and
  `getblock <hash> 3`. Verbosity level 3 returns previous-output data for every input, which is what
  lets the indexer resolve inscription movements without a separate transaction index. That
  verbosity level exists in Bitcoin Core 25.0 and later, so run 25.0 or newer.
- **ZeroMQ.** Only the `hashblock` topic is used. `rawblock`, `rawtx`, and `hashtx` are not read.

A matching `bitcoin.conf` fragment:

```ini
server=1
rpcuser=indexer
rpcpassword=change-me
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
zmqpubhashblock=tcp://127.0.0.1:18543
```

The indexer waits until `getblockchaininfo` reports `initial_block_download = false` and
`blocks == headers` on ten consecutive polls, one second apart, before it starts. Until then it logs
`bitcoind has not reached chain tip, trying again...` once per second.

### PostgreSQL

One database per index. With Ordinals, BRC-20, and Runes all enabled you need three:

```sql
CREATE DATABASE ordinals;
CREATE DATABASE brc20;
CREATE DATABASE runes;
```

PostgreSQL 17 or newer is recommended. The development compose file in this repository pins
PostgreSQL 15, which is enough for tests.

The indexer creates and migrates its own tables. It needs a role that can create tables, types, and
indexes in its database.

## System requirements

| Resource | Guidance |
| --- | --- |
| CPU | The indexer parallelizes block download and parsing across cores. Set `resources.cpu_core_available` to the cores you are willing to give it. The internal thread pool uses `cpu_core_available - 2`, with a floor of 1. |
| Memory | 16 GB is a reasonable floor for a mainnet Ordinals index. Peak use is driven by `indexer_channel_capacity` and the two LRU caches. |
| Disk | SSD or NVMe. Postgres holds inscription content as `BYTEA` in the `inscriptions` table, so the Ordinals database grows with total inscribed bytes. The Ordinals RocksDB block archive lives separately under `storage.working_dir`. |
| File descriptors | At least 4096. `resources.ulimit` defaults to 2048 and should be raised alongside the OS limit. |

Sizing detail is in [Performance and sizing](performance.md).

## Build from source

Requires the Rust toolchain pinned in `rust-toolchain.toml` (1.85) and a C/C++ toolchain with LLVM
and Clang available, because `rocksdb` and `zmq` build native code.

```console
git clone https://github.com/bitcoinuniverseio/bitcoin-indexer.git
cd bitcoin-indexer
cargo bitcoin-indexer-install
```

`cargo bitcoin-indexer-install` is an alias defined in `.cargo/config.toml`. It expands to
`cargo install --path components/cli --locked --force`, so the binary lands in `~/.cargo/bin`.

To build without installing:

```console
cargo build --release
```

The release container build uses `cargo build --features release --release`.

Verify the toolchain and dependency graph without linking:

```console
cargo check --workspace
```

## Build with Docker

Three images are defined:

| Dockerfile | Produces |
| --- | --- |
| `dockerfiles/components/bitcoin-indexer.dockerfile` | The Rust indexer binary |
| `dockerfiles/components/ordinals-api.dockerfile` | The Ordinals read API |
| `dockerfiles/components/runes-api.dockerfile` | The Runes read API |

```console
docker build -t bitcoin-indexer -f dockerfiles/components/bitcoin-indexer.dockerfile .
```

The indexer image is a two-stage build on `rust:bullseye`, installing LLVM and Clang 18 plus the
compression libraries RocksDB needs, and the runtime stage is `debian:bullseye-slim`.

This fork does not publish container images. The image tags in `.github/workflows/ci.yaml` point at
the upstream Docker Hub namespace, and the publishing jobs are gated off for this repository. See
[Releases and versioning](releases.md).

## Build the read APIs

Each API is an independent npm workspace. Node.js 24.19.0 is what CI pins.

```console
cd api/ordinals
npm ci
npm run build
npm run start
```

Same shape under `api/runes`. Configuration is by environment variable, documented in
[Configuration](configuration.md).

## First run

```console
bitcoin-indexer config new --mainnet
```

This writes `./Indexer.toml` in the current directory. Edit it before starting anything, then:

```console
bitcoin-indexer ordinals database migrate --config-path ./Indexer.toml
bitcoin-indexer ordinals service start --config-path ./Indexer.toml
```

`service start` also runs migrations on startup, so the explicit `database migrate` step is
optional. Running it separately is useful when you want migrations to happen in a controlled
maintenance window.

> `config new --regtest` writes `network = "regtest"`, which the configuration parser rejects. Use
> `network = "devnet"` for regtest. See [Configuration](configuration.md).
