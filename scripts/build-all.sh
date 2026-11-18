#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# السكربت الشامل لبناء وتحزيم لغة لسان واستوديو سطح المكتب عبر المنصات
# يدعم: Linux (x86_64 / ARM64 Raspberry Pi)، macOS (Apple Silicon / Intel / Universal)، Windows
# ==============================================================================

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p "$ROOT_DIR/dist"
OS_NAME="$(uname -s)"

echo "🚀 بدء التحزيم الشامل لمنصة لسان 1.0 على النظام: $OS_NAME ($(uname -m))"

case "$OS_NAME" in
    Linux)
        bash "$ROOT_DIR/scripts/package-linux.sh"

        # في حال توفر مترجم ARM64 المتقاطع نبني حزمة Raspberry Pi 4 / 5 تلقائياً أيضاً
        if [ "$(uname -m)" = "x86_64" ] && command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
            if rustup target list --installed 2>/dev/null | grep -q "aarch64-unknown-linux-gnu"; then
                echo "🍓 تم اكتشاف أدوات البناء المتقاطع لـ Raspberry Pi ARM64؛ جاري التحزيم..."
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
                    bash "$ROOT_DIR/scripts/package-linux.sh" aarch64-unknown-linux-gnu
            fi
        fi
        ;;
    Darwin)
        if rustup target list --installed 2>/dev/null | grep -q "aarch64-apple-darwin" && \
           rustup target list --installed 2>/dev/null | grep -q "x86_64-apple-darwin"; then
            bash "$ROOT_DIR/scripts/package-macos.sh" aarch64-apple-darwin
            bash "$ROOT_DIR/scripts/package-macos.sh" x86_64-apple-darwin
            SKIP_BUILD=1 bash "$ROOT_DIR/scripts/package-macos.sh" universal
        else
            bash "$ROOT_DIR/scripts/package-macos.sh"
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        if command -v pwsh >/dev/null 2>&1; then
            pwsh -NoProfile -ExecutionPolicy Bypass -File "$ROOT_DIR/scripts/package-windows.ps1"
        else
            powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$ROOT_DIR/scripts/package-windows.ps1"
        fi
        ;;
    *)
        echo "❌ نظام التشغيل $OS_NAME غير معروف لهذا السكربت."
        exit 1
        ;;
esac

echo ""
echo "🎉 اكتمل بناء الحزم بنجاح داخل المجلد dist/:"
ls -lh "$ROOT_DIR/dist"
