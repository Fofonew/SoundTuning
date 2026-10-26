#!/usr/bin/env bash
set -euo pipefail

version="${1:-}"
if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Usage : scripts/release.sh X.Y.Z   (ex. scripts/release.sh 0.2.0)" >&2
    exit 1
fi

cd "$(git rev-parse --show-toplevel)"
if [[ "$(git branch --show-current)" != "main" ]]; then
    echo "Les releases partent de main." >&2
    exit 1
fi
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Des modifications ne sont pas commitées." >&2
    exit 1
fi
if git rev-parse -q --verify "refs/tags/v$version" >/dev/null; then
    echo "Le tag v$version existe déjà." >&2
    exit 1
fi

perl -0pi -e 's/(\[workspace\.package\][^\[]*?\nversion = ")[^"]*(")/${1}'"$version"'${2}/s' Cargo.toml
cargo check --quiet --workspace
git add Cargo.toml Cargo.lock
git commit --quiet -m "version $version"
git tag -a "v$version" -m "SoundTuning $version"

echo "Version $version prête (commit et tag v$version créés en local)."
echo "Pour publier : git push origin main && git push origin v$version"
