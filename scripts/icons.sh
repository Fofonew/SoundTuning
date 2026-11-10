#!/usr/bin/env bash
# Regenerates the application icons from the SVG sources (macOS only: needs rsvg-convert, iconutil, python3).
set -euo pipefail

icon_dir="$(cd "$(dirname "$0")/../crates/app/assets/icon" && pwd)"
work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

render() {
  rsvg-convert --width "$2" --height "$2" "$icon_dir/$1" --output "$3"
}

iconset="$work_dir/SoundTuning.iconset"
mkdir "$iconset"
for size in 16 32 128 256 512; do
  render soundtuning-macos.svg "$size" "$iconset/icon_${size}x${size}.png"
  render soundtuning-macos.svg "$((size * 2))" "$iconset/icon_${size}x${size}@2x.png"
done
iconutil --convert icns "$iconset" --output "$icon_dir/SoundTuning.icns"

ico_pngs=()
for size in 16 24 32 48 64 128 256; do
  render soundtuning.svg "$size" "$work_dir/ico_$size.png"
  ico_pngs+=("$work_dir/ico_$size.png")
done
python3 -I - "$icon_dir/SoundTuning.ico" "${ico_pngs[@]}" <<'PY'
import struct, sys
out, *paths = sys.argv[1:]
images = [open(path, "rb").read() for path in paths]
header = struct.pack("<HHH", 0, 1, len(images))
offset = len(header) + 16 * len(images)
entries, data = b"", b""
for png in images:
    width, height = struct.unpack(">II", png[16:24])
    entries += struct.pack("<BBBBHHII", width % 256, height % 256, 0, 0, 1, 32, len(png), offset + len(data))
    data += png
open(out, "wb").write(header + entries + data)
PY

render soundtuning.svg 256 "$icon_dir/soundtuning-256.png"
echo "icons written to $icon_dir"
