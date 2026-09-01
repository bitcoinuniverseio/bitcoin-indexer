# Support

## Read these first

Most questions are answered in the repository:

| Question | Document |
| --- | --- |
| How do I run it? | [`docs/install.md`](docs/install.md) and [`docs/configuration.md`](docs/configuration.md) |
| What does this command do? | [`docs/cli.md`](docs/cli.md) |
| Why is it slow, stuck, or failing? | [`docs/troubleshooting.md`](docs/troubleshooting.md) |
| What do the tables mean? | [`docs/database.md`](docs/database.md) |
| What does the API return? | [`docs/api/README.md`](docs/api/README.md) and the OpenAPI documents next to it |
| How does it handle reorgs? Does it see the mempool? | [`docs/reorgs-and-mempool.md`](docs/reorgs-and-mempool.md) |
| What should I monitor? | [`docs/operations.md`](docs/operations.md) |
| Which version is this? | [`docs/releases.md`](docs/releases.md) |

## Where to ask

The issue tracker on this repository is turned off.

| Kind of question | Where |
| --- | --- |
| A bug in protocol indexing logic, or a question about upstream behaviour | [`hirosystems/bitcoin-indexer`](https://github.com/hirosystems/bitcoin-indexer), which is the upstream project |
| Ordinal theory primitives in `components/ord` | [`ordinals/ord`](https://github.com/ordinals/ord) |
| A change you want to make here | Open a pull request. See [CONTRIBUTING.md](CONTRIBUTING.md). |
| A suspected vulnerability | Private reporting only. See [SECURITY.md](SECURITY.md). |
| Bitcoin Universe products and protocols generally | [docs.bitcoinuniverse.io](https://docs.bitcoinuniverse.io/) |

## What is not supported

- **Bitcoin Universe does not operate a public deployment of these APIs.** This repository is source
  code you run yourself. No hosted endpoint here is a supported product surface.
- **There is no service level agreement.** This repository is `experimental` in the Bitcoin Universe
  lifecycle and carries no Bitcoin Universe release tag. See [`docs/releases.md`](docs/releases.md).
- **Transaction status, wallet recovery, and asset support questions do not belong here.** This
  software indexes blocks; it holds no keys and moves no funds.

## Filing a good report

When you do open a pull request or an upstream issue, include:

- the commit you built, since the version number in `Cargo.toml` is upstream's;
- the network, and which index (Ordinals, BRC-20, or Runes);
- the relevant part of `Indexer.toml` with every credential removed;
- the log lines around the failure, and the value of `last_indexed_block_height` from the metrics
  endpoint if the indexer is running.
