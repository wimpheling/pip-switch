#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <version> <target-triple> <output-dir>" >&2
  exit 2
fi

version="$1"
target_triple="$2"
output_dir="$3"
package_dir="pip-switch-${version}-${target_triple}"
stage="${output_dir}/${package_dir}"

rm -rf "$stage"
mkdir -p \
  "$stage/bin" \
  "$stage/share/doc/pip-switch" \
  "$stage/share/systemd/user" \
  "$stage/lib/udev/rules.d"

cp target/release/pip-switch "$stage/bin/"
cp target/release/pip-switch-daemon "$stage/bin/"
cp README.md "$stage/share/doc/pip-switch/"
cp packaging/linux/pip-switch-daemon.service "$stage/share/systemd/user/"
cp packaging/udev/60-pip-switch-msi.rules "$stage/lib/udev/rules.d/"

tar -C "$output_dir" -czf "${output_dir}/${package_dir}.tar.gz" "$package_dir"
