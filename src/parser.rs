use crate::ast::{Expr, Function, Op, Stmt};
use crate::lexer::{Token, TokenKind};
use std::mem::discriminant;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, String> {
    Parser { tokens, cursor: 0, loop_depth: 0, function_depth: 0 }.program()
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    loop_depth: usize,
    function_depth: usize,
}

impl Parser {
    fn current(&self) -> &Token { &self.tokens[self.cursor] }
    fn peek_next(&self) -> Option<&Token> { self.tokens.get(self.cursor + 1) }
    fn is(&self, kind: &TokenKind) -> bool {
        discriminant(&self.current().kind) == discriminant(kind)
    }
    fn take(&mut self, kind: &TokenKind) -> bool {
        if self.is(kind) { self.cursor += 1; true } else { false }
    }
    fn take_as(&mut self) -> bool {
        if self.take(&TokenKind::As) {
            return true;
        }
        if let TokenKind::Identifier(ref name) = self.current().kind {
            if name == "ك" {
                self.cursor += 1;
                return true;
            }
        }
        false
    }
    fn error(&self, reason: &str) -> String {
        format!("السطر {}، العمود {}: {}", self.current().line, self.current().column, reason)
    }
    fn expect(&mut self, kind: &TokenKind, reason: &str) -> Result<(), String> {
        if self.take(kind) { Ok(()) } else { Err(self.error(reason)) }
    }
    fn identifier(&mut self) -> Result<String, String> {
        match self.current().kind.clone() {
            TokenKind::Identifier(name) => { self.cursor += 1; Ok(name) }
            _ => Err(self.error("متوقع اسم عربي لمتغير أو دالة")),
        }
    }
    fn program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is(&TokenKind::Eof) {
            if self.take(&TokenKind::Semicolon) { continue; }
            statements.push(self.statement()?);
        }
        Ok(statements)
    }
    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(&TokenKind::LeftBrace, "متوقع '{' لبدء الكتلة")?;
        let mut statements = Vec::new();
        while !self.is(&TokenKind::RightBrace) && !self.is(&TokenKind::Eof) {
            if self.take(&TokenKind::Semicolon) { continue; }
            statements.push(self.statement()?);
        }
        self.expect(&TokenKind::RightBrace, "متوقع '}' لإنهاء الكتلة")?;
        Ok(statements)
    }
    fn statement(&mut self) -> Result<Stmt, String> {
        let line = self.current().line;
        let col = self.current().column;
        let stmt = self.raw_statement()?;
        Ok(Stmt::Spanned(Box::new(stmt), line, col))
    }

    fn raw_statement(&mut self) -> Result<Stmt, String> {
        if self.take(&TokenKind::Import) {
            let (module, is_string) = match self.current().kind.clone() {
                TokenKind::String(path) => { self.cursor += 1; (path, true) }
                TokenKind::Identifier(name) => { self.cursor += 1; (name, false) }
                _ => return Err(self.error("استورد تتطلب اسم وحدة أو مسار ملف")),
            };
            let alias = if self.take_as() {
                Some(self.identifier()?)
            } else {
                None
            };
            self.semicolon()?;
            if is_string && alias.is_none() {
                return Ok(Stmt::Import(module));
            } else {
                return Ok(Stmt::ImportModule { module, alias });
            }
        }
        if self.take(&TokenKind::From) {
            let module = match self.current().kind.clone() {
                TokenKind::String(path) => { self.cursor += 1; path }
                TokenKind::Identifier(name) => { self.cursor += 1; name }
                _ => return Err(self.error("من تتطلب اسم وحدة أو مسار ملف")),
            };
            self.expect(&TokenKind::Import, "متوقع 'استورد' بعد اسم الوحدة في تعليمة 'من'")?;
            let mut items = Vec::new();
            loop {
                items.push(self.identifier()?);
                if !self.take(&TokenKind::Comma) { break; }
            }
            self.semicolon()?;
            return Ok(Stmt::FromImport { module, items });
        }
        if self.take(&TokenKind::With) {
            let has_paren = self.take(&TokenKind::LeftParen);
            let expr = self.expression()?;
            let alias = if self.take_as() {
                Some(self.identifier()?)
            } else {
                None
            };
            if has_paren {
                self.expect(&TokenKind::RightParen, "متوقع ')' بعد تعبير مع")?;
            }
            let body = self.block()?;
            return Ok(Stmt::With { expr, alias, body });
        }

        if self.take(&TokenKind::Function) {
            let func = self.function_declaration()?;
            return Ok(Stmt::Function(func));
        }
        if self.take(&TokenKind::Class) {
            let stmt = self.class_declaration()?;
            return Ok(stmt);
        }
        if self.take(&TokenKind::Let) {
            let name = self.identifier()?;
            let mut names = vec![name];
            while self.take(&TokenKind::Comma) {
                names.push(self.identifier()?);
            }
            self.expect(&TokenKind::Equal, "متوقع '=' لتعريف المتغير")?;
            let value = self.expression()?;
            self.semicolon()?;
            if names.len() == 1 {
                return Ok(Stmt::Let(names.pop().unwrap(), value));
            } else {
                return Ok(Stmt::LetUnpack(names, value));
            }
        }
        if self.take(&TokenKind::Print) {
            self.expect(&TokenKind::LeftParen, "متوقع '(' بعد اطبع")?;
            let expr = self.expression()?;
            self.expect(&TokenKind::RightParen, "متوقع ')' بعد اطبع")?;
            self.semicolon()?;
            return Ok(Stmt::Print(expr));
        }
        if self.take(&TokenKind::If) {
            self.expect(&TokenKind::LeftParen, "متوقع '(' بعد إذا")?;
            let cond = self.expression()?;
            self.expect(&TokenKind::RightParen, "متوقع ')' بعد الشرط")?;
            let yes = self.block()?;
            let no = if self.take(&TokenKind::Else) { Some(self.block()?) } else { None };
            return Ok(Stmt::If(cond, yes, no));
        }
        if self.take(&TokenKind::ForEach) {
            let has_paren = self.take(&TokenKind::LeftParen);
            let first_name = self.identifier()?;
            let mut names = vec![first_name];
            while self.take(&TokenKind::Comma) {
                names.push(self.identifier()?);
            }
            self.expect(&TokenKind::In, "متوقع 'في' داخل حلقة لكل")?;
            let iterable = self.expression()?;
            if has_paren {
                self.expect(&TokenKind::RightParen, "متوقع ')' بعد تعبير لكل")?;
            }
            self.loop_depth += 1;
            let body = self.block();
            self.loop_depth -= 1;
            if names.len() == 1 {
                return Ok(Stmt::ForEach(names.pop().unwrap(), iterable, body?));
            } else {
                return Ok(Stmt::ForEachUnpack(names, iterable, body?));
            }
        }
        if self.take(&TokenKind::While) {
            self.expect(&TokenKind::LeftParen, "متوقع '(' بعد طالما")?;
            let cond = self.expression()?;
            self.expect(&TokenKind::RightParen, "متوقع ')' بعد الشرط")?;
            self.loop_depth += 1;
            let body = self.block();
            self.loop_depth -= 1;
            return Ok(Stmt::While(cond, body?));
        }
        if self.take(&TokenKind::Return) {
            if self.function_depth == 0 { return Err(self.error("لا يجوز استخدام أرجع خارج الدالة")); }
            let prev_line = if self.cursor > 0 { self.tokens[self.cursor - 1].line } else { 0 };
            let is_empty = self.is(&TokenKind::Semicolon)
                || self.is(&TokenKind::RightBrace)
                || self.is(&TokenKind::Eof)
                || (self.current().line > prev_line);
            let val = if is_empty { None } else { Some(self.expression()?) };
            self.semicolon()?;
            return Ok(Stmt::Return(val));
        }
        if self.take(&TokenKind::Break) {
            if self.loop_depth == 0 { return Err(self.error("لا يجوز استخدام توقف خارج الحلقة")); }
            self.semicolon()?;
            return Ok(Stmt::Break);
        }
        if self.take(&TokenKind::Continue) {
            if self.loop_depth == 0 { return Err(self.error("لا يجوز استخدام تابع خارج الحلقة")); }
            self.semicolon()?;
            return Ok(Stmt::Continue);
        }
        if self.take(&TokenKind::Throw) {
            let expr = self.expression()?;
            self.semicolon()?;
            return Ok(Stmt::Throw(expr));
        }
        if self.take(&TokenKind::Try) {
            let try_body = self.block()?;
            self.expect(&TokenKind::Catch, "متوقع 'التقط' أو 'أمسك' بعد كتلة حاول")?;
            let mut catch_var = None;
            if self.take(&TokenKind::LeftParen) {
                catch_var = Some(self.identifier()?);
                self.expect(&TokenKind::RightParen, "متوقع ')' بعد متغير الخطأ")?;
            }
            let catch_body = self.block()?;
            let finally_body = if self.take(&TokenKind::Finally) {
                Some(self.block()?)
            } else {
                None
            };
            return Ok(Stmt::TryCatch {
                try_body,
                catch_var,
                catch_body,
                finally_body,
            });
        }
        if self.take(&TokenKind::Match) {
            self.expect(&TokenKind::LeftParen, "متوقع '(' بعد اختر")?;
            let target = self.expression()?;
            self.expect(&TokenKind::RightParen, "متوقع ')' بعد تعبير اختر")?;
            self.expect(&TokenKind::LeftBrace, "متوقع '{' لكتلة اختر")?;
            let mut cases = Vec::new();
            let mut default_case = None;
            while !self.is(&TokenKind::RightBrace) && !self.is(&TokenKind::Eof) {
                if self.take(&TokenKind::Case) {
                    let val = self.expression()?;
                    self.expect(&TokenKind::Colon, "متوقع ':' بعد تعبير حالة")?;
                    let mut stmts = Vec::new();
                    while !self.is(&TokenKind::Case) && !self.is(&TokenKind::Default) && !self.is(&TokenKind::RightBrace) && !self.is(&TokenKind::Eof) {
                        stmts.push(self.statement()?);
                    }
                    cases.push((val, stmts));
                } else if self.take(&TokenKind::Default) {
                    if default_case.is_some() {
                        return Err(self.error("لا يمكن تكرار حالة 'افتراضي' في جملة اختر"));
                    }
                    self.expect(&TokenKind::Colon, "متوقع ':' بعد افتراضي")?;
                    let mut stmts = Vec::new();
                    while !self.is(&TokenKind::Case) && !self.is(&TokenKind::Default) && !self.is(&TokenKind::RightBrace) && !self.is(&TokenKind::Eof) {
                        stmts.push(self.statement()?);
                    }
                    default_case = Some(stmts);
                } else {
                    return Err(self.error("متوقع 'حالة' أو 'افتراضي' داخل كتلة اختر"));
                }
            }
            self.expect(&TokenKind::RightBrace, "متوقع '}' لإغلاق كتلة اختر")?;
            return Ok(Stmt::Match {
                target,
                cases,
                default_case,
            });
        }
        if self.is(&TokenKind::LeftBrace) { return Ok(Stmt::Block(self.block()?)); }
        let expr = self.expression()?;
        self.semicolon()?;
        Ok(Stmt::Expr(expr))
    }

    fn function_declaration(&mut self) -> Result<Function, String> {
        let name = self.identifier()?;
        self.expect(&TokenKind::LeftParen, "متوقع '(' بعد اسم الدالة")?;
        let mut params = Vec::new();
        let mut defaults = Vec::new();
        let mut vararg = None;
        let mut seen_default = false;
        if !self.is(&TokenKind::RightParen) {
            loop {
                if self.take(&TokenKind::Star) {
                    let vname = self.identifier()?;
                    vararg = Some(vname);
                    if self.take(&TokenKind::Comma) {
                        if !self.is(&TokenKind::RightParen) {
                            return Err(self.error("معامل المتغيرات *يجب أن يكون في نهاية قائمة المعاملات"));
                        }
                    }
                    break;
                }
                let param_name = self.identifier()?;
                if params.contains(&param_name) { return Err(self.error("معامل مكرر في الدالة")); }
                params.push(param_name);
                if self.take(&TokenKind::Equal) {
                    seen_default = true;
                    let default_expr = self.expression()?;
                    defaults.push(Some(default_expr));
                } else {
                    if seen_default {
                        return Err(self.error("لا يمكن وضع معامل بدون قيمة افتراضية بعد معامل ذي قيمة افتراضية"));
                    }
                    defaults.push(None);
                }
                if !self.take(&TokenKind::Comma) { break; }
            }
        }
        self.expect(&TokenKind::RightParen, "متوقع ')' بعد معاملات الدالة")?;
        let enclosing_loop_depth = self.loop_depth;
        self.loop_depth = 0;
        self.function_depth += 1;
        let body = if self.take(&TokenKind::FatArrow) {
            let expr = self.expression()?;
            self.semicolon()?;
            Ok(vec![Stmt::Return(Some(expr))])
        } else {
            self.block()
        };
        self.function_depth -= 1;
        self.loop_depth = enclosing_loop_depth;
        Ok(Function { name, params, defaults, vararg, body: body? })
    }

    fn class_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.identifier()?;
        let parent = if self.take(&TokenKind::Extends) {
            Some(self.identifier()?)
        } else {
            None
        };
        self.expect(&TokenKind::LeftBrace, "متوقع '{' لبدء جسم الصنف")?;
        let mut methods = Vec::new();
        while !self.is(&TokenKind::RightBrace) && !self.is(&TokenKind::Eof) {
            if self.take(&TokenKind::Function) {
                let func = self.function_declaration()?;
                methods.push(func);
            } else if self.take(&TokenKind::Semicolon) {
                // Ignore stray semicolons
            } else {
                return Err(self.error("جسم الصنف يجب أن يحتوي على دوال معرّفة بـ 'دالة'"));
            }
        }
        self.expect(&TokenKind::RightBrace, "متوقع '}' لإنهاء جسم الصنف")?;
        Ok(Stmt::Class(crate::ast::ClassDef { name, parent, methods }))
    }

    fn semicolon(&mut self) -> Result<(), String> {
        if self.take(&TokenKind::Semicolon) {
            while self.take(&TokenKind::Semicolon) {}
            return Ok(());
        }
        if self.is(&TokenKind::RightBrace) || self.is(&TokenKind::Eof) {
            return Ok(());
        }
        if self.cursor > 0 {
            let prev_line = self.tokens[self.cursor - 1].line;
            let current_line = self.current().line;
            if current_line > prev_line {
                return Ok(());
            }
        }
        Err(self.error("متوقع ';' أو '؛' أو سطر جديد لإنهاء التعليمة"))
    }
    fn expression(&mut self) -> Result<Expr, String> {
        let line = self.current().line;
        let col = self.current().column;
        let expr = self.assignment()?;
        Ok(Expr::Spanned(Box::new(expr), line, col))
    }
    fn assignment(&mut self) -> Result<Expr, String> {
        let lhs = self.or()?;
        if let Expr::Variable(first_name) = lhs.unwrap_spanned().clone() {
            if self.is(&TokenKind::Comma) {
                let saved_cursor = self.cursor;
                let mut names = vec![first_name];
                let mut is_unpack = true;
                while self.take(&TokenKind::Comma) {
                    if let Ok(next_name) = self.identifier() {
                        names.push(next_name);
                    } else {
                        is_unpack = false;
                        break;
                    }
                }
                if is_unpack && self.take(&TokenKind::Equal) {
                    let rhs = self.assignment()?;
                    return Ok(Expr::AssignUnpack(names, Box::new(rhs)));
                } else {
                    self.cursor = saved_cursor;
                }
            }
        }
        if self.take(&TokenKind::Equal) {
            let rhs = self.assignment()?;
            match lhs.unwrap_spanned() {
                Expr::Variable(name) => Ok(Expr::Assign(name.clone(), Box::new(rhs))),
                Expr::GetProperty(target, prop) => Ok(Expr::SetProperty(target.clone(), prop.clone(), Box::new(rhs))),
                Expr::Index(target, index) => match target.unwrap_spanned() {
                    Expr::Variable(name) => Ok(Expr::AssignIndex(name.clone(), index.clone(), Box::new(rhs))),
                    _ => Err(self.error("إسناد عناصر القائمة أو القاموس يتطلب متغيرًا مباشرًا")),
                },
                _ => Err(self.error("الطرف الأيسر للإسناد يجب أن يكون اسم متغير أو خاصية")),
            }
        } else { Ok(lhs) }
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;
        while self.take(&TokenKind::OrOr) {
            expr = Expr::Binary(Box::new(expr), Op::Or, Box::new(self.and()?));
        }
        Ok(expr)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        while self.take(&TokenKind::AndAnd) {
            expr = Expr::Binary(Box::new(expr), Op::And, Box::new(self.equality()?));
        }
        Ok(expr)
    }
    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;
        loop {
            let op = if self.take(&TokenKind::EqualEqual) { Op::Equal }
                else if self.take(&TokenKind::BangEqual) { Op::NotEqual }
                else { break };
            expr = Expr::Binary(Box::new(expr), op, Box::new(self.comparison()?));
        }
        Ok(expr)
    }
    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;
        loop {
            let op = if self.take(&TokenKind::Less) { Op::Less }
                else if self.take(&TokenKind::LessEqual) { Op::LessEqual }
                else if self.take(&TokenKind::Greater) { Op::Greater }
                else if self.take(&TokenKind::GreaterEqual) { Op::GreaterEqual }
                else if self.take(&TokenKind::In) { Op::In }
                else if self.is(&TokenKind::Bang) && self.peek_next().map(|t| &t.kind) == Some(&TokenKind::In) {
                    self.cursor += 2;
                    Op::NotIn
                }
                else { break };
            expr = Expr::Binary(Box::new(expr), op, Box::new(self.term()?));
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;
        loop {
            let op = if self.take(&TokenKind::Plus) { Op::Add }
                else if self.take(&TokenKind::Minus) { Op::Subtract }
                else { break };
            expr = Expr::Binary(Box::new(expr), op, Box::new(self.factor()?));
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;
        loop {
            let op = if self.take(&TokenKind::Star) { Op::Multiply }
                else if self.take(&TokenKind::Slash) { Op::Divide }
                else if self.take(&TokenKind::Percent) { Op::Modulo }
                else { break };
            expr = Expr::Binary(Box::new(expr), op, Box::new(self.unary()?));
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if self.take(&TokenKind::Bang) { return Ok(Expr::Unary(Op::Not, Box::new(self.unary()?))); }
        if self.take(&TokenKind::Minus) { return Ok(Expr::Unary(Op::Negate, Box::new(self.unary()?))); }
        self.power()
    }
    fn power(&mut self) -> Result<Expr, String> {
        let base = self.call()?;
        if self.take(&TokenKind::Power) {
            // Right associative: 2 ** 3 ** 2 == 512.
            return Ok(Expr::Binary(Box::new(base), Op::Power, Box::new(self.unary()?)));
        }
        Ok(base)
    }
    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        loop {
            if self.take(&TokenKind::Dot) {
                let prop = self.identifier()?;
                if self.take(&TokenKind::LeftParen) {
                    let mut args = Vec::new();
                    if !self.is(&TokenKind::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if !self.take(&TokenKind::Comma) { break; }
                        }
                    }
                    self.expect(&TokenKind::RightParen, "متوقع ')' بعد وسائط استدعاء الطريقة")?;
                    expr = Expr::MethodCall(Box::new(expr), prop, args);
                } else {
                    expr = Expr::GetProperty(Box::new(expr), prop);
                }
                continue;
            }
            if self.take(&TokenKind::LeftBracket) {
                if self.take(&TokenKind::Colon) {
                    let end = if !self.is(&TokenKind::RightBracket) && !self.is(&TokenKind::Colon) {
                        Some(Box::new(self.expression()?))
                    } else {
                        None
                    };
                    let step = if self.take(&TokenKind::Colon) {
                        if !self.is(&TokenKind::RightBracket) {
                            Some(Box::new(self.expression()?))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    self.expect(&TokenKind::RightBracket, "متوقع ']' بعد التقطيع")?;
                    expr = Expr::Slice(Box::new(expr), None, end, step);
                    continue;
                }
                let first_expr = self.expression()?;
                if self.take(&TokenKind::Colon) {
                    let end = if !self.is(&TokenKind::RightBracket) && !self.is(&TokenKind::Colon) {
                        Some(Box::new(self.expression()?))
                    } else {
                        None
                    };
                    let step = if self.take(&TokenKind::Colon) {
                        if !self.is(&TokenKind::RightBracket) {
                            Some(Box::new(self.expression()?))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    self.expect(&TokenKind::RightBracket, "متوقع ']' بعد التقطيع")?;
                    expr = Expr::Slice(Box::new(expr), Some(Box::new(first_expr)), end, step);
                    continue;
                }
                self.expect(&TokenKind::RightBracket, "متوقع ']' بعد الفهرس")?;
                expr = Expr::Index(Box::new(expr), Box::new(first_expr));
                continue;
            }
            if !self.take(&TokenKind::LeftParen) { break; }
            let mut args = Vec::new();
            if !self.is(&TokenKind::RightParen) {
                loop {
                    args.push(self.expression()?);
                    if !self.take(&TokenKind::Comma) { break; }
                }
            }
            self.expect(&TokenKind::RightParen, "متوقع ')' بعد وسائط الاستدعاء")?;
            expr = match expr.unwrap_spanned() {
                Expr::Variable(name) => Expr::Call(name.clone(), args),
                _ => return Err(self.error("الاستدعاء يحتاج إلى اسم دالة")),
            };
        }
        Ok(expr)
    }
    fn primary(&mut self) -> Result<Expr, String> {
        let kind = self.current().kind.clone();
        match kind {
            TokenKind::Number(n) => { self.cursor += 1; Ok(Expr::Number(n)) }
            TokenKind::String(s) => { self.cursor += 1; Ok(Expr::Text(s)) }
            TokenKind::FString(s) => {
                self.cursor += 1;
                let segments = self.parse_fstring(&s)?;
                Ok(Expr::FString(segments))
            }
            TokenKind::True => { self.cursor += 1; Ok(Expr::Bool(true)) }
            TokenKind::False => { self.cursor += 1; Ok(Expr::Bool(false)) }
            TokenKind::Null => { self.cursor += 1; Ok(Expr::Null) }
            TokenKind::Function => {
                self.cursor += 1;
                self.expect(&TokenKind::LeftParen, "متوقع '(' بعد 'دالة' في الدالة المجهولة")?;
                let mut params = Vec::new();
                if !self.is(&TokenKind::RightParen) {
                    loop {
                        let name = self.identifier()?;
                        if params.contains(&name) { return Err(self.error("معامل مكرر في الدالة")); }
                        params.push(name);
                        if !self.take(&TokenKind::Comma) { break; }
                    }
                }
                self.expect(&TokenKind::RightParen, "متوقع ')' بعد معاملات الدالة")?;
                let enclosing_loop_depth = self.loop_depth;
                self.loop_depth = 0;
                self.function_depth += 1;
                let body = if self.take(&TokenKind::FatArrow) {
                    let expr = self.expression()?;
                    Ok(vec![Stmt::Return(Some(expr))])
                } else {
                    self.block()
                };
                self.function_depth -= 1;
                self.loop_depth = enclosing_loop_depth;
                Ok(Expr::Lambda(params, body?))
            }
            TokenKind::New => {
                self.cursor += 1;
                let class_name = self.identifier()?;
                self.expect(&TokenKind::LeftParen, "متوقع '(' بعد اسم الصنف")?;
                let mut args = Vec::new();
                if !self.is(&TokenKind::RightParen) {
                    loop {
                        args.push(self.expression()?);
                        if !self.take(&TokenKind::Comma) { break; }
                    }
                }
                self.expect(&TokenKind::RightParen, "متوقع ')' بعد وسائط إنشاء الصنف")?;
                Ok(Expr::Call(class_name, args))
            }
            TokenKind::LeftBracket => {
                self.cursor += 1;
                if self.take(&TokenKind::RightBracket) {
                    return Ok(Expr::List(Vec::new()));
                }
                let first = self.expression()?;
                if self.take(&TokenKind::ForEach) {
                    let var = self.identifier()?;
                    self.expect(&TokenKind::In, "متوقع 'في' بعد اسم المتغير في التوليد المختصر")?;
                    let iterable = self.expression()?;
                    let condition = if self.take(&TokenKind::If) {
                        Some(Box::new(self.expression()?))
                    } else {
                        None
                    };
                    self.expect(&TokenKind::RightBracket, "متوقع ']' لإغلاق التوليد المختصر")?;
                    return Ok(Expr::ListComprehension {
                        expr: Box::new(first),
                        var,
                        iterable: Box::new(iterable),
                        condition,
                    });
                }
                let mut items = vec![first];
                while self.take(&TokenKind::Comma) {
                    if self.is(&TokenKind::RightBracket) { break; }
                    items.push(self.expression()?);
                }
                self.expect(&TokenKind::RightBracket, "متوقع ']' لإنهاء القائمة")?;
                Ok(Expr::List(items))
            }
            TokenKind::LeftBrace => {
                self.cursor += 1;
                let mut pairs = Vec::new();
                if !self.is(&TokenKind::RightBrace) {
                    loop {
                        let key = match self.current().kind.clone() {
                            TokenKind::String(key) => { self.cursor += 1; key },
                            _ => return Err(self.error("يجب أن يكون مفتاح القاموس نصًا بين علامتي اقتباس")),
                        };
                        self.expect(&TokenKind::Colon, "متوقع ':' بعد مفتاح القاموس")?;
                        let value = self.expression()?;
                        pairs.push((key, value));
                        if !self.take(&TokenKind::Comma) { break; }
                    }
                }
                self.expect(&TokenKind::RightBrace, "متوقع '}' لإغلاق القاموس")?;
                Ok(Expr::Map(pairs))
            }
            TokenKind::Identifier(s) => { self.cursor += 1; Ok(Expr::Variable(s)) }
            TokenKind::LeftParen => {
                self.cursor += 1;
                let expr = self.expression()?;
                self.expect(&TokenKind::RightParen, "متوقع ')' بعد التعبير")?;
                Ok(expr)
            }
            _ => Err(self.error("تعبير غير مكتمل أو رمز غير متوقع")),
        }
    }

    fn parse_fstring(&self, raw: &str) -> Result<Vec<Expr>, String> {
        let mut segments = Vec::new();
        let chars: Vec<char> = raw.chars().collect();
        let mut i = 0;
        let mut current_lit = String::new();
        while i < chars.len() {
            if chars[i] == '{' {
                if i + 1 < chars.len() && chars[i + 1] == '{' {
                    current_lit.push('{');
                    i += 2;
                    continue;
                }
                if !current_lit.is_empty() {
                    segments.push(Expr::Text(current_lit.clone()));
                    current_lit.clear();
                }
                i += 1; // skip '{'
                let mut expr_str = String::new();
                let mut depth = 1;
                while i < chars.len() {
                    let c = chars[i];
                    if c == '"' {
                        expr_str.push(c);
                        i += 1;
                        while i < chars.len() {
                            let sc = chars[i];
                            expr_str.push(sc);
                            if sc == '"' { i += 1; break; }
                            if sc == '\\' && i + 1 < chars.len() {
                                i += 1;
                                expr_str.push(chars[i]);
                            }
                            i += 1;
                        }
                        continue;
                    }
                    if c == '{' { depth += 1; }
                    else if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    expr_str.push(c);
                    i += 1;
                }
                if depth != 0 {
                    return Err(self.error("قوس '{' غير مغلق داخل نص f-string"));
                }
                let tokens = crate::lexer::tokenize(&expr_str)
                    .map_err(|e| format!("خطأ في تعبير داخل f-string: {}", e))?;
                let mut sub_parser = Parser { tokens, cursor: 0, loop_depth: self.loop_depth, function_depth: self.function_depth };
                let parsed_expr = sub_parser.expression()
                    .map_err(|e| format!("خطأ في تعبير داخل f-string: {}", e))?;
                segments.push(parsed_expr);
            } else if chars[i] == '}' {
                if i + 1 < chars.len() && chars[i + 1] == '}' {
                    current_lit.push('}');
                    i += 2;
                    continue;
                }
                return Err(self.error("قوس '}' غير متوقع في f-string"));
            } else {
                current_lit.push(chars[i]);
                i += 1;
            }
        }
        if !current_lit.is_empty() {
            segments.push(Expr::Text(current_lit));
        }
        Ok(segments)
    }
}
