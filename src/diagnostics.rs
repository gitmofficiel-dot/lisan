/// وحدة التشخيص ومصحح الأخطاء والاقتراحات الذكية للغة لسان

/// تطبيع النصوص العربية لتسهيل المقارنة الذكية واكتشاف الأخطاء الإملائية الشائعة
pub fn normalize_arabic(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            // توحيد الألفات
            'أ' | 'إ' | 'آ' | 'ٱ' => result.push('ا'),
            // توحيد التاء المربوطة والهاء
            'ة' => result.push('ه'),
            // توحيد الألف المقصورة والياء
            'ى' => result.push('ي'),
            // إهمال التطويل (الكشيدة)
            'ـ' => continue,
            // إهمال حركات التشكيل والتنوين
            '\u{064B}'..='\u{065F}' | '\u{0670}' => continue,
            other => result.push(other),
        }
    }
    result
}

/// حساب مسافة ليفنشتاين (Levenshtein Distance) لقياس التقارب بين سلسلتين
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (m, n) = (a_chars.len(), b_chars.len());

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1)
                .min(prev[j] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// البحث عن أفضل اقتراح ذكي من قائمة المرشحين
pub fn find_suggestion<'a, I>(input: &str, candidates: I) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let norm_input = normalize_arabic(input);
    let mut best_candidate: Option<(&'a str, usize)> = None;

    for cand in candidates {
        if cand == input {
            continue;
        }
        let norm_cand = normalize_arabic(cand);
        // 1. تطابق تام بعد التطبيع (مثل النتيجة vs النتيجه)
        if norm_input == norm_cand {
            return Some(cand.to_string());
        }

        // 2. حساب مسافة ليفنشتاين
        let dist = levenshtein_distance(&norm_input, &norm_cand);
        let max_allowed = if norm_input.chars().count() <= 3 { 1 } else { 2 };

        if dist <= max_allowed {
            match best_candidate {
                Some((_, best_dist)) if dist < best_dist => {
                    best_candidate = Some((cand, dist));
                }
                None => {
                    best_candidate = Some((cand, dist));
                }
                _ => {}
            }
        }
    }

    best_candidate.map(|(name, _)| name.to_string())
}

/// قائمة أسماء كافة الدوال القياسية والكلمات المفتاحية للاقتراح
pub const ALL_BUILTIN_FUNCTIONS: &[&str] = &[
    "اطبع", "طول", "نوع", "نص", "عدد", "أضف", "مدى", "يحتوي",
    "مفاتيح", "قيم", "احذف", "قسّم", "ادمج", "استبدل", "اقتطع", "نظّف",
    "مطلق", "جذر", "قرّب", "أرضية", "سقف", "قوة", "أدنى", "أعلى",
    "اقرأ_ملف", "اكتب_ملف", "ألحق_ملف", "ملف_موجود", "مجلد_موجود", "أنشئ_مجلد",
    "قائمة_ملفات", "احذف_ملف", "مسار_حالي", "اطلب_ويب", "جلب_ويب",
    "وضع_المنفذ", "اكتب_رقمي", "اقرأ_رقمي", "اكتب_تماثلي", "اقرأ_تماثلي",
    "انتظر_ملي", "تأخير", "انتظر_ميكرو", "وقت_التشغيل", "نبضة_صوت", "أوقف_صوت",
    "تسلسلي_ابدأ", "تسلسلي_اطبع", "تسلسلي_متاح", "تسلسلي_اقرأ", "خريطة_قيمة", "قيد_قيمة", "ترجم_أردوينو", "إعادة_تشغيل", "اعادة_تشغيل",
    "أيقونة", "قائمة_الأيقونات", "نسق_تصميم", "عنصر_بطاقة", "عنصر_زر", "عنصر_إحصائية",
    "عنصر_مفتاح", "عنصر_تنبيه", "عنصر_شارة", "عنصر_حقل", "عنصر_شريط_تقدم",
    "بناء_تطبيق_هاتف", "تصدير_تطبيق_هاتف", "بناء_صفحة_ويب", "تصدير_صفحة_ويب", "تصدير_موقع",
    "تحقق", "تحقق_يساوي", "تحقق_صحيح", "تحقق_خطأ", "تحقق_خطا", "صف", "مجموعة",
    "أي", "اي", "كل", "مرقم", "سحاب",
    "جا", "جتا", "ظا", "لوغاريتم", "أسي", "ط", "هـ", "طابع_زمني", "خلط", "عناصر", "نظام_تشغيل", "افتح",
    "عكس", "فرز", "مجموع", "متوسط", "ابحث", "يبدأ_ب", "ينتهي_ب", "كرر",
    "أحرف_كبيرة", "أحرف_صغيرة", "ادخل", "الآن", "عدد_صحيح", "عشوائي",
    "عدد_عشوائي", "عنصر_عشوائي", "بيئة", "وسائط", "تحليل_json", "ترميز_json", "تنسيق",
    "تجزئة_sha256", "توقيع_hmac", "ترميز_ست_عشري", "فك_ست_عشري", "معرف_فريد",
    "شفر_نص", "فك_تشفير_نص", "يطابق_نمط", "استخرج_نمط", "استبدل_نمط",
    "معلومات_النظام", "نم", "انتظر_ثواني", "وقت_دقيق_ملي",
    "عرض_قالب", "تنظيف_html", "تنظيف_صفحة", "بناء_استجابة_http", "بناء_استجابة_شبكية",
    "تحليل_استعلام_url", "تحليل_استعلام_رابط", "طلب_http_متقدم", "طلب_شبكي_متقدم", "طلب_شبكي_آمن", "تشغيل_تطبيق_ويب",
    "مسار_ادمج", "اسم_ملف", "امتداد_ملف", "مجلد_أب", "انسخ_ملف", "انقل_ملف", "حجم_ملف", "احذف_مجلد",
    "نفّذ_أمر", "بصمة_crc32", "بصمة_تحقق", "ضغط_rle", "ضغط_تكراري", "فك_ضغط_rle", "فك_ضغط_تكراري",
    "ترميز_جيسون", "تحليل_جيسون", "ترميز_رابط", "فك_رابط", "تجزئة_آمنة", "توقيع_رسالة",
    "استورد_بايثون", "استورد_جافاسكربت", "بايثون_متوفر", "بايثون_استدعِ", "بايثون_سمة", "بايثون_نفّذ",
    "جافاسكربت_متوفر", "جافاسكربت_استدعِ", "جافاسكربت_نفّذ",
    "انتظر_قيمة", "تشغيل_متوازٍ", "تشغيل_متوازي", "طلب_شبكي_متوازي",
    "مكتبة_نظام_متوفرة", "استدعِ_دالة_نظام", "استدع_دالة_نظام",
    "مفتاح_مقبس_حي", "مصافحة_مقبس_حي", "تصدير_كاباسيتور", "تصدير_تاوري", "تصدير_فيجما",
];

/// جدول التحويل التلقائي من الكلمات المفتاحية الإنجليزية إلى العربية الخالصة في لسان
pub fn english_keyword_to_arabic(word: &str) -> Option<&'static str> {
    match word {
        "if" => Some("إذا"),
        "else" | "elif" => Some("وإلا"),
        "while" => Some("طالما"),
        "for" => Some("لكل"),
        "in" => Some("في"),
        "def" | "function" | "fn" | "func" => Some("دالة"),
        "return" => Some("أرجع"),
        "import" | "require" => Some("استورد"),
        "from" => Some("من"),
        "as" => Some("كـ"),
        "class" | "struct" => Some("صنف"),
        "extends" => Some("يرث"),
        "new" => Some("جديد"),
        "this" | "self" => Some("هذا"),
        "true" | "True" => Some("صحيح"),
        "false" | "False" => Some("خطأ"),
        "null" | "None" | "nil" | "undefined" => Some("لاشيء"),
        "and" => Some("و"),
        "or" => Some("أو"),
        "not" => Some("ليس"),
        "try" => Some("حاول"),
        "except" | "catch" => Some("التقط"),
        "finally" => Some("أخيرًا"),
        "raise" | "throw" => Some("ارمِ"),
        "break" => Some("توقف"),
        "continue" => Some("تابع"),
        "match" | "switch" => Some("طابق"),
        "case" => Some("حالة"),
        "default" => Some("افتراضي"),
        "let" | "var" | "const" => Some("دع"),
        "print" | "console" => Some("اطبع"),
        "with" => Some("مع"),
        "async" => Some("دالة_غير_متزامنة"),
        "await" => Some("انتظر"),
        "pass" => Some("// كتلة فارغة"),
        _ => None,
    }
}

/// جدول الأسماء العربية الخالصة مقابل الأسماء القديمة التي كانت تحتوي مقاطع لاتينية
/// الصيغة: (الاسم_القديم، الاسم_العربي_الخالص)
pub const PURE_ARABIC_ALIASES: &[(&str, &str)] = &[
    // الذكاء الاصطناعي والرسوم المتجهة
    ("مصنف_جيران_knn", "مصنف_الجيران_الأقرب"),
    ("لوحة_رسم_svg", "لوحة_رسم_متجه"),
    // الويب والشبكة والأنظمة
    ("طلب_http_متقدم", "طلب_شبكي_متقدم"),
    ("عميل_api", "عميل_واجهة_برمجية"),
    ("بناء_استجابة_http", "بناء_استجابة_شبكية"),
    ("تنظيف_html", "تنظيف_صفحة"),
    ("تحليل_استعلام_url", "تحليل_استعلام_رابط"),
    ("ترميز_json", "ترميز_جيسون"),
    ("تحليل_json", "تحليل_جيسون"),
    ("ترميز_url", "ترميز_رابط"),
    ("فك_url", "فك_رابط"),
    ("ترميز_csv", "ترميز_جدول_نصي"),
    ("تحليل_csv", "تحليل_جدول_نصي"),
    ("تجزئة_sha256", "تجزئة_آمنة"),
    ("توقيع_hmac", "توقيع_رسالة"),
    ("بصمة_crc32", "بصمة_تحقق"),
    ("ضغط_rle", "ضغط_تكراري"),
    ("فك_ضغط_rle", "فك_ضغط_تكراري"),
    // الروبوتات والبروتوكولات الصناعية
    ("متحكم_pid", "متحكم_تناسبي_تكاملي"),
    ("لورا_lora", "لورا"),
    ("شريحة_اتصال_gsm", "شريحة_اتصال_خلوي"),
    ("سائق_خطوي_a4988", "سائق_محرك_خطوي"),
    ("مراقب_بطارية_bms", "مراقب_البطارية"),
    // المتحكمات الدقيقة والحساسات
    ("شاشة_oled", "شاشة_عضوية"),
    ("قارئ_rfid", "قارئ_بطاقات_لاسلكي"),
    ("محدد_مواقع_gps", "محدد_مواقع"),
    ("ساعة_الوقت_الحقيقي_rtc", "ساعة_الوقت_الحقيقي"),
    ("حساس_حركة_pir", "حساس_حركة"),
    ("حساس_غاز_mq", "حساس_غاز"),
    ("مستقبل_أشعة_ir", "مستقبل_أشعة_تحت_حمراء"),
    ("خلية_وزن_hx711", "خلية_وزن"),
    ("حساس_نبض_max30102", "حساس_نبض"),
    ("قبعة_محركات_pca9685", "قبعة_محركات"),
    ("شاشة_حبر_إلكتروني_epaper", "شاشة_حبر_إلكتروني"),
    ("مصفوفة_ليد_8x8", "مصفوفة_ليد"),
    ("لوحة_مفاتيح_4x4", "لوحة_مفاتيح_مصفوفية"),
    ("عميل_mqtt", "عميل_رسائل_الأشياء"),
    ("بلوتوث_ble", "بلوتوث_منخفض_الطاقة"),
    ("ذاكرة_eeprom", "ذاكرة_دائمة"),
    ("بطاقة_ذاكرة_sd", "بطاقة_ذاكرة"),
    ("اتصال_مباشر_espnow", "اتصال_مباشر_إسب"),
    ("واي_فاي_esp", "واي_فاي_إسب"),
    ("خادم_ويب_esp", "خادم_ويب_إسب"),
    ("مقبس_ويب_esp", "مقبس_ويب_إسب"),
    ("كاميرا_esp32", "كاميرا_إسب"),
    ("بلوتوث_تسلسلي_esp", "بلوتوث_تسلسلي_إسب"),
    ("نوم_عميق_esp", "نوم_عميق_إسب"),
    ("حساس_لمس_esp", "حساس_لمس_إسب"),
    ("عميل_http_esp", "عميل_شبكي_إسب"),
    ("محول_رقمي_esp", "محول_رقمي_إسب"),
    ("محول_تماثلي_esp", "محول_تماثلي_إسب"),
    ("تحديث_هوائي_ota", "تحديث_هوائي"),
    ("ذاكرة_تفضيلات_nvs", "ذاكرة_تفضيلات"),
    ("ذاكرة_فلاش_spiffs", "ذاكرة_فلاش_ملفات"),
    ("توقيت_شبكي_ntp", "توقيت_شبكي"),
    ("مؤقت_عتادي_stm32", "مؤقت_عتادي_ستم"),
    ("ذاكرة_فلاش_stm32", "ذاكرة_فلاش_ستم"),
    ("مراقب_النظام_iwdg", "مراقب_النظام"),
    ("ناقل_can_stm32", "ناقل_كان_ستم"),
    ("مشفر_عتادي_stm32", "مشفر_عتادي_ستم"),
    ("وضع_توفير_طاقة_stm32", "وضع_توفير_طاقة_ستم"),
    ("ناقل_مباشر_dma", "ناقل_ذاكرة_مباشر"),
    ("منفذ_usb_cdc", "منفذ_تسلسلي_افتراضي"),
    ("مهمة_متوازية_rtos", "مهمة_متوازية"),
    ("ناقل_spi_stm32", "ناقل_طرفي_ستم"),
    ("ناقل_i2c_stm32", "ناقل_ثنائي_ستم"),
    ("محول_رقمي_stm32", "محول_رقمي_ستم"),
    ("محول_تماثلي_stm32", "محول_تماثلي_ستم"),
    ("منفذ_تسلسلي_stm32", "منفذ_تسلسلي_ستم"),
    ("منفذ_gpio_راسبيري", "منفذ_عام_راسبيري"),
    ("ناقل_spi_راسبيري", "ناقل_طرفي_راسبيري"),
    ("ناقل_i2c_راسبيري", "ناقل_ثنائي_راسبيري"),
    ("نبضة_pwm_راسبيري", "نبضة_عرض_راسبيري"),
    ("قبعة_الحساسات_sensehat", "قبعة_الحساسات"),
    ("شاشة_لمس_dsi", "شاشة_لمس_راسبيري"),
    ("وحدة_pio_بيكو", "وحدة_إدخال_مبرمجة_بيكو"),
    ("واي_فاي_بيكو_w", "واي_فاي_بيكو"),
    ("ناقل_i2c_بيكو", "ناقل_ثنائي_بيكو"),
    ("ناقل_spi_بيكو", "ناقل_طرفي_بيكو"),
    ("منفذ_uart_بيكو", "منفذ_تسلسلي_بيكو"),
    ("ناقل_i2c", "ناقل_ثنائي"),
    ("ناقل_spi", "ناقل_طرفي"),
    ("محول_adc", "محول_تماثلي_رقمي"),
    ("منفذ_dac", "منفذ_تماثلي"),
    ("مروحة_pwm", "مروحة_متغيرة"),
    ("وحدة_plc", "وحدة_تحكم_منطقي"),
];

/// تحويل اسم عربي خالص إلى الاسم الداخلي المعتمَد عند التشغيل
pub fn pure_arabic_to_canonical(name: &str) -> Option<&'static str> {
    for &(legacy, pure_ar) in PURE_ARABIC_ALIASES {
        if pure_ar == name {
            return Some(legacy);
        }
    }
    None
}

/// إرجاع البديل العربي الخالص إذا استُخدم اسم قديم يحتوي مقاطع لاتينية
pub fn legacy_to_pure_arabic(name: &str) -> Option<&'static str> {
    for &(legacy, pure_ar) in PURE_ARABIC_ALIASES {
        if legacy == name {
            return Some(pure_ar);
        }
    }
    None
}

/// تشخيص مُهيكل صديق للبشر ووكلاء الذكاء الاصطناعي
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredDiagnostic {
    pub code: String,
    pub severity: String, // "error" | "warning"
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub suggestion: Option<String>,
    pub replacement: Option<String>,
}

fn escape_json_diag(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

impl StructuredDiagnostic {
    pub fn to_json(&self) -> String {
        let sug = match &self.suggestion {
            Some(s) => format!("\"{}\"", escape_json_diag(s)),
            None => "null".to_string(),
        };
        let rep = match &self.replacement {
            Some(r) => format!("\"{}\"", escape_json_diag(r)),
            None => "null".to_string(),
        };
        format!(
            "{{\"code\":\"{}\",\"severity\":\"{}\",\"line\":{},\"column\":{},\"message\":\"{}\",\"suggestion\":{},\"replacement\":{}}}",
            escape_json_diag(&self.code),
            escape_json_diag(&self.severity),
            self.line,
            self.column,
            escape_json_diag(&self.message),
            sug,
            rep
        )
    }
}

pub fn diagnostics_to_json(diags: &[StructuredDiagnostic]) -> String {
    let items: Vec<String> = diags.iter().map(|d| d.to_json()).collect();
    format!("[{}]", items.join(","))
}

/// فحص قاعدة «العربية فقط»: يكتشف الكلمات المفتاحية الإنجليزية والأسماء المهجورة ذات المقاطع اللاتينية
pub fn lint_arabic_only(source: &str) -> Vec<StructuredDiagnostic> {
    let mut out = Vec::new();
    let tokens = match crate::lexer::tokenize(source) {
        Ok(t) => t,
        Err(_) => return out,
    };

    for (idx, tok) in tokens.iter().enumerate() {
        if let crate::lexer::TokenKind::Identifier(ref word) = tok.kind {
            // تجاهل ما بعد النقطة (مثل استدعاء دوال كائن خارجي)
            let after_dot = idx > 0 && matches!(tokens[idx - 1].kind, crate::lexer::TokenKind::Dot);
            if !after_dot {
                if let Some(arabic_kw) = english_keyword_to_arabic(word) {
                    out.push(StructuredDiagnostic {
                        code: "ل0001".to_string(),
                        severity: "error".to_string(),
                        line: tok.line,
                        column: tok.column,
                        message: format!(
                            "لسان لغة عربية خالصة: الكلمة المفتاحية الإنجليزية '{}' غير مسموحة. استخدم '{}' بدلاً منها.",
                            word, arabic_kw
                        ),
                        suggestion: Some(format!("استبدل '{}' بـ '{}'", word, arabic_kw)),
                        replacement: Some(arabic_kw.to_string()),
                    });
                } else if let Some(pure_ar) = legacy_to_pure_arabic(word) {
                    out.push(StructuredDiagnostic {
                        code: "ل0004".to_string(),
                        severity: "warning".to_string(),
                        line: tok.line,
                        column: tok.column,
                        message: format!(
                            "الاسم '{}' يحتوي مقطعاً لاتينياً (مهجور مؤقتاً). يُفضّل الاسم العربي الخالص '{}'.",
                            word, pure_ar
                        ),
                        suggestion: Some(format!("استخدم '{}' بدلاً من '{}'", pure_ar, word)),
                        replacement: Some(pure_ar.to_string()),
                    });
                }
            }
        }
    }
    out
}

/// فحص تدريجي للأنواع الاختيارية (Gradual Typing): يتحقق من تطابق القيم الابتدائية وإرجاع الدوال
pub fn check_gradual_types(source: &str) -> Vec<StructuredDiagnostic> {
    use crate::lexer::TokenKind;
    let mut out = Vec::new();
    let tokens = match crate::lexer::tokenize(source) {
        Ok(t) => t,
        Err(_) => return out,
    };

    fn normalize_type_name(t: &str) -> &str {
        match t {
            "رقم" | "عدد" | "عدد_صحيح" | "عشري" => "عدد",
            "نص" | "سلسلة" => "نص",
            "منطقي" | "منطق" => "منطقي",
            "قائمة" | "مصفوفة" => "قائمة",
            "قاموس" | "خريطة" => "قاموس",
            "فارغ" | "لاشيء" => "فارغ",
            "أي" | "اي" => "أي",
            other => other,
        }
    }

    fn infer_simple_literal(tokens: &[crate::lexer::Token], pos: usize) -> Option<&'static str> {
        let tok = tokens.get(pos)?;
        let next = tokens.get(pos + 1).map(|t| &t.kind);
        let is_compound = matches!(
            next,
            Some(
                TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Star
                    | TokenKind::Slash
                    | TokenKind::Percent
                    | TokenKind::Power
                    | TokenKind::EqualEqual
                    | TokenKind::BangEqual
                    | TokenKind::Less
                    | TokenKind::LessEqual
                    | TokenKind::Greater
                    | TokenKind::GreaterEqual
                    | TokenKind::AndAnd
                    | TokenKind::OrOr
                    | TokenKind::LeftBracket
                    | TokenKind::Dot
            )
        );
        if is_compound {
            return None;
        }
        match &tok.kind {
            TokenKind::Number(_) => Some("عدد"),
            TokenKind::String(_) | TokenKind::FString(_) => Some("نص"),
            TokenKind::True | TokenKind::False => Some("منطقي"),
            TokenKind::Null => Some("فارغ"),
            TokenKind::LeftBracket => Some("قائمة"),
            TokenKind::LeftBrace => Some("قاموس"),
            _ => None,
        }
    }

    let mut i = 0;
    while i < tokens.len() {
        // 1) دع اسم: نوع = قيمة
        if matches!(tokens[i].kind, TokenKind::Let) {
            if let (
                Some(crate::lexer::Token { kind: TokenKind::Identifier(var_name), .. }),
                Some(crate::lexer::Token { kind: TokenKind::Colon, .. }),
                Some(type_tok),
            ) = (tokens.get(i + 1), tokens.get(i + 2), tokens.get(i + 3))
            {
                let declared_raw = match &type_tok.kind {
                    TokenKind::Identifier(t) => Some(t.as_str()),
                    TokenKind::Null => Some("فارغ"),
                    _ => None,
                };
                if let Some(declared_raw) = declared_raw {
                    let declared = normalize_type_name(declared_raw);
                    // ابحث عن '=' بعد تحديد النوع (مع تجاوز [...] إن وُجدت)
                    let mut eq_idx = i + 4;
                    if matches!(tokens.get(eq_idx).map(|t| &t.kind), Some(TokenKind::LeftBracket)) {
                        let mut depth = 0usize;
                        while eq_idx < tokens.len() {
                            match tokens[eq_idx].kind {
                                TokenKind::LeftBracket => depth += 1,
                                TokenKind::RightBracket => {
                                    depth = depth.saturating_sub(1);
                                    if depth == 0 { eq_idx += 1; break; }
                                }
                                _ => {}
                            }
                            eq_idx += 1;
                        }
                    }
                    if matches!(tokens.get(eq_idx).map(|t| &t.kind), Some(TokenKind::Equal)) {
                        if let Some(actual) = infer_simple_literal(&tokens, eq_idx + 1) {
                            if declared != "أي" && declared != actual && matches!(declared, "عدد" | "نص" | "منطقي" | "قائمة" | "قاموس" | "فارغ") {
                                out.push(StructuredDiagnostic {
                                    code: "ل0003".to_string(),
                                    severity: "error".to_string(),
                                    line: type_tok.line,
                                    column: type_tok.column,
                                    message: format!(
                                        "عدم تطابق النوع للمتغير '{}': النوع المصرّح به '{}' لكن القيمة من نوع '{}'.",
                                        var_name, declared_raw, actual
                                    ),
                                    suggestion: Some(format!("غيّر القيمة لتكون من نوع '{}' أو عدّل النوع إلى '{}'", declared_raw, actual)),
                                    replacement: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2) دالة اسم(...) -> نوع_إرجاع { ... أرجع قيمة ... }
        if matches!(tokens[i].kind, TokenKind::Function) {
            if let Some(crate::lexer::Token { kind: TokenKind::Identifier(fn_name), .. }) = tokens.get(i + 1) {
                let mut j = i + 2;
                let mut paren_depth = 0usize;
                while j < tokens.len() {
                    match tokens[j].kind {
                        TokenKind::LeftParen => paren_depth += 1,
                        TokenKind::RightParen => {
                            paren_depth = paren_depth.saturating_sub(1);
                            if paren_depth == 0 { j += 1; break; }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if matches!(tokens.get(j).map(|t| &t.kind), Some(TokenKind::Arrow)) {
                    if let Some(ret_tok) = tokens.get(j + 1) {
                        let ret_raw = match &ret_tok.kind {
                            TokenKind::Identifier(t) => Some(t.as_str()),
                            TokenKind::Null => Some("فارغ"),
                            _ => None,
                        };
                        if let Some(ret_raw) = ret_raw {
                            let expected_ret = normalize_type_name(ret_raw);
                            // انتقل إلى بداية كتلة الدالة '{'
                            let mut k = j + 2;
                            while k < tokens.len() && !matches!(tokens[k].kind, TokenKind::LeftBrace) {
                                k += 1;
                            }
                            if k < tokens.len() {
                                let mut brace_depth = 0usize;
                                let mut nested_fn_depth = 0usize;
                                while k < tokens.len() {
                                    match tokens[k].kind {
                                        TokenKind::Function => nested_fn_depth += 1,
                                        TokenKind::LeftBrace => brace_depth += 1,
                                        TokenKind::RightBrace => {
                                            brace_depth = brace_depth.saturating_sub(1);
                                            if nested_fn_depth > 0 && brace_depth == 1 {
                                                nested_fn_depth -= 1;
                                            }
                                            if brace_depth == 0 { break; }
                                        }
                                        TokenKind::Return if brace_depth >= 1 && nested_fn_depth == 0 => {
                                            if let Some(actual) = infer_simple_literal(&tokens, k + 1) {
                                                if expected_ret != "أي"
                                                    && expected_ret != actual
                                                    && matches!(expected_ret, "عدد" | "نص" | "منطقي" | "قائمة" | "قاموس" | "فارغ")
                                                {
                                                    out.push(StructuredDiagnostic {
                                                        code: "ل0003".to_string(),
                                                        severity: "error".to_string(),
                                                        line: tokens[k].line,
                                                        column: tokens[k].column,
                                                        message: format!(
                                                            "عدم تطابق نوع الإرجاع في الدالة '{}': المتوقع '{}' لكن جملة 'أرجع' تعيد '{}'.",
                                                            fn_name, ret_raw, actual
                                                        ),
                                                        suggestion: Some(format!("أرجع قيمة من نوع '{}' أو عدّل نوع الإرجاع إلى '{}'", ret_raw, actual)),
                                                        replacement: None,
                                                    });
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                    k += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }

    out
}

/// استخراج رقم السطر والعمود من رسالة خطأ عربية (بلا اعتماد على وحدة lsp)
pub fn parse_error_location(err_msg: &str) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    if let Some(s_idx) = err_msg.find("السطر ") {
        let digits: String = err_msg[s_idx + "السطر ".len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(l) = digits.parse::<usize>() {
            if l > 0 { line = l; }
        }
    }
    if let Some(c_idx) = err_msg.find("العمود ") {
        let digits: String = err_msg[c_idx + "العمود ".len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(c) = digits.parse::<usize>() {
            if c > 0 { col = c; }
        }
    }
    (line, col)
}

/// فحص شامل للشيفرة: القواعد النحوية + قاعدة العربية فقط + الأنواع الاختيارية
pub fn diagnose_source(source: &str) -> Vec<StructuredDiagnostic> {
    let mut diags = Vec::new();
    let arabic_lints = lint_arabic_only(source);
    let has_english_kw_error = arabic_lints.iter().any(|d| d.code == "ل0001");
    diags.extend(arabic_lints);

    match crate::lexer::tokenize(source) {
        Ok(tokens) => {
            if let Err(err) = crate::parser::parse(tokens) {
                if !has_english_kw_error {
                    let (line, col) = parse_error_location(&err);
                    diags.push(StructuredDiagnostic {
                        code: "ل0002".to_string(),
                        severity: "error".to_string(),
                        line,
                        column: col,
                        message: err,
                        suggestion: None,
                        replacement: None,
                    });
                }
            } else {
                diags.extend(check_gradual_types(source));
            }
        }
        Err(err) => {
            let (line, col) = parse_error_location(&err);
            diags.push(StructuredDiagnostic {
                code: "ل0002".to_string(),
                severity: "error".to_string(),
                line,
                column: col,
                message: err,
                suggestion: None,
                replacement: None,
            });
        }
    }

    diags
}

pub fn suggest_variable_error(name: &str, available: &[&str]) -> String {
    if let Some(arabic_kw) = english_keyword_to_arabic(name) {
        return format!(
            "[ل0001] لسان لغة عربية خالصة: '{}' كلمة مفتاحية إنجليزية غير مسموحة. استخدم '{}' بدلاً منها.",
            name, arabic_kw
        );
    }
    if let Some(suggestion) = find_suggestion(name, available.iter().copied()) {
        format!("متغير غير معرّف: '{}' (هل تقصد '{}'؟)", name, suggestion)
    } else {
        format!("متغير غير معرّف: '{}'", name)
    }
}

pub fn suggest_function_error(name: &str, available: &[&str]) -> String {
    if let Some(arabic_kw) = english_keyword_to_arabic(name) {
        return format!(
            "[ل0001] لسان لغة عربية خالصة: '{}' كلمة مفتاحية إنجليزية غير مسموحة. استخدم '{}' بدلاً منها.",
            name, arabic_kw
        );
    }
    let all_candidates = ALL_BUILTIN_FUNCTIONS.iter().copied().chain(available.iter().copied());
    if let Some(suggestion) = find_suggestion(name, all_candidates) {
        format!("دالة غير معرّفة: '{}' (هل تقصد '{}'؟)", name, suggestion)
    } else {
        format!("دالة غير معرّفة: '{}'", name)
    }
}

pub fn suggest_dict_key_error(key: &str, available_keys: &[&str]) -> String {
    if let Some(suggestion) = find_suggestion(key, available_keys.iter().copied()) {
        format!("المفتاح '{}' غير موجود في القاموس (هل تقصد '{}'؟)", key, suggestion)
    } else {
        format!("المفتاح '{}' غير موجود في القاموس", key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_arabic() {
        assert_eq!(normalize_arabic("أحمد"), "احمد");
        assert_eq!(normalize_arabic("النتيجةُ"), "النتيجه");
        assert_eq!(normalize_arabic("مُسْتَخْدَم"), "مستخدم");
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("اطبع", "اطببع"), 1);
        assert_eq!(levenshtein_distance("مجموع", "مجمموع"), 1);
        assert_eq!(levenshtein_distance("قائمة", "قائمة"), 0);
    }

    #[test]
    fn test_suggestions() {
        assert_eq!(
            find_suggestion("اطببع", ALL_BUILTIN_FUNCTIONS.iter().copied()),
            Some("اطبع".to_string())
        );
        assert_eq!(
            find_suggestion("النتيجه", ["النتيجة", "العمر", "المعدل"].iter().copied()),
            Some("النتيجة".to_string())
        );
    }
}
