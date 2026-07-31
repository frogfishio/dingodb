#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const ignoredDirectories = new Set([
  ".git",
  "target",
  "node_modules",
  "dist",
  ".astro",
]);

function walk(directory, files = []) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (ignoredDirectories.has(entry.name)) continue;
    const child = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(child, files);
    else if (entry.name.endsWith(".md")) files.push(child);
  }
  return files;
}

function localPath(target) {
  if (
    !target ||
    target.startsWith("#") ||
    target.startsWith("/") ||
    /^[a-z][a-z0-9+.-]*:/i.test(target)
  ) {
    return null;
  }
  const withoutFragment = target.split("#", 1)[0];
  if (!withoutFragment) return null;
  try {
    return decodeURI(withoutFragment);
  } catch {
    return withoutFragment;
  }
}

const failures = [];
for (const file of walk(".")) {
  const text = fs.readFileSync(file, "utf8");
  const targets = [];
  for (const match of text.matchAll(/\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
    targets.push(match[1]);
  }
  for (const match of text.matchAll(/^(\s*\[[^\]]+\]:\s*)(\S+)/gm)) {
    targets.push(match[2]);
  }
  for (const target of targets) {
    const local = localPath(target);
    if (!local) continue;
    const resolved = path.resolve(path.dirname(file), local);
    if (!fs.existsSync(resolved)) failures.push(`${file}: ${target}`);
  }
}

if (failures.length > 0) {
  console.error("Broken local Markdown links:");
  for (const failure of failures) console.error(`  ${failure}`);
  process.exit(1);
}

console.log("Documentation links OK");
