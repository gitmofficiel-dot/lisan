use crate::runtime::Value;
use std::iter::Peekable;
use std::str::Chars;

/// تحليل نص بتنسيق JSON إلى قيم لسان
pub fn parse_json(source: &str) -> Result<Value, String> {
    let mut parser = JsonParser::new(source);
    let val = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.chars.peek().is_some() {
        return Err("خطأ في تحليل JSON: رموز إضافية بعد نهاية بنية البيانات".into());
    }
    Ok(val)
}

/// تحويل قيمة لسان إلى نص بتنسيق JSON
pub fn to_json_string(value: &Value) -> Result<String, String> {
    let mut out = String::new();
    serialize_value(value, &mut out)?;
    Ok(out)
}

struct JsonParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> JsonParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        let &first = self.chars.peek().ok_or_else(|| "خطأ في تحليل JSON: نص فارغ أو غير مكتمل".to_string())?;

        match first {
            '{' => self.parse_object(),
            '[' => self.parse_array(),
            '"' => self.parse_string().map(Value::Text),
            't' => self.parse_literal("true", Value::Bool(true)),
            'f' => self.parse_literal("false", Value::Bool(false)),
            'n' => self.parse_literal("null", Value::Null),
            'ص' => self.parse_literal("صحيح", Value::Bool(true)),
            'خ' => self.parse_literal("خطأ", Value::Bool(false)),
            'ل' => self.parse_literal("لاشيء", Value::Null),
            '-' | '0'..='9' => self.parse_number(),
            other => Err(format!("خطأ في تحليل JSON: رمز غير متوقع '{}'", other)),
        }
    }

    fn parse_literal(&mut self, expected: &str, value: Value) -> Result<Value, String> {
        for ch in expected.chars() {
            match self.chars.next() {
                Some(c) if c == ch => {}
                _ => return Err(format!("خطأ في تحليل JSON: قيمة غير صالحة، متوقع '{}'", expected)),
            }
        }
        Ok(value)
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.chars.next() != Some('"') {
            return Err("خطأ في تحليل JSON: متوقع علامة اقتباس بداية النص".into());
        }
        let mut s = String::new();
        while let Some(c) = self.chars.next() {
            match c {
                '"' => return Ok(s),
                '\\' => {
                    let esc = self.chars.next().ok_or("خطأ في تحليل JSON: رمز هروب غير مكتمل")?;
                    match esc {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\x08'),
                        'f' => s.push('\x0c'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            let mut hex = String::with_capacity(4);
                            for _ in 0..4 {
                                hex.push(self.chars.next().ok_or("رمز Unicode في JSON غير مكتمل")?);
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| "رمز Unicode في JSON غير صحيح")?;
                            let ch = char::from_u32(code).ok_or("رمز Unicode في JSON غير صالح")?;
                            s.push(ch);
                        }
                        other => return Err(format!("رمز هروب غير معروف '\\{}' في JSON", other)),
                    }
                }
                normal => s.push(normal),
            }
        }
        Err("خطأ في تحليل JSON: نص غير مغلق".into())
    }

    fn parse_number(&mut self) -> Result<Value, String> {
        let mut num_str = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E' {
                num_str.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        let n: f64 = num_str.parse().map_err(|_| format!("رقم غير صالح في JSON: '{}'", num_str))?;
        if !n.is_finite() {
            return Err("الرقم في JSON خارج النطاق المسموح".into());
        }
        Ok(Value::Number(n))
    }

    fn parse_array(&mut self) -> Result<Value, String> {
        self.chars.next(); // skip '['
        let mut items = Vec::new();
        self.skip_whitespace();
        if let Some(&']') = self.chars.peek() {
            self.chars.next();
            return Ok(Value::List(items));
        }

        loop {
            let item = self.parse_value()?;
            items.push(item);
            self.skip_whitespace();
            match self.chars.next() {
                Some(',') => self.skip_whitespace(),
                Some(']') => break,
                Some(other) => return Err(format!("خطأ في مصفوفة JSON: متوقع ',' أو ']' ولكن وُجد '{}'", other)),
                None => return Err("خطأ في مصفوفة JSON: مصفوفة غير مغلقة بـ ']'".into()),
            }
        }
        Ok(Value::List(items))
    }

    fn parse_object(&mut self) -> Result<Value, String> {
        self.chars.next(); // skip '{'
        let mut pairs = Vec::new();
        self.skip_whitespace();
        if let Some(&'}') = self.chars.peek() {
            self.chars.next();
            return Ok(Value::Map(pairs));
        }

        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            match self.chars.next() {
                Some(':') => {}
                Some(other) => return Err(format!("خطأ في كائن JSON: متوقع ':' بعد المفتاح ولكن وُجد '{}'", other)),
                None => return Err("خطأ في كائن JSON: غير مكتمل".into()),
            }
            let val = self.parse_value()?;
            pairs.push((key, val));

            self.skip_whitespace();
            match self.chars.next() {
                Some(',') => self.skip_whitespace(),
                Some('}') => break,
                Some(other) => return Err(format!("خطأ في كائن JSON: متوقع ',' أو '}}' ولكن وُجد '{}'", other)),
                None => return Err("خطأ في كائن JSON: كائن غير مغلق بـ '}}'".into()),
            }
        }
        Ok(Value::Map(pairs))
    }
}

fn serialize_value(value: &Value, out: &mut String) -> Result<(), String> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => {
            if !n.is_finite() {
                return Err("لا يمكن تحويل قيمة عددية غير منتهية إلى JSON".into());
            }
            if n.fract() == 0.0 && n.abs() <= 9_007_199_254_740_991.0 {
                out.push_str(&format!("{}", *n as i64));
            } else {
                out.push_str(&format!("{}", n));
            }
        }
        Value::Text(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\x08' => out.push_str("\\b"),
                    '\x0c' => out.push_str("\\f"),
                    other => out.push(other),
                }
            }
            out.push('"');
        }
        Value::List(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                serialize_value(item, out)?;
            }
            out.push(']');
        }
        Value::Map(pairs) => {
            out.push('{');
            for (i, (key, val)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                for c in key.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        other => out.push(other),
                    }
                }
                out.push_str("\": ");
                serialize_value(val, out)?;
            }
            out.push('}');
        }
        Value::Instance { fields, .. } => {
            out.push('{');
            let borrowed = fields.borrow();
            for (i, (key, val)) in borrowed.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push('"');
                for c in key.chars() {
                    match c {
                        '"' => out.push_str("\\\""),
                        '\\' => out.push_str("\\\\"),
                        other => out.push(other),
                    }
                }
                out.push_str("\": ");
                serialize_value(val, out)?;
            }
            out.push('}');
        }
        Value::Lambda { .. } => {
            return Err("لا يمكن تحويل دالة مجهولة إلى JSON".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_roundtrip_primitives() {
        assert_eq!(parse_json("123.5").unwrap(), Value::Number(123.5));
        assert_eq!(parse_json("\"مرحبا بك\"").unwrap(), Value::Text("مرحبا بك".into()));
        assert_eq!(parse_json("true").unwrap(), Value::Bool(true));
        assert_eq!(parse_json("false").unwrap(), Value::Bool(false));
        assert_eq!(parse_json("null").unwrap(), Value::Null);
    }

    #[test]
    fn test_json_roundtrip_complex() {
        let json = r#"{"اسم": "لسان", "الإصدار": 0.9, "مزايا": ["سريع", "عربي", true], "نشط": true}"#;
        let parsed = parse_json(json).unwrap();
        let serialized = to_json_string(&parsed).unwrap();
        let reparsed = parse_json(&serialized).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
