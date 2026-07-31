#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const ignoredDirectories = new Set([
  ".git",
  "target",
  "node_modules",
  "dist",
  ".astro",
  "tools",
]);

const ignoredFiles = new Set([
  "scripts/check-residiuum-identity.mjs",
  "doc/done/rebrand/REBRAND.md",
  "doc/done/rebrand/REBRAND_CHANGELOG.md",
  "doc/done/rebrand/REBRAND_INVENTORY.md",
  "doc/done/rebrand/REBRAND_PROTOCOL_IDENTITY_RESET.md",
  "doc/done/rebrand/WEB_REBRAND_INVENTORY.md",
  "web/docs.residiuumdb.org/public/_redirects",
  "web/docs.residiuumdb.org/package-lock.json",
  "web/residiuumdb.org/package-lock.json",
]);

const textExtensions = new Set([
  "",
  ".css",
  ".html",
  ".js",
  ".json",
  ".md",
  ".mjs",
  ".rs",
  ".scss",
  ".sh",
  ".toml",
  ".ts",
  ".txt",
  ".yaml",
  ".yml",
]);

const forbidden = [
  ["former Dingo identity", /dingo(?!db)/i],
  ["former DQL identity", /dql/i],
  ["former DRE identity", /DRE|Dre|(?<![A-Za-z0-9])dre(?![A-Za-z0-9])/],
  ["incorrect Residuum spelling", /residuum(?!db)/i],
  ["former .dingo media suffix", /\.dingo\b/i],
  ["former identity encoded as hex", /64696e676f|44494e474f/i],
  [
    "former D-prefixed protocol magic",
    /D(?:QRY0002|HYDRA01|CAT0001|CHM0001|SEGC001|TK00001|DED0001|IDX000[123]|CHKPT01|TIER001|MPC0001|SIX0001|CHIMR01|VL1|CSR0002|ENV0001)/,
  ],
];

function walk(directory, files = []) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (ignoredDirectories.has(entry.name)) continue;
    const child = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      walk(child, files);
    } else if (textExtensions.has(path.extname(entry.name))) {
      files.push(child);
    }
  }
  return files;
}

const failures = [];
for (const file of walk(".")) {
  const normalized = file.replaceAll(path.sep, "/").replace(/^\.\//, "");
  if (ignoredFiles.has(normalized)) continue;
  const text = fs.readFileSync(file, "utf8");
  for (const [label, pattern] of forbidden) {
    const match = text.match(pattern);
    if (!match) continue;
    const line = text.slice(0, match.index).split("\n").length;
    failures.push(`${normalized}:${line}: ${label}: ${JSON.stringify(match[0])}`);
  }
}

if (failures.length > 0) {
  console.error("Former or misspelled product identities found:");
  for (const failure of failures) console.error(`  ${failure}`);
  process.exit(1);
}

console.log("Residiuum identity reset clean");
