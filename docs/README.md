# Bitcoin Indexer documentation

Everything here is written against the code in this repository. If a statement here and the code
disagree, the code is right and this documentation is a bug.

## Start here

| If you want to | Read |
| --- | --- |
| Understand the moving parts | [Architecture](architecture.md) |
| Get it running | [Installation](install.md), then [Configuration](configuration.md) |
| Drive it | [CLI reference](cli.md) |
| Know how long a sync takes | [Synchronization](synchronization.md) |
| Query the data directly | [Database model](database.md) |
| Understand reorg and mempool behaviour | [Reorgs and mempool](reorgs-and-mempool.md) |
| Integrate over HTTP | [API reference](api/README.md) |
| Run it in production | [Operations](operations.md), [Performance and sizing](performance.md) |
| Harden it | [Security](security.md) |
| Change it | [Testing](testing.md), [Releases and versioning](releases.md) |
| Fix something that broke | [Troubleshooting](troubleshooting.md) |
| Know where the code came from | [Upstream relationship](upstream.md) |

## The one-paragraph summary

`bitcoin-indexer` is a Rust binary that reads Bitcoin blocks from Bitcoin Core, extracts Ordinals
inscription state, BRC-20 token state, and Runes state from them, and writes that state into
PostgreSQL. It keeps a small in-memory model of recent block headers so it can detect forks and
roll blocks back out of Postgres when the node changes its mind about the best chain. Two Node.js
Fastify services read those Postgres databases and expose read-only REST endpoints. Nothing in this
repository reads the mempool, signs anything, or writes to Bitcoin.

## Scope boundaries worth stating plainly

- Three protocols only: Ordinals, BRC-20, Runes. Nothing else in the Bitcoin Universe protocol set
  is indexed by this repository.
- Confirmed blocks only. See [Reorgs and mempool](reorgs-and-mempool.md).
- Read-only HTTP surface. See [API reference](api/README.md).
- This repository carries no Bitcoin Universe release tag. See [Releases and versioning](releases.md).
