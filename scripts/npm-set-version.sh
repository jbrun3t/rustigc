#!/bin/sh
# cargo-release pre-release-hook for rustigc-wasm. so npm inherit the version

set -e
# cargo-release calls the hook on a dry run too.
if [ "$DRY_RUN" = true ]; then
    echo "would set the npm versions to $NEW_VERSION"
    exit 0
fi
cd "$CRATE_ROOT"
npm pkg set version="$NEW_VERSION" --include-workspace-root -w utils -w score
npm pkg set peerDependencies.rustigc-wasm="^$NEW_VERSION" -w utils
npm pkg set dependencies.rustigc-wasm="^$NEW_VERSION" dependencies.rustigc-wasm-utils="^$NEW_VERSION" -w score
