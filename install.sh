#!/bin/sh
set -e

# ==============================================================================
# سكربت تثبيت لغة البرمجة العربية "لسان" واستوديو لسان لسطح المكتب
# يدعم: Linux (x86_64 & ARM64 / Raspberry Pi 4 & 5) و macOS (Apple Silicon & Intel)
# الاستخدام: curl -sSf https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.sh | sh
# ==============================================================================

REPO="gitmofficiel-dot/lisan"
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "🔍 جاري فحص بيئة التشغيل ($OS / $ARCH)..."

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64|amd64)
        ASSET="lisan-linux-x86_64.tar.gz"
        ;;
      aarch64|arm64)
        ASSET="lisan-linux-arm64-raspberrypi.tar.gz"
        ;;
      *)
        echo "❌ المعمارية $ARCH غير مدعومة حالياً بحزمة مسبقة البناء لنظام لينكس."
        echo "💡 يمكنك البناء من المصدر عبر: cargo install --git https://github.com/$REPO"
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64|aarch64)
        ASSET="lisan-macos-arm64.tar.gz"
        ;;
      x86_64)
        ASSET="lisan-macos-x86_64.tar.gz"
        ;;
      *)
        echo "❌ المعمارية $ARCH غير مدعومة حالياً لنظام macOS."
        exit 1
        ;;
    esac
    ;;
  *)
    echo "❌ نظام التشغيل $OS غير مدعوم عبر سكربت يونكس."
    echo "🪟 على نظام ويندوز شغّل في PowerShell:"
    echo "   irm https://raw.githubusercontent.com/$REPO/main/install.ps1 | iex"
    exit 1
    ;;
esac

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

echo "📥 جاري تنزيل أحدث إصدار من لغة لسان ($ASSET)..."
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$ASSET"

if command -v curl >/dev/null 2>&1; then
  curl -sSfL "$DOWNLOAD_URL" -o "$TEMP_DIR/$ASSET"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TEMP_DIR/$ASSET" "$DOWNLOAD_URL"
else
  echo "❌ يتطلب التنزيل وجود curl أو wget على النظام."
  exit 1
fi

echo "📦 جاري فك الضغط وتثبيت لسان في $INSTALL_DIR..."
tar -xzf "$TEMP_DIR/$ASSET" -C "$TEMP_DIR"

FOUND_BIN="$TEMP_DIR/lisan"
if [ ! -f "$FOUND_BIN" ]; then
  FOUND_BIN="$(find "$TEMP_DIR" -type f -name lisan | head -n 1)"
fi

if [ -z "$FOUND_BIN" ] || [ ! -f "$FOUND_BIN" ]; then
  echo "❌ تعذّر العثور على الملف التنفيذي lisan داخل الأرشيف."
  exit 1
fi

if [ -w "$INSTALL_DIR" ]; then
  mv "$FOUND_BIN" "$INSTALL_DIR/lisan"
  chmod +x "$INSTALL_DIR/lisan"
else
  sudo mv "$FOUND_BIN" "$INSTALL_DIR/lisan"
  sudo chmod +x "$INSTALL_DIR/lisan"
fi

# تثبيت أمثلة لسان محلياً إن توفرت داخل الأرشيف
if [ -d "$TEMP_DIR/examples" ] && [ -n "${HOME:-}" ]; then
  SHARE_DIR="$HOME/.local/share/lisan"
  mkdir -p "$SHARE_DIR/examples"
  cp -r "$TEMP_DIR/examples/"* "$SHARE_DIR/examples/" 2>/dev/null || true
fi

# تثبيت اختصار تطبيق استوديو لسان لسطح المكتب على أنظمة Linux و Raspberry Pi OS
if [ "$OS" = "Linux" ] && [ -n "${HOME:-}" ] && [ -d "$HOME/.local/share/applications" ]; then
  ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
  mkdir -p "$ICON_DIR"
  cat > "$ICON_DIR/lisan.svg" <<'SVG'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <rect x="16" y="16" width="224" height="224" rx="52" fill="#0f172a" stroke="#0284c7" stroke-width="10"/>
  <circle cx="192" cy="64" r="12" fill="#10b981"/>
  <path d="M168 76 L168 158 C168 182 150 196 120 196 C92 196 76 181 76 160 C76 144 88 134 104 134" fill="none" stroke="#0ea5e9" stroke-width="20" stroke-linecap="round"/>
  <text x="128" y="118" text-anchor="middle" font-family="Tahoma, Arial, sans-serif" font-size="38" font-weight="bold" fill="#f8fafc">ض</text>
</svg>
SVG

  cat > "$HOME/.local/share/applications/lisan.desktop" <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Lisan Studio
Name[ar]=استوديو لسان
Comment=Arabic Programming Language & Desktop IDE
Comment[ar]=بيئة تطوير لغة البرمجة العربية لسان لسطح المكتب والمتحكمات
Exec=$INSTALL_DIR/lisan --studio
Icon=lisan
Terminal=false
Categories=Development;IDE;Education;Electronics;
StartupNotify=true
EOF
  chmod 644 "$HOME/.local/share/applications/lisan.desktop"
  echo "🖥️ تم تثبيت اختصار تطبيق (استوديو لسان) في قائمة تطبيقات سطح المكتب!"
fi

echo ""
echo "✨ تم تثبيت لغة لسان بنجاح! 🎉"
echo "النسخة المثبتة:"
"$INSTALL_DIR/lisan" --version 2>/dev/null || true
echo ""
echo "🚀 طرق التشغيل السريع:"
echo "  lisan                 فتح الطرفية التفاعلية (REPL)"
echo "  lisan --studio        تشغيل تطبيق استوديو لسان لسطح المكتب"
echo "  lisan ملف.lsn         ترجمة ملف لسان إلى برنامج آلة أصلي"
echo ""
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo "⚠️ ملاحظة: أضف $INSTALL_DIR إلى متغير المسارات PATH لديك:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac
