# Contributing

Thanks for wanting to improve this indexer. Read this before opening a pull request.

## Decide where the change belongs first

This repository is a fork of
[`hirosystems/bitcoin-indexer`](https://github.com/hirosystems/bitcoin-indexer), and
`components/ord` is vendored from the [`ord`](https://github.com/ordinals/ord) reference
implementation. A fix in the wrong place gets lost at the next merge from upstream.

| Change | Where to send it |
| --- | --- |
| Protocol indexing bug or feature | Prefer upstream `hirosystems/bitcoin-indexer`, then merge here |
| Ordinal theory primitives in `components/ord` | Prefer `ordinals/ord` |
| CI, runners, packaging, documentation | Here |
| Something Bitcoin Universe operates that upstream would not take | Here, and note it in `docs/upstream.md` |

Full context is in [`docs/upstream.md`](docs/upstream.md).

## Development setup

You need Rust 1.85 (pinned in `rust-toolchain.toml`), Node.js 24.19.0, Docker, and a C/C++ toolchain
with LLVM and Clang for the `rocksdb` and `zmq` native builds. See
[`docs/install.md`](docs/install.md).

```console
docker compose -f dockerfiles/docker-compose.dev.postgres.yml up -d
cargo check --workspace
cargo test --workspace
```

For the APIs:

```console
cd api/ordinals   # or api/runes
npm ci
npm run test
```

## Before you open a pull request

Run what CI runs. It is faster than waiting for a queued runner.

```console
# Rust
cargo fmt --all -- --check
cargo bitcoin-indexer-clippy
cargo test --workspace

# APIs, from api/ordinals and api/runes
npm run lint:eslint
npm run lint:prettier
npm run lint:unused-exports
npm run test
```

Add or update tests for every behaviour you change. [`docs/testing.md`](docs/testing.md) describes
what each suite covers.

## Branches and commits

- `develop` is the default and working branch. Branch from it and target it.
- Write [conventional commits](https://www.conventionalcommits.org/). Upstream's release automation
  parses them, and keeping the convention keeps merges from upstream clean.
- Keep the diff scoped to the change. Do not reformat adjacent code.

## Rules specific to this repository

**Migrations are append-only.** Migration files are embedded into the binary at compile time and
Refinery verifies checksums of already applied versions. Never edit a migration that has shipped; add
a new `V<n>__<description>.sql`. See [`docs/database.md`](docs/database.md).

**Route schema changes require regenerating the API contracts.** The OpenAPI 3.1 documents in
`docs/api/` are generated, not hand-edited:

```console
node scripts/generate-openapi-contracts.mjs
```

Commit the regenerated files with the change that caused them.

**CI runs on self-hosted runners only.** Never introduce `ubuntu-latest`, `windows-latest`, or
`macos-latest`. Match the runner label the existing workflows use.

**Keep the documentation true.** If your change makes a statement in `docs/` or `README.md` wrong,
fix it in the same pull request. Every claim in that tree is supposed to trace to code.

## Documentation style

- Plain, direct sentences. Short paragraphs. A table or a diagram beats three paragraphs of prose.
- No placeholder sections, no aspirational features described as shipped, no untested examples.
- Prefer stating a limitation plainly over leaving it unsaid.

## Mutation testing

`.github/workflows/pr-mutants.yml` runs `cargo-mutants` on pull requests. The guidance carried over
from upstream on how to read caught, missed, timeout, and unviable mutants, and when to apply
`#[cfg_attr(test, mutants::skip)]`, is in [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md).

## Reporting bugs

The issue tracker is turned off on this repository, so a bug report goes in the pull request that
fixes it, or upstream when the bug is upstream's. Read
[`docs/troubleshooting.md`](docs/troubleshooting.md) first: a good number of reports are covered
there. See [SUPPORT.md](SUPPORT.md) for where to take questions.

Whichever route you take, include the commit you built, the network, the relevant part of
`Indexer.toml` with credentials removed, and the log output around the failure.

For security issues, none of the above. See [SECURITY.md](SECURITY.md).
