#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# سكربت تحزيم لغة لسان وحزمة تطبيق macOS الرسمية (Lisan Studio.app)
# يدعم Apple Silicon (M1/M2/M3/M4)، ومعالجات Intel، وحزمة Universal Binary
# ==============================================================================

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-}"
SKIP_BUILD="${SKIP_BUILD:-0}"

if [ -z "$MODE" ]; then
    case "$(uname -m)" in
        arm64|aarch64) MODE="aarch64-apple-darwin" ;;
        *)             MODE="x86_64-apple-darwin" ;;
    esac
fi

mkdir -p "$ROOT_DIR/dist"
STAGE_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGE_DIR"' EXIT

if [ "$MODE" = "universal" ]; then
    ARCH_LABEL="universal"
    if [ "$SKIP_BUILD" != "1" ]; then
        echo "🔨 جاري بناء نسخة macOS الموحدة (Apple Silicon + Intel)..."
        cargo build --release --target aarch64-apple-darwin
        cargo build --release --target x86_64-apple-darwin
    fi
    mkdir -p "$ROOT_DIR/target/universal-macos/release"
    lipo -create \
        -output "$ROOT_DIR/target/universal-macos/release/lisan" \
        "$ROOT_DIR/target/aarch64-apple-darwin/release/lisan" \
        "$ROOT_DIR/target/x86_64-apple-darwin/release/lisan"
    BIN_PATH="$ROOT_DIR/target/universal-macos/release/lisan"
else
    case "$MODE" in
        aarch64-apple-darwin|arm64)
            TARGET="aarch64-apple-darwin"
            ARCH_LABEL="arm64"
            ;;
        x86_64-apple-darwin|x86_64)
            TARGET="x86_64-apple-darwin"
            ARCH_LABEL="x86_64"
            ;;
        *)
            echo "❌ هدف macOS غير مدعوم: $MODE"
            exit 1
            ;;
    esac

    if [ "$SKIP_BUILD" != "1" ]; then
        echo "🔨 جاري بناء نواة لسان لنظام macOS ($TARGET)..."
        cargo build --release --target "$TARGET"
    fi
    BIN_PATH="$ROOT_DIR/target/$TARGET/release/lisan"
fi

if [ ! -f "$BIN_PATH" ]; then
    echo "❌ لم يتم العثور على الملف التنفيذي: $BIN_PATH"
    exit 1
fi

# 1. أرشيف سطر الأوامر (.tar.gz) المتوافق مع install.sh
TAR_NAME="lisan-macos-${ARCH_LABEL}.tar.gz"
CLI_STAGE="$STAGE_DIR/cli"
mkdir -p "$CLI_STAGE/examples" "$CLI_STAGE/editors"
cp "$BIN_PATH" "$CLI_STAGE/lisan"
chmod 755 "$CLI_STAGE/lisan"
cp -r "$ROOT_DIR/examples/"* "$CLI_STAGE/examples/" 2>/dev/null || true
cp -r "$ROOT_DIR/editors/vscode" "$CLI_STAGE/editors/" 2>/dev/null || true
cp "$ROOT_DIR/README.md" "$CLI_STAGE/README.md"
cp "$ROOT_DIR/LICENSE" "$CLI_STAGE/LICENSE"

tar -czf "$ROOT_DIR/dist/$TAR_NAME" -C "$CLI_STAGE" .
echo "✅ تم إنشاء أرشيف macOS: dist/$TAR_NAME"

# 2. حزمة تطبيق سطح المكتب (dist/Lisan Studio.app)
APP_BUNDLE="$ROOT_DIR/dist/Lisan Studio.app"
rm -rf "$APP_BUNDLE"
mkdir -p \
    "$APP_BUNDLE/Contents/MacOS" \
    "$APP_BUNDLE/Contents/Resources/examples" \
    "$APP_BUNDLE/Contents/Resources/vscode"

cp "$ROOT_DIR/desktop/Info.plist" "$APP_BUNDLE/Contents/Info.plist"
cp "$BIN_PATH" "$APP_BUNDLE/Contents/MacOS/lisan"
chmod 755 "$APP_BUNDLE/Contents/MacOS/lisan"

cat > "$APP_BUNDLE/Contents/MacOS/lisan-studio" <<'LAUNCHER'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
RES_DIR="$(cd "$DIR/../Resources" && pwd)"
cd "$RES_DIR" || exit 1
exec "$DIR/lisan" --studio
LAUNCHER
chmod 755 "$APP_BUNDLE/Contents/MacOS/lisan-studio"

if [ -f "$ROOT_DIR/assets/logo.svg" ]; then
    cp "$ROOT_DIR/assets/logo.svg" "$APP_BUNDLE/Contents/Resources/logo.svg"
else
    cat > "$APP_BUNDLE/Contents/Resources/logo.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <rect x="16" y="16" width="224" height="224" rx="52" fill="#0f172a" stroke="#0284c7" stroke-width="10"/>
  <circle cx="192" cy="64" r="12" fill="#10b981"/>
  <path d="M168 76 L168 158 C168 182 150 196 120 196 C92 196 76 181 76 160 C76 144 88 134 104 134" fill="none" stroke="#0ea5e9" stroke-width="20" stroke-linecap="round"/>
  <text x="128" y="118" text-anchor="middle" font-family="Tahoma, Arial, sans-serif" font-size="38" font-weight="bold" fill="#f8fafc">ض</text>
</svg>
SVG
fi

cp -r "$ROOT_DIR/examples/"* "$APP_BUNDLE/Contents/Resources/examples/" 2>/dev/null || true
cp -r "$ROOT_DIR/editors/vscode/"* "$APP_BUNDLE/Contents/Resources/vscode/" 2>/dev/null || true

APP_ZIP="Lisan-Studio-macOS-${ARCH_LABEL}.zip"
rm -f "$ROOT_DIR/dist/$APP_ZIP"
if command -v ditto >/dev/null 2>&1; then
    ditto -c -k --sequesterRsrc --keepParent "$APP_BUNDLE" "$ROOT_DIR/dist/$APP_ZIP"
elif command -v zip >/dev/null 2>&1; then
    (cd "$ROOT_DIR/dist" && zip -qry "$APP_ZIP" "Lisan Studio.app")
fi
echo "✅ تم إنشاء حزمة التطبيق: dist/Lisan Studio.app و dist/$APP_ZIP"

if command -v shasum >/dev/null 2>&1; then
    (cd "$ROOT_DIR/dist" && shasum -a 256 "$TAR_NAME" > "${TAR_NAME}.sha256")
    if [ -f "$ROOT_DIR/dist/$APP_ZIP" ]; then
        (cd "$ROOT_DIR/dist" && shasum -a 256 "$APP_ZIP" > "${APP_ZIP}.sha256")
    fi
fi
