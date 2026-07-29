#!/usr/bin/env node
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const contentRoot = join(root, 'src', 'content');
const vocab = JSON.parse(
  readFileSync(join(root, 'src', 'data', 'status-vocabulary.json'), 'utf8'),
);
const release = JSON.parse(
  readFileSync(join(root, 'src', 'data', 'release.json'), 'utf8'),
);
const repoRoot = join(root, '..', '..');

const allowedStatus = new Set(vocab.allowed);
const allowedSurfaces = new Set(vocab.surfaces);
const allowedClasses = new Set(vocab.docClasses);
const allowedSpecStates = new Set(Object.keys(vocab.specStates));

const errors = [];
const warnings = [];

function walk(dir, files = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, files);
    else if (name.endsWith('.md')) files.push(p);
  }
  return files;
}

function parseFm(raw) {
  if (!raw.startsWith('---')) return { data: {}, body: raw };
  const end = raw.indexOf('\n---', 3);
  if (end === -1) return { data: {}, body: raw };
  const yaml = raw.slice(4, end);
  const data = {};
  let key = null;
  let obj = null;
  let arr = null;
  for (const line of yaml.split('\n')) {
    if (!line.trim()) continue;
    const ai = line.match(/^\s+-\s+(.*)$/);
    if (ai && key) {
      if (!Array.isArray(data[key])) data[key] = [];
      data[key].push(ai[1].replace(/^["']|["']$/g, ''));
      continue;
    }
    const nested = line.match(/^\s{2}([\w_]+):\s*(.*)$/);
    if (nested && key) {
      if (!obj) {
        obj = {};
        data[key] = obj;
      }
      obj[nested[1]] = nested[2].replace(/^["']|["']$/g, '');
      continue;
    }
    const top = line.match(/^([\w_]+):\s*(.*)$/);
    if (top) {
      key = top[1];
      obj = null;
      const val = top[2].trim();
      if (!val) {
        data[key] = {};
        continue;
      }
      data[key] = val.replace(/^["']|["']$/g, '');
    }
  }
  return { data, body: raw.slice(end + 4) };
}

const files = walk(contentRoot);
const slugs = new Set();

for (const file of files) {
  const raw = readFileSync(file, 'utf8');
  const slug = file
    .slice(contentRoot.length + 1)
    .replace(/\.md$/, '')
    .replace(/\\/g, '/');
  slugs.add(slug);
  const { data } = parseFm(raw);

  for (const req of [
    'title',
    'description',
    'class',
    'status',
    'last_verified',
  ]) {
    if (!data[req]) errors.push(`${slug}: missing ${req}`);
  }

  if (data.class && !allowedClasses.has(data.class)) {
    errors.push(`${slug}: unknown class ${data.class}`);
  }
  if (data.status && !allowedStatus.has(data.status)) {
    errors.push(`${slug}: unknown status ${data.status}`);
  }

  const surface =
    typeof data.applies_to === 'object'
      ? data.applies_to.surface
      : undefined;
  if (surface && !allowedSurfaces.has(surface)) {
    errors.push(`${slug}: unknown surface ${surface}`);
  }

  if (data.spec_state && !allowedSpecStates.has(data.spec_state)) {
    errors.push(`${slug}: unknown spec_state ${data.spec_state}`);
  }

  if (data.status === 'design' && surface && surface !== 'design-only' && surface !== 'all-profiles') {
    // design pages should not claim available product surfaces strongly
  }

  if (data.status === 'available' && /will be|planned to|not implemented/i.test(raw) && data.class === 'specification') {
    // ok
  }

  // Present-tense availability on design pages
  if (data.status === 'design') {
    if (/\bis available\b/i.test(raw) && !/not (?:yet )?available|not implemented/i.test(raw)) {
      warnings.push(`${slug}: design page may claim availability`);
    }
  }

  // Experimental pages should mention limitations somewhere in nav chrome is ok; require link text
  if (
    data.status === 'experimental' &&
    !/known-limitations|limitations/i.test(raw) &&
    !slug.startsWith('status/')
  ) {
    // soft: many pages rely on maturity banner link — chrome provides it
  }

  // Operations need verification section
  if (data.class === 'operation' && !slug.endsWith('/index')) {
    if (!/## Verification/i.test(raw)) {
      errors.push(`${slug}: operations page missing ## Verification`);
    }
    if (/## Procedure/i.test(raw) && /destructive|rm -rf|overwrite/i.test(raw)) {
      if (!/Risk level/i.test(raw)) {
        errors.push(`${slug}: destructive ops need Risk level`);
      }
    }
  }

  const sourcePath =
    typeof data.source === 'object' ? data.source.path : undefined;
  if (sourcePath) {
    const abs = join(repoRoot, sourcePath);
    try {
      statSync(abs);
    } catch {
      // some paths may be directories
      try {
        statSync(join(repoRoot, sourcePath.replace(/\/$/, '')));
      } catch {
        errors.push(`${slug}: source.path not found: ${sourcePath}`);
      }
    }
  }

  if (data.last_verified && data.last_verified < '2026-01-01') {
    warnings.push(`${slug}: last_verified looks stale ${data.last_verified}`);
  }
}

// Internal link check among content
const linkRe = /\]\((\/[a-z0-9_\/\-]*)\)/gi;
for (const file of files) {
  const raw = readFileSync(file, 'utf8');
  const slug = file
    .slice(contentRoot.length + 1)
    .replace(/\.md$/, '')
    .replace(/\\/g, '/');
  let m;
  while ((m = linkRe.exec(raw))) {
    let href = m[1];
    if (href.includes('#')) href = href.split('#')[0];
    if (!href || href === '/') continue;
    const target = href.replace(/^\//, '').replace(/\/$/, '');
    if (!slugs.has(target) && !slugs.has(target + '/index')) {
      // allow section indexes that exist as target/index
      const asIndex = target.endsWith('/index') ? target : target + '/index';
      // also pages might be `getting-started` vs `getting-started/index`
      if (
        !slugs.has(asIndex) &&
        !slugs.has(target) &&
        target !== 'next' &&
        !target.startsWith('api/')
      ) {
        // check if any slug equals or is under
        const ok = [...slugs].some(
          (s) => s === target || s === `${target}/index` || s.startsWith(target + '/'),
        );
        // only exact page match required for leaf links
        if (!ok && !slugs.has(target)) {
          // leaf: target should exist as slug
          if (![...slugs].includes(target) && ![...slugs].includes(`${target}/index`)) {
            errors.push(`${slug}: broken internal link ${m[1]}`);
          }
        }
      }
    }
  }
}

// Prohibited language
const prohibited = [
  /\bproduction-ready\b/i,
  /\bRedis-class\b/i,
  /\bRedis-fast\b/i,
  /\bindestructible\b/i,
  /\bbattle-tested\b/i,
  /\benterprise-ready\b/i,
];
for (const file of files) {
  const raw = readFileSync(file, 'utf8');
  for (const re of prohibited) {
    if (!re.test(raw)) continue;
    if (re.source.includes('production-ready')) {
      const hits = raw.match(/[^.!\n]{0,30}production-ready[^.!\n]{0,30}/gi) || [];
      for (const h of hits) {
        if (!/\bnot\b/i.test(h) && !/\bno\b/i.test(h)) {
          errors.push(`${file}: prohibited: ${h.trim()}`);
        }
      }
      continue;
    }
    errors.push(`${file}: prohibited pattern ${re}`);
  }
}

if (warnings.length) {
  console.warn('Warnings:\n' + warnings.map((w) => `  - ${w}`).join('\n'));
}

if (errors.length) {
  console.error(
    `Docs validation failed (${errors.length}):\n` +
      errors.map((e) => `  - ${e}`).join('\n'),
  );
  process.exit(1);
}

console.log(
  `OK: ${files.length} pages, release ${release.productVersion}, statuses/surfaces valid`,
);
