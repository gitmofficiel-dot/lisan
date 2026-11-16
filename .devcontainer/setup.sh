#!/usr/bin/env bash
set -e

echo "🚀 إعداد بيئة لسان في GitHub Codespaces (Initializing Lisan)..."

# 1. التأكد من تثبيت هدف wasm
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# 2. بناء لسان في وضع الإصدار
echo "🔨 بناء مترجم ومفسّر لسان (Building Lisan release)..."
cargo build --release

# 3. تثبيت لسان في المسار العام للنظام ليتاح أمر lisan مباشرة في الطرفية
echo "📦 تثبيت أمر lisan في /usr/local/bin..."
if command -v sudo >/dev/null 2>&1; then
    sudo cp target/release/lisan /usr/local/bin/lisan
    sudo chmod +x /usr/local/bin/lisan
else
    cp target/release/lisan /usr/local/bin/lisan
    chmod +x /usr/local/bin/lisan
fi

# 4. تثبيت إضافة لسان في Visual Studio Code داخل Codespaces
echo "🎨 تثبيت ملحق لسان في VS Code (Installing Lisan Editor Extension)..."
for ext_dir in "$HOME/.vscode-server/extensions" "$HOME/.vscode-remote/extensions"; do
    mkdir -p "$ext_dir"
    rm -rf "$ext_dir/lisan-lang.lisan-arabic-lang-1.0.0"
    cp -r editors/vscode "$ext_dir/lisan-lang.lisan-arabic-lang-1.0.0"
done

# 5. بناء حزمة WebAssembly للويب
echo "🌐 تحديث حزمة WebAssembly..."
cargo build --lib --target wasm32-unknown-unknown --release 2>/dev/null || true
if [ -f target/wasm32-unknown-unknown/release/lisan.wasm ]; then
    cp target/wasm32-unknown-unknown/release/lisan.wasm web/lisan.wasm
fi

echo "✅ تم إعداد بيئة Codespaces للسان بنجاح!"
