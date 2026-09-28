#!/bin/bash
set -e

REQUIRED=(
  Cargo.toml
  src/lib.rs
  src/commands.rs
  src/error.rs
  permissions/default.toml
  guest-js/package.json
  guest-js/tsconfig.json
  guest-js/src/index.ts
  README.md
  LICENSE
  .gitignore
)

echo "==> Checking required files"
for f in "${REQUIRED[@]}"; do
  [ -f "$f" ] || { echo "MISSING: $f"; exit 1; }
done
echo "  all files present"

if command -v cargo >/dev/null 2>&1; then
  echo "==> cargo check"
  cargo check --quiet
  echo "  cargo check OK"
fi

if command -v bun >/dev/null 2>&1; then
  echo "==> guest-js typecheck + build + test"
  (cd guest-js && bun install --silent && bun run typecheck && bun run build && bun test)
  if [ ! -f guest-js/dist/index.js ] || [ ! -f guest-js/dist/index.mjs ]; then
    echo "ERROR: guest-js dual ESM/CJS build missing"
    exit 1
  fi
  echo "  guest-js build OK"
fi

echo "==> All checks passed"
