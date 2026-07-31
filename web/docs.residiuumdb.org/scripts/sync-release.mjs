#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

const siteRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = join(siteRoot, '..', '..');
const versionPath = join(repoRoot, 'VERSION');
const releasePath = join(siteRoot, 'src', 'data', 'release.json');

if (!existsSync(versionPath)) {
  console.error('VERSION missing');
  process.exit(1);
}

const productVersion = readFileSync(versionPath, 'utf8').trim();
let rev = 'unknown';
try {
  rev = execSync('git rev-parse --short HEAD', {
    cwd: repoRoot,
    encoding: 'utf8',
  }).trim();
} catch {
  /* keep */
}

const release = JSON.parse(readFileSync(releasePath, 'utf8'));
const today = new Date().toISOString().slice(0, 10);
release.productVersion = productVersion;
release.productLine = productVersion.split('.').slice(0, 2).join('.');
release.sourceRevision = rev;
delete release.dingoSourceRevision;
release.generatedAt = today;
release.lastVerified = today;
writeFileSync(releasePath, JSON.stringify(release, null, 2) + '\n');
console.log(`Synced docs release ${productVersion} @ ${rev}`);