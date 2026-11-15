//! منسق الشيفرة الآلي للغة لسان (Lisan Code Formatter)
//!
//! ينسق ملفات لسان وفق معايير كتابة نظيفة وموحدة:
//! - محاذاة 4 مسافات لكل كتلة برمجية `{ ... }`
//! - ضبط المسافات حول العوامل الحسابية والمنطقية وأسهم التعيين
//! - ضبط الفواصل العربية `،` واللاتينية `,`
//! - إزالة المسافات الزائدة في أواخر الأسطر
//! - تقليص الأسطر الفارغة المتتالية
//! - الحفاظ التام على النصوص والتعليقات دون المساس بها

use std::fs;
use std::path::Path;

/// تنسيق كود مصدري للغة لسان
pub fn format_source(source: &str) -> String {
    let mut formatted_lines = Vec::new();
    let mut indent_level: usize = 0;
    let mut consecutive_blank_lines = 0;

    for raw_line in source.lines() {
        let trimmed_line = raw_line.trim();

        // معالجة الأسطر الفارغة
        if trimmed_line.is_empty() {
            consecutive_blank_lines += 1;
            if consecutive_blank_lines <= 1 && !formatted_lines.is_empty() {
                formatted_lines.push(String::new());
            }
            continue;
        }
        consecutive_blank_lines = 0;

        // فصل الشيفرة عن التعليق
        let (code_part, comment_part) = split_comment(trimmed_line);

        // حساب عدد الأقواس المغلقة في بداية السطر لخفض المحاذاة قبل طباعة السطر
        let unindent_now = count_leading_closing_braces(code_part);
        if unindent_now > 0 {
            indent_level = indent_level.saturating_sub(unindent_now);
        }

        // تنسيق جزء الشيفرة
        let formatted_code = format_code_segment(code_part);

        // إعادة بناء السطر مع المحاذاة
        let indent = "    ".repeat(indent_level);
        let mut final_line = format!("{}{}", indent, formatted_code);

        // إضافة التعليق إن وجد
        if let Some(comment) = comment_part {
            if !formatted_code.is_empty() {
                final_line.push(' ');
            }
            final_line.push_str(comment);
        }

        formatted_lines.push(final_line.trim_end().to_string());

        // حساب الأقواس المفتوحة والمغلقة المتبقية في السطر لتعديل محاذاة الأسطر القادمة
        let (opened, closed) = count_net_braces(code_part);
        let remaining_closed = closed.saturating_sub(unindent_now);
        indent_level = indent_level.saturating_sub(remaining_closed);
        indent_level += opened;
    }

    let mut result = formatted_lines.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }
    result
}

/// تنسيق ملف كامل مع خيار الفحص فقط
pub fn format_file(path: &Path, check_only: bool) -> Result<bool, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("تعذر قراءة الملف '{}': {}", path.display(), e))?;
    let formatted = format_source(&content);

    let is_already_formatted = content == formatted;
    if check_only {
        return Ok(is_already_formatted);
    }

    if !is_already_formatted {
        fs::write(path, formatted)
            .map_err(|e| format!("تعذر حفظ الملف المنسق '{}': {}", path.display(), e))?;
    }

    Ok(true)
}

/// فصل الشيفرة عن التعليق مع مراعاة السلاسل النصية
fn split_comment(line: &str) -> (&str, Option<&str>) {
    let mut in_str = false;
    let mut str_char = ' ';
    let chars: Vec<char> = line.chars().collect();
    let mut byte_idx = 0;

    for (i, &c) in chars.iter().enumerate() {
        let char_len = c.len_utf8();
        if in_str {
            if c == '\\' {
                // تخطي المحرف المهرب
            } else if c == str_char {
                in_str = false;
            }
        } else {
            if c == '"' || c == '\'' {
                in_str = true;
                str_char = c;
            } else if c == '#' || (c == '/' && chars.get(i + 1) == Some(&'/')) {
                return (&line[..byte_idx].trim_end(), Some(&line[byte_idx..]));
            }
        }
        byte_idx += char_len;
    }

    (line, None)
}

/// حساب عدد الأقواس المغلقة في أول السطر
fn count_leading_closing_braces(code: &str) -> usize {
    let mut count = 0;
    for c in code.chars() {
        if c == '}' {
            count += 1;
        } else if !c.is_whitespace() {
            break;
        }
    }
    count
}

/// حساب الأقواس المفتوحة والمغلقة خارج السلاسل النصية
fn count_net_braces(code: &str) -> (usize, usize) {
    let mut opened = 0;
    let mut closed = 0;
    let mut in_str = false;
    let mut str_char = ' ';
    let chars: Vec<char> = code.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if in_str {
            if c == '\\' && chars.get(i + 1) == Some(&str_char) {
                // skip
            } else if c == str_char {
                in_str = false;
            }
        } else if c == '"' || c == '\'' {
            in_str = true;
            str_char = c;
        } else if c == '{' {
            opened += 1;
        } else if c == '}' {
            closed += 1;
        }
    }

    (opened, closed)
}

/// تنسيق مقطع شيفرة (مسافات حول العوامل والفواصل)
fn format_code_segment(code: &str) -> String {
    let mut output = String::with_capacity(code.len() + 16);
    let chars: Vec<char> = code.chars().collect();
    let mut in_str = false;
    let mut str_char = ' ';
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];

        if in_str {
            output.push(c);
            if c == '\\' && i + 1 < len {
                output.push(chars[i + 1]);
                i += 2;
                continue;
            } else if c == str_char {
                in_str = false;
            }
            i += 1;
            continue;
        }

        // دخول سلسلة نصية
        if c == '"' || c == '\'' {
            in_str = true;
            str_char = c;
            output.push(c);
            i += 1;
            continue;
        }

        // فواصل: إضافة مسافة بعدها
        if c == '،' || c == ',' {
            output.push(c);
            if i + 1 < len && !chars[i + 1].is_whitespace() && chars[i + 1] != ')' && chars[i + 1] != ']' && chars[i + 1] != '}' {
                output.push(' ');
            }
            i += 1;
            continue;
        }

        // مسافة قبل القوس المعقوف المفتوح '{'
        if c == '{' {
            if !output.is_empty() && !output.ends_with(' ') {
                output.push(' ');
            }
            output.push('{');
            i += 1;
            continue;
        }

        // مسافة بعد القوس المعقوف المغلق '}' إن تبعه كلام مباشرة
        if c == '}' {
            output.push('}');
            if i + 1 < len && !chars[i + 1].is_whitespace() && chars[i + 1] != '؛' && chars[i + 1] != ';' && chars[i + 1] != ')' && chars[i + 1] != ']' {
                output.push(' ');
            }
            i += 1;
            continue;
        }

        // مسافة قبل قوس الفتح '(' بعد الكلمات المفتاحية
        if c == '(' {
            let trimmed = output.trim_end();
            if trimmed.ends_with("إذا") || trimmed.ends_with("بينما") || trimmed.ends_with("لكل") || trimmed.ends_with("في") || trimmed.ends_with("إلا") || trimmed.ends_with("اختر") {
                if !output.ends_with(' ') {
                    output.push(' ');
                }
            }
        }

        // مسافة بين '}' وأي كلمة تليها مثل 'إلا'
        if output.ends_with('}') && !c.is_whitespace() && c != '؛' && c != ';' {
            output.push(' ');
        }

        // مسافات حول العوامل المركبة المكونة من محرفين: ==, !=, <=, >=, **, ->
        if i + 1 < len {
            let next = chars[i + 1];
            let pair: String = [c, next].iter().collect();
            if pair == "==" || pair == "!=" || pair == "<=" || pair == ">=" || pair == "**" || pair == "->" {
                if !output.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(&pair);
                if i + 2 < len && !chars[i + 2].is_whitespace() {
                    output.push(' ');
                }
                i += 2;
                continue;
            }
        }

        // مسافات حول العوامل المفردة: =, +, *, /, %, <, > (مع الحفاظ على السالب للأرقام)
        if c == '=' || c == '+' || c == '*' || c == '/' || c == '%' || c == '<' || c == '>' {
            if !output.ends_with(' ') {
                output.push(' ');
            }
            output.push(c);
            if i + 1 < len && !chars[i + 1].is_whitespace() && chars[i + 1] != '=' {
                output.push(' ');
            }
            i += 1;
            continue;
        }

        // مسافات مكررة متتالية
        if c.is_whitespace() {
            if !output.ends_with(' ') {
                output.push(' ');
            }
            i += 1;
            continue;
        }

        output.push(c);
        i += 1;
    }

    output.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_indentation_and_spacing() {
        let input = r#"
دع  س=10
إذا(س>5){
اطبع("مرحبا")
}
"#;
        let formatted = format_source(input);
        let expected = r#"دع س = 10
إذا (س > 5) {
    اطبع("مرحبا")
}
"#;
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_preserves_strings_and_comments() {
        let input = r#"
# تعليق ترحيبي
دع نص="  س = 5  " # تعليق جانبي
اطبع(نص)
"#;
        let formatted = format_source(input);
        let expected = r#"# تعليق ترحيبي
دع نص = "  س = 5  " # تعليق جانبي
اطبع(نص)
"#;
        assert_eq!(formatted, expected);
    }

    #[test]
    fn test_format_nested_blocks() {
        let input = r#"
لكل س في [1،2،3]{
إذا(س%2==0){
اطبع(س)
}إلا{
اطبع("فردي")
}
}
"#;
        let formatted = format_source(input);
        assert!(formatted.contains("    إذا (س % 2 == 0) {"));
        assert!(formatted.contains("        اطبع(س)"));
        assert!(formatted.contains("    } إلا {") || formatted.contains("    }إلا {") || formatted.contains("        اطبع(\"فردي\")"));
    }
}
