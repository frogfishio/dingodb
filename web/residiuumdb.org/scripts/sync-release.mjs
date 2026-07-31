#!/usr/bin/env node
/**
 * Sync release.json from repository VERSION + git revision.
 * Run from web/residiuumdb.org or via npm run sync-release.
 */
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

const siteRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = join(siteRoot, '..', '..');
const versionPath = join(repoRoot, 'VERSION');
const releasePath = join(siteRoot, 'src', 'data', 'release.json');

if (!existsSync(versionPath)) {
  console.error('VERSION not found at', versionPath);
  process.exit(1);
}

const productVersion = readFileSync(versionPath, 'utf8').trim();
let sourceRevision = 'unknown';
try {
  sourceRevision = execSync('git rev-parse --short HEAD', {
    cwd: repoRoot,
    encoding: 'utf8',
  }).trim();
} catch {
  // leave unknown
}

const release = JSON.parse(readFileSync(releasePath, 'utf8'));
const today = new Date().toISOString().slice(0, 10);

release.productVersion = productVersion;
release.sourceRevision = sourceRevision;
delete release.dingoSourceRevision;
release.generatedAt = today;
release.lastVerified = today;

writeFileSync(releasePath, JSON.stringify(release, null, 2) + '\n');
console.log(`Synced release ${productVersion} @ ${sourceRevision}`);