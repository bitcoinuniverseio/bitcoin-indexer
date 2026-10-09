# Reporting a vulnerability

Use GitHub private vulnerability reporting for this repository:

**https://github.com/bitcoinuniverseio/bitcoin-indexer/security/advisories/new**

It stays private to the maintainers until an advisory is published. The issue tracker on this
repository is turned off, so there is no public place to file one by accident.

Do not put a suspected vulnerability in a pull request, a commit message, or a pull request comment.
All three are public the moment they are written.

## What to include

Enough to reproduce it, and nothing more than you are comfortable sending:

- what you did and what happened, against which command or endpoint;
- what you expected instead, and why the difference matters;
- the commit you built. `Cargo.toml` carries an upstream version number, so the commit hash is the
  useful identifier here.

No proof of exploitation is needed. A clear description of the flaw is enough.

## Send it upstream too, when it belongs there

This repository is a fork of
[`hirosystems/bitcoin-indexer`](https://github.com/hirosystems/bitcoin-indexer), and
`components/ord` is vendored from [`ordinals/ord`](https://github.com/ordinals/ord). A finding in
code this fork has not modified affects everyone running the upstream project, so please report it
upstream as well. [`docs/upstream.md`](docs/upstream.md) records what this fork changes and what it
takes unmodified.

## Scope

In scope:

- The `bitcoin-indexer` binary and everything under `components/`.
- The two read APIs under `api/`.
- The SQL migrations under `migrations/`.
- The build and CI configuration in this repository.

[`docs/security.md`](docs/security.md) records the trust boundaries this software assumes, and is the
right thing to read first if you are unsure whether something is a finding or a stated boundary. In
particular, these are documented properties rather than vulnerabilities:

- Postgres connections are made without TLS.
- The read APIs have no authentication, no rate limiting, and permissive CORS by design. They are
  meant to sit behind a proxy that supplies all three.
- The indexer metrics server binds `0.0.0.0`, and both APIs bind a metrics port and a profiler port
  in their default mode. Restricting those to a private interface is the operator's job.
- `GET /inscriptions/{id}/content` returns arbitrary attacker-chosen bytes under an
  attacker-chosen content type, because that is what an inscription is. Rendering it safely is the
  responsibility of whatever displays it.

Out of scope:

- Any Bitcoin Universe deployment or hosted endpoint. Report those against the service in question.
- Vulnerabilities in Bitcoin Core, PostgreSQL, or other dependencies. Report those to their
  projects.
- Findings that require an attacker who already has shell access to the indexer host or credentials
  for its databases.

## What happens next

Reports are read by the maintainers of this repository. If a fix lands here, it lands on `develop`;
if it belongs upstream, we will say so and point you at the upstream report.
