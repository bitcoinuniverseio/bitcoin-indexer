# Upstream relationship

## What this repository is

`bitcoinuniverseio/bitcoin-indexer` is a GitHub fork of
[`hirosystems/bitcoin-indexer`](https://github.com/hirosystems/bitcoin-indexer), licensed
Apache-2.0. The fork relationship is recorded on the repository itself, and the Apache-2.0 `LICENSE`
file at the root is the upstream license carried forward unchanged.

The upstream project was previously named `ordhook`. Traces of that name are still in the tree and
are not mistakes:

- `ordhook.code-workspace` at the root
- the `ORDHOOK_MAINTENANCE` and `ORDHOOK_TEST_PG_*` environment variables
- the RocksDB directory name `hord.rocksdb`

## What Bitcoin Universe changed

The fork is a maintenance and infrastructure fork, not a functional divergence. The commits made
here since the fork point are, in substance:

| Area | Change |
| --- | --- |
| CI runners | All workflows moved off GitHub-hosted runners onto the self-hosted and RunsOn fleet |
| CI gating | Upstream-only jobs (semantic release, Docker Hub publishing, Vercel deployment) are gated off for this repository, because they need upstream credentials this fork does not have |
| CI additions | A production Rust gate and a changed-source hygiene check |
| Toolchain pinning | Node pinned to 24.19.0, Rust coverage tooling pinned to 1.85 |
| Performance | Batched block input lookups in the Ordinals index |
| Tests | Isolated PostgreSQL fixtures for the Ordinals suite |
| Documentation | This `docs/` tree, the rewritten README, and the OpenAPI 3.1 contracts under `docs/api/` |

Protocol logic, the database schema, and the API surface are upstream's.

## Versions and tags

**The version tags in this repository are upstream's release line, not Bitcoin Universe releases.**
Tags `v2.2.x` and `v3.0.x`, the `CHANGELOG.md` entries, and the `3.0.0` version in
`Cargo.toml` all came across with the fork and refer to upstream releases.

Bitcoin Universe has cut no release from this repository. The `docs.manifest.json` therefore records
`lifecycle: experimental` with no `releasedRef` and no `releaseVersion`, and an `upstream` block
naming the upstream project, its license, and the fork relationship. Do not read an upstream tag as
a statement that Bitcoin Universe has validated that commit.

See [Releases and versioning](releases.md) for what CI does and does not do here.

## Vendored code

`components/ord` is a trimmed copy of the
[`ord`](https://github.com/ordinals/ord) reference implementation, declaring version `0.22.2` in its
`Cargo.toml`. It supplies sat arithmetic, rarity, degree and epoch maths, inscription envelope
parsing, charms, and media type classification. It was vendored by upstream, not by this fork, and it
carries no separate license header inside this repository. Treat it as third-party code: prefer
upstreaming fixes to `ord` over patching the copy.

`components/runes` depends on the published `ordinals` crate (version `0.0.15`, aliased as
`ordinals-parser`) for runestone parsing.

## Where to send changes

| Change | Where |
| --- | --- |
| Protocol indexing bug or feature | Prefer [`hirosystems/bitcoin-indexer`](https://github.com/hirosystems/bitcoin-indexer), then merge upstream here |
| Ordinal theory primitives in `components/ord` | Prefer [`ordinals/ord`](https://github.com/ordinals/ord) |
| CI, runners, packaging, documentation | Here |
| Anything Bitcoin Universe operates that upstream would not take | Here, and note the divergence in this file |

Upstream security fixes arrive through a merge from upstream rather than through a release of this
repository. See [Security](security.md).

## Attribution

The upstream authors are credited in the Apache-2.0 `LICENSE`, in `CHANGELOG.md`, and in the
`package.json` author fields of both API services. Keep those intact.
