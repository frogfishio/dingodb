#!/usr/bin/env node

import {
  mkdir,
  readdir,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(root, 'dist');
const assets = join(dist, 'assets');
const server = join(dist, 'server');

await rm(assets, { recursive: true, force: true });
await rm(server, { recursive: true, force: true });
await mkdir(assets, { recursive: true });

for (const entry of await readdir(dist)) {
  if (entry === 'assets' || entry === 'server' || entry === '.openai') continue;
  await rename(join(dist, entry), join(assets, entry));
}

await mkdir(server, { recursive: true });
await writeFile(
  join(server, 'index.js'),
  `async function asset(request, env, pathname) {
  return env.ASSETS.fetch(new Request(new URL(pathname, request.url), request));
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    let response = await env.ASSETS.fetch(request);
    if (response.status !== 404) return response;

    const path = url.pathname.endsWith("/")
      ? url.pathname + "index.html"
      : url.pathname + "/index.html";
    response = await asset(request, env, path);
    if (response.status !== 404) return response;

    const notFound = await asset(request, env, "/404.html");
    return new Response(notFound.body, {
      status: 404,
      headers: notFound.headers,
    });
  },
};
`,
  'utf8',
);

console.log('Prepared static Astro output for Sites.');
