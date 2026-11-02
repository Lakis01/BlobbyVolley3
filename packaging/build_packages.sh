#!/bin/bash
set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
VERSION="${1:-$(grep -m1 '^version = ' "$REPO_ROOT/Cargo.toml" | cut -d '"' -f2)}"

echo "Building Blobby Volley 3 Release Packages (v$VERSION)..."
mkdir -p "$DIST_DIR"

# 1. Ensure release binary exists and is stripped
cd "$REPO_ROOT"
cargo build --release
mkdir -p /tmp/blobby_build
cp target/release/blobby /tmp/blobby_build/blobby
strip /tmp/blobby_build/blobby

# 2. Build Linux Portable .tar.gz bundle
echo "Packaging .tar.gz bundle..."
PORTABLE_DIR="/tmp/blobby-volley-3-v$VERSION-linux-x86_64"
rm -rf "$PORTABLE_DIR"
mkdir -p "$PORTABLE_DIR"
cp /tmp/blobby_build/blobby "$PORTABLE_DIR/"
cp packaging/blobby.desktop "$PORTABLE_DIR/"
cp packaging/blobby.png "$PORTABLE_DIR/"
cp README.md "$PORTABLE_DIR/"
cp COPYING "$PORTABLE_DIR/"

tar -czf "$DIST_DIR/blobby-volley-3-v$VERSION-linux-x86_64.tar.gz" -C /tmp "blobby-volley-3-v$VERSION-linux-x86_64"
echo "Created: $DIST_DIR/blobby-volley-3-v$VERSION-linux-x86_64.tar.gz"

# 3. Build Debian / Ubuntu .deb package
echo "Packaging Debian (.deb) package..."
DEB_BUILD="/tmp/blobby_deb_build"
rm -rf "$DEB_BUILD"
mkdir -p "$DEB_BUILD/DEBIAN"
mkdir -p "$DEB_BUILD/usr/bin"
mkdir -p "$DEB_BUILD/usr/share/applications"
mkdir -p "$DEB_BUILD/usr/share/pixmaps"
mkdir -p "$DEB_BUILD/usr/share/doc/blobby-volley-3"

cat <<EOF > "$DEB_BUILD/DEBIAN/control"
Package: blobby-volley-3
Version: $VERSION
Section: games
Priority: optional
Architecture: amd64
Maintainer: gratis01 <gratis01@local>
Description: Blobby Volley 3 - Remastered 2026 Edition in Rust
 A modern, high-performance port and complete visual remaster of the
 legendary Blobby Volley head-to-head multiplayer volleyball game.
EOF

cp /tmp/blobby_build/blobby "$DEB_BUILD/usr/bin/"
chmod 755 "$DEB_BUILD/usr/bin/blobby"
cp packaging/blobby.desktop "$DEB_BUILD/usr/share/applications/"
cp packaging/blobby.png "$DEB_BUILD/usr/share/pixmaps/"
cp README.md "$DEB_BUILD/usr/share/doc/blobby-volley-3/"
cp COPYING "$DEB_BUILD/usr/share/doc/blobby-volley-3/copyright"

# Create deb using ar
cd "$DEB_BUILD"
echo "2.0" > debian-binary
tar -czf control.tar.gz -C "$DEB_BUILD/DEBIAN" control
tar -czf data.tar.gz -C "$DEB_BUILD" usr
ar rcs "$DIST_DIR/blobby-volley-3_${VERSION}_amd64.deb" debian-binary control.tar.gz data.tar.gz
echo "Created: $DIST_DIR/blobby-volley-3_${VERSION}_amd64.deb"

# 4. Copy Arch Linux PKGBUILD
cp "$REPO_ROOT/packaging/PKGBUILD" "$DIST_DIR/PKGBUILD"
echo "Created: $DIST_DIR/PKGBUILD"

# Clean temporary files
rm -rf /tmp/blobby_build "$PORTABLE_DIR" "$DEB_BUILD"
echo "All release packages built successfully in $DIST_DIR!"
