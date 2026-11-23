#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# سكربت تحزيم لغة لسان واستوديو لسان لأنظمة Linux و Raspberry Pi (ARM64 / x86_64)
# ينتج:
#   - dist/lisan-linux-<arch>.tar.gz
#   - dist/lisan_1.2.0_<deb_arch>.deb (حزمة Debian / Ubuntu / Raspberry Pi OS)
# ==============================================================================

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="1.2.0"
HOST_ARCH="$(uname -m)"
TARGET="${1:-}"
SKIP_BUILD="${SKIP_BUILD:-0}"

if [ -z "$TARGET" ]; then
    case "$HOST_ARCH" in
        x86_64|amd64)
            TARGET="x86_64-unknown-linux-gnu"
            ;;
        aarch64|arm64)
            TARGET="aarch64-unknown-linux-gnu"
            ;;
        *)
            echo "❌ المعمارية $HOST_ARCH غير معروفة؛ مرر الهدف يدوياً مثل x86_64-unknown-linux-gnu"
            exit 1
            ;;
    esac
fi

case "$TARGET" in
    x86_64-*)
        TAR_NAME="lisan-linux-x86_64.tar.gz"
        DEB_ARCH="amd64"
        ;;
    aarch64-*|arm64-*)
        TAR_NAME="lisan-linux-arm64-raspberrypi.tar.gz"
        DEB_ARCH="arm64"
        ;;
    *)
        TAR_NAME="lisan-linux-${TARGET}.tar.gz"
        DEB_ARCH="amd64"
        ;;
esac

if [ "$SKIP_BUILD" != "1" ]; then
    echo "🔨 جاري بناء نواة لسان واستوديو سطح المكتب للهدف: $TARGET ..."
    cargo build --release --target "$TARGET"
fi

BIN_PATH="$ROOT_DIR/target/$TARGET/release/lisan"
if [ ! -f "$BIN_PATH" ]; then
    echo "❌ لم يتم العثور على الملف التنفيذي المبني في: $BIN_PATH"
    exit 1
fi

mkdir -p "$ROOT_DIR/dist"
STAGE_BASE="$(mktemp -d)"
trap 'rm -rf "$STAGE_BASE"' EXIT

# 1. إنشاء أرشيف التوزيع المحمول (.tar.gz)
TAR_STAGE="$STAGE_BASE/tar"
mkdir -p "$TAR_STAGE/examples" "$TAR_STAGE/editors" "$TAR_STAGE/desktop"
cp "$BIN_PATH" "$TAR_STAGE/lisan"
chmod 755 "$TAR_STAGE/lisan"
cp -r "$ROOT_DIR/examples/"* "$TAR_STAGE/examples/" 2>/dev/null || true
cp -r "$ROOT_DIR/editors/vscode" "$TAR_STAGE/editors/" 2>/dev/null || true
cp "$ROOT_DIR/desktop/lisan.desktop" "$TAR_STAGE/desktop/lisan.desktop"
cp "$ROOT_DIR/README.md" "$TAR_STAGE/README.md"
cp "$ROOT_DIR/LICENSE" "$TAR_STAGE/LICENSE"

tar -czf "$ROOT_DIR/dist/$TAR_NAME" -C "$TAR_STAGE" .
echo "✅ تم إنشاء الأرشيف: dist/$TAR_NAME"

# 2. إنشاء حزمة التثبيت الرسمية (.deb) لأنظمة Debian / Ubuntu / Raspberry Pi OS
if command -v dpkg-deb >/dev/null 2>&1; then
    DEB_NAME="lisan_${VERSION}_${DEB_ARCH}.deb"
    DEB_STAGE="$STAGE_BASE/deb/lisan_${VERSION}_${DEB_ARCH}"
    mkdir -p \
        "$DEB_STAGE/DEBIAN" \
        "$DEB_STAGE/usr/bin" \
        "$DEB_STAGE/usr/share/applications" \
        "$DEB_STAGE/usr/share/icons/hicolor/scalable/apps" \
        "$DEB_STAGE/usr/share/lisan/examples" \
        "$DEB_STAGE/usr/share/lisan/vscode" \
        "$DEB_STAGE/usr/share/doc/lisan"

    cp "$BIN_PATH" "$DEB_STAGE/usr/bin/lisan"
    chmod 755 "$DEB_STAGE/usr/bin/lisan"

    cp "$ROOT_DIR/desktop/lisan.desktop" "$DEB_STAGE/usr/share/applications/lisan.desktop"
    chmod 644 "$DEB_STAGE/usr/share/applications/lisan.desktop"

    if [ -f "$ROOT_DIR/assets/logo.svg" ]; then
        cp "$ROOT_DIR/assets/logo.svg" "$DEB_STAGE/usr/share/icons/hicolor/scalable/apps/lisan.svg"
    else
        cat > "$DEB_STAGE/usr/share/icons/hicolor/scalable/apps/lisan.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <rect x="16" y="16" width="224" height="224" rx="52" fill="#0f172a" stroke="#0284c7" stroke-width="10"/>
  <circle cx="192" cy="64" r="12" fill="#10b981"/>
  <path d="M168 76 L168 158 C168 182 150 196 120 196 C92 196 76 181 76 160 C76 144 88 134 104 134" fill="none" stroke="#0ea5e9" stroke-width="20" stroke-linecap="round"/>
  <text x="128" y="118" text-anchor="middle" font-family="Tahoma, Arial, sans-serif" font-size="38" font-weight="bold" fill="#f8fafc">ض</text>
</svg>
SVG
    fi
    chmod 644 "$DEB_STAGE/usr/share/icons/hicolor/scalable/apps/lisan.svg"

    # PNG icons (identical on every desktop, no font dependency)
    for SZ in 64 128 256 512; do
        if [ -f "$ROOT_DIR/assets/icon-$SZ.png" ]; then
            mkdir -p "$DEB_STAGE/usr/share/icons/hicolor/${SZ}x${SZ}/apps"
            cp "$ROOT_DIR/assets/icon-$SZ.png" "$DEB_STAGE/usr/share/icons/hicolor/${SZ}x${SZ}/apps/lisan.png"
            chmod 644 "$DEB_STAGE/usr/share/icons/hicolor/${SZ}x${SZ}/apps/lisan.png"
        fi
    done

    cp -r "$ROOT_DIR/examples/"* "$DEB_STAGE/usr/share/lisan/examples/" 2>/dev/null || true
    cp -r "$ROOT_DIR/editors/vscode/"* "$DEB_STAGE/usr/share/lisan/vscode/" 2>/dev/null || true
    cp "$ROOT_DIR/README.md" "$DEB_STAGE/usr/share/doc/lisan/README.md"
    cp "$ROOT_DIR/LICENSE" "$DEB_STAGE/usr/share/doc/lisan/copyright"

    INSTALLED_KB="$(du -sk "$DEB_STAGE/usr" | awk '{print $1}')"
    cat > "$DEB_STAGE/DEBIAN/control" <<EOF
Package: lisan
Version: ${VERSION}
Section: devel
Priority: optional
Architecture: ${DEB_ARCH}
Installed-Size: ${INSTALLED_KB}
Recommends: clang
Maintainer: Lisan Programming Language Team <team@lisan.dev>
Homepage: https://github.com/gitmofficiel-dot/lisan
Description: Lisan Arabic Programming Language & Desktop Studio IDE
 Modern native Arabic programming language, bytecode VM, LLVM native compiler,
 Lisan Studio Desktop IDE, and hardware transpilers for Arduino, ESP32, STM32,
 and Raspberry Pi (GPIO/I2C/SPI/Camera).
EOF
    chmod 755 "$DEB_STAGE/DEBIAN"
    chmod 644 "$DEB_STAGE/DEBIAN/control"

    if dpkg-deb --help 2>&1 | grep -q -- "--root-owner-group"; then
        dpkg-deb --build --root-owner-group "$DEB_STAGE" "$ROOT_DIR/dist/$DEB_NAME" >/dev/null
    else
        dpkg-deb --build "$DEB_STAGE" "$ROOT_DIR/dist/$DEB_NAME" >/dev/null
    fi
    echo "✅ تم إنشاء حزمة ديبيان/أوبونتو: dist/$DEB_NAME"
else
    echo "ℹ️ أداة dpkg-deb غير مثبتة؛ تم تخطي بناء ملف .deb والاكتفاء بأرشيف .tar.gz"
fi

# 3. توليد بصمات التحقق SHA-256
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$ROOT_DIR/dist" && sha256sum "$TAR_NAME" > "${TAR_NAME}.sha256")
    if [ -f "$ROOT_DIR/dist/lisan_${VERSION}_${DEB_ARCH}.deb" ]; then
        (cd "$ROOT_DIR/dist" && sha256sum "lisan_${VERSION}_${DEB_ARCH}.deb" > "lisan_${VERSION}_${DEB_ARCH}.deb.sha256")
    fi
elif command -v shasum >/dev/null 2>&1; then
    (cd "$ROOT_DIR/dist" && shasum -a 256 "$TAR_NAME" > "${TAR_NAME}.sha256")
fi
