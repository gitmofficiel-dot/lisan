#!/bin/sh
set -e

# ==============================================================================
# سكربت تثبيت لغة البرمجة العربية "لسان" (Lisan Programming Language Installer)
# الاستخدام: curl -sSf https://raw.githubusercontent.com/gitmofficiel-dot/lisan/main/install.sh | sh
# ==============================================================================

REPO="gitmofficiel-dot/lisan"
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "🔍 جاري فحص بيئة التشغيل..."

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)
        ASSET="lisan-linux-x86_64.tar.gz"
        ;;
      *)
        echo "❌ المعمارية $ARCH غير مدعومة حالياً بشكل مسبق البناء لنظام لينكس."
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64)
        ASSET="lisan-macos-arm64.tar.gz"
        ;;
      x86_64)
        ASSET="lisan-macos-x86_64.tar.gz"
        ;;
      *)
        echo "❌ المعمارية $ARCH غير مدعومة حالياً لنظام ماك."
        exit 1
        ;;
    esac
    ;;
  *)
    echo "❌ نظام التشغيل $OS غير مدعوم عبر سكربت التثبيت التلقائي."
    echo "يرجى تنزيل الحزمة الجاهزة مباشرة من صفحة الإصدارات في غيت هاب: https://github.com/$REPO/releases"
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

echo "📥 جاري تنزيل أحدث إصدار من لسان ($ASSET)..."
DOWNLOAD_URL="https://github.com/$REPO/releases/latest/download/$ASSET"

if command -v curl >/dev/null 2>&1; then
  curl -sSfL "$DOWNLOAD_URL" -o "$TEMP_DIR/$ASSET"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TEMP_DIR/$ASSET" "$DOWNLOAD_URL"
else
  echo "❌ يتطلب التنزيل وجود curl أو wget على النظام."
  exit 1
fi

echo "📦 جاري فك الضغط وتثبيت الملف التنفيذي في $INSTALL_DIR..."
tar -xzf "$TEMP_DIR/$ASSET" -C "$TEMP_DIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TEMP_DIR/lisan" "$INSTALL_DIR/lisan"
  chmod +x "$INSTALL_DIR/lisan"
else
  sudo mv "$TEMP_DIR/lisan" "$INSTALL_DIR/lisan"
  sudo chmod +x "$INSTALL_DIR/lisan"
fi

echo ""
echo "✨ تم تثبيت لغة لسان بنجاح! 🎉"
echo "النسخة المثبتة:"
"$INSTALL_DIR/lisan" --version 2>/dev/null || "$INSTALL_DIR/lisan" || true
echo ""
echo "لبدء الاستخدام اكتب في الطرفية:"
echo "  lisan --repl"
echo ""
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo "⚠️ ملاحظة: تأكد من إضافة $INSTALL_DIR إلى متغير المسارات PATH لديك."
  echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi
