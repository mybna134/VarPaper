#!/bin/bash
set -e

# Build wayvid AppImage
# Requires: appimagetool, linuxdeploy, Rust toolchain
# Optional: upx for compression

VERSION="${1:-0.3.0}"
ARCH="$(uname -m)"
BUILD_DIR="$(pwd)/build"
APPDIR="${BUILD_DIR}/wayvid.AppDir"

echo "🚀 Building wayvid AppImage v${VERSION} for ${ARCH}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo "📋 Checking prerequisites..."

check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}❌ $1 not found${NC}"
        echo "   Install: $2"
        return 1
    else
        echo -e "${GREEN}✅ $1 found${NC}"
        return 0
    fi
}

MISSING=0
check_command cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" || MISSING=1
check_command flutter "https://docs.flutter.dev/get-started/install/linux" || MISSING=1
check_command appimagetool "wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH}.AppImage -O ~/.local/bin/appimagetool && chmod +x ~/.local/bin/appimagetool" || MISSING=1

if [ $MISSING -eq 1 ]; then
    echo -e "${RED}❌ Missing prerequisites. Install them and try again.${NC}"
    exit 1
fi

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf "${BUILD_DIR}"
mkdir -p "${APPDIR}"

# Build wayvid in release mode (unless NO_BUILD=1)
if [ "${NO_BUILD}" = "1" ]; then
    echo "⚡ Skipping build (using precompiled binaries from target/release)"
    cd "$(git rev-parse --show-toplevel)"
else
    echo "🔨 Building wayvid (release mode)..."
    cd "$(git rev-parse --show-toplevel)"
    flutter pub get
    flutter build linux --release
    cargo build --release -p wayvid-ctl
fi

# Create AppDir structure
echo "📦 Creating AppDir structure..."
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/lib"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/wayvid"
mkdir -p "${APPDIR}/usr/share/doc/wayvid"

# Copy the Flutter bundle and CLI (wayvid-gui remains the public entrypoint).
echo "📋 Copying binaries..."
if [ -x build/linux/x64/release/bundle/wayvid-gui ]; then
    mkdir -p "${APPDIR}/usr/lib/wayvid"
    cp -a build/linux/x64/release/bundle/. "${APPDIR}/usr/lib/wayvid/"
    install -m755 packaging/wayvid-gui-wrapper "${APPDIR}/usr/bin/wayvid-gui"
    echo "   ✅ Copied wayvid-gui Flutter bundle"
else
    echo -e "${RED}❌ Flutter wayvid-gui bundle not found${NC}"
    exit 1
fi

if [ -f target/release/wayvid-ctl ]; then
    cp target/release/wayvid-ctl "${APPDIR}/usr/bin/"
    echo "   ✅ Copied wayvid-ctl (CLI control tool)"
else
    echo -e "${YELLOW}⚠️  wayvid-ctl not found (optional)${NC}"
fi

# Strip binaries if not already stripped
if command -v strip &> /dev/null; then
    echo "🔪 Stripping binaries..."
    [ -f "${APPDIR}/usr/bin/wayvid-gui" ] && strip "${APPDIR}/usr/bin/wayvid-gui"
    [ -f "${APPDIR}/usr/bin/wayvid-ctl" ] && strip "${APPDIR}/usr/bin/wayvid-ctl"
fi

# Optional: Compress binaries with UPX
if command -v upx &> /dev/null; then
    echo "📦 Compressing binaries with UPX..."
    [ -f "${APPDIR}/usr/bin/wayvid-gui" ] && upx --best --lzma "${APPDIR}/usr/bin/wayvid-gui" || true
    [ -f "${APPDIR}/usr/bin/wayvid-ctl" ] && upx --best --lzma "${APPDIR}/usr/bin/wayvid-ctl" || true
fi

# Copy desktop file
echo "📋 Copying desktop file..."
cp packaging/appimage/wayvid.desktop "${APPDIR}/usr/share/applications/"
cp packaging/appimage/wayvid.desktop "${APPDIR}/"

# Copy or generate icon
echo "🎨 Setting up icon..."
ICON_SOURCE=""
if [ -f "packaging/appimage/wayvid.png" ]; then
    ICON_SOURCE="packaging/appimage/wayvid.png"
elif [ -f "assets/icon.png" ]; then
    ICON_SOURCE="assets/icon.png"
else
    echo -e "${YELLOW}⚠️  No icon found, creating placeholder...${NC}"
    # Create a simple placeholder icon using ImageMagick if available
    if command -v convert &> /dev/null; then
        convert -size 256x256 xc:transparent \
                -fill '#5865F2' -draw "roundrectangle 20,20 236,236 40,40" \
                -fill white -pointsize 72 -gravity center \
                -annotate +0+0 "WV" \
                packaging/appimage/wayvid.png
        ICON_SOURCE="packaging/appimage/wayvid.png"
    else
        echo -e "${YELLOW}⚠️  ImageMagick not found, skipping icon${NC}"
    fi
fi

if [ -n "$ICON_SOURCE" ]; then
    cp "$ICON_SOURCE" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/wayvid.png"
    cp "$ICON_SOURCE" "${APPDIR}/wayvid.png"
fi

# Copy AppRun script
echo "📋 Copying AppRun..."
cp packaging/appimage/AppRun "${APPDIR}/"
chmod +x "${APPDIR}/AppRun"

# Copy configuration and documentation
echo "📚 Copying documentation..."
cp configs/config.example.yaml "${APPDIR}/usr/share/wayvid/"
cp README.md "${APPDIR}/usr/share/doc/wayvid/"
cp docs/*.md "${APPDIR}/usr/share/doc/wayvid/" 2>/dev/null || true

# Copy dependencies (libmpv, GUI libs)
echo "📦 Copying dependencies..."
copy_lib() {
    local lib="$1"
    local lib_path=$(ldconfig -p | grep "$lib" | awk '{print $NF}' | head -n 1)
    if [ -n "$lib_path" ] && [ -f "$lib_path" ]; then
        echo "   Copying $lib_path"
        cp -L "$lib_path" "${APPDIR}/usr/lib/"
        # Copy symlinks
        local lib_name=$(basename "$lib_path")
        local lib_dir=$(dirname "$lib_path")
        find "$lib_dir" -name "${lib_name%.*}*" -type l -exec cp -P {} "${APPDIR}/usr/lib/" \;
    else
        echo -e "${YELLOW}   ⚠️  $lib not found (optional)${NC}"
    fi
}

# Copy libmpv and its dependencies
copy_lib "libmpv.so"
copy_lib "libwayland-client.so"
copy_lib "libwayland-egl.so"
copy_lib "libEGL.so"
copy_lib "libGL.so"

# Copy GUI dependencies (only if wayvid-gui is present)
if [ -f "${APPDIR}/usr/bin/wayvid-gui" ]; then
    echo "📦 Copying GUI dependencies..."
    copy_lib "libfontconfig.so"
    copy_lib "libxkbcommon.so"
fi

# Create AppImage
echo "🎁 Creating AppImage..."
cd "${BUILD_DIR}"

# Set version info
export VERSION="${VERSION}"
export ARCH="${ARCH}"

# Create AppImage with appimagetool
APPIMAGE_NAME="wayvid-${VERSION}-${ARCH}.AppImage"
appimagetool --appimage-extract-and-run "${APPDIR}" "${APPIMAGE_NAME}" || \
appimagetool "${APPDIR}" "${APPIMAGE_NAME}"

# Make executable
chmod +x "${APPIMAGE_NAME}"

# Calculate size and checksum
SIZE=$(du -h "${APPIMAGE_NAME}" | cut -f1)
SHA256=$(sha256sum "${APPIMAGE_NAME}" | cut -d' ' -f1)

echo ""
echo -e "${GREEN}✅ AppImage built successfully!${NC}"
echo ""
echo "📦 Package: ${APPIMAGE_NAME}"
echo "📏 Size: ${SIZE}"
echo "🔐 SHA256: ${SHA256}"
echo ""
echo "📍 Location: ${BUILD_DIR}/${APPIMAGE_NAME}"
echo ""
echo "🧪 Test it:"
echo "   ./${APPIMAGE_NAME} --version"
echo "   ./${APPIMAGE_NAME} --help"
echo ""
echo "🚀 Usage:"
echo "   ./${APPIMAGE_NAME}                    # Run wayvid"
echo "   ./${APPIMAGE_NAME} ctl status         # Run wayvid-ctl"
if [ -f "${APPDIR}/usr/bin/wayvid-gui" ]; then
    echo "   ./${APPIMAGE_NAME} gui                # Run wayvid-gui (GUI control panel)"
fi
echo ""

# Create symlink for convenience
ln -sf "${APPIMAGE_NAME}" "${BUILD_DIR}/wayvid-latest-${ARCH}.AppImage"
echo "🔗 Symlink created: wayvid-latest-${ARCH}.AppImage"
