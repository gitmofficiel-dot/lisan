#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Lisan Studio IDE Builder & Rebrander (استوديو لسان — مبني على Code-OSS / VSCodium)
# يطبّق الهوية العربية الكاملة (الاسم + الشعار + السمات + إضافة لسان v1.2 + اللغة العربية)
#
# الاستخدام:
#   ./ide/build-ide.sh --source-tar <VSCodium.tar.gz> --target-dir /media/mohammed/External/Lisan-Studio [--install-desktop]
# ==============================================================================

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_TAR=""
TARGET_DIR=""
LISAN_BIN=""
INSTALL_DESKTOP=0
INSTALL_ARABIC_PACK=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --source-tar) SOURCE_TAR="$2"; shift 2 ;;
    --target-dir) TARGET_DIR="$2"; shift 2 ;;
    --lisan-bin) LISAN_BIN="$2"; shift 2 ;;
    --install-desktop) INSTALL_DESKTOP=1; shift ;;
    --with-arabic-pack) INSTALL_ARABIC_PACK=1; shift ;;
    --no-arabic-pack) INSTALL_ARABIC_PACK=0; shift ;;
    *) echo "خيار غير معروف: $1"; exit 2 ;;
  esac
done

if [[ -z "$TARGET_DIR" ]]; then
  echo "الاستخدام: $0 --source-tar <archive.tar.gz> --target-dir <مجلد_التثبيت> [--install-desktop]"
  exit 2
fi

mkdir -p "$TARGET_DIR"

if [[ -n "$SOURCE_TAR" ]]; then
  echo "=== [1/6] فك حزمة Code-OSS / VSCodium إلى $TARGET_DIR ==="
  tar -xzf "$SOURCE_TAR" -C "$TARGET_DIR"
fi

APP_DIR="$TARGET_DIR/resources/app"
if [[ ! -d "$APP_DIR" ]]; then
  echo "❌ لم يتم العثور على $APP_DIR — تأكد من المسار أو الأرشيف."
  exit 1
fi

echo "=== [2/6] تطبيق هوية واسم «استوديو لسان — Lisan Studio» على product.json ==="
python3 - "$APP_DIR/product.json" "$REPO_ROOT/ide/product.patch.json" <<'PY'
import json, sys
prod_path, patch_path = sys.argv[1], sys.argv[2]
with open(prod_path, "r", encoding="utf-8") as f:
    product = json.load(f)
with open(patch_path, "r", encoding="utf-8") as f:
    patch = json.load(f)
product.update(patch)
with open(prod_path, "w", encoding="utf-8") as f:
    json.dump(product, f, ensure_ascii=False, indent=2)
print("Updated product.json:", product.get("nameLong"))
PY

echo "=== [3/6] استبدال الأيقونات والشعار بشعار لغة لسان الرسمي ==="
if [[ -f "$REPO_ROOT/assets/icon-512.png" ]]; then
  mkdir -p "$APP_DIR/resources/linux"
  cp "$REPO_ROOT/assets/icon-512.png" "$APP_DIR/resources/linux/code.png"
  cp "$REPO_ROOT/assets/icon-512.png" "$TARGET_DIR/lisan-studio.png"
fi
if [[ -f "$REPO_ROOT/assets/favicon.ico" ]]; then
  cp "$REPO_ROOT/assets/favicon.ico" "$APP_DIR/out/media/code-icon.ico" 2>/dev/null || true
fi
if [[ -f "$REPO_ROOT/assets/icon-512.svg" ]]; then
  cp "$REPO_ROOT/assets/icon-512.svg" "$APP_DIR/out/media/code-icon.svg" 2>/dev/null || true
fi

echo "=== [4/6] دمج إضافة لغة لسان v1.2 + إضافة تجربة وسمات استوديو لسان داخل المحرر ==="
rm -rf "$APP_DIR/extensions/lisan-arabic-lang" "$APP_DIR/extensions/lisan-studio-experience"
mkdir -p "$APP_DIR/extensions/lisan-arabic-lang" "$APP_DIR/extensions/lisan-studio-experience"
cp -r "$REPO_ROOT/editors/vscode/"* "$APP_DIR/extensions/lisan-arabic-lang/"
cp -r "$REPO_ROOT/ide/extensions/lisan-studio-experience/"* "$APP_DIR/extensions/lisan-studio-experience/"

# تنزيل حزمة اللغة العربية للواجهة من Open VSX إن توفّر اتصال
if [[ "$INSTALL_ARABIC_PACK" -eq 1 ]]; then
  echo "=== [5/6] تنزيل حزمة واجهة اللغة العربية (Open VSX) ==="
  TMP_AR="$(mktemp -d)"
  AR_URL="$(curl -sSf "https://open-vsx.org/api/ms-ceintl/vscode-language-pack-ar/latest" 2>/dev/null | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d.get("files",{}).get("download",""))' 2>/dev/null || true)"
  if [[ -n "$AR_URL" ]]; then
    if curl -sSfL "$AR_URL" -o "$TMP_AR/ar.vsix"; then
      python3 -c "import zipfile,sys; zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])" "$TMP_AR/ar.vsix" "$TMP_AR/unpacked"
      if [[ -d "$TMP_AR/unpacked/extension" ]]; then
        rm -rf "$APP_DIR/extensions/ms-ceintl.vscode-language-pack-ar"
        cp -r "$TMP_AR/unpacked/extension" "$APP_DIR/extensions/ms-ceintl.vscode-language-pack-ar"
        echo "✅ تم دمج حزمة اللغة العربية بنجاح!"
      fi
    fi
  else
    echo "⚠️ تعذر تنزيل حزمة اللغة العربية حالياً (يمكن تثبيتها لاحقاً من داخل المحرر)."
  fi
  rm -rf "$TMP_AR"
fi

echo "=== [6/6] إنشاء ملف التشغيل lisan-studio والإعدادات الافتراضية ==="
USER_DATA_DIR="$TARGET_DIR/user-data"
EXT_DIR="$TARGET_DIR/extensions"
mkdir -p "$USER_DATA_DIR/User" "$EXT_DIR"

if [[ ! -f "$USER_DATA_DIR/User/settings.json" ]]; then
  cp "$REPO_ROOT/ide/default-settings.json" "$USER_DATA_DIR/User/settings.json"
fi
if [[ ! -f "$USER_DATA_DIR/User/argv.json" ]]; then
  cat > "$USER_DATA_DIR/User/argv.json" <<'ARGV'
{
  "enable-crash-reporter": false
}
ARGV
fi

# دمج الملف التنفيذي لمترجم لغة لسان (lisan) مباشرة داخل حزمة المحرر ليعمل فور التحميل بلا تثبيت إضافي
mkdir -p "$TARGET_DIR/bin"
if [[ -z "$LISAN_BIN" ]]; then
  for cand in "$HOME/.local/bin/lisan" "$REPO_ROOT/target/release/lisan" "$(command -v lisan 2>/dev/null || true)"; do
    if [[ -n "$cand" && -f "$cand" ]]; then
      LISAN_BIN="$cand"
      break
    fi
  done
fi
if [[ -n "$LISAN_BIN" && -f "$LISAN_BIN" ]]; then
  cp "$LISAN_BIN" "$TARGET_DIR/bin/lisan"
  chmod +x "$TARGET_DIR/bin/lisan"
  echo "✅ تم دمج مترجم لغة لسان مباشرة داخل المحرر: $TARGET_DIR/bin/lisan"
fi

# إنشاء السكربت التنفيذي lisan-studio داخل الحزمة (يتتبع الروابط الرمزية ويدعم الأقراص الخارجية وUbuntu 24.04)
cat > "$TARGET_DIR/bin/lisan-studio" <<'EOF'
#!/usr/bin/env bash
REAL_SCRIPT="$(readlink -f "${BASH_SOURCE[0]}")"
SELF_DIR="$(cd "$(dirname "${REAL_SCRIPT}")/.." && pwd)"
export PATH="${SELF_DIR}/bin:${HOME}/.local/bin:${PATH}"

EXTRA_ARGS=()
if [[ -f "${SELF_DIR}/chrome-sandbox" ]]; then
  SANDBOX_PERM="$(stat -c '%u:%a' "${SELF_DIR}/chrome-sandbox" 2>/dev/null || echo '')"
  if [[ "${SANDBOX_PERM}" != "0:4755" ]]; then
    EXTRA_ARGS+=(--no-sandbox)
  fi
fi

exec "${SELF_DIR}/bin/codium" \
  "${EXTRA_ARGS[@]}" \
  --user-data-dir="${SELF_DIR}/user-data" \
  --extensions-dir="${SELF_DIR}/extensions" \
  "$@"
EOF
chmod +x "$TARGET_DIR/bin/lisan-studio"

if [[ "$INSTALL_DESKTOP" -eq 1 ]]; then
  mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" "$HOME/.local/share/icons/hicolor/512x512/apps" "$HOME/.local/share/icons/hicolor/scalable/apps"
  ln -sf "$TARGET_DIR/bin/lisan-studio" "$HOME/.local/bin/lisan-studio"
  ln -sf "$TARGET_DIR/bin/lisan-studio" "$HOME/.local/bin/lisan-code"

  cp "$REPO_ROOT/assets/icon-512.png" "$HOME/.local/share/icons/hicolor/512x512/apps/lisan-studio.png"
  cp "$REPO_ROOT/assets/icon-512.svg" "$HOME/.local/share/icons/hicolor/scalable/apps/lisan-studio.svg"

  cat > "$HOME/.local/share/applications/lisan-studio-ide.desktop" <<DESKTOP
[Desktop Entry]
Version=1.0
Type=Application
Name=Lisan Studio IDE
Name[ar]=استوديو لسان (محرر الشيفرة)
GenericName=Arabic Code Editor
GenericName[ar]=محرر البرمجة العربية لسان
Comment=Custom Arabic IDE powered by Code-OSS & Lisan v1.2.0
Comment[ar]=محرر استوديو لسان العربي المتكامل للذكاء الاصطناعي والويب والتطبيقات والمتحكمات
Exec=$HOME/.local/bin/lisan-studio %F
Icon=lisan-studio
Terminal=false
Categories=Development;IDE;TextEditor;
MimeType=text/plain;inode/directory;
StartupWMClass=Lisan Studio
StartupNotify=true
DESKTOP
  chmod 644 "$HOME/.local/share/applications/lisan-studio-ide.desktop"
  update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
  echo "🖥️ تم تثبيت اختصار «استوديو لسان (محرر الشيفرة)» في قائمة التطبيقات وأمر الطرفية lisan-studio!"
fi

echo "✅ اكتمل بناء وتجهيز استوديو لسان (Lisan Studio IDE) في: $TARGET_DIR"
