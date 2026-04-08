#!/bin/bash
set -e
VERSION="$1"

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

echo "Updating versions to $VERSION..."

# Update Cargo.toml - only the [package] section version (first occurrence)
sed -i "0,/^version = /s/version = \".*\"/version = \"$VERSION\"/" klask-rs/Cargo.toml
grep -q "version = \"$VERSION\"" klask-rs/Cargo.toml || { echo "❌ Failed to update Cargo.toml"; exit 1; }
echo "  ✅ klask-rs/Cargo.toml"

# Update package.json version using node
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('klask-react/package.json', 'utf8'));
pkg.version = process.argv[1];
fs.writeFileSync('klask-react/package.json', JSON.stringify(pkg, null, 2) + '\n');
" "$VERSION"
node -e "const p=require('./klask-react/package.json'); if(p.version !== process.argv[1]) process.exit(1);" "$VERSION" || { echo "❌ Failed to update package.json"; exit 1; }
echo "  ✅ klask-react/package.json"

# Update Chart.yaml version and appVersion
sed -i "s/^version:.*/version: $VERSION/" charts/klask/Chart.yaml
sed -i "s/^appVersion:.*/appVersion: \"$VERSION\"/" charts/klask/Chart.yaml
grep -q "^version: $VERSION" charts/klask/Chart.yaml || { echo "❌ Failed to update Chart.yaml version"; exit 1; }
echo "  ✅ charts/klask/Chart.yaml"

echo "✅ All versions updated to $VERSION"
