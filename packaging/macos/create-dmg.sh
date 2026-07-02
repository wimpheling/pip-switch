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
stage="${output_dir}/${name}"
app="${stage}/pip-switch.app"

rm -rf "$stage"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources/bin" "$stage/bin"

sed "s/__VERSION__/${version}/g" packaging/macos/Info.plist > "$app/Contents/Info.plist"
cp target/release/pip-switch-daemon "$app/Contents/MacOS/"
cp target/release/pip-switch "$app/Contents/Resources/bin/"
cp target/release/pip-switch "$stage/bin/"
cp README.md "$stage/"

chmod +x "$app/Contents/MacOS/pip-switch-daemon" "$stage/bin/pip-switch"

hdiutil create \
  -volname "pip-switch ${version}" \
  -srcfolder "$stage" \
  -ov \
  -format UDZO \
  "${output_dir}/${name}.dmg"
