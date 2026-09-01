#!/usr/bin/env node
// Regenerates the published OpenAPI 3.1 contracts under docs/api/.
//
// The two Fastify APIs already emit OpenAPI 3.0.3 through `npm run generate:openapi`
// (see api/<name>/util/openapi-generator.ts). This script runs those generators and
// rewrites the result as OpenAPI 3.1, replacing the upstream Hiro branding and the
// upstream public server URL with values that are true for this repository.
//
// Usage:
//   node scripts/generate-openapi-contracts.mjs
//
// Requirements: dependencies installed in api/ordinals and api/runes (`npm ci`).

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const outDir = join(repoRoot, 'docs', 'api');

const APIS = [
  {
    dir: join(repoRoot, 'api', 'ordinals'),
    out: join(outDir, 'ordinals-openapi.json'),
    title: 'Bitcoin Indexer Ordinals API',
    env: {},
    description: [
      'Read-only REST API over the Ordinals and BRC-20 indexes produced by this repository.',
      '',
      'Every operation is a GET. The API holds no keys, signs nothing, and never writes to the',
      'index; it only reads the Postgres databases the Rust indexer fills.',
      '',
      'The server mounts the same routes under two prefixes: `/ordinals/v1` (documented here)',
      'and `/ordinals`. Prefer the versioned prefix.',
      '',
      'Inscription and inscription-transfer responses carry an `ETag`; send `If-None-Match` to',
      'get a `304` when nothing changed.',
      '',
      'This document is generated from the route schemas in `api/ordinals/src`. Regenerate it',
      'with `node scripts/generate-openapi-contracts.mjs`.',
    ].join('\n'),
  },
  {
    dir: join(repoRoot, 'api', 'runes'),
    out: join(outDir, 'runes-openapi.json'),
    title: 'Bitcoin Indexer Runes API',
    // The Runes route modules read ENV at import time, so the generator needs a
    // syntactically valid database configuration even though it never connects.
    env: {
      RUNES_PGHOST: 'localhost',
      RUNES_PGPORT: '5432',
      RUNES_PGUSER: 'postgres',
      RUNES_PGPASSWORD: 'postgres',
      RUNES_PGDATABASE: 'runes',
    },
    description: [
      'Read-only REST API over the Runes index produced by this repository.',
      '',
      'Every operation is a GET. The API holds no keys, signs nothing, and never writes to the',
      'index; it only reads the Postgres database the Rust indexer fills.',
      '',
      'The server mounts the same routes under two prefixes: `/runes/v1` (documented here) and',
      '`/runes`. Prefer the versioned prefix.',
      '',
      'This document is generated from the route schemas in `api/runes/src`. Regenerate it with',
      '`node scripts/generate-openapi-contracts.mjs`.',
    ].join('\n'),
  },
];

function run(cmd, args, cwd, env) {
  execFileSync(cmd, args, {
    cwd,
    stdio: 'inherit',
    shell: process.platform === 'win32',
    env: { ...process.env, ...env },
  });
}

if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });

for (const api of APIS) {
  if (!existsSync(join(api.dir, 'node_modules'))) {
    throw new Error(`Install dependencies first: (cd ${api.dir} && npm ci)`);
  }
  // @hirosystems/api-toolkit reads .git-info at import time.
  if (!existsSync(join(api.dir, '.git-info'))) {
    run(join(api.dir, 'node_modules', '.bin', 'api-toolkit-git-info'), [], api.dir, {});
  }
  run('npm', ['run', 'generate:openapi'], api.dir, api.env);

  const doc = JSON.parse(readFileSync(join(api.dir, 'tmp', 'openapi.json'), 'utf8'));

  doc.openapi = '3.1.0';
  doc.info = {
    title: api.title,
    description: api.description,
    // The path prefix is the only version this interface actually declares.
    version: 'v1',
    license: { name: 'Apache-2.0', identifier: 'Apache-2.0' },
  };
  doc.externalDocs = {
    url: 'https://github.com/bitcoinuniverseio/bitcoin-indexer',
    description: 'Source repository',
  };
  // Deployments are operator-chosen, so the contract is relative to whatever host serves it.
  doc.servers = [{ url: '/', description: 'Root of a deployed API instance' }];

  // The inscription content route replies with the raw inscription bytes under the
  // inscription's own content type. TypeBox describes that as `instanceOf: Uint8Array`,
  // which is a TypeBox extension rather than anything a client can act on.
  const content = doc.paths?.['/ordinals/v1/inscriptions/{id}/content']?.get?.responses?.['200'];
  if (content) {
    content.description = "Raw inscription bytes, served under the inscription's own content type";
    content.content = { '*/*': { schema: { type: 'string', format: 'binary' } } };
  }

  writeFileSync(api.out, `${JSON.stringify(doc, null, 2)}\n`);
  console.log(`wrote ${api.out} (${Object.keys(doc.paths).length} paths)`);
}
