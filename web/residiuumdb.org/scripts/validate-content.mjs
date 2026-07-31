#!/usr/bin/env node
/**
 * Claim/status validation for residiuumdb.org (WEBSITE_SPEC §5, §10, §15).
 * Fails the build on unknown status values, unknown claim IDs, or
 * verified_for mismatch with release metadata.
 */
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const data = (name) =>
  JSON.parse(readFileSync(join(root, 'src', 'data', name), 'utf8'));

const release = data('release.json');
const vocab = data('status-vocabulary.json');
const claims = data('claims.json');
const capabilities = data('capabilities.json');
const roadmap = data('roadmap.json');

const allowed = new Set(vocab.allowed);
const errors = [];

function checkStatus(status, where) {
  if (!allowed.has(status)) {
    errors.push(`${where}: unknown status "${status}"`);
  }
}

const claimIds = new Set();
for (const claim of claims) {
  if (!claim.id) errors.push('claim missing id');
  if (claimIds.has(claim.id)) errors.push(`duplicate claim id: ${claim.id}`);
  claimIds.add(claim.id);
  checkStatus(claim.status, claim.id);
  if (claim.verified_for !== release.productVersion) {
    errors.push(
      `${claim.id}: verified_for ${claim.verified_for} != release ${release.productVersion}`,
    );
  }
}

function walkCaps(list, label) {
  for (const item of list) {
    checkStatus(item.status, `${label}:${item.id || item.name}`);
    if (item.claim_id) {
      if (!claimIds.has(item.claim_id)) {
        errors.push(`${label}:${item.id} references unknown claim ${item.claim_id}`);
      }
    }
  }
}

walkCaps(capabilities.deploymentProfiles, 'profile');
walkCaps(capabilities.worksToday, 'worksToday');
walkCaps(capabilities.beingBuilt, 'beingBuilt');

for (const track of roadmap.tracks) {
  checkStatus(track.status, `roadmap:${track.id}`);
  if (track.status === 'available') {
    errors.push(`roadmap:${track.id} must not use Available badge`);
  }
}

if (capabilities.verified_for !== release.productVersion) {
  errors.push(
    `capabilities.verified_for ${capabilities.verified_for} != release ${release.productVersion}`,
  );
}

/** Positive prohibited phrases (WEBSITE_SPEC §5.4). Negated forms are allowed. */
const prohibited = [
  { re: /\bindestructible\b/i, allowIf: /\bnot\s+indestructible\b/i },
  { re: /\bunbreakable\b/i, allowIf: /\bnot\s+unbreakable\b/i },
  { re: /\bcannot lose data\b/i },
  { re: /\balways survives\b/i },
  { re: /\bRedis-fast\b/i },
  { re: /\bRedis-class\b/i },
  { re: /\bfaster than MongoDB\b/i },
  { re: /\benterprise-ready\b/i },
  { re: /\bbattle-tested\b/i },
  { re: /\bcloud-native object storage\b/i },
  { re: /\bfull transactions\b/i },
  { re: /\bsecure by mathematics\b/i },
  { re: /\bmathematically impossible to breach\b/i },
  {
    re: /\bauthenticated continuation tokens\b/i,
    allowIf: /do not claim[\s\S]{0,40}authenticated continuation tokens/i,
  },
];

function walk(dir, files = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    const st = statSync(p);
    if (st.isDirectory()) walk(p, files);
    else if (/\.(astro|md|mdx|html)$/.test(name)) files.push(p);
  }
  return files;
}

const pages = walk(join(root, 'src'));
for (const file of pages) {
  const text = readFileSync(file, 'utf8');
  // production-ready: only flag if not negated nearby
  for (const m of text.match(/[^.!?\n]{0,40}production-ready[^.!?\n]{0,40}/gi) || []) {
    if (!/\bnot\b/i.test(m) && !/\bno\b/i.test(m)) {
      errors.push(`${file}: prohibited language: ${m.trim()}`);
    }
  }
  for (const m of text.match(/[^.!?\n]{0,40}production cluster[^.!?\n]{0,40}/gi) || []) {
    if (!/\bnot\b/i.test(m) && !/\bno\b/i.test(m) && !/Do not choose/i.test(m)) {
      errors.push(`${file}: prohibited language: ${m.trim()}`);
    }
  }
  for (const { re, allowIf } of prohibited) {
    if (re.test(text)) {
      if (allowIf && allowIf.test(text)) continue;
      errors.push(`${file}: matches prohibited pattern ${re}`);
    }
  }
}

if (errors.length) {
  console.error('Content validation failed:\n' + errors.map((e) => `  - ${e}`).join('\n'));
  process.exit(1);
}

console.log(`OK: ${claimIds.size} claims, statuses valid, release ${release.productVersion}`);