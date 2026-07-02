#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: $0 <version> <target-triple> <output-dir>" >&2
  exit 2
fi

version="$1"
target_triple="$2"
output_dir="$3"
name="pip-switch-${version}-${target_triple}"
work_dir="${output_dir}/${name}-pkg"
root_dir="${work_dir}/root"
component_pkg="${work_dir}/${name}-component.pkg"
product_pkg="${output_dir}/${name}.pkg"

rm -rf "$work_dir" "$product_pkg"
mkdir -p \
  "$root_dir/usr/local/bin" \
  "$root_dir/usr/local/share/doc/pip-switch"

cp target/release/pip-switch "$root_dir/usr/local/bin/"
cp README.md "$root_dir/usr/local/share/doc/pip-switch/README.md"

chmod +x "$root_dir/usr/local/bin/pip-switch"

if [[ "${PIP_SWITCH_PKG_DRY_RUN:-}" == "1" ]]; then
  find "$root_dir" -type f | sort
  exit 0
fi

pkgbuild \
  --root "$root_dir" \
  --identifier "dev.pip-switch" \
  --version "$version" \
  --install-location "/" \
  --scripts packaging/macos/scripts \
  "$component_pkg"

productbuild \
  --package "$component_pkg" \
  "$product_pkg"
