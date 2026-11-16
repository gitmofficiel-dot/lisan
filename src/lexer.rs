#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Let, Print, If, Else, While, ForEach, In, Function, Return, Import, Break, Continue, True, False, Null,
    Try, Catch, Finally, Throw, Match, Case, Default,
    Class, New, Extends,
    With, As, From,
    Identifier(String), Number(f64), String(String), FString(String),
    LeftParen, RightParen, LeftBrace, RightBrace, LeftBracket, RightBracket, Comma, Colon, Semicolon, Dot, FatArrow,
    Plus, Minus, Star, Power, Slash, Percent,
    Equal, EqualEqual, Bang, BangEqual,
    Less, LessEqual, Greater, GreaterEqual, AndAnd, OrOr, Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;
    let mut line = 1;
    let mut column = 1;
    let mut result = Vec::new();

    while i < chars.len() {
        let c = chars[i];
        if c == '\n' {
            i += 1;
            line += 1;
            column = 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            column += 1;
            continue;
        }
        if c == '#' || (c == '/' && chars.get(i + 1) == Some(&'/')) {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                column += 1;
            }
            continue;
        }

        let start_line = line;
        let start_col = column;
        let start = i;
        let kind = match c {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ',' | '،' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' | '؛' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' if chars.get(i + 1) == Some(&'*') => { i += 1; column += 1; TokenKind::Power }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' if chars.get(i + 1) == Some(&'=') => { i += 1; column += 1; TokenKind::EqualEqual }
            '=' if chars.get(i + 1) == Some(&'>') => { i += 1; column += 1; TokenKind::FatArrow }
            '=' => TokenKind::Equal,
            '!' if chars.get(i + 1) == Some(&'=') => { i += 1; column += 1; TokenKind::BangEqual }
            '!' => TokenKind::Bang,
            '<' if chars.get(i + 1) == Some(&'=') => { i += 1; column += 1; TokenKind::LessEqual }
            '<' => TokenKind::Less,
            '>' if chars.get(i + 1) == Some(&'=') => { i += 1; column += 1; TokenKind::GreaterEqual }
            '>' => TokenKind::Greater,
            '&' if chars.get(i + 1) == Some(&'&') => { i += 1; column += 1; TokenKind::AndAnd }
            '|' if chars.get(i + 1) == Some(&'|') => { i += 1; column += 1; TokenKind::OrOr }
            'ن' | 'f' | 'F' if chars.get(i + 1) == Some(&'"') => {
                i += 1; // skip prefix 'ن' / 'f'
                column += 1;
                i += 1; // skip opening quote '"'
                column += 1;
                let mut s = String::new();
                let mut closed = false;
                let mut brace_depth: usize = 0;
                while i < chars.len() {
                    let x = chars[i];
                    if brace_depth == 0 && x == '"' {
                        closed = true;
                        break;
                    }
                    if x == '\n' {
                        return Err(format!("السطر {}، العمود {}: نص غير مغلق", start_line, start_col));
                    }
                    if brace_depth == 0 {
                        if x == '{' && chars.get(i + 1) == Some(&'{') {
                            s.push('{');
                            s.push('{');
                            i += 2;
                            column += 2;
                            continue;
                        }
                        if x == '}' && chars.get(i + 1) == Some(&'}') {
                            s.push('}');
                            s.push('}');
                            i += 2;
                            column += 2;
                            continue;
                        }
                        if x == '{' {
                            brace_depth += 1;
                            s.push(x);
                            i += 1;
                            column += 1;
                            continue;
                        }
                        if x == '\\' {
                            i += 1;
                            column += 1;
                            let escaped = chars.get(i).ok_or_else(|| format!("السطر {}: تسلسل هروب غير مكتمل", start_line))?;
                            s.push(match escaped {
                                'n' => '\n', 't' => '\t', 'r' => '\r', '"' => '"', '\\' => '\\',
                                _ => return Err(format!("السطر {}: تسلسل هروب غير معروف", start_line)),
                            });
                            i += 1;
                            column += 1;
                            continue;
                        }
                        s.push(x);
                        i += 1;
                        column += 1;
                    } else {
                        if x == '"' {
                            s.push(x);
                            i += 1;
                            column += 1;
                            let mut str_closed = false;
                            while i < chars.len() {
                                let sx = chars[i];
                                s.push(sx);
                                if sx == '"' { str_closed = true; break; }
                                if sx == '\\' && i + 1 < chars.len() {
                                    i += 1;
                                    column += 1;
                                    s.push(chars[i]);
                                }
                                i += 1;
                                column += 1;
                            }
                            if !str_closed {
                                return Err(format!("السطر {}، العمود {}: نص غير مغلق داخل تعبير f-string", start_line, start_col));
                            }
                            i += 1;
                            column += 1;
                            continue;
                        }
                        if x == '{' {
                            brace_depth += 1;
                        } else if x == '}' {
                            brace_depth -= 1;
                        }
                        s.push(x);
                        i += 1;
                        column += 1;
                    }
                }
                if !closed {
                    return Err(format!("السطر {}، العمود {}: نص غير مغلق", start_line, start_col));
                }
                TokenKind::FString(s)
            }
            '"' => {
                i += 1;
                column += 1;
                let mut s = String::new();
                let mut closed = false;
                while i < chars.len() {
                    let x = chars[i];
                    if x == '"' { closed = true; break; }
                    if x == '\n' {
                        return Err(format!("السطر {}، العمود {}: نص غير مغلق", start_line, start_col));
                    }
                    if x == '\\' {
                        i += 1;
                        column += 1;
                        let escaped = chars.get(i).ok_or_else(|| format!("السطر {}: تسلسل هروب غير مكتمل", start_line))?;
                        s.push(match escaped {
                            'n' => '\n', 't' => '\t', 'r' => '\r', '"' => '"', '\\' => '\\',
                            _ => return Err(format!("السطر {}: تسلسل هروب غير معروف", start_line)),
                        });
                    } else { s.push(x); }
                    i += 1;
                    column += 1;
                }
                if !closed {
                    return Err(format!("السطر {}، العمود {}: نص غير مغلق", start_line, start_col));
                }
                TokenKind::String(s)
            }
         _ if is_lisan_digit(c) => {
            let mut num = String::new();
            while i < chars.len() {
                if let Some(d) = normalize_digit(chars[i]) {
                    num.push(d);
                    i += 1;
                    column += 1;
                } else {
                    break;
                }
            }
            if chars.get(i).copied().map(is_decimal_sep).unwrap_or(false)
                && chars.get(i + 1).copied().map(is_lisan_digit).unwrap_or(false)
            {
                num.push('.');
                i += 1;
                column += 1;
                while i < chars.len() {
                    if let Some(d) = normalize_digit(chars[i]) {
                        num.push(d);
                        i += 1;
                        column += 1;
                    } else {
                        break;
                    }
                }
            }
            if matches!(chars.get(i).copied(), Some('e' | 'E')) {
                num.push(chars[i]);
                i += 1;
                column += 1;
                if matches!(chars.get(i).copied(), Some('+' | '-')) {
                    num.push(chars[i]);
                    i += 1;
                    column += 1;
                }
                if !chars.get(i).copied().map(is_lisan_digit).unwrap_or(false) {
                    return Err(format!("Invalid scientific notation at line {} col {}", start_line, start_col));
                }
                while i < chars.len() {
                    if let Some(d) = normalize_digit(chars[i]) {
                        num.push(d);
                        i += 1;
                        column += 1;
                    } else {
                        break;
                    }
                }
            }
            i -= 1;
            column -= 1;
            let parsed: f64 = num.parse().map_err(|_| format!("Invalid number at line {}", start_line))?;
            if !parsed.is_finite() { return Err(format!("Number out of range at line {}", start_line)); }
            TokenKind::Number(parsed)
         }  
            _ if is_ident_start(c) => {
                while i + 1 < chars.len() && is_ident_continue(chars[i + 1]) {
                    i += 1;
                    column += 1;
                }
                let word: String = chars[start..=i].iter().collect();
                match word.as_str() {
                    "دع" | "ليكن" | "متغير" => TokenKind::Let,
                    "اطبع" => TokenKind::Print,
                    "إذا" | "اذا" => TokenKind::If,
                    "وإلا" | "والا" => TokenKind::Else,
                    "طالما" | "بينما" => TokenKind::While,
                    "لكل" => TokenKind::ForEach,
                    "في" => TokenKind::In,
                    "دالة" => TokenKind::Function,
                    "استورد" => TokenKind::Import,
                    "أرجع" | "ارجع" => TokenKind::Return,
                    "توقف" => TokenKind::Break,
                    "تابع" => TokenKind::Continue,
                    "صحيح" => TokenKind::True,
                    "خطأ" | "خطا" => TokenKind::False,
                    "لاشيء" => TokenKind::Null,
                    "و" => TokenKind::AndAnd,
                    "أو" | "او" => TokenKind::OrOr,
                    "ليس" => TokenKind::Bang,
                    "حاول" => TokenKind::Try,
                    "التقط" | "أمسك" | "امسك" => TokenKind::Catch,
                    "أخيرًا" | "أخيرا" | "اخيرا" => TokenKind::Finally,
                    "ارمِ" | "ارم" | "ألقِ" | "الق" => TokenKind::Throw,
                    "اختر" => TokenKind::Match,
                    "حالة" => TokenKind::Case,
                    "افتراضي" => TokenKind::Default,
                    "صنف" | "فئة" | "هيكل" => TokenKind::Class,
                    "يرث" | "يمتد" => TokenKind::Extends,
                    "جديد" => TokenKind::New,
                    "مع" => TokenKind::With,
                    "كـ" | "باسم" | "كاسم" | "as" => TokenKind::As,
                    "من" | "from" => TokenKind::From,
                    _ => match normalize_arabic_keyword(&word).as_str() {
                    "\u{0627}\u{0630}\u{0627}" => TokenKind::If,
                    "\u{0627}\u{0644}\u{0627}" => TokenKind::Else,
                    "\u{0648}\u{0627}\u{0644}\u{0627}" => TokenKind::Else,
                    "\u{062F}\u{0627}\u{0644}\u{0647}" => TokenKind::Function,
                    "\u{0627}\u{0631}\u{062C}\u{0639}" => TokenKind::Return,
                    "\u{0627}\u{0633}\u{062A}\u{064A}\u{0631}\u{0627}\u{062F}" => TokenKind::Import,
                    "\u{062E}\u{0637}\u{0627}" => TokenKind::False,
                    "\u{0627}\u{0648}" => TokenKind::OrOr,
                    "\u{0627}\u{0645}\u{0633}\u{0643}" => TokenKind::Catch,
                    "\u{0627}\u{062E}\u{064A}\u{0631}\u{0627}" => TokenKind::Finally,
                    "\u{0646}\u{0647}\u{0627}\u{0626}\u{064A}\u{0627}" => TokenKind::Finally,
                    "\u{0627}\u{0631}\u{0645}" => TokenKind::Throw,
                    "\u{0627}\u{0637}\u{0644}\u{0642}" => TokenKind::Throw,
                    "\u{0627}\u{0641}\u{062A}\u{0631}\u{0627}\u{0636}\u{064A}" => TokenKind::Default,
                    "\u{0641}\u{0626}\u{0647}" => TokenKind::Class,
                    "\u{062D}\u{0627}\u{0644}\u{0647}" => TokenKind::Case,
                    "\u{0627}\u{0637}\u{0628}\u{0639}" => TokenKind::Print,
                    "\u{0645}\u{062A}\u{063A}\u{064A}\u{0631}" => TokenKind::Let,
                    "\u{0644}\u{064A}\u{0643}\u{0646}" => TokenKind::Let,
                    "\u{0635}\u{062D}\u{064A}\u{062D}" => TokenKind::True,
                    "\u{0641}\u{0627}\u{0631}\u{063A}" => TokenKind::Null,
                    "\u{0644}\u{064A}\u{0633}" => TokenKind::Bang,
                    "\u{062C}\u{062F}\u{064A}\u{062F}" => TokenKind::New,
                    "\u{0644}\u{0643}\u{0644}" => TokenKind::ForEach,
                    "\u{0637}\u{0627}\u{0644}\u{0645}\u{0627}" => TokenKind::While,
                    "\u{0628}\u{064A}\u{0646}\u{0645}\u{0627}" => TokenKind::While,
                    "\u{062A}\u{0648}\u{0642}\u{0641}" => TokenKind::Break,
                    "\u{0627}\u{0633}\u{062A}\u{0645}\u{0631}" => TokenKind::Continue,
                    _ => TokenKind::Identifier(word),
                },
                }
            }
            _ => return Err(format!("السطر {}، العمود {}: رمز غير معروف '{}'", line, column, c)),
        };
        result.push(Token { kind, line: start_line, column: start_col });
        i += 1;
        column += 1;
    }
    result.push(Token { kind: TokenKind::Eof, line, column });
    Ok(result)
}

fn is_ident_start(c: char) -> bool { c.is_alphabetic() || c == '_' }
fn is_ident_continue(c: char) -> bool { is_ident_start(c) || is_lisan_digit(c) || matches!(c, '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{0640}') }
fn is_lisan_digit(c: char) -> bool { normalize_digit(c).is_some() }
fn is_decimal_sep(c: char) -> bool { matches!(c, '.' | '\u{066B}') }
fn normalize_digit(c: char) -> Option<char> {
    match c {
        '0'..='9' => Some(c),
        '\u{0660}'..='\u{0669}' => Some((b'0' + (c as u32 - 0x0660) as u8) as char),
        '\u{06F0}'..='\u{06F9}' => Some((b'0' + (c as u32 - 0x06F0) as u8) as char),
        _ => None,
    }
}
fn normalize_arabic_keyword(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.chars() {
        match c {
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{0640}' => {}
            '\u{0622}' | '\u{0623}' | '\u{0625}' | '\u{0671}' => out.push('\u{0627}'),
            '\u{0629}' => out.push('\u{0647}'),
            '\u{0649}' => out.push('\u{064A}'),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arabic_and_persian_numerals() {
        let tokens = tokenize("\u{0661}\u{0662}\u{0663} \u{0663}\u{066B}\u{0661}\u{0664} \u{06F4}\u{06F5}").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Number(n) if (n - 123.0).abs() < f64::EPSILON));
        assert!(matches!(tokens[1].kind, TokenKind::Number(n) if (n - 3.14).abs() < 1e-10));
        assert!(matches!(tokens[2].kind, TokenKind::Number(n) if (n - 45.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_normalized_arabic_keywords() {
        let tokens = tokenize("\u{0625}\u{0630}\u{0627} \u{0627}\u{0630}\u{0627} \u{062F}\u{0627}\u{0644}\u{0629} \u{062F}\u{0627}\u{0644}\u{0647}").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::If));
        assert!(matches!(tokens[1].kind, TokenKind::If));
        assert!(matches!(tokens[2].kind, TokenKind::Function));
        assert!(matches!(tokens[3].kind, TokenKind::Function));
    }
}
