# Bitcoin Indexer

Index Bitcoin meta-protocols (Ordinals inscriptions, BRC-20, and Runes) from a Bitcoin Core node
into Postgres, and read them back over two REST APIs.

This is the Bitcoin Universe fork of [`hirosystems/bitcoin-indexer`](https://github.com/hirosystems/bitcoin-indexer),
Apache-2.0. See [Upstream relationship](docs/upstream.md) for what that means for versions, tags,
and support.

| | |
| --- | --- |
| Language | Rust (workspace, toolchain pinned to 1.85) plus two Node.js APIs |
| Storage | PostgreSQL, one database per index, plus a local RocksDB block archive for Ordinals |
| Chain input | Bitcoin Core over JSON-RPC and ZeroMQ |
| Protocols indexed | Ordinals inscriptions, BRC-20, Runes |
| Networks | Ordinals and BRC-20: `mainnet`, `testnet`, `signet`, `devnet` (regtest). Runes: `mainnet` only. |
| Lifecycle in Universe | Experimental. No Bitcoin Universe release tag exists yet. |

## What this is

A block-driven indexer. It reads Bitcoin blocks in order, derives protocol state from them, and
writes that state into Postgres tables that the two bundled read APIs serve. It tracks chain forks
in memory and rolls indexed blocks back out of Postgres when the node reorganizes.

## What this is not

- **Not a mempool indexer.** It subscribes only to the `hashblock` ZeroMQ topic. Unconfirmed
  transactions are never read, stored, or served. Every row in every table comes from a mined block.
- **Not a wallet or a signer.** It holds no keys, builds no transactions, and broadcasts nothing.
- **Not a general Bitcoin explorer.** It stores protocol state, not a full transaction index.
- **Not a multi-protocol indexer beyond the three above.** Alkanes, Atomicals, Stamps, TAP, Bitmap,
  and the rest of the Bitcoin Universe protocol set are indexed elsewhere and are out of scope here.
- **Not a writable API.** Every route in both APIs is a `GET`. There are no mutation endpoints,
  and no authentication, because there is nothing to authorize.

## Documentation

| Document | What it answers |
| --- | --- |
| [Architecture](docs/architecture.md) | Components, threads, and how a block becomes a row |
| [Installation](docs/install.md) | Build from source, Docker, system requirements |
| [Configuration](docs/configuration.md) | Every TOML key and every environment variable |
| [CLI reference](docs/cli.md) | Every command, flag, and exit behaviour |
| [Synchronization](docs/synchronization.md) | Initial sync, streaming, start heights, resource use |
| [Database model](docs/database.md) | Tables, keys, and how migrations run |
| [Reorgs and mempool](docs/reorgs-and-mempool.md) | Fork tracking, rollback, confirmation depth |
| [API reference](docs/api/README.md) | Route shape, OpenAPI contracts, caching, errors |
| [Operations](docs/operations.md) | Metrics, health, alerting, backup, upgrade, recovery |
| [Performance and sizing](docs/performance.md) | What to provision and what the knobs do |
| [Security](docs/security.md) | Trust boundaries and what to keep off the public network |
| [Testing](docs/testing.md) | How to run each suite and what it covers |
| [Releases and versioning](docs/releases.md) | What CI does in this fork, and what it does not |
| [Troubleshooting](docs/troubleshooting.md) | Organized by the symptom you actually see |
| [Upstream relationship](docs/upstream.md) | Fork provenance, vendored code, attribution |

## Quick start

You need a fully synced Bitcoin Core node with JSON-RPC and ZeroMQ `hashblock` enabled, and a
PostgreSQL server. Full detail is in [Installation](docs/install.md).

```console
git clone https://github.com/bitcoinuniverseio/bitcoin-indexer.git
cd bitcoin-indexer
cargo bitcoin-indexer-install
```

Generate a configuration file, edit it, then start an index:

```console
bitcoin-indexer config new --mainnet     # writes ./Indexer.toml
bitcoin-indexer ordinals service start --config-path ./Indexer.toml
```

Runes uses the same shape:

```console
bitcoin-indexer runes service start --config-path ./Indexer.toml
```

Then run either read API against the databases the indexer filled. See
[API reference](docs/api/README.md).

## Repository layout

```
components/bitcoind    Block download pipeline, ZeroMQ stream, fork tracking (BlockPool)
components/ord         Vendored subset of the ord reference implementation, v0.22.2
components/ordinals    Inscription indexing, BRC-20 meta-protocol, Postgres writes
components/runes       Runes indexing and Postgres writes
components/config      TOML configuration parsing and defaults
components/postgres    Shared connection pool helpers
components/cli         The `bitcoin-indexer` binary
migrations/            Refinery SQL migrations, one directory per database
api/ordinals           Fastify read API for inscriptions and BRC-20
api/runes              Fastify read API for Runes
docs/                  This documentation
dockerfiles/           Container builds and the development Postgres compose file
```

## Contributing, support, security

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SUPPORT.md](SUPPORT.md)
- [SECURITY.md](SECURITY.md)

## License

Apache-2.0. See [LICENSE](LICENSE).
