use crate::ast::{Expr, Function, Op, Stmt};
use std::collections::HashMap;
use std::fmt;

const MAX_STEPS: usize = 250_000;
const MAX_CALL_DEPTH: usize = 128;
const MAX_LIST_SIZE: usize = 10_000;

/// Flow through nested statements; loops consume Break/Continue, functions consume Return.
#[derive(Debug, PartialEq)]
enum Flow {
    Next,
    Return(Value),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Bool(bool),
    Null,
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Bool(true) => write!(f, "صحيح"),
            Value::Bool(false) => write!(f, "خطأ"),
            Value::Null => write!(f, "لاشيء"),
            Value::Map(pairs) => {
                write!(f, "{{")?;
                for (i, (key, value)) in pairs.iter().enumerate() {
                    if i > 0 { write!(f, "، ")?; }
                    write!(f, "{:?}: ", key)?;
                    match value {
                        Value::Text(text) => write!(f, "{:?}", text)?,
                        other => write!(f, "{}", other)?,
                    }
                }
                write!(f, "}}")
            }
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, "، ")?; }
                    match item {
                        Value::Text(s) => write!(f, "{:?}", s)?,
                        value => write!(f, "{}", value)?,
                    }
                }
                write!(f, "]")
            }
        }
    }
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
    functions: HashMap<String, Function>,
    output: Vec<String>,
    steps: usize,
    call_depth: usize,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            output: Vec::new(),
            steps: 0,
            call_depth: 0,
        }
    }
}

impl Interpreter {
    pub fn run_source(&mut self, source: &str) -> Result<Vec<String>, String> {
        let tokens = crate::lexer::tokenize(source)?;
        let statements = crate::parser::parse(tokens)?;
        self.run_program(&statements)
    }

    /// Run a UTF-8 .lsn file and its imports relative to each source file.
    pub fn run_file(&mut self, path: impl AsRef<std::path::Path>) -> Result<Vec<String>, String> {
        let statements = crate::modules::load_file(path.as_ref())?;
        self.run_program(&statements)
    }

    /// Execute an already parsed, import-resolved program. Used by the bundler.
    pub fn run_program(&mut self, statements: &[Stmt]) -> Result<Vec<String>, String> {
        self.output.clear();
        self.steps = 0;
        match self.execute_statements(&statements, false)? {
            Flow::Next => {},
            Flow::Return(_) => return Err("لا يجوز استخدام أرجع خارج الدالة".into()),
            Flow::Break | Flow::Continue => return Err("لا يجوز استخدام تحكم حلقة خارج الحلقة".into()),
        }
        Ok(std::mem::take(&mut self.output))
    }

    fn tick(&mut self) -> Result<(), String> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            Err("تجاوز البرنامج الحد الآمن لعدد خطوات التنفيذ".into())
        } else { Ok(()) }
    }

    fn execute_statements(&mut self, stmts: &[Stmt], scoped: bool) -> Result<Flow, String> {
        if scoped { self.scopes.push(HashMap::new()); }
        let result = (|| {
            for stmt in stmts {
                match self.execute(stmt)? {
                    Flow::Next => {},
                    signal => return Ok(signal),
                }
            }
            Ok(Flow::Next)
        })();
        if scoped { self.scopes.pop(); }
        result
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<Flow, String> {
        self.tick()?;
        match stmt {
            Stmt::Import(_) => {
                return Err("استورد تعمل فقط عند تشغيل ملف .lsn وليس من النص التفاعلي".into());
            }
            Stmt::Let(name, expr) => {
                let value = self.eval(expr)?;
                let scope = self.scopes.last_mut().expect("scope exists");
                if scope.contains_key(name) {
                    return Err(format!("المتغير '{}' معرّف من قبل في هذا النطاق", name));
                }
                scope.insert(name.clone(), value);
            }
            Stmt::Print(expr) => {
                let value = self.eval(expr)?;
                self.output.push(value.to_string());
            }
            Stmt::Expr(expr) => { self.eval(expr)?; }
            Stmt::Block(stmts) => { return self.execute_statements(stmts, true); }
            Stmt::If(cond, yes, no) => {
                if self.eval_bool(cond)? { return self.execute_statements(yes, true); }
                if let Some(no) = no { return self.execute_statements(no, true); }
            }
            Stmt::While(cond, stmts) => {
                loop {
                    self.tick()?;
                    if !self.eval_bool(cond)? { break; }
                    match self.execute_statements(stmts, true)? {
                        Flow::Next | Flow::Continue => {},
                        Flow::Break => break,
                        signal @ Flow::Return(_) => return Ok(signal),
                    }
                }
            }
            Stmt::ForEach(name, iterable, stmts) => {
                let values = match self.eval(iterable)? {
                    Value::List(items) => items,
                    Value::Text(s) => s.chars().map(|c| Value::Text(c.to_string())).collect(),
                    _ => return Err("حلقة لكل تتطلب قائمة أو نصًا".into()),
                };
                for value in values {
                    self.tick()?;
                    self.scopes.push(HashMap::from([(name.clone(), value)]));
                    let result = self.execute_statements(stmts, false);
                    self.scopes.pop();
                    match result? {
                        Flow::Next | Flow::Continue => {},
                        Flow::Break => break,
                        signal @ Flow::Return(_) => return Ok(signal),
                    }
                }
            }
            Stmt::Function(f) => { self.functions.insert(f.name.clone(), f.clone()); }
            Stmt::Return(expr) => {
                let value = match expr { Some(expr) => self.eval(expr)?, None => Value::Null };
                return Ok(Flow::Return(value));
            }
            Stmt::Break => return Ok(Flow::Break),
            Stmt::Continue => return Ok(Flow::Continue),
            Stmt::Throw(expr) => {
                let value = self.eval(expr)?;
                return Err(value.to_string());
            }
            Stmt::TryCatch { try_body, catch_var, catch_body, finally_body } => {
                let try_res = self.execute_statements(try_body, true);
                let mut flow = match try_res {
                    Ok(signal) => signal,
                    Err(err_msg) => {
                        self.scopes.push(HashMap::new());
                        if let Some(var) = catch_var {
                            self.scopes.last_mut().unwrap().insert(var.clone(), Value::Text(err_msg));
                        }
                        let catch_res = self.execute_statements(catch_body, false);
                        self.scopes.pop();
                        catch_res?
                    }
                };
                if let Some(finally) = finally_body {
                    let fin_res = self.execute_statements(finally, true)?;
                    if matches!(fin_res, Flow::Return(_) | Flow::Break | Flow::Continue) {
                        flow = fin_res;
                    }
                }
                return Ok(flow);
            }
            Stmt::Match { target, cases, default_case } => {
                let target_val = self.eval(target)?;
                let mut matched = false;
                for (case_expr, case_stmts) in cases {
                    let case_val = self.eval(case_expr)?;
                    if target_val == case_val {
                        matched = true;
                        let flow = self.execute_statements(case_stmts, true)?;
                        if flow != Flow::Next { return Ok(flow); }
                        break;
                    }
                }
                if !matched {
                    if let Some(def_stmts) = default_case {
                        let flow = self.execute_statements(def_stmts, true)?;
                        if flow != Flow::Next { return Ok(flow); }
                    }
                }
            }
        }
        Ok(Flow::Next)
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name)).cloned()
            .ok_or_else(|| format!("متغير غير معرّف: '{}'", name))
    }

    fn eval_bool(&mut self, expr: &Expr) -> Result<bool, String> {
        match self.eval(expr)? {
            Value::Bool(b) => Ok(b),
            _ => Err("الشرط يجب أن ينتج قيمة منطقية (صحيح أو خطأ)".into()),
        }
    }

    fn eval(&mut self, expr: &Expr) -> Result<Value, String> {
        self.tick()?;
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::List(items) => {
                if items.len() > MAX_LIST_SIZE { return Err("القائمة تجاوزت الحد الأقصى".into()); }
                let mut values = Vec::with_capacity(items.len());
                for item in items { values.push(self.eval(item)?); }
                Ok(Value::List(values))
            }
            Expr::Map(pairs) => {
                if pairs.len() > MAX_LIST_SIZE { return Err("القاموس تجاوز الحد الأقصى".into()); }
                let mut result: Vec<(String, Value)> = Vec::new();
                for (key, item) in pairs {
                    if result.iter().any(|(existing, _)| existing == key) {
                        return Err(format!("مفتاح مكرر في القاموس: '{}'", key));
                    }
                    let value = self.eval(item)?;
                    result.push((key.clone(), value));
                }
                Ok(Value::Map(result))
            }
            Expr::Variable(name) => self.get_variable(name),
            Expr::Assign(name, expr) => {
                let value = self.eval(expr)?;
                for scope in self.scopes.iter_mut().rev() {
                    if scope.contains_key(name) {
                        scope.insert(name.clone(), value.clone());
                        return Ok(value);
                    }
                }
                Err(format!("لا يمكن الإسناد لمتغير غير معرّف: '{}'", name))
            }
            Expr::Index(target, index) => {
                let value = self.eval(target)?;
                let key = self.eval(index)?;
                match value {
                    Value::List(items) => {
                        let idx = checked_index(key)?;
                        items.get(idx).cloned().ok_or_else(|| format!("الفهرس {} خارج حدود القائمة", idx))
                    }
                    Value::Text(text) => {
                        let idx = checked_index(key)?;
                        text.chars().nth(idx).map(|c| Value::Text(c.to_string()))
                            .ok_or_else(|| format!("الفهرس {} خارج حدود النص", idx))
                    }
                    Value::Map(items) => {
                        let key = checked_key(key)?;
                        items.iter().find(|(stored, _)| stored == &key)
                            .map(|(_, value)| value.clone())
                            .ok_or_else(|| format!("مفتاح غير موجود في القاموس: '{}'", key))
                    }
                    _ => Err("الفهرسة تتطلب قائمة أو نصًا أو قاموسًا".into()),
                }
            }
            Expr::AssignIndex(name, index, expr) => {
                let key = self.eval(index)?;
                let value = self.eval(expr)?;
                for scope in self.scopes.iter_mut().rev() {
                    if let Some(target) = scope.get_mut(name) {
                        return match target {
                            Value::List(items) => {
                                let idx = checked_index(key)?;
                                if idx >= items.len() {
                                    Err(format!("الفهرس {} خارج حدود القائمة", idx))
                                } else {
                                    items[idx] = value.clone();
                                    Ok(value)
                                }
                            }
                            Value::Map(items) => {
                                let key = checked_key(key)?;
                                if let Some((_, existing)) = items.iter_mut().find(|(name, _)| name == &key) {
                                    *existing = value.clone();
                                } else {
                                    if items.len() >= MAX_LIST_SIZE { return Err("القاموس تجاوز الحد الأقصى".into()); }
                                    items.push((key, value.clone()));
                                }
                                Ok(value)
                            }
                            _ => Err("إسناد عنصر يتطلب متغير قائمة أو قاموس".into()),
                        };
                    }
                }
                Err(format!("متغير غير معرّف: '{}'", name))
            }
            Expr::Unary(op, expr) => {
                let value = self.eval(expr)?;
                match (op, value) {
                    (Op::Negate, Value::Number(n)) => finite_number(-n),
                    (Op::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err("نوع قيمة غير صالح للعملية الأحادية".into()),
                }
            }
            Expr::Binary(left, Op::And, right) => {
                let lhs = self.eval_bool(left)?;
                if !lhs { return Ok(Value::Bool(false)); }
                Ok(Value::Bool(self.eval_bool(right)?))
            }
            Expr::Binary(left, Op::Or, right) => {
                let lhs = self.eval_bool(left)?;
                if lhs { return Ok(Value::Bool(true)); }
                Ok(Value::Bool(self.eval_bool(right)?))
            }
            Expr::Binary(left, op, right) => {
                let lhs = self.eval(left)?;
                let rhs = self.eval(right)?;
                self.binary(lhs, *op, rhs)
            }
            Expr::Call(name, args) => self.call(name, args),
        }
    }

    fn binary(&self, lhs: Value, op: Op, rhs: Value) -> Result<Value, String> {
        match op {
            Op::Equal => return Ok(Value::Bool(lhs == rhs)),
            Op::NotEqual => return Ok(Value::Bool(lhs != rhs)),
            Op::Add => {
                return match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => finite_number(a + b),
                    (Value::Text(a), b) => Ok(Value::Text(a + &b.to_string())),
                    (a, Value::Text(b)) => Ok(Value::Text(a.to_string() + &b)),
                    (Value::List(mut a), Value::List(b)) if a.len() + b.len() <= MAX_LIST_SIZE => {
                        a.extend(b);
                        Ok(Value::List(a))
                    }
                    (Value::List(_), Value::List(_)) => Err("القائمة تجاوزت الحد الأقصى".into()),
                    _ => Err("الجمع يحتاج رقمين أو نصًا أو قائمتين".into()),
                }
            }
            _ => {}
        }
        match (lhs, rhs) {
            (Value::Number(a), Value::Number(b)) => {
                match op {
                    Op::Subtract => finite_number(a - b),
                    Op::Multiply => finite_number(a * b),
                    Op::Power => finite_number(a.powf(b)),
                    Op::Divide if b != 0.0 => finite_number(a / b),
                    Op::Modulo if b != 0.0 => finite_number(a % b),
                    Op::Divide | Op::Modulo => Err("القسمة على صفر ممنوعة".into()),
                    Op::Less => Ok(Value::Bool(a < b)),
                    Op::LessEqual => Ok(Value::Bool(a <= b)),
                    Op::Greater => Ok(Value::Bool(a > b)),
                    Op::GreaterEqual => Ok(Value::Bool(a >= b)),
                    _ => Err("عملية غير صالحة على الأرقام".into()),
                }
            }
            _ => Err("عملية حسابية/مقارنة تحتاج قيمتين رقميتين".into()),
        }
    }

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<Value, String> {
        if !self.functions.contains_key(name) && is_builtin(name) {
            let mut values = Vec::with_capacity(args.len());
            for arg in args { values.push(self.eval(arg)?); }
            return call_builtin(name, values);
        }
        let function = self.functions.get(name).cloned()
            .ok_or_else(|| format!("دالة غير معرّفة: '{}'", name))?;
        if args.len() != function.params.len() {
            return Err(format!("الدالة '{}' تتوقع {} معاملات، استلمت {}",
                name, function.params.len(), args.len()));
        }
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err("تجاوز الحد الآمن لعمق استدعاء الدوال".into());
        }
        let mut local = HashMap::new();
        for (param, arg) in function.params.iter().zip(args) {
            local.insert(param.clone(), self.eval(arg)?);
        }
        // Local variables of the caller are invisible; globals are visible and writable.
        let globals = self.scopes[0].clone();
        let previous_scopes = std::mem::replace(&mut self.scopes, vec![globals, local]);
        self.call_depth += 1;
        let result = self.execute_statements(&function.body, false);
        self.call_depth -= 1;
        let updated_globals = self.scopes[0].clone();
        self.scopes = previous_scopes;
        self.scopes[0] = updated_globals;
        match result? {
            Flow::Return(value) => Ok(value),
            Flow::Next => Ok(Value::Null),
            Flow::Break | Flow::Continue => Err("تحكم حلقة خارج الحلقة في الدالة".into()),
        }
    }
}

fn finite_number(value: f64) -> Result<Value, String> {
    if value.is_finite() { Ok(Value::Number(value)) }
    else { Err("الناتج الحسابي خارج النطاق المسموح للأعداد".into()) }
}

fn checked_index(index: Value) -> Result<usize, String> {
    match index {
        Value::Number(n) if n.is_finite() && n >= 0.0 && n.fract() == 0.0 && n < usize::MAX as f64 => Ok(n as usize),
        _ => Err("الفهرس يجب أن يكون عددًا صحيحًا غير سالب".into()),
    }
}

fn checked_key(key: Value) -> Result<String, String> {
    match key {
        Value::Text(s) => Ok(s),
        _ => Err("مفتاح القاموس يجب أن يكون نصًا".into()),
    }
}

fn is_builtin(name: &str) -> bool {
    matches!(name,
        "طول" | "نوع" | "نص" | "عدد" | "أضف" | "اضف" | "مدى" | "يحتوي" |
        "مفاتيح" | "قيم" | "احذف" | "قسّم" | "قسم" | "ادمج" | "استبدل" |
        "اقتطع" | "نظّف" | "نظف" | "مطلق" | "جذر" | "قرّب" | "قرب" |
        "أرضية" | "ارضية" | "سقف" | "قوة" |
        "أدنى" | "ادنى" | "أعلى" | "اعلى" | "اقرأ_ملف" | "اكتب_ملف" |
        "تحقق" | "تحقق_يساوي" |
        "عكس" | "اعكس" | "فرز" | "مجموع" | "متوسط" | "ابحث" |
        "يبدأ_ب" | "يبدا_ب" | "ينتهي_ب" | "كرر" |
        "أحرف_كبيرة" | "احرف_كبيرة" | "أحرف_صغيرة" | "احرف_صغيرة" |
        "ادخل" | "ملف_موجود" | "ألحق_ملف" | "الحق_ملف" | "الآن" | "الان" | "عدد_صحيح")
}

fn require_args(name: &str, values: &[Value], count: usize) -> Result<(), String> {
    if values.len() == count { Ok(()) }
    else { Err(format!("الدالة '{}' تتوقع {} معاملات، استلمت {}", name, count, values.len())) }
}

fn parse_latin_number(source: &str) -> Result<f64, String> {
    let text = source.trim();
    if text.chars().any(|c| matches!(c, '٠'..='٩' | '۰'..='۹' | '٫')) {
        return Err("استخدم الأرقام اللاتينية 0-9 والنقطة العشرية '.' فقط".into());
    }
    let number: f64 = text.parse().map_err(|_| "لا يمكن تحويل النص إلى رقم".to_string())?;
    if number.is_finite() { Ok(number) } else { Err("الرقم خارج النطاق المسموح".into()) }
}

fn expect_integer(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) if n.is_finite() && n.fract() == 0.0
            && n.abs() <= 9_007_199_254_740_991.0 => Ok(*n),
        _ => Err("مدى تتطلب أعدادًا صحيحة دقيقة ضمن النطاق المسموح".into()),
    }
}

fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "تحقق" => {
            require_args(name, &args, 1)?;
            match args[0] {
                Value::Bool(true) => Ok(Value::Bool(true)),
                Value::Bool(false) => Err("فشل التحقق: الشرط غير صحيح".into()),
                _ => Err("تحقق تتطلب قيمة منطقية".into()),
            }
        }
        "تحقق_يساوي" => {
            require_args(name, &args, 2)?;
            if args[0] == args[1] { Ok(Value::Bool(true)) }
            else { Err(format!("فشل التحقق: المتوقع {} لكن الفعلي {}", args[1], args[0])) }
        }
        "طول" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::List(items) => Ok(Value::Number(items.len() as f64)),
                Value::Text(text) => Ok(Value::Number(text.chars().count() as f64)),
                Value::Map(items) => Ok(Value::Number(items.len() as f64)),
                _ => Err("طول يقبل قائمة أو نصًا فقط".into()),
            }
        }
        "نوع" => {
            require_args(name, &args, 1)?;
            let kind = match args[0] {
                Value::Number(_) => "رقم",
                Value::Text(_) => "نص",
                Value::Bool(_) => "منطقي",
                Value::Null => "لاشيء",
                Value::List(_) => "قائمة",
                Value::Map(_) => "قاموس",
            };
            Ok(Value::Text(kind.into()))
        }
        "نص" => {
            require_args(name, &args, 1)?;
            Ok(Value::Text(args[0].to_string()))
        }
        "عدد" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(n) => Ok(Value::Number(*n)),
                Value::Text(s) => Ok(Value::Number(parse_latin_number(s)?)),
                _ => Err("عدد يقبل رقمًا أو نصًا رقميًا".into()),
            }
        }
        "أضف" | "اضف" => {
            require_args(name, &args, 2)?;
            let mut items = match &args[0] {
                Value::List(items) => items.clone(),
                _ => return Err("أضف تتطلب قائمة كمعامل أول".into()),
            };
            if items.len() >= MAX_LIST_SIZE { return Err("القائمة تجاوزت الحد الأقصى".into()); }
            items.push(args[1].clone());
            Ok(Value::List(items))
        }
        "يحتوي" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::List(items), sought) => Ok(Value::Bool(items.contains(sought))),
                (Value::Text(text), Value::Text(sought)) => Ok(Value::Bool(text.contains(sought))),
                (Value::Map(pairs), Value::Text(key)) => Ok(Value::Bool(pairs.iter().any(|(k, _)| k == key))),
                _ => Err("يحتوي تتطلب قائمة أو نصين".into()),
            }
        }
        "مفاتيح" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Map(pairs) => Ok(Value::List(pairs.iter().map(|(key, _)| Value::Text(key.clone())).collect())),
                _ => Err("مفاتيح تتطلب قاموسًا".into()),
            }
        }
        "قيم" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Map(pairs) => Ok(Value::List(pairs.iter().map(|(_, val)| val.clone()).collect())),
                _ => Err("قيم تتطلب قاموسًا".into()),
            }
        }
        "احذف" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Map(pairs), Value::Text(key)) => {
                    let mut new = pairs.clone();
                    new.retain(|(name, _)| name != key);
                    Ok(Value::Map(new))
                }
                (Value::List(items), Value::Number(_)) => {
                    let index = checked_index(args[1].clone())?;
                    if index >= items.len() { return Err("الفهرس خارج حدود القائمة".into()); }
                    let mut new = items.clone();
                    new.remove(index);
                    Ok(Value::List(new))
                }
                _ => Err("احذف تتطلب قاموسًا ومفتاحًا نصيًا، أو قائمة وفهرسًا".into()),
            }
        }
        "قسّم" | "قسم" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(text), Value::Text(sep)) => {
                    let items: Vec<_> = text.split(sep).map(|v| Value::Text(v.to_owned())).collect();
                    if items.len() > MAX_LIST_SIZE { return Err("القائمة تجاوزت الحد الأقصى".into()); }
                    Ok(Value::List(items))
                }
                _ => Err("قسّم تتطلب نصين".into()),
            }
        }
        "ادمج" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::List(items), Value::Text(sep)) => {
                    let strings: Result<Vec<&str>, String> = items.iter().map(|x| match x {
                        Value::Text(v) => Ok(v.as_str()),
                        _ => Err("ادمج تتطلب قائمة من النصوص".into()),
                    }).collect();
                    Ok(Value::Text(strings?.join(sep)))
                }
                _ => Err("ادمج تتطلب قائمة ونصًا للفاصل".into()),
            }
        }
        "استبدل" => {
            require_args(name, &args, 3)?;
            match (&args[0], &args[1], &args[2]) {
                (Value::Text(source), Value::Text(from), Value::Text(to)) => {
                    if from.is_empty() { return Err("لا يجوز أن يكون النص المستبدَل فارغًا".into()); }
                    Ok(Value::Text(source.replace(from, to)))
                }
                _ => Err("استبدل تتطلب ثلاثة نصوص".into()),
            }
        }
        "اقتطع" => {
            require_args(name, &args, 3)?;
            match &args[0] {
                Value::Text(source) => {
                    let first = checked_index(args[1].clone())?;
                    let end = checked_index(args[2].clone())?;
                    let chars: Vec<char> = source.chars().collect();
                    if first > end || end > chars.len() { return Err("حدود الاقتطاع غير صحيحة".into()); }
                    Ok(Value::Text(chars[first..end].iter().collect()))
                }
                _ => Err("اقتطع تتطلب نصًا وفهرسين".into()),
            }
        }
        "نظّف" | "نظف" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(text) => Ok(Value::Text(text.trim().to_string())),
                _ => Err("نظّف تتطلب نصًا".into()),
            }
        }
        "مطلق" | "جذر" | "قرّب" | "قرب" => {
            require_args(name, &args, 1)?;
            match args[0] {
                Value::Number(num) => {
                    let result = match name {
                        "مطلق" => num.abs(),
                        "جذر" if num >= 0.0 => num.sqrt(),
                        "جذر" => return Err("الجذر التربيعي لعدد سالب غير مدعوم".into()),
                        _ => num.round(),
                    };
                    if !result.is_finite() { return Err("الناتج خارج نطاق الأعداد".into()); }
                    Ok(Value::Number(result))
                }
                _ => Err("تتطلب الدالة عددًا".into()),
            }
        }
        "أدنى" | "ادنى" | "أعلى" | "اعلى" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(if name == "أدنى" || name == "ادنى" { a.min(*b) } else { a.max(*b) })),
                _ => Err("المقارنة تتطلب رقمين".into()),
            }
        }
        "اقرأ_ملف" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(path) => {
                    let metadata = std::fs::metadata(path).map_err(|e| format!("تعذّرت قراءة الملف: {}", e))?;
                    if metadata.len() > 2_000_000 { return Err("حجم الملف يتجاوز 2 ميغابايت".into()); }
                    std::fs::read_to_string(path).map(Value::Text).map_err(|e| format!("تعذّرت قراءة النص UTF-8: {}", e))
                }
                _ => Err("اقرأ_ملف تتطلب مسارًا نصيًا".into()),
            }
        }
        "اكتب_ملف" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(path), Value::Text(contents)) => {
                    if contents.len() > 2_000_000 { return Err("حجم النص يتجاوز 2 ميغابايت".into()); }
                    std::fs::write(path, contents).map_err(|e| format!("تعذّرت كتابة الملف: {}", e))?;
                    Ok(Value::Bool(true))
                }
                _ => Err("اكتب_ملف تتطلب مسارًا ونصًا".into()),
            }
        }
        "مدى" => {
            if !(1..=3).contains(&args.len()) {
                return Err("مدى تتوقع من معامل واحد إلى ثلاثة معاملات".into());
            }
            let (start, end, step) = match args.len() {
                1 => (0.0, expect_integer(&args[0])?, 1.0),
                2 => (expect_integer(&args[0])?, expect_integer(&args[1])?, 1.0),
                _ => (expect_integer(&args[0])?, expect_integer(&args[1])?, expect_integer(&args[2])?),
            };
            if step == 0.0 { return Err("خطوة مدى لا يجوز أن تكون صفرًا".into()); }
            let raw_length = (end - start) / step;
            let length = if raw_length <= 0.0 { 0.0 } else { raw_length.ceil() };
            if !length.is_finite() || length > MAX_LIST_SIZE as f64 {
                return Err("مدى تجاوز الحد الأقصى للقائمة".into());
            }
            let mut out = Vec::with_capacity(length as usize);
            for i in 0..length as usize {
                out.push(Value::Number(start + (i as f64) * step));
            }
            Ok(Value::List(out))
        }
        "عكس" | "اعكس" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::List(items) => {
                    let mut rev = items.clone();
                    rev.reverse();
                    Ok(Value::List(rev))
                }
                Value::Text(text) => {
                    let rev: String = text.chars().rev().collect();
                    Ok(Value::Text(rev))
                }
                _ => Err("عكس تتطلب قائمة أو نصًا".into()),
            }
        }
        "فرز" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::List(items) => {
                    let mut sorted = items.clone();
                    if sorted.is_empty() {
                        return Ok(Value::List(sorted));
                    }
                    let all_numbers = sorted.iter().all(|x| matches!(x, Value::Number(_)));
                    let all_strings = sorted.iter().all(|x| matches!(x, Value::Text(_)));
                    if all_numbers {
                        sorted.sort_by(|a, b| match (a, b) {
                            (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            _ => unreachable!(),
                        });
                        Ok(Value::List(sorted))
                    } else if all_strings {
                        sorted.sort_by(|a, b| match (a, b) {
                            (Value::Text(x), Value::Text(y)) => x.cmp(y),
                            _ => unreachable!(),
                        });
                        Ok(Value::List(sorted))
                    } else {
                        Err("فرز تتطلب قائمة متجانسة من الأرقام أو النصوص".into())
                    }
                }
                _ => Err("فرز تتطلب قائمة".into()),
            }
        }
        "مجموع" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::List(items) => {
                    let mut sum = 0.0;
                    for item in items {
                        match item {
                            Value::Number(n) => sum += *n,
                            _ => return Err("مجموع تتطلب قائمة من الأرقام فقط".into()),
                        }
                    }
                    finite_number(sum)
                }
                _ => Err("مجموع تتطلب قائمة".into()),
            }
        }
        "متوسط" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::List(items) => {
                    if items.is_empty() {
                        return Err("لا يمكن حساب متوسط قائمة فارغة".into());
                    }
                    let mut sum = 0.0;
                    for item in items {
                        match item {
                            Value::Number(n) => sum += *n,
                            _ => return Err("متوسط تتطلب قائمة من الأرقام فقط".into()),
                        }
                    }
                    finite_number(sum / (items.len() as f64))
                }
                _ => Err("متوسط تتطلب قائمة".into()),
            }
        }
        "ابحث" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::List(items), target) => {
                    let pos = items.iter().position(|x| x == target).map(|i| i as f64).unwrap_or(-1.0);
                    Ok(Value::Number(pos))
                }
                (Value::Text(haystack), Value::Text(needle)) => {
                    if let Some(byte_idx) = haystack.find(needle) {
                        let char_idx = haystack[..byte_idx].chars().count() as f64;
                        Ok(Value::Number(char_idx))
                    } else {
                        Ok(Value::Number(-1.0))
                    }
                }
                _ => Err("ابحث تتطلب قائمة وعنصرًا للبحث، أو نصين".into()),
            }
        }
        "يبدأ_ب" | "يبدا_ب" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(s), Value::Text(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
                _ => Err("يبدأ_ب تتطلب نصين".into()),
            }
        }
        "ينتهي_ب" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(s), Value::Text(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
                _ => Err("ينتهي_ب تتطلب نصين".into()),
            }
        }
        "كرر" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(s), Value::Number(n)) => {
                    let count = checked_index(Value::Number(*n))?;
                    let total_len = s.len().saturating_mul(count);
                    if total_len > 2_000_000 {
                        return Err("النص المكرر تجاوز الحد الأقصى (2 ميغابايت)".into());
                    }
                    Ok(Value::Text(s.repeat(count)))
                }
                _ => Err("كرر تتطلب نصًا وعددًا صحيحًا موجبًا".into()),
            }
        }
        "أحرف_كبيرة" | "احرف_كبيرة" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(s) => Ok(Value::Text(s.to_uppercase())),
                _ => Err("أحرف_كبيرة تتطلب نصًا".into()),
            }
        }
        "أحرف_صغيرة" | "احرف_صغيرة" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(s) => Ok(Value::Text(s.to_lowercase())),
                _ => Err("أحرف_صغيرة تتطلب نصًا".into()),
            }
        }
        "أرضية" | "ارضية" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(n) => finite_number(n.floor()),
                _ => Err("أرضية تتطلب عددًا".into()),
            }
        }
        "سقف" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(n) => finite_number(n.ceil()),
                _ => Err("سقف تتطلب عددًا".into()),
            }
        }
        "قوة" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Number(base), Value::Number(exp)) => {
                    if *base == 0.0 && *exp < 0.0 {
                        return Err("القسمة على صفر أثناء حساب القوة".into());
                    }
                    let res = base.powf(*exp);
                    finite_number(res)
                }
                _ => Err("قوة تتطلب رقمين (الأساس والأس)".into()),
            }
        }
        "ادخل" => {
            if args.len() > 1 { return Err("ادخل تقبل رسالة اختيارية واحدة".into()); }
            if let Some(Value::Text(prompt)) = args.first() {
                use std::io::Write;
                print!("{}", prompt);
                let _ = std::io::stdout().flush();
            }
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).map_err(|e| format!("تعذّرت قراءة الإدخال: {}", e))?;
            Ok(Value::Text(line.trim_end_matches(['\n', '\r']).to_string()))
        }
        "ملف_موجود" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(path) => Ok(Value::Bool(std::path::Path::new(path).is_file())),
                _ => Err("ملف_موجود تتطلب مسارًا نصيًا".into()),
            }
        }
        "ألحق_ملف" | "الحق_ملف" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(path), Value::Text(contents)) => {
                    use std::io::Write;
                    if contents.len() > 2_000_000 { return Err("حجم النص يتجاوز 2 ميغابايت".into()); }
                    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)
                        .map_err(|e| format!("تعذّر فتح الملف: {}", e))?;
                    file.write_all(contents.as_bytes()).map_err(|e| format!("تعذّرت الكتابة: {}", e))?;
                    Ok(Value::Bool(true))
                }
                _ => Err("ألحق_ملف تتطلب مسارًا ونصًا".into()),
            }
        }
        "الآن" | "الان" => {
            require_args(name, &args, 0)?;
            let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("خطأ في قراءة الوقت: {}", e))?.as_secs_f64();
            Ok(Value::Number(secs))
        }
        "عدد_صحيح" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(n) => finite_number(n.trunc()),
                Value::Text(s) => finite_number(parse_latin_number(s)?.trunc()),
                _ => Err("عدد_صحيح تقبل رقمًا أو نصًا رقميًا".into()),
            }
        }
        _ => Err(format!("دالة غير معرّفة: '{}'", name)),
    }
}
