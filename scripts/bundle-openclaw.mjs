#!/usr/bin/env zx

/**
 * bundle-openclaw.mjs
 *
 * Bundles the openclaw npm package with ALL its dependencies (including
 * transitive ones) into a self-contained directory (build/openclaw/) for
 * Tauri's bundle.resources to pick up.
 *
 * pnpm uses a content-addressable virtual store with symlinks. A naive copy
 * of node_modules/openclaw/ will miss runtime dependencies entirely. Even
 * copying only direct siblings misses transitive deps (e.g. @clack/prompts
 * depends on @clack/core which lives in a separate virtual store entry).
 *
 * This script performs a recursive BFS through pnpm's virtual store to
 * collect every transitive dependency into a flat node_modules structure.
 * It also prunes unnecessary files (type defs, source maps, docs, tests) to
 * shrink the bundle and reduce macOS code-signing overhead — Tauri has no
 * afterPack hook, so the cleanup must happen before Tauri copies resources.
 */

import 'zx/globals';
import crypto from 'node:crypto';

const ROOT = path.resolve(__dirname, '..');
const OUTPUT = path.join(ROOT, 'build', 'openclaw');
const NODE_MODULES = path.join(ROOT, 'node_modules');

echo`📦 Bundling openclaw for Tauri resources...`;

// 1. Resolve the real path of node_modules/openclaw (follows pnpm symlink)
const openclawLink = path.join(NODE_MODULES, 'openclaw');
if (!fs.existsSync(openclawLink)) {
  echo`❌ node_modules/openclaw not found. Run pnpm install first.`;
  process.exit(1);
}

const openclawReal = fs.realpathSync(openclawLink);
echo`   openclaw resolved: ${openclawReal}`;

// 2. Clean and create output directory
if (fs.existsSync(OUTPUT)) {
  fs.rmSync(OUTPUT, { recursive: true });
}
fs.mkdirSync(OUTPUT, { recursive: true });

// 3. Copy openclaw package itself to OUTPUT root
echo`   Copying openclaw package...`;
fs.cpSync(openclawReal, OUTPUT, { recursive: true, dereference: true });

// 4. Recursively collect ALL transitive dependencies via pnpm virtual store BFS
//
// pnpm structure example:
//   .pnpm/openclaw@ver/node_modules/
//     openclaw/          <- real files
//     chalk/             <- symlink -> .pnpm/chalk@ver/node_modules/chalk
//     @clack/prompts/    <- symlink -> .pnpm/@clack+prompts@ver/node_modules/@clack/prompts
//
//   .pnpm/@clack+prompts@ver/node_modules/
//     @clack/prompts/    <- real files
//     @clack/core/       <- symlink (transitive dep, NOT in openclaw's siblings!)
//
// We BFS from openclaw's virtual store node_modules, following each symlink
// to discover the target's own virtual store node_modules and its deps.

const collected = new Map(); // realPath -> packageName (for deduplication)
const queue = []; // BFS queue of virtual-store node_modules dirs to visit

/**
 * Given a real path of a package, find the containing virtual-store node_modules.
 * e.g. .pnpm/chalk@5.4.1/node_modules/chalk -> .pnpm/chalk@5.4.1/node_modules
 * e.g. .pnpm/@clack+core@0.4.1/node_modules/@clack/core -> .pnpm/@clack+core@0.4.1/node_modules
 */
function getVirtualStoreNodeModules(realPkgPath) {
  let dir = realPkgPath;
  while (dir !== path.dirname(dir)) {
    if (path.basename(dir) === 'node_modules') {
      return dir;
    }
    dir = path.dirname(dir);
  }
  return null;
}

/**
 * List all package entries in a virtual-store node_modules directory.
 * Handles both regular packages (chalk) and scoped packages (@clack/prompts).
 * Returns array of { name, fullPath }.
 */
function listPackages(nodeModulesDir) {
  const result = [];
  if (!fs.existsSync(nodeModulesDir)) return result;

  for (const entry of fs.readdirSync(nodeModulesDir)) {
    if (entry === '.bin') continue;

    const entryPath = path.join(nodeModulesDir, entry);
    const stat = fs.lstatSync(entryPath);

    if (entry.startsWith('@')) {
      // Scoped package: read sub-entries
      if (stat.isDirectory() || stat.isSymbolicLink()) {
        const resolvedScope = stat.isSymbolicLink() ? fs.realpathSync(entryPath) : entryPath;
        // Check if this is actually a scoped directory or a package
        try {
          const scopeEntries = fs.readdirSync(entryPath);
          for (const sub of scopeEntries) {
            result.push({
              name: `${entry}/${sub}`,
              fullPath: path.join(entryPath, sub),
            });
          }
        } catch {
          // Not a directory, skip
        }
      }
    } else {
      result.push({ name: entry, fullPath: entryPath });
    }
  }
  return result;
}

// Start BFS from openclaw's virtual store node_modules
const openclawVirtualNM = getVirtualStoreNodeModules(openclawReal);
if (!openclawVirtualNM) {
  echo`❌ Could not determine pnpm virtual store for openclaw`;
  process.exit(1);
}

echo`   Virtual store root: ${openclawVirtualNM}`;
queue.push({ nodeModulesDir: openclawVirtualNM, skipPkg: 'openclaw' });

while (queue.length > 0) {
  const { nodeModulesDir, skipPkg } = queue.shift();
  const packages = listPackages(nodeModulesDir);

  for (const { name, fullPath } of packages) {
    // Skip the package that owns this virtual store entry (it's the package itself, not a dep)
    if (name === skipPkg) continue;

    let realPath;
    try {
      realPath = fs.realpathSync(fullPath);
    } catch {
      continue; // broken symlink, skip
    }

    if (collected.has(realPath)) continue; // already visited
    collected.set(realPath, name);

    // Find this package's own virtual store node_modules to discover ITS deps
    const depVirtualNM = getVirtualStoreNodeModules(realPath);
    if (depVirtualNM && depVirtualNM !== nodeModulesDir) {
      // Determine the package's "self name" in its own virtual store
      // For scoped: @clack/core -> skip "@clack/core" when scanning
      queue.push({ nodeModulesDir: depVirtualNM, skipPkg: name });
    }
  }
}

echo`   Found ${collected.size} total packages (direct + transitive)`;

// 5. Copy all collected packages into OUTPUT/node_modules/ (flat structure)
//
// IMPORTANT: BFS guarantees direct deps are encountered before transitive deps.
// When the same package name appears at different versions (e.g. chalk@5 from
// openclaw directly, chalk@4 from a transitive dep), we keep the FIRST one
// (direct dep version) and skip later duplicates. This prevents version
// conflicts like CJS chalk@4 overwriting ESM chalk@5.
const outputNodeModules = path.join(OUTPUT, 'node_modules');
fs.mkdirSync(outputNodeModules, { recursive: true });

const copiedNames = new Set(); // Track package names already copied
let copiedCount = 0;
let skippedDupes = 0;

for (const [realPath, pkgName] of collected) {
  if (copiedNames.has(pkgName)) {
    skippedDupes++;
    continue; // Keep the first version (closer to openclaw in dep tree)
  }
  copiedNames.add(pkgName);

  const dest = path.join(outputNodeModules, pkgName);

  try {
    // Ensure parent directory exists (for scoped packages like @clack/core)
    fs.mkdirSync(path.dirname(dest), { recursive: true });
    fs.cpSync(realPath, dest, { recursive: true, dereference: true });
    copiedCount++;
  } catch (err) {
    echo`   ⚠️  Skipped ${pkgName}: ${err.message}`;
  }
}

// 6. Prune unnecessary files (type defs, source maps, docs, tests) to shrink
// the bundle and reduce macOS code-signing overhead. Must happen here because
// Tauri has no afterPack hook — the source must already be pruned before the
// Tauri bundler copies it into Resources/.
function cleanupUnnecessaryFiles(dir) {
  let removedCount = 0;

  function walk(currentDir) {
    const entries = fs.readdirSync(currentDir, { withFileTypes: true });

    for (const entry of entries) {
      const fullPath = path.join(currentDir, entry.name);

      if (entry.isDirectory()) {
        if (entry.name === 'test' || entry.name === 'tests' ||
            entry.name === '__tests__' || entry.name === '.github' ||
            entry.name === 'docs' || entry.name === 'examples') {
          try {
            fs.rmSync(fullPath, { recursive: true, force: true });
            removedCount++;
          } catch {
            // ignore
          }
        } else {
          walk(fullPath);
        }
      } else if (entry.isFile()) {
        const name = entry.name;
        if (name.endsWith('.d.ts') || name.endsWith('.d.ts.map') ||
            name.endsWith('.js.map') || name.endsWith('.mjs.map') ||
            name.endsWith('.ts.map') || name === '.DS_Store' ||
            name === 'README.md' || name === 'CHANGELOG.md' ||
            name === 'LICENSE.md' || name === 'CONTRIBUTING.md' ||
            name.endsWith('.md.txt') || name.endsWith('.markdown') ||
            name === 'tsconfig.json' || name === '.npmignore' ||
            name === '.eslintrc' || name === '.prettierrc') {
          try {
            fs.rmSync(fullPath, { force: true });
            removedCount++;
          } catch {
            // ignore
          }
        }
      }
    }
  }

  walk(dir);
  return removedCount;
}

echo`   Pruning type defs, source maps, docs, tests...`;
const removedCount = cleanupUnnecessaryFiles(OUTPUT);
echo`   Pruned ${removedCount} files/directories`;

// 7. Filter koffi multi-platform native binaries. pnpm already filters
// optionalDeps-style packages (@napi-rs/canvas, @img/sharp, ...) per host;
// koffi ships all 18 platform binaries inside build/koffi/<arch>/, so we
// strip the ones we don't need.
const KOFFI_PLATFORM_MAP = {
  'darwin-arm64': 'darwin_arm64',
  'darwin-x64':   'darwin_x64',
  'linux-arm64':  'linux_arm64',
  'linux-x64':    'linux_x64',
  'win32-arm64':  'win32_arm64',
  'win32-x64':    'win32_x64',
};
const hostKey = `${process.platform}-${process.arch}`;
const koffiKeep = KOFFI_PLATFORM_MAP[hostKey];
const koffiBase = path.join(OUTPUT, 'node_modules', 'koffi', 'build', 'koffi');
if (koffiKeep && fs.existsSync(koffiBase)) {
  let dropped = 0;
  for (const entry of fs.readdirSync(koffiBase)) {
    if (entry !== koffiKeep) {
      fs.rmSync(path.join(koffiBase, entry), { recursive: true, force: true });
      dropped++;
    }
  }
  echo`   Filtered koffi → kept ${koffiKeep}, dropped ${dropped} platforms`;
} else if (!koffiKeep) {
  echo`   ⚠️  Unknown host ${hostKey}; leaving koffi platforms untouched`;
}

// 8. Write version fingerprint (openclaw version + bundle-time short hash)
// used by the Rust extractor to decide if re-extraction is needed.
const openclawPkg = JSON.parse(fs.readFileSync(path.join(OUTPUT, 'package.json'), 'utf8'));
const bundleId = `${openclawPkg.version}-${crypto.createHash('sha1').update(String(Date.now())).digest('hex').slice(0, 8)}`;
const versionPath = path.join(ROOT, 'build', 'openclaw.version');
fs.writeFileSync(versionPath, bundleId, 'utf8');
echo`   Wrote version: ${bundleId}`;

// 9. Tar+gzip into a single resource. Using the system tar (BSD/GNU compatible
// flags) — present on macOS/Linux and Win10 1809+. CI on older Windows would
// need to switch to a Node tar package.
const archivePath = path.join(ROOT, 'build', 'openclaw.tar.gz');
if (fs.existsSync(archivePath)) fs.rmSync(archivePath, { force: true });
echo`   Creating openclaw.tar.gz...`;
await $`tar -czf ${archivePath} -C ${path.join(ROOT, 'build')} openclaw`;
const archiveSize = (fs.statSync(archivePath).size / (1024 * 1024)).toFixed(1);
echo`   ✓ openclaw.tar.gz ready (${archiveSize} MB)`;

// 10. Verify the bundle
const entryExists = fs.existsSync(path.join(OUTPUT, 'openclaw.mjs'));
const distExists = fs.existsSync(path.join(OUTPUT, 'dist', 'entry.js'));

echo``;
echo`✅ Bundle complete: ${OUTPUT}`;
echo`   Unique packages copied: ${copiedCount}`;
echo`   Duplicate versions skipped: ${skippedDupes}`;
echo`   Total discovered: ${collected.size}`;
echo`   openclaw.mjs: ${entryExists ? '✓' : '✗'}`;
echo`   dist/entry.js: ${distExists ? '✓' : '✗'}`;
echo`   openclaw.tar.gz: ${archiveSize} MB`;

if (!entryExists || !distExists) {
  echo`❌ Bundle verification failed!`;
  process.exit(1);
}
