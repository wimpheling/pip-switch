#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <tag>" >&2
  exit 2
fi

tag="${1#refs/tags/}"
version="$(packaging/scripts/release-version.sh)"

if [[ "$tag" != "v${version}" ]]; then
  echo "release tag ${tag} does not match Cargo version ${version}; expected v${version}" >&2
  exit 1
fi

echo "$version"
