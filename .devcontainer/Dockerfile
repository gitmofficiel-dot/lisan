FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# تثبيت الأدوات العتادية للمتحكمات الدقيقة وأدوات التطوير المتقدمة
# Install embedded toolchains (Arduino, ESP32, STM32), build tools, and utilities
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get install -y --no-install-recommends \
        clang \
        llvm \
        lld \
        gcc-arm-none-eabi \
        libnewlib-arm-none-eabi \
        gdb-multiarch \
        gcc-avr \
        binutils-avr \
        avr-libc \
        avrdude \
        openocd \
        pkg-config \
        libssl-dev \
        curl \
        wget \
        unzip \
    && rm -rf /var/lib/apt/lists/*

# تثبيت arduino-cli لتطوير وتجربة الأردوينو و ESP32 و STM32 سحابياً
RUN curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | BINDIR=/usr/local/bin sh

# تثبيت esptool لمتحكمات ESP32
RUN pip3 install --no-cache-dir --break-system-packages esptool 2>/dev/null || pip3 install --no-cache-dir esptool 2>/dev/null || true

# إضافة هدف WebAssembly في Rust ومكونات التطوير الأساسية
RUN rustup target add wasm32-unknown-unknown \
    && rustup component add clippy rustfmt
