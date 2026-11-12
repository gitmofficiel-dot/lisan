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
    "اقرأ_ملف", "اكتب_ملف", "ألحق_ملف", "ملف_موجود", "تحقق", "تحقق_يساوي",
    "عكس", "فرز", "مجموع", "متوسط", "ابحث", "يبدأ_ب", "ينتهي_ب", "كرر",
    "أحرف_كبيرة", "أحرف_صغيرة", "ادخل", "الآن", "عدد_صحيح", "عشوائي",
    "عدد_عشوائي", "عنصر_عشوائي", "بيئة", "وسائط", "تحليل_json", "ترميز_json", "تنسيق",
];

pub fn suggest_variable_error(name: &str, available: &[&str]) -> String {
    if let Some(suggestion) = find_suggestion(name, available.iter().copied()) {
        format!("متغير غير معرّف: '{}' (هل تقصد '{}'؟)", name, suggestion)
    } else {
        format!("متغير غير معرّف: '{}'", name)
    }
}

pub fn suggest_function_error(name: &str, available: &[&str]) -> String {
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
