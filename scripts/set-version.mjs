#!/usr/bin/env node
/**
 * Bump package / Tauri / Cargo version for CI releases.
 * Usage: node scripts/set-version.mjs 0.1.42
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error('usage: node scripts/set-version.mjs <semver>');
  process.exit(1);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

function replaceFile(rel, replacer) {
  const file = path.join(root, rel);
  const before = fs.readFileSync(file, 'utf8');
  const after = replacer(before);
  if (after === before) {
    throw new Error(`version not updated in ${rel}`);
  }
  fs.writeFileSync(file, after);
  console.log(`updated ${rel} → ${version}`);
}

replaceFile('package.json', (s) =>
  s.replace(/"version":\s*"[^"]+"/, `"version": "${version}"`),
);
replaceFile('src-tauri/tauri.conf.json', (s) =>
  s.replace(/"version":\s*"[^"]+"/, `"version": "${version}"`),
);
replaceFile('src-tauri/Cargo.toml', (s) =>
  s.replace(/^version = "[^"]+"/m, `version = "${version}"`),
);
replaceFile('src-tauri/Cargo.lock', (s) =>
  s.replace(/(name = "app"\n)version = "[^"]+"/, `$1version = "${version}"`),
);
