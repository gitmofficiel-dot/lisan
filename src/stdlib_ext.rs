//! المكتبة القياسية الموسّعة للغة لسان (التشفير، الأنماط القياسية، معلومات المنصة، وقاعدة البيانات المدمجة).
//! مبنية بالكامل بلغة Rust الصافية دون أي اعتماديات خارجية.

use crate::runtime::Value;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// جميع أسماء الدوال المدمجة الموسّعة (بالعربية والإنجليزية)
pub const ALL_EXT_BUILTINS: &[&str] = &[
    // التشفير والتجزئة والترميز الست عشري والمعرّفات الفريدة
    "تجزئة_sha256",
    "تجزئه_sha256",
    "sha256",
    "توقيع_hmac",
    "hmac_sha256",
    "ترميز_ست_عشري",
    "hex_encode",
    "فك_ست_عشري",
    "hex_decode",
    "معرف_فريد",
    "uuid",
    "شفر_نص",
    "encrypt_text",
    "فك_تشفير_نص",
    "decrypt_text",
    // مطابقة الأنماط والتعابير النمطية (Regex)
    "يطابق_نمط",
    "regex_match",
    "استخرج_نمط",
    "regex_find_all",
    "استبدل_نمط",
    "regex_replace",
    // أدوات المنصة والنظام والوقت الدقيق
    "معلومات_النظام",
    "system_info",
    "نم",
    "انتظر_ثواني",
    "sleep",
    "وقت_دقيق_ملي",
    "time_ms",
];

/// التحقق مما إذا كان الاسم يشير إلى دالة قياسية موسّعة
pub fn is_ext_builtin(name: &str) -> bool {
    ALL_EXT_BUILTINS.contains(&name)
}

fn require_ext_args(name: &str, args: &[Value], expected: usize) -> Result<(), String> {
    if args.len() != expected {
        Err(format!(
            "الدالة '{}' تتوقع {} معاملات، استلمت {}",
            name,
            expected,
            args.len()
        ))
    } else {
        Ok(())
    }
}

fn expect_text<'a>(name: &str, val: &'a Value) -> Result<&'a str, String> {
    match val {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("الدالة '{}' تتطلب نصًا", name)),
    }
}

/// تنفيذ دالة مدمجة من المكتبة القياسية الموسّعة
pub fn eval_ext_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        // ==================== 1. التشفير والتجزئة ====================
        "تجزئة_sha256" | "تجزئه_sha256" | "sha256" => {
            require_ext_args(name, &args, 1)?;
            let text = expect_text(name, &args[0])?;
            Ok(Value::Text(sha256_hex(text.as_bytes())))
        }
        "توقيع_hmac" | "hmac_sha256" => {
            require_ext_args(name, &args, 2)?;
            let msg = expect_text(name, &args[0])?;
            let secret = expect_text(name, &args[1])?;
            Ok(Value::Text(hmac_sha256_hex(msg.as_bytes(), secret.as_bytes())))
        }
        "ترميز_ست_عشري" | "hex_encode" => {
            require_ext_args(name, &args, 1)?;
            let text = expect_text(name, &args[0])?;
            Ok(Value::Text(hex_encode_bytes(text.as_bytes())))
        }
        "فك_ست_عشري" | "hex_decode" => {
            require_ext_args(name, &args, 1)?;
            let hex_str = expect_text(name, &args[0])?;
            let bytes = hex_decode_bytes(hex_str)?;
            let decoded = String::from_utf8(bytes)
                .map_err(|_| "البيانات المفكوكة من الست عشري ليست نص UTF-8 صالحًا".to_string())?;
            Ok(Value::Text(decoded))
        }
        "معرف_فريد" | "uuid" => {
            require_ext_args(name, &args, 0)?;
            Ok(Value::Text(generate_uuid_v4()))
        }
        "شفر_نص" | "encrypt_text" => {
            require_ext_args(name, &args, 2)?;
            let plaintext = expect_text(name, &args[0])?;
            let password = expect_text(name, &args[1])?;
            Ok(Value::Text(encrypt_text_stream(plaintext, password)))
        }
        "فك_تشفير_نص" | "decrypt_text" => {
            require_ext_args(name, &args, 2)?;
            let ciphertext_hex = expect_text(name, &args[0])?;
            let password = expect_text(name, &args[1])?;
            let plaintext = decrypt_text_stream(ciphertext_hex, password)?;
            Ok(Value::Text(plaintext))
        }

        // ==================== 2. الأنماط والتعابير النمطية ====================
        "يطابق_نمط" | "regex_match" => {
            require_ext_args(name, &args, 2)?;
            let source = expect_text(name, &args[0])?;
            let pattern = expect_text(name, &args[1])?;
            let matched = regex_is_match(source, pattern)?;
            Ok(Value::Bool(matched))
        }
        "استخرج_نمط" | "regex_find_all" => {
            require_ext_args(name, &args, 2)?;
            let source = expect_text(name, &args[0])?;
            let pattern = expect_text(name, &args[1])?;
            let matches = regex_find_all(source, pattern)?;
            Ok(Value::list(matches.into_iter().map(Value::Text).collect()))
        }
        "استبدل_نمط" | "regex_replace" => {
            require_ext_args(name, &args, 3)?;
            let source = expect_text(name, &args[0])?;
            let pattern = expect_text(name, &args[1])?;
            let replacement = expect_text(name, &args[2])?;
            let replaced = regex_replace_all(source, pattern, replacement)?;
            Ok(Value::Text(replaced))
        }

        // ==================== 3. معلومات النظام والمنصة والوقت ====================
        "معلومات_النظام" | "system_info" => {
            require_ext_args(name, &args, 0)?;
            Ok(build_system_info())
        }
        "نم" | "انتظر_ثواني" | "sleep" => {
            require_ext_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(secs) => {
                    if !secs.is_finite() {
                        return Err("مدة الانتظار يجب أن تكون رقمًا منتهيًا".into());
                    }
                    let clamped = secs.max(0.0).min(3600.0);
                    #[cfg(not(target_arch = "wasm32"))]
                    if clamped > 0.0 {
                        std::thread::sleep(Duration::from_secs_f64(clamped));
                    }
                    #[cfg(target_arch = "wasm32")]
                    let _ = clamped;
                    Ok(Value::Bool(true))
                }
                _ => Err(format!("الدالة '{}' تتطلب عدد الثواني كرقم", name)),
            }
        }
        "وقت_دقيق_ملي" | "time_ms" => {
            require_ext_args(name, &args, 0)?;
            #[cfg(not(target_arch = "wasm32"))]
            {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("تعذّرت قراءة الوقت الدقيق: {}", e))?;
                Ok(Value::Number(now.as_secs_f64() * 1000.0))
            }
            #[cfg(target_arch = "wasm32")]
            {
                let ticks = ENTROPY_COUNTER.fetch_add(1, Ordering::Relaxed);
                Ok(Value::Number(ticks as f64))
            }
        }

        _ => Err(format!("دالة قياسية موسّعة غير معروفة: '{}'", name)),
    }
}

// ============================================================================
// 1. خوارزميات التشفير والتجزئة (FIPS 180-4 SHA-256 + HMAC + UUID v4 + Cipher)
// ============================================================================

const SHA256_H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// حساب تجزئة SHA-256 وفق معيار FIPS 180-4 وإرجاع 32 بايت
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut state = SHA256_H0;

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(data.len() + 72);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while (padded.len() + 8) % 64 != 0 {
        padded.push(0x00);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            let base = i * 4;
            w[i] = u32::from_be_bytes([
                chunk[base],
                chunk[base + 1],
                chunk[base + 2],
                chunk[base + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// حساب تجزئة SHA-256 كسلسلة ست عشرية صغيرة (64 محرفًا)
pub fn sha256_hex(data: &[u8]) -> String {
    hex_encode_bytes(&sha256_bytes(data))
}

/// حساب توقيع HMAC-SHA256 وفق معيار RFC 2104
pub fn hmac_sha256_bytes(message: &[u8], key: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let hashed_key = sha256_bytes(key);
        key_block[..32].copy_from_slice(&hashed_key);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner_input = Vec::with_capacity(64 + message.len());
    inner_input.extend_from_slice(&ipad);
    inner_input.extend_from_slice(message);
    let inner_hash = sha256_bytes(&inner_input);

    let mut outer_input = [0u8; 96];
    outer_input[..64].copy_from_slice(&opad);
    outer_input[64..].copy_from_slice(&inner_hash);
    sha256_bytes(&outer_input)
}

/// حساب توقيع HMAC-SHA256 كسلسلة ست عشرية صغيرة (64 محرفًا)
pub fn hmac_sha256_hex(message: &[u8], key: &[u8]) -> String {
    hex_encode_bytes(&hmac_sha256_bytes(message, key))
}

/// ترميز مصفوفة بايتات إلى سلسلة ست عشرية صغيرة
pub fn hex_encode_bytes(bytes: &[u8]) -> String {
    const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX_LOWER[(b >> 4) as usize] as char);
        out.push(HEX_LOWER[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_nibble(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("محرف ست عشري غير صالح: '{}'", b as char)),
    }
}

/// فك ترميز سلسلة ست عشرية إلى بايتات
pub fn hex_decode_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let trimmed = hex.trim();
    if trimmed.len() % 2 != 0 {
        return Err("النص الست عشري يجب أن يكون ذا طول زوجي".into());
    }
    let raw = trimmed.as_bytes();
    let mut out = Vec::with_capacity(raw.len() / 2);
    let mut i = 0;
    while i < raw.len() {
        let hi = hex_nibble(raw[i])?;
        let lo = hex_nibble(raw[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

static ENTROPY_COUNTER: AtomicU64 = AtomicU64::new(0x9e37_79b9_7f4a_7c15);

/// توليد 16 بايت عشوائية عالية الإنتروبيا بالاعتماد على الوقت والعداد ومؤشرات الذاكرة وSHA-256
fn random_entropy_16() -> [u8; 16] {
    let counter = ENTROPY_COUNTER.fetch_add(0xbf58_476d_1ce4_e5b9, Ordering::Relaxed);
    #[cfg(not(target_arch = "wasm32"))]
    let (nanos, stack_addr, pid) = {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let mono = Instant::now();
        let addr = &mono as *const Instant as usize as u64;
        let p = std::process::id() as u64;
        (n, addr, p)
    };
    #[cfg(target_arch = "wasm32")]
    let (nanos, stack_addr, pid) = {
        let dummy = counter;
        let addr = &dummy as *const u64 as usize as u64;
        (counter as u128, addr, 1u64)
    };

    // خلط xorshift64* إضافي قبل تمريره عبر SHA-256
    let mut x = counter
        ^ (nanos as u64)
        ^ ((nanos >> 64) as u64).rotate_left(21)
        ^ stack_addr.rotate_right(13)
        ^ pid.rotate_left(37);
    if x == 0 {
        x = 0x2545_f491_4f6c_dd1d;
    }
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    let mixed = x.wrapping_mul(0x2545_f491_4f6c_dd1d);

    let mut seed_material = [0u8; 40];
    seed_material[0..8].copy_from_slice(&counter.to_le_bytes());
    seed_material[8..24].copy_from_slice(&nanos.to_le_bytes());
    seed_material[24..32].copy_from_slice(&stack_addr.to_le_bytes());
    seed_material[32..40].copy_from_slice(&mixed.to_le_bytes());

    let digest = sha256_bytes(&seed_material);
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest[..16]);
    out
}

/// توليد معرّف فريد عالميًا بصيغة RFC 4122 UUID v4
pub fn generate_uuid_v4() -> String {
    let mut b = random_entropy_16();
    // ضبط الإصدار 4 (0100xxxx) والمتغير RFC 4122 (10xxxxxx)
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;

    format!(
        "{}-{}-{}-{}-{}",
        hex_encode_bytes(&b[0..4]),
        hex_encode_bytes(&b[4..6]),
        hex_encode_bytes(&b[6..8]),
        hex_encode_bytes(&b[8..10]),
        hex_encode_bytes(&b[10..16]),
    )
}

fn derive_stream_subkeys(password: &[u8], nonce: &[u8; 16]) -> ([u8; 32], [u8; 32]) {
    let mut material = Vec::with_capacity(16 + password.len());
    material.extend_from_slice(nonce);
    material.extend_from_slice(password);
    let enc_key = hmac_sha256_bytes(b"lisan-stream-enc-v1", &material);
    let mac_key = hmac_sha256_bytes(b"lisan-stream-mac-v1", &material);
    (enc_key, mac_key)
}

fn apply_sha256_keystream(input: &[u8], enc_key: &[u8; 32], nonce: &[u8; 16]) -> Vec<u8> {
    let mut output = Vec::with_capacity(input.len());
    let mut block_input = [0u8; 24];
    block_input[..16].copy_from_slice(nonce);

    for (counter, chunk) in input.chunks(32).enumerate() {
        block_input[16..24].copy_from_slice(&(counter as u64).to_be_bytes());
        let keystream = hmac_sha256_bytes(&block_input, enc_key);
        for (i, &byte) in chunk.iter().enumerate() {
            output.push(byte ^ keystream[i]);
        }
    }
    output
}

/// تشفير نص متناظر مع مصادقة (Nonce + XOR-HMAC-SHA256 Keystream + Tag) مرمّز بالست عشري
pub fn encrypt_text_stream(plaintext: &str, password: &str) -> String {
    let nonce = random_entropy_16();
    let (enc_key, mac_key) = derive_stream_subkeys(password.as_bytes(), &nonce);
    let ciphertext = apply_sha256_keystream(plaintext.as_bytes(), &enc_key, &nonce);

    let mut auth_data = Vec::with_capacity(16 + ciphertext.len());
    auth_data.extend_from_slice(&nonce);
    auth_data.extend_from_slice(&ciphertext);
    let full_tag = hmac_sha256_bytes(&auth_data, &mac_key);

    let mut packet = Vec::with_capacity(16 + ciphertext.len() + 16);
    packet.extend_from_slice(&nonce);
    packet.extend_from_slice(&ciphertext);
    packet.extend_from_slice(&full_tag[..16]);
    hex_encode_bytes(&packet)
}

/// فك تشفير نص مشفر بواسطة `encrypt_text_stream` والتحقق من سلامة كلمة المرور والبيانات
pub fn decrypt_text_stream(ciphertext_hex: &str, password: &str) -> Result<String, String> {
    let packet = hex_decode_bytes(ciphertext_hex)?;
    if packet.len() < 32 {
        return Err("فشل فك التشفير: النص المشفر قصير جدًا أو غير صالح".into());
    }

    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&packet[..16]);
    let ciphertext = &packet[16..packet.len() - 16];
    let expected_tag = &packet[packet.len() - 16..];

    let (enc_key, mac_key) = derive_stream_subkeys(password.as_bytes(), &nonce);

    let mut auth_data = Vec::with_capacity(16 + ciphertext.len());
    auth_data.extend_from_slice(&nonce);
    auth_data.extend_from_slice(ciphertext);
    let computed_tag = hmac_sha256_bytes(&auth_data, &mac_key);

    // مقارنة بزمن ثابت لتجنب هجمات التوقيت
    let mut diff = 0u8;
    for (a, b) in expected_tag.iter().zip(computed_tag[..16].iter()) {
        diff |= a ^ b;
    }
    if diff != 0 {
        return Err("فشل فك التشفير: كلمة المرور غير صحيحة أو البيانات تالفة".into());
    }

    let plain_bytes = apply_sha256_keystream(ciphertext, &enc_key, &nonce);
    String::from_utf8(plain_bytes)
        .map_err(|_| "فشل فك التشفير: الناتج ليس نص UTF-8 صالحًا".into())
}

// ============================================================================
// 2. محرك مطابقة الأنماط والتعابير النمطية (Pure-Rust Unicode Regex Engine)
// ============================================================================

const MAX_REGEX_STEPS: usize = 500_000;

#[derive(Debug, Clone)]
enum RegexNode {
    Concat(Vec<RegexNode>),
    Alt(Box<RegexNode>, Box<RegexNode>),
    Repeat {
        node: Box<RegexNode>,
        min: usize,
        max: Option<usize>,
    },
    AnchorStart,
    AnchorEnd,
    Any,
    Digit(bool),
    Word(bool),
    Space(bool),
    Literal(char),
    CharClass {
        negated: bool,
        items: Vec<ClassItem>,
    },
}

#[derive(Debug, Clone)]
enum ClassItem {
    Single(char),
    Range(char, char),
    Digit(bool),
    Word(bool),
    Space(bool),
}

fn is_regex_digit(c: char) -> bool {
    c.is_ascii_digit() || ('\u{0660}'..='\u{0669}').contains(&c)
}

fn is_regex_word(c: char) -> bool {
    c == '_'
        || c.is_alphanumeric()
        || matches!(c, '\u{064B}'..='\u{065F}' | '\u{0670}')
}

fn is_regex_space(c: char) -> bool {
    c.is_whitespace()
}

struct RegexParser {
    chars: Vec<char>,
    pos: usize,
}

impl RegexParser {
    fn new(pattern: &str) -> Self {
        Self {
            chars: pattern.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn parse(&mut self) -> Result<RegexNode, String> {
        let node = self.parse_alt()?;
        if let Some(unexpected) = self.peek() {
            return Err(format!("محرف غير متوقع في النمط: '{}'", unexpected));
        }
        Ok(node)
    }

    fn parse_alt(&mut self) -> Result<RegexNode, String> {
        let mut left = self.parse_concat()?;
        while self.peek() == Some('|') {
            self.next();
            let right = self.parse_concat()?;
            left = RegexNode::Alt(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<RegexNode, String> {
        let mut items = Vec::new();
        while let Some(ch) = self.peek() {
            if ch == ')' || ch == '|' {
                break;
            }
            items.push(self.parse_quantified()?);
        }
        if items.len() == 1 {
            Ok(items.pop().unwrap())
        } else {
            Ok(RegexNode::Concat(items))
        }
    }

    fn parse_quantified(&mut self) -> Result<RegexNode, String> {
        let atom = self.parse_atom()?;
        if matches!(atom, RegexNode::AnchorStart | RegexNode::AnchorEnd) {
            return Ok(atom);
        }
        match self.peek() {
            Some('*') => {
                self.next();
                Ok(RegexNode::Repeat {
                    node: Box::new(atom),
                    min: 0,
                    max: None,
                })
            }
            Some('+') => {
                self.next();
                Ok(RegexNode::Repeat {
                    node: Box::new(atom),
                    min: 1,
                    max: None,
                })
            }
            Some('?') => {
                self.next();
                Ok(RegexNode::Repeat {
                    node: Box::new(atom),
                    min: 0,
                    max: Some(1),
                })
            }
            Some('{') => {
                if let Some((min, max)) = self.try_parse_brace_quantifier()? {
                    Ok(RegexNode::Repeat {
                        node: Box::new(atom),
                        min,
                        max,
                    })
                } else {
                    Ok(atom)
                }
            }
            _ => Ok(atom),
        }
    }

    fn try_parse_brace_quantifier(&mut self) -> Result<Option<(usize, Option<usize>)>, String> {
        let saved = self.pos;
        self.next(); // consume '{'
        let mut min_str = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                min_str.push(c);
                self.next();
            } else {
                break;
            }
        }
        if min_str.is_empty() {
            self.pos = saved;
            return Ok(None);
        }
        let min: usize = min_str
            .parse()
            .map_err(|_| "عدد تكرار غير صالح في النمط".to_string())?;

        match self.peek() {
            Some('}') => {
                self.next();
                Ok(Some((min, Some(min))))
            }
            Some(',') => {
                self.next();
                if self.peek() == Some('}') {
                    self.next();
                    return Ok(Some((min, None)));
                }
                let mut max_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        max_str.push(c);
                        self.next();
                    } else {
                        break;
                    }
                }
                if max_str.is_empty() || self.peek() != Some('}') {
                    self.pos = saved;
                    return Ok(None);
                }
                self.next(); // consume '}'
                let max: usize = max_str
                    .parse()
                    .map_err(|_| "الحد الأعلى للتكرار غير صالح في النمط".to_string())?;
                if max < min {
                    return Err("الحد الأعلى للتكرار أصغر من الحد الأدنى في النمط".into());
                }
                Ok(Some((min, Some(max))))
            }
            _ => {
                self.pos = saved;
                Ok(None)
            }
        }
    }

    fn parse_atom(&mut self) -> Result<RegexNode, String> {
        let ch = self
            .next()
            .ok_or_else(|| "نهاية غير متوقعة في النمط".to_string())?;
        match ch {
            '^' => Ok(RegexNode::AnchorStart),
            '$' => Ok(RegexNode::AnchorEnd),
            '.' => Ok(RegexNode::Any),
            '*' | '+' | '?' => Err(format!("محدد التكرار '{}' يجب أن يسبقه عنصر في النمط", ch)),
            '(' => {
                let inner = self.parse_alt()?;
                if self.next() != Some(')') {
                    return Err("قوس '(' غير مغلق في النمط".into());
                }
                Ok(inner)
            }
            '[' => self.parse_char_class(),
            '\\' => self.parse_escape(),
            other => Ok(RegexNode::Literal(other)),
        }
    }

    fn parse_escape(&mut self) -> Result<RegexNode, String> {
        let esc = self
            .next()
            .ok_or_else(|| "رمز هروب '\\' غير مكتمل في نهاية النمط".to_string())?;
        match esc {
            'd' => Ok(RegexNode::Digit(true)),
            'D' => Ok(RegexNode::Digit(false)),
            'w' => Ok(RegexNode::Word(true)),
            'W' => Ok(RegexNode::Word(false)),
            's' => Ok(RegexNode::Space(true)),
            'S' => Ok(RegexNode::Space(false)),
            'n' => Ok(RegexNode::Literal('\n')),
            'r' => Ok(RegexNode::Literal('\r')),
            't' => Ok(RegexNode::Literal('\t')),
            other => Ok(RegexNode::Literal(other)),
        }
    }

    fn parse_class_escape(&mut self) -> Result<ClassItem, String> {
        let esc = self
            .next()
            .ok_or_else(|| "رمز هروب '\\' غير مكتمل داخل فئة الأحرف".to_string())?;
        match esc {
            'd' => Ok(ClassItem::Digit(true)),
            'D' => Ok(ClassItem::Digit(false)),
            'w' => Ok(ClassItem::Word(true)),
            'W' => Ok(ClassItem::Word(false)),
            's' => Ok(ClassItem::Space(true)),
            'S' => Ok(ClassItem::Space(false)),
            'n' => Ok(ClassItem::Single('\n')),
            'r' => Ok(ClassItem::Single('\r')),
            't' => Ok(ClassItem::Single('\t')),
            other => Ok(ClassItem::Single(other)),
        }
    }

    fn parse_char_class(&mut self) -> Result<RegexNode, String> {
        let negated = if self.peek() == Some('^') {
            self.next();
            true
        } else {
            false
        };

        let mut items = Vec::new();
        let mut first = true;

        loop {
            let ch = self
                .next()
                .ok_or_else(|| "فئة الأحرف '[' غير مغلقة بـ ']' في النمط".to_string())?;
            if ch == ']' && !first {
                break;
            }
            first = false;

            let item = if ch == '\\' {
                self.parse_class_escape()?
            } else {
                ClassItem::Single(ch)
            };

            if let ClassItem::Single(start_ch) = item {
                if self.peek() == Some('-') && self.chars.get(self.pos + 1) != Some(&']') {
                    self.next(); // consume '-'
                    let end_raw = self
                        .next()
                        .ok_or_else(|| "نطاق أحرف غير مكتمل داخل '[' في النمط".to_string())?;
                    let end_ch = if end_raw == '\\' {
                        match self.parse_class_escape()? {
                            ClassItem::Single(c) => c,
                            _ => return Err("نهاية نطاق الأحرف داخل '[' يجب أن تكون حرفًا مفردًا".into()),
                        }
                    } else {
                        end_raw
                    };
                    if start_ch > end_ch {
                        return Err(format!(
                            "نطاق أحرف معكوس [{}-{}] في النمط",
                            start_ch, end_ch
                        ));
                    }
                    items.push(ClassItem::Range(start_ch, end_ch));
                    continue;
                }
            }

            items.push(item);
        }

        Ok(RegexNode::CharClass { negated, items })
    }
}

fn char_matches_class(c: char, negated: bool, items: &[ClassItem]) -> bool {
    let mut matched = false;
    for item in items {
        let hit = match item {
            ClassItem::Single(expected) => c == *expected,
            ClassItem::Range(lo, hi) => (*lo..=*hi).contains(&c),
            ClassItem::Digit(sense) => is_regex_digit(c) == *sense,
            ClassItem::Word(sense) => is_regex_word(c) == *sense,
            ClassItem::Space(sense) => is_regex_space(c) == *sense,
        };
        if hit {
            matched = true;
            break;
        }
    }
    if negated {
        !matched
    } else {
        matched
    }
}

#[derive(Clone)]
enum MatchStep<'a> {
    Node(&'a RegexNode),
    RepeatMore {
        node: &'a RegexNode,
        min: usize,
        max: Option<usize>,
        count: usize,
        last_pos: usize,
    },
}

fn run_steps<'a>(
    mut stack: Vec<MatchStep<'a>>,
    chars: &[char],
    mut pos: usize,
    steps: &mut usize,
) -> Result<Option<usize>, String> {
    loop {
        *steps += 1;
        if *steps > MAX_REGEX_STEPS {
            return Err("تجاوزت مطابقة النمط الحد الأقصى لخطوات البحث الآمن".into());
        }

        let Some(step) = stack.pop() else {
            return Ok(Some(pos));
        };

        match step {
            MatchStep::Node(node) => match node {
                RegexNode::Concat(children) => {
                    for child in children.iter().rev() {
                        stack.push(MatchStep::Node(child));
                    }
                }
                RegexNode::Alt(left, right) => {
                    let mut left_stack = stack.clone();
                    left_stack.push(MatchStep::Node(left));
                    if let Some(end) = run_steps(left_stack, chars, pos, steps)? {
                        return Ok(Some(end));
                    }
                    stack.push(MatchStep::Node(right));
                }
                RegexNode::Repeat { node, min, max } => {
                    stack.push(MatchStep::RepeatMore {
                        node,
                        min: *min,
                        max: *max,
                        count: 0,
                        last_pos: usize::MAX,
                    });
                }
                RegexNode::AnchorStart => {
                    if pos != 0 {
                        return Ok(None);
                    }
                }
                RegexNode::AnchorEnd => {
                    if pos != chars.len() {
                        return Ok(None);
                    }
                }
                RegexNode::Any => {
                    if pos < chars.len() && chars[pos] != '\n' {
                        pos += 1;
                    } else {
                        return Ok(None);
                    }
                }
                RegexNode::Digit(sense) => {
                    if pos < chars.len() && (is_regex_digit(chars[pos]) == *sense) {
                        pos += 1;
                    } else {
                        return Ok(None);
                    }
                }
                RegexNode::Word(sense) => {
                    if pos < chars.len() && (is_regex_word(chars[pos]) == *sense) {
                        pos += 1;
                    } else {
                        return Ok(None);
                    }
                }
                RegexNode::Space(sense) => {
                    if pos < chars.len() && (is_regex_space(chars[pos]) == *sense) {
                        pos += 1;
                    } else {
                        return Ok(None);
                    }
                }
                RegexNode::Literal(expected) => {
                    if pos < chars.len() && chars[pos] == *expected {
                        pos += 1;
                    } else {
                        return Ok(None);
                    }
                }
                RegexNode::CharClass { negated, items } => {
                    if pos < chars.len() && char_matches_class(chars[pos], *negated, items) {
                        pos += 1;
                    } else {
                        return Ok(None);
                    }
                }
            },
            MatchStep::RepeatMore {
                node,
                min,
                max,
                count,
                last_pos,
            } => {
                // منع الحلقات اللانهائية على التطابقات الصفرية الطول بعد استيفاء الحد الأدنى
                if count > min && pos == last_pos {
                    continue;
                }
                if count < min {
                    stack.push(MatchStep::RepeatMore {
                        node,
                        min,
                        max,
                        count: count + 1,
                        last_pos: pos,
                    });
                    stack.push(MatchStep::Node(node));
                } else {
                    let can_grow = max.map_or(true, |m| count < m);
                    if can_grow {
                        let mut greedy_stack = stack.clone();
                        greedy_stack.push(MatchStep::RepeatMore {
                            node,
                            min,
                            max,
                            count: count + 1,
                            last_pos: pos,
                        });
                        greedy_stack.push(MatchStep::Node(node));
                        if let Some(end) = run_steps(greedy_stack, chars, pos, steps)? {
                            return Ok(Some(end));
                        }
                    }
                    // التراجع والمحاولة بالبقية الحالية في المكدس
                }
            }
        }
    }
}

fn match_at(ast: &RegexNode, chars: &[char], start: usize) -> Result<Option<usize>, String> {
    let mut steps = 0;
    run_steps(vec![MatchStep::Node(ast)], chars, start, &mut steps)
}

/// التحقق مما إذا كان النمط يطابق النص (كليًا إن احتوى مرساة أو جزئيًا في أي موضع)
pub fn regex_is_match(source: &str, pattern: &str) -> Result<bool, String> {
    let ast = RegexParser::new(pattern).parse()?;
    let chars: Vec<char> = source.chars().collect();
    for start in 0..=chars.len() {
        if match_at(&ast, &chars, start)?.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// استخراج جميع التطابقات غير المتداخلة للنمط من النص
pub fn regex_find_all(source: &str, pattern: &str) -> Result<Vec<String>, String> {
    let ast = RegexParser::new(pattern).parse()?;
    let chars: Vec<char> = source.chars().collect();
    let mut results = Vec::new();
    let mut pos = 0;

    while pos <= chars.len() {
        if let Some(end) = match_at(&ast, &chars, pos)? {
            let matched_str: String = chars[pos..end].iter().collect();
            results.push(matched_str);
            if end == pos {
                pos += 1;
            } else {
                pos = end;
            }
        } else {
            pos += 1;
        }
    }
    Ok(results)
}

/// استبدال جميع التطابقات غير المتداخلة للنمط في النص بنص بديل
pub fn regex_replace_all(
    source: &str,
    pattern: &str,
    replacement: &str,
) -> Result<String, String> {
    let ast = RegexParser::new(pattern).parse()?;
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut pos = 0;

    while pos <= chars.len() {
        if let Some(end) = match_at(&ast, &chars, pos)? {
            out.push_str(replacement);
            if end == pos {
                if pos < chars.len() {
                    out.push(chars[pos]);
                }
                pos += 1;
            } else {
                pos = end;
            }
        } else {
            if pos < chars.len() {
                out.push(chars[pos]);
            }
            pos += 1;
        }
    }
    Ok(out)
}

// ============================================================================
// 3. معلومات النظام والمنصة
// ============================================================================

fn build_system_info() -> Value {
    #[cfg(not(target_arch = "wasm32"))]
    let (cores, home_dir, temp_dir, os_name) = {
        let c = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1) as f64;
        let h = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        let t = std::env::temp_dir().to_string_lossy().into_owned();
        (c, h, t, std::env::consts::OS.to_string())
    };
    #[cfg(target_arch = "wasm32")]
    let (cores, home_dir, temp_dir, os_name) = (
        1.0,
        "/wasm".to_string(),
        "/tmp".to_string(),
        "wasm32".to_string(),
    );

    Value::map(vec![
        ("النظام".to_string(), Value::Text(os_name)),
        ("المعمارية".to_string(), Value::Text(std::env::consts::ARCH.to_string())),
        ("عائلة_النظام".to_string(), Value::Text(std::env::consts::FAMILY.to_string())),
        ("الأنوية".to_string(), Value::Number(cores)),
        ("مجلد_المستخدم".to_string(), Value::Text(home_dir)),
        ("مجلد_مؤقت".to_string(), Value::Text(temp_dir)),
        ("إصدار_لسان".to_string(), Value::Text("1.2.0".to_string())),
    ])
}

// ============================================================================
// 4. الأصناف القياسية بلغة لسان (قاعدة_بيانات، تشفير_أمان، محلل_أنماط)
// ============================================================================

/// المكتبة القياسية الكائنية المدمجة بلغة لسان لقواعد البيانات المستديمة والتشفير والأنماط
pub const STDLIB_EXT_LISAN: &str = r##"
# المكتبة القياسية الموسّعة للغة لسان: قواعد البيانات المستديمة، التشفير، ومحلل الأنماط

صنف قاعدة_بيانات {
    دالة بناء(مسار_الملف = "") {
        هذا.مسار = مسار_الملف
        هذا.جداول = {}
        إذا (مسار_الملف != "") {
            إذا (ملف_موجود(مسار_الملف)) {
                دع محتوى_الملف = نظف(اقرأ_ملف(مسار_الملف))
                إذا (محتوى_الملف != "") {
                    هذا.جداول = تحليل_json(محتوى_الملف)
                }
            }
        }
    }

    دالة أنشئ_جدول(اسم_الجدول) {
        دع جداول = هذا.جداول
        إذا (ليس يحتوي(جداول، اسم_الجدول)) {
            جداول[اسم_الجدول] = []
            هذا.جداول = جداول
        }
        أرجع صحيح
    }

    دالة أدرج(اسم_الجدول، سجل_قاموس) {
        هذا.أنشئ_جدول(اسم_الجدول)
        دع جداول = هذا.جداول
        دع قائمة_السجلات = جداول[اسم_الجدول]
        إذا (ليس يحتوي(سجل_قاموس، "معرف")) {
            دع أعلى_معرف = 0
            لكل عنصر في قائمة_السجلات {
                إذا (يحتوي(عنصر، "معرف")) {
                    دع رقم_المعرف = عنصر["معرف"]
                    إذا (نوع(رقم_المعرف) == "رقم") {
                        إذا (رقم_المعرف > أعلى_معرف) {
                            أعلى_معرف = رقم_المعرف
                        }
                    }
                }
            }
            سجل_قاموس["معرف"] = أعلى_معرف + 1
        }
        قائمة_السجلات = أضف(قائمة_السجلات، سجل_قاموس)
        جداول[اسم_الجدول] = قائمة_السجلات
        هذا.جداول = جداول
        أرجع سجل_قاموس
    }

    دالة الكل(اسم_الجدول) {
        دع جداول = هذا.جداول
        إذا (ليس يحتوي(جداول، اسم_الجدول)) {
            أرجع []
        }
        أرجع جداول[اسم_الجدول]
    }

    دالة ابحث_بحقل(اسم_الجدول، اسم_الحقل، القيمة) {
        دع السجلات = هذا.الكل(اسم_الجدول)
        دع النتائج = []
        لكل سجل في السجلات {
            إذا (يحتوي(سجل، اسم_الحقل)) {
                إذا (سجل[اسم_الحقل] == القيمة) {
                    النتائج = أضف(النتائج، سجل)
                }
            }
        }
        أرجع النتائج
    }

    دالة احذف_بمعرف(اسم_الجدول، المعرف) {
        دع جداول = هذا.جداول
        إذا (ليس يحتوي(جداول، اسم_الجدول)) {
            أرجع خطأ
        }
        دع السجلات = جداول[اسم_الجدول]
        دع المتبقية = []
        دع تم_الحذف = خطأ
        لكل سجل في السجلات {
            دع يطابق_الحذف = خطأ
            إذا (يحتوي(سجل، "معرف")) {
                إذا (سجل["معرف"] == المعرف) {
                    يطابق_الحذف = صحيح
                    تم_الحذف = صحيح
                }
            }
            إذا (ليس يطابق_الحذف) {
                المتبقية = أضف(المتبقية، سجل)
            }
        }
        جداول[اسم_الجدول] = المتبقية
        هذا.جداول = جداول
        أرجع تم_الحذف
    }

    دالة عدد_السجلات(اسم_الجدول) {
        أرجع طول(هذا.الكل(اسم_الجدول))
    }

    دالة أسماء_الجداول() {
        أرجع مفاتيح(هذا.جداول)
    }

    دالة احفظ() {
        إذا (هذا.مسار != "") {
            دع نص_جيسون = ترميز_json(هذا.جداول)
            اكتب_ملف(هذا.مسار، نص_جيسون)
            أرجع صحيح
        }
        أرجع خطأ
    }
}

صنف تشفير_أمان {
    دالة بناء(مفتاح_افتراضي = "") {
        هذا.مفتاح = مفتاح_افتراضي
    }

    دالة تجزئة(نص_المدخل) {
        أرجع تجزئة_sha256(نص_المدخل)
    }

    دالة وقع(رسالة، مفتاح_سري = "") {
        دع مفتاح_فعلي = مفتاح_سري
        إذا (مفتاح_فعلي == "") {
            مفتاح_فعلي = هذا.مفتاح
        }
        أرجع توقيع_hmac(رسالة، مفتاح_فعلي)
    }

    دالة تحقق_توقيع(رسالة، التوقيع_المتوقع، مفتاح_سري = "") {
        دع محسوب = هذا.وقع(رسالة، مفتاح_سري)
        أرجع محسوب == التوقيع_المتوقع
    }

    دالة شفر(نص_واضح، كلمة_مرور = "") {
        دع كلمة_فعلية = كلمة_مرور
        إذا (كلمة_فعلية == "") {
            كلمة_فعلية = هذا.مفتاح
        }
        أرجع شفر_نص(نص_واضح، كلمة_فعلية)
    }

    دالة فك_تشفير(نص_مشفر، كلمة_مرور = "") {
        دع كلمة_فعلية = كلمة_مرور
        إذا (كلمة_فعلية == "") {
            كلمة_فعلية = هذا.مفتاح
        }
        أرجع فك_تشفير_نص(نص_مشفر، كلمة_فعلية)
    }

    دالة معرف_جديد() {
        أرجع معرف_فريد()
    }

    دالة إلى_ست_عشري(نص_عادي) {
        أرجع ترميز_ست_عشري(نص_عادي)
    }

    دالة من_ست_عشري(نص_مرمز) {
        أرجع فك_ست_عشري(نص_مرمز)
    }
}

صنف محلل_أنماط {
    دالة بناء(نمط) {
        هذا.نمط = نمط
    }

    دالة يطابق(نص_المصدر) {
        أرجع يطابق_نمط(نص_المصدر، هذا.نمط)
    }

    دالة استخرج(نص_المصدر) {
        أرجع استخرج_نمط(نص_المصدر، هذا.نمط)
    }

    دالة استبدل(نص_المصدر، البديل) {
        أرجع استبدل_نمط(نص_المصدر، هذا.نمط، البديل)
    }

    دالة عدد_التطابقات(نص_المصدر) {
        أرجع طول(استخرج_نمط(نص_المصدر، هذا.نمط))
    }
}
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_nist_vectors() {
        // NIST FIPS 180-4 standard vectors
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn test_hmac_sha256_rfc4231() {
        let mac = hmac_sha256_hex(b"what do ya want for nothing?", b"Jefe");
        assert_eq!(
            mac,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn test_cipher_arabic_utf8_roundtrip_and_tamper_detection() {
        let original = "بسم الله الرحمن الرحيم — لغة لسان العربية المتكاملة 2026!";
        let password = "مفتاح_تشفير_عربي_سري_للغاية";

        let encrypted = encrypt_text_stream(original, password);
        assert_ne!(encrypted, original);
        assert!(encrypted.len() >= 64);

        let decrypted = decrypt_text_stream(&encrypted, password).expect("decryption must succeed");
        assert_eq!(decrypted, original);

        // كلمة مرور خاطئة يجب أن تفشل بأمان
        assert!(decrypt_text_stream(&encrypted, "كلمة_مرور_خاطئة").is_err());

        // نص فارغ
        let empty_enc = encrypt_text_stream("", password);
        assert_eq!(decrypt_text_stream(&empty_enc, password).unwrap(), "");
    }

    #[test]
    fn test_uuid_v4_rfc4122_format() {
        let u1 = generate_uuid_v4();
        let u2 = generate_uuid_v4();
        assert_eq!(u1.len(), 36);
        assert_ne!(u1, u2);
        let bytes = u1.as_bytes();
        assert_eq!(bytes[8], b'-');
        assert_eq!(bytes[13], b'-');
        assert_eq!(bytes[14], b'4');
        assert_eq!(bytes[18], b'-');
        assert!(matches!(bytes[19], b'8' | b'9' | b'a' | b'b'));
        assert_eq!(bytes[23], b'-');
    }

    #[test]
    fn test_regex_matching_extraction_and_replacement() {
        // مطابقة المرساة ^ و $
        assert!(regex_is_match("lisan2026", "^[a-z]+\\d+$").unwrap());
        assert!(!regex_is_match("lisan2026!", "^[a-z]+\\d+$").unwrap());

        // استخراج الأرقام والكلمات العربية والإنجليزية والبريد الإلكتروني
        let nums = regex_find_all("السعر 150 ريال والخصم 25 بالمئة", "\\d+").unwrap();
        assert_eq!(nums, vec!["150", "25"]);

        let words = regex_find_all("لغة لسان_1 رائعة!", "\\w+").unwrap();
        assert_eq!(words, vec!["لغة", "لسان_1", "رائعة"]);

        let emails = regex_find_all(
            "راسلنا على admin@lisan.dev أو support@arabic-code.org الآن",
            "[a-zA-Z0-9._+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]+",
        )
        .unwrap();
        assert_eq!(emails, vec!["admin@lisan.dev", "support@arabic-code.org"]);

        // فئة الأحرف المنفية والاستبدال
        let masked = regex_replace_all("رقم البطاقة: 4532-9910-0042", "\\d+", "XXXX").unwrap();
        assert_eq!(masked, "رقم البطاقة: XXXX-XXXX-XXXX");
    }

    #[test]
    fn test_stdlib_ext_lisan_parses_cleanly() {
        let tokens = crate::lexer::tokenize(STDLIB_EXT_LISAN).expect("STDLIB_EXT_LISAN must tokenize");
        let stmts = crate::parser::parse(tokens).expect("STDLIB_EXT_LISAN must parse");
        assert_eq!(stmts.len(), 3);
    }
}
