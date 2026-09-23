#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"
binary="${CARGO_TARGET_DIR:-$repo/target}/release/hdsl"

if [[ "${HDSL_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build --release --locked -p hdsl
  "${MDBOOK:-mdbook}" build docs/user
fi

version="$(awk '/^\[workspace.package\]/{section=1;next} section && /^version =/{gsub(/"/, "", $3);print $3;exit}' Cargo.toml)"
arch="$(dpkg --print-architecture)"
if [[ -z "$version" || ( "$arch" != "amd64" && "$arch" != "arm64" ) ]]; then
  echo "Unsupported package version or architecture" >&2
  exit 1
fi
test -x "$binary"
test -f docs/user/book/index.html

mkdir -p dist
stage="$(mktemp -d "${TMPDIR:-/tmp}/hdsl-package.XXXXXX")"
trap 'rm -rf "$stage"' EXIT

portable="$stage/hdsl-$version-linux-$arch"
mkdir -p "$portable"
install -m 0755 "$binary" "$portable/hdsl"
install -m 0644 LICENSE NOTICE crates/ui/ui/assets/hdsl-mark.svg "$portable/"
cp -a docs/user/book "$portable/help"
find "$portable/help" -type d -exec chmod 0755 {} +
find "$portable/help" -type f -exec chmod 0644 {} +
touch "$portable/portable.flag"
tar -C "$stage" -czf "dist/hdsl-$version-linux-$arch-portable.tar.gz" "$(basename "$portable")"

deb="$stage/deb"
mkdir -p "$deb/DEBIAN" "$deb/usr/bin" "$deb/usr/share/hdsl" \
  "$deb/usr/share/doc/hdsl" "$deb/usr/share/applications" \
  "$deb/usr/share/icons/hicolor/scalable/apps"
install -m 0755 "$binary" "$deb/usr/bin/hdsl"
cp -a docs/user/book "$deb/usr/share/hdsl/help"
find "$deb/usr/share/hdsl/help" -type d -exec chmod 0755 {} +
find "$deb/usr/share/hdsl/help" -type f -exec chmod 0644 {} +
install -m 0644 LICENSE "$deb/usr/share/doc/hdsl/copyright"
install -m 0644 NOTICE "$deb/usr/share/doc/hdsl/NOTICE"
install -m 0644 crates/ui/ui/assets/hdsl-mark.svg \
  "$deb/usr/share/icons/hicolor/scalable/apps/hdsl.svg"
cat > "$deb/usr/share/applications/hdsl.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=HDSL
Comment=DeepSeek Harness launcher
Exec=hdsl
Icon=hdsl
Terminal=false
Categories=Development;Utility;
DESKTOP
cat > "$deb/DEBIAN/control" <<CONTROL
Package: hdsl
Version: $version
Section: devel
Priority: optional
Architecture: $arch
Maintainer: HDSL contributors
Depends: libc6, libfontconfig1, libgl1, libxkbcommon0
Description: DeepSeek Harness desktop launcher
 HDSL manages isolated DeepSeek Harness versions, web instances and plugins.
CONTROL
dpkg-deb --build --root-owner-group "$deb" "dist/hdsl_${version}_${arch}.deb"

(
  cd dist
  sha256sum "hdsl-$version-linux-$arch-portable.tar.gz" "hdsl_${version}_${arch}.deb" \
    > "SHA256SUMS-linux-$arch.txt"
)
echo "Created Linux $arch portable archive and Debian package"
