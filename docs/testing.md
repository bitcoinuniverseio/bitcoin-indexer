# Testing

Three suites, all needing a PostgreSQL instance except the pure unit tests.

| Suite | Location | Runner |
| --- | --- | --- |
| Rust workspace | `components/**` | `cargo test --workspace` |
| Ordinals API | `api/ordinals/tests` | `jest --runInBand` |
| Runes API | `api/runes/tests` | `jest --runInBand` |

## Start the test database

Every suite that touches Postgres expects one on `localhost:5432` with user `postgres`, password
`postgres`, database `postgres`.

```console
docker compose -f dockerfiles/docker-compose.dev.postgres.yml up -d
```

Tear it down with:

```console
docker compose -f dockerfiles/docker-compose.dev.postgres.yml down -v -t 0
```

Override the connection for the Rust suite with `ORDHOOK_TEST_PG_HOST`, `ORDHOOK_TEST_PG_PORT`,
`ORDHOOK_TEST_PG_USER`, `ORDHOOK_TEST_PG_PASSWORD`, and `ORDHOOK_TEST_PG_DATABASE`.

## Rust suite

```console
cargo test --workspace
```

`.cargo/config.toml` sets `RUST_TEST_THREADS = "1"`, so tests run serially. That is deliberate:
several of them share the same test database.

Around 290 test functions across the workspace. Coverage by component:

| Component | What is covered |
| --- | --- |
| `components/bitcoind` | Fork tracking in `block_pool`, including reorg scenarios and orphan handling; block cursor and byte-level parsing; height range helpers |
| `components/ord` | Sat arithmetic, rarity, degree, epoch, height, inscription envelope parsing, inscription ids, charms, media classification |
| `components/ordinals` | Inscription sequencing and indexing, BRC-20 verification and cache behaviour, Postgres reads and writes, Prometheus metric recording |
| `components/runes` | Rune parsing and validation, ledger and balance cache behaviour, Postgres reads and writes |

To run one component:

```console
cargo test -p ordinals
cargo test -p bitcoind
```

There is also a containerized runner that builds the Docker build stage and runs the whole suite
inside it, which is the closest match to CI:

```console
./scripts/run-tests.sh
```

It starts Postgres, builds the image, runs `cargo test --workspace --no-fail-fast`, and cleans up.

## API suites

Each API is its own npm project.

```console
cd api/ordinals
npm ci
npm run test
```

Targeted runs in `api/ordinals`:

```console
npm run test:api      # ./tests/api
npm run test:brc-20   # ./tests/brc-20
```

`api/ordinals/tests` covers inscriptions, satoshis, statistics, status, ETag cache behaviour, and
the ordinal satoshi calculations, plus a BRC-20 suite. `api/runes/tests` covers the API surface and
the configurable result limit driven by `API_RESULTS_MAX_LIMIT`.

The API suites use the same helper scripts CI uses to manage the database:

```console
npm run testenv:run    # start postgres
npm run testenv:logs   # follow its logs
npm run testenv:stop   # stop and remove
```

## Lint, format, and type checks

```console
# Rust
cargo fmt --all -- --check
cargo bitcoin-indexer-clippy

# APIs, from api/ordinals or api/runes
npm run lint:eslint
npm run lint:prettier
npm run lint:unused-exports
npx tsc --noEmit -p tsconfig.json
```

`cargo bitcoin-indexer-clippy` is an alias in `.cargo/config.toml` that runs clippy across tests,
features, and targets with the lint set this project allows. `cargo bitcoin-indexer-fmt` applies the
import grouping this repository uses.

## What CI runs

`.github/workflows/ci.yaml` runs, on self-hosted runners:

| Job | Command |
| --- | --- |
| `api-lint` | ESLint, Prettier, unused exports, for both API suites |
| `api-test` | `npm run test -- --coverage` for both API suites against a Postgres container |
| `rustfmt` | `cargo fmt --check` |
| `clippy` | The project clippy alias |
| `test` | Doc tests and `cargo test` against Postgres |

`.github/workflows/pr-mutants.yml` runs mutation testing on pull requests.
`.github/workflows/universe-production.yml` adds a formatting and source hygiene gate over changed
Rust files.

All workflows target self-hosted runners. Do not introduce `ubuntu-latest`, `windows-latest`, or
`macos-latest`; match the runner label the existing workflows use.

## Regenerating the API contracts

Route schema changes must be reflected in `docs/api/*.json`:

```console
node scripts/generate-openapi-contracts.mjs
```

This requires `npm ci` to have been run in both API directories. See the
[API reference](api/README.md).
