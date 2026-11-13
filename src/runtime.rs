use crate::ast::{Expr, Function, Op, Stmt};
use std::collections::HashMap;
use std::fmt;

/// حدود التنفيذ. وضع «الصندوق الآمن» يحمي المتصفح والاختبارات من البرامج اللانهائية،
/// أما الوضع الكامل فيشغّل البرامج الحقيقية بلا حد للخطوات كما في بايثون.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// أقصى عدد لخطوات التنفيذ؛ `None` تعني بلا حد.
    pub max_steps: Option<usize>,
    /// أقصى عمق لاستدعاء الدوال (بايثون: 1000).
    pub max_call_depth: usize,
    /// أقصى عدد لعناصر القائمة أو القاموس.
    pub max_list_size: usize,
}

impl Limits {
    /// حدود آمنة للمتصفح (WebAssembly) والاختبارات وتنفيذ الشيفرة غير الموثوقة.
    pub const fn sandbox() -> Self {
        Self { max_steps: Some(250_000), max_call_depth: 32, max_list_size: 10_000 }
    }

    /// حدود البرامج الحقيقية من سطر الأوامر: بلا حد للخطوات، وعمق استدعاء مثل بايثون.
    /// تتطلب تشغيل المفسّر على خيط بمكدس كبير (انظر `run_with_large_stack`).
    pub const fn full() -> Self {
        Self { max_steps: None, max_call_depth: 1000, max_list_size: 50_000_000 }
    }
}

impl Default for Limits {
    fn default() -> Self { Self::sandbox() }
}

/// حجم المكدس اللازم لتشغيل عمق استدعاء 1000 في المفسّر الشجري.
pub const LARGE_STACK_BYTES: usize = 512 * 1024 * 1024;

/// تشغيل دالة على خيط بمكدس كبير حتى يتسع للاستدعاءات العودية العميقة.
pub fn run_with_large_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .name("lisan-main".into())
        .stack_size(LARGE_STACK_BYTES)
        .spawn(f)
        .expect("تعذّر إنشاء خيط التنفيذ")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}

thread_local! {
    /// حد حجم القوائم للخيط الحالي؛ تستخدمه الدوال المدمجة التي لا تملك وصولًا للمفسّر.
    static LIST_LIMIT: std::cell::Cell<usize> = std::cell::Cell::new(Limits::sandbox().max_list_size);
}

fn max_list_size() -> usize {
    LIST_LIMIT.with(|limit| limit.get())
}

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
    Instance {
        class_name: String,
        fields: std::rc::Rc<std::cell::RefCell<Vec<(String, Value)>>>,
    },
    Lambda {
        params: Vec<String>,
        body: Vec<Stmt>,
        captured: Vec<(String, Value)>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Bool(true) => write!(f, "صحيح"),
            Value::Bool(false) => write!(f, "خطأ"),
            Value::Null => write!(f, "لاشيء"),
            Value::Lambda { params, .. } => write!(f, "<دالة({})>", params.join("، ")),
            Value::Instance { class_name, fields } => {
                write!(f, "{} {{", class_name)?;
                let borrowed = fields.borrow();
                for (i, (key, value)) in borrowed.iter().enumerate() {
                    if i > 0 { write!(f, "، ")?; }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
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
    pub scopes: Vec<HashMap<String, Value>>,
    pub functions: HashMap<String, Function>,
    pub classes: HashMap<String, crate::ast::ClassDef>,
    output: Vec<String>,
    steps: usize,
    call_depth: usize,
    limits: Limits,
    /// عند التفعيل تُطبع المخرجات فورًا على الشاشة بدل تجميعها حتى نهاية البرنامج.
    stream_output: bool,
    pub current_span: Option<(usize, usize)>,
    pub call_stack: Vec<(String, Option<(usize, usize)>)>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::with_limits(Limits::sandbox())
    }
}

impl Interpreter {
    /// مفسّر بحدود تنفيذ محددة.
    pub fn with_limits(limits: Limits) -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            classes: HashMap::new(),
            output: Vec::new(),
            steps: 0,
            call_depth: 0,
            limits,
            stream_output: false,
            current_span: None,
            call_stack: Vec::new(),
        }
    }

    /// مفسّر للبرامج الحقيقية من سطر الأوامر: بلا حد للخطوات، وطباعة فورية للمخرجات.
    /// يجب تشغيله داخل `run_with_large_stack`.
    pub fn for_cli() -> Self {
        let mut interpreter = Self::with_limits(Limits::full());
        interpreter.stream_output = true;
        interpreter
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    fn emit(&mut self, line: String) {
        if self.stream_output {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            let _ = writeln!(handle, "{}", line);
            let _ = handle.flush();
        } else {
            self.output.push(line);
        }
    }

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
        self.current_span = None;
        self.call_stack.clear();
        let list_limit = self.limits.max_list_size;
        let previous_limit = LIST_LIMIT.with(|limit| limit.replace(list_limit));
        let result = self.execute_statements(&statements, false);
        LIST_LIMIT.with(|limit| limit.set(previous_limit));
        match result {
            Ok(Flow::Next) => {},
            Ok(Flow::Return(_)) => {
                let err = self.format_runtime_error("لا يجوز استخدام أرجع خارج الدالة");
                self.current_span = None;
                self.call_stack.clear();
                return Err(err);
            }
            Ok(Flow::Break) | Ok(Flow::Continue) => {
                let err = self.format_runtime_error("لا يجوز استخدام تحكم حلقة خارج الحلقة");
                self.current_span = None;
                self.call_stack.clear();
                return Err(err);
            }
            Err(err) => {
                let formatted = self.format_runtime_error(&err);
                self.current_span = None;
                self.call_stack.clear();
                return Err(formatted);
            }
        }
        Ok(std::mem::take(&mut self.output))
    }

    /// صياغة خطأ التشغيل مع تتبع مسار الاستدعاء وأرقام السطور (Traceback).
    pub fn format_runtime_error(&self, message: &str) -> String {
        if message.starts_with("تتبع مسار التنفيذ") || message.starts_with("خطأ تشغيل") {
            return message.to_string();
        }
        let mut out = String::new();
        if !self.call_stack.is_empty() {
            out.push_str("تتبع مسار التنفيذ (آخر استدعاء هو الأحدث):\n");
            for (fn_name, span) in &self.call_stack {
                match span {
                    Some((l, c)) => out.push_str(&format!("  السطر {}، العمود {}: في «{}»\n", l, c, fn_name)),
                    None => out.push_str(&format!("  في «{}»\n", fn_name)),
                }
            }
        }
        let location = match self.current_span {
            Some((l, c)) => format!("السطر {}، العمود {}: ", l, c),
            None => String::new(),
        };
        out.push_str(&format!("خطأ تشغيل: {}{}", location, message));
        out
    }

    fn tick(&mut self) -> Result<(), String> {
        self.steps += 1;
        match self.limits.max_steps {
            Some(max) if self.steps > max => {
                Err("تجاوز البرنامج الحد الآمن لعدد خطوات التنفيذ".into())
            }
            _ => Ok(()),
        }
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

    fn execute(&mut self, mut stmt: &Stmt) -> Result<Flow, String> {
        self.tick()?;
        let mut prev_span = None;
        while let Stmt::Spanned(inner, line, col) = stmt {
            if prev_span.is_none() {
                prev_span = Some(self.current_span);
            }
            self.current_span = Some((*line, *col));
            stmt = inner;
        }
        let res = self.execute_unspanned(stmt);
        if res.is_ok() {
            if let Some(prev) = prev_span {
                self.current_span = prev;
            }
        }
        res
    }

    fn execute_unspanned(&mut self, stmt: &Stmt) -> Result<Flow, String> {
        match stmt {
            Stmt::Import(name) => {
                if let Some(builtin_src) = crate::modules::resolve_builtin_module(name) {
                    let tokens = crate::lexer::tokenize(builtin_src)?;
                    let stmts = crate::parser::parse(tokens)?;
                    for s in &stmts {
                        match s.unwrap_spanned() {
                            Stmt::Let(var_name, expr) => {
                                let val = self.eval(expr)?;
                                self.scopes[0].insert(var_name.clone(), val);
                            }
                            Stmt::Function(func) => {
                                self.functions.insert(func.name.clone(), func.clone());
                            }
                            Stmt::Class(cls) => {
                                self.classes.insert(cls.name.clone(), cls.clone());
                            }
                            _ => {
                                self.execute(s)?;
                            }
                        }
                    }
                    return Ok(Flow::Next);
                }
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
                self.emit(value.to_string());
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
            Stmt::Class(c) => {
                if c.parent.as_deref() == Some(c.name.as_str()) {
                    return Err(format!("لا يجوز للصنف '{}' أن يرث من نفسه", c.name));
                }
                self.classes.insert(c.name.clone(), c.clone());
            }
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
                let stack_len = self.call_stack.len();
                let prev_span = self.current_span;
                let try_res = self.execute_statements(try_body, true);
                let mut flow = match try_res {
                    Ok(signal) => signal,
                    Err(err_msg) => {
                        self.call_stack.truncate(stack_len);
                        self.current_span = prev_span;
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
            Stmt::Spanned(_, _, _) => unreachable!(),
        }
        Ok(Flow::Next)
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        if let Some(val) = self.scopes.iter().rev().find_map(|scope| scope.get(name)) {
            return Ok(val.clone());
        }
        if let Some(f) = self.functions.get(name) {
            return Ok(Value::Lambda {
                params: f.params.clone(),
                body: f.body.clone(),
                captured: Vec::new(),
            });
        }
        let mut available: Vec<&str> = Vec::new();
        for scope in &self.scopes {
            for k in scope.keys() {
                available.push(k.as_str());
            }
        }
        for f in self.functions.keys() {
            available.push(f.as_str());
        }
        for c in self.classes.keys() {
            available.push(c.as_str());
        }
        Err(crate::diagnostics::suggest_variable_error(name, &available))
    }

    fn eval_bool(&mut self, expr: &Expr) -> Result<bool, String> {
        match self.eval(expr)? {
            Value::Bool(b) => Ok(b),
            _ => Err("الشرط يجب أن ينتج قيمة منطقية (صحيح أو خطأ)".into()),
        }
    }

    fn eval(&mut self, mut expr: &Expr) -> Result<Value, String> {
        self.tick()?;
        let mut prev_span = None;
        while let Expr::Spanned(inner, line, col) = expr {
            if prev_span.is_none() {
                prev_span = Some(self.current_span);
            }
            self.current_span = Some((*line, *col));
            expr = inner;
        }
        let res = self.eval_unspanned(expr);
        if res.is_ok() {
            if let Some(prev) = prev_span {
                self.current_span = prev;
            }
        }
        res
    }

    fn eval_unspanned(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Text(s) => Ok(Value::Text(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::List(items) => {
                if items.len() > max_list_size() { return Err("القائمة تجاوزت الحد الأقصى".into()); }
                let mut values = Vec::with_capacity(items.len());
                for item in items { values.push(self.eval(item)?); }
                Ok(Value::List(values))
            }
            Expr::Map(pairs) => {
                if pairs.len() > max_list_size() { return Err("القاموس تجاوز الحد الأقصى".into()); }
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
                        if let Some((_, value)) = items.iter().find(|(stored, _)| stored == &key) {
                            Ok(value.clone())
                        } else {
                            let available_keys: Vec<&str> = items.iter().map(|(k, _)| k.as_str()).collect();
                            Err(crate::diagnostics::suggest_dict_key_error(&key, &available_keys))
                        }
                    }
                    Value::Instance { class_name: _, ref fields } => {
                        let key = checked_key(key)?;
                        let borrowed = fields.borrow();
                        if let Some((_, value)) = borrowed.iter().find(|(k, _)| k == &key) {
                            Ok(value.clone())
                        } else {
                            let available_keys: Vec<&str> = borrowed.iter().map(|(k, _)| k.as_str()).collect();
                            Err(crate::diagnostics::suggest_dict_key_error(&key, &available_keys))
                        }
                    }
                    _ => Err("الفهرسة تتطلب قائمة أو نصًا أو قاموسًا أو كائنًا".into()),
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
                                    if items.len() >= max_list_size() { return Err("القاموس تجاوز الحد الأقصى".into()); }
                                    items.push((key, value.clone()));
                                }
                                Ok(value)
                            }
                            Value::Instance { fields, .. } => {
                                let key = checked_key(key)?;
                                let mut borrowed = fields.borrow_mut();
                                if let Some((_, existing)) = borrowed.iter_mut().find(|(name, _)| name == &key) {
                                    *existing = value.clone();
                                } else {
                                    borrowed.push((key, value.clone()));
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
            Expr::GetProperty(target, prop) => self.eval_get_property(target, prop),
            Expr::SetProperty(target, prop, expr) => self.eval_set_property(target, prop, expr),
            Expr::MethodCall(target, method_name, args) => self.eval_method_call(target, method_name, args),
            Expr::Lambda(params, body) => {
                let mut captured = Vec::new();
                for scope in self.scopes.iter().skip(1) {
                    for (k, v) in scope {
                        if let Some((_, existing)) = captured.iter_mut().find(|(ek, _): &&mut (String, Value)| ek == k) {
                            *existing = v.clone();
                        } else {
                            captured.push((k.clone(), v.clone()));
                        }
                    }
                }
                Ok(Value::Lambda {
                    params: params.clone(),
                    body: body.clone(),
                    captured,
                })
            }
            Expr::Spanned(_, _, _) => unreachable!(),
        }
    }

    fn find_class_method(&self, start_class: &str, method_name: &str) -> Result<Option<(String, Function)>, String> {
        let mut current = Some(start_class.to_string());
        let mut depth = 0;
        while let Some(cname) = current {
            depth += 1;
            if depth > 32 {
                return Err("تجاوز حد عمق الوراثة بين الأصناف (قد توجد حلقة وراثة)".into());
            }
            let cdef = self.classes.get(&cname).ok_or_else(|| format!("تعريف الصنف '{}' غير موجود", cname))?;
            if let Some(m) = cdef.methods.iter().find(|m| m.name == method_name).cloned() {
                return Ok(Some((cname, m)));
            }
            current = cdef.parent.clone();
        }
        Ok(None)
    }

    fn find_class_constructor(&self, start_class: &str) -> Result<Option<(String, Function)>, String> {
        let mut current = Some(start_class.to_string());
        let mut depth = 0;
        while let Some(cname) = current {
            depth += 1;
            if depth > 32 {
                return Err("تجاوز حد عمق الوراثة بين الأصناف".into());
            }
            let cdef = self.classes.get(&cname).ok_or_else(|| format!("تعريف الصنف '{}' غير موجود", cname))?;
            if let Some(m) = cdef.methods.iter().find(|m| m.name == "بناء" || m.name == "إنشاء" || m.name == "تهيئة").cloned() {
                return Ok(Some((cname, m)));
            }
            current = cdef.parent.clone();
        }
        Ok(None)
    }

    fn collect_class_method_names(&self, start_class: &str) -> Vec<String> {
        let mut names = Vec::new();
        let mut current = Some(start_class.to_string());
        let mut depth = 0;
        while let Some(cname) = current {
            depth += 1;
            if depth > 32 { break; }
            if let Some(cdef) = self.classes.get(&cname) {
                for m in &cdef.methods {
                    if !names.contains(&m.name) {
                        names.push(m.name.clone());
                    }
                }
                current = cdef.parent.clone();
            } else {
                break;
            }
        }
        names
    }

    #[inline(never)]
    fn invoke_method(&mut self, self_val: Value, owner_class: &str, method: &Function, evaluated_args: Vec<Value>) -> Result<Value, String> {
        if method.params.len() != evaluated_args.len() {
            return Err(format!("الطريقة '{}.{}' تتوقع {} معاملات، استلمت {}",
                owner_class, method.name, method.params.len(), evaluated_args.len()));
        }
        if self.call_depth >= self.limits.max_call_depth {
            return Err("تجاوز الحد الآمن لعمق استدعاء الدوال".into());
        }
        let mut local = HashMap::new();
        local.insert("هذا".to_string(), self_val.clone());
        local.insert("ذاتي".to_string(), self_val);
        local.insert("__class_owner".to_string(), Value::Text(owner_class.to_string()));
        for (param, arg) in method.params.iter().zip(evaluated_args) {
            local.insert(param.clone(), arg);
        }
        let globals = self.scopes[0].clone();
        let previous_scopes = std::mem::replace(&mut self.scopes, vec![globals, local]);
        self.call_depth += 1;
        let result = self.execute_statements(&method.body, false);
        self.call_depth -= 1;
        let updated_globals = self.scopes[0].clone();
        self.scopes = previous_scopes;
        self.scopes[0] = updated_globals;
        match result? {
            Flow::Return(value) => Ok(value),
            Flow::Next => Ok(Value::Null),
            Flow::Break | Flow::Continue => Err("تحكم حلقة خارج الحلقة في الطريقة".into()),
        }
    }

    #[inline(never)]
    fn invoke_lambda(&mut self, params: &[String], body: &[Stmt], captured: &[(String, Value)], evaluated_args: Vec<Value>) -> Result<Value, String> {
        if params.len() != evaluated_args.len() {
            return Err(format!("الدالة تتوقع {} معاملات، استلمت {}", params.len(), evaluated_args.len()));
        }
        if self.call_depth >= self.limits.max_call_depth {
            return Err("تجاوز الحد الآمن لعمق استدعاء الدوال".into());
        }
        let mut local = HashMap::new();
        for (k, v) in captured {
            local.insert(k.clone(), v.clone());
        }
        for (param, arg) in params.iter().zip(evaluated_args) {
            local.insert(param.clone(), arg);
        }
        let globals = self.scopes[0].clone();
        let previous_scopes = std::mem::replace(&mut self.scopes, vec![globals, local]);
        self.call_depth += 1;
        let result = self.execute_statements(body, false);
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

    #[inline(never)]
    fn eval_get_property(&mut self, target: &Expr, prop: &str) -> Result<Value, String> {
        let value = self.eval(target)?;
        match value {
            Value::Instance { class_name: _, ref fields } => {
                let borrowed = fields.borrow();
                if let Some((_, val)) = borrowed.iter().find(|(k, _)| k == prop) {
                    return Ok(val.clone());
                }
                let available: Vec<&str> = borrowed.iter().map(|(k, _)| k.as_str()).collect();
                Err(crate::diagnostics::suggest_dict_key_error(prop, &available))
            }
            Value::Text(ref text) => match prop {
                "طول" => Ok(Value::Number(text.chars().count() as f64)),
                _ => Err(format!("خاصية '{}' غير معروفة للنصوص", prop)),
            },
            Value::List(ref items) => match prop {
                "طول" => Ok(Value::Number(items.len() as f64)),
                _ => Err(format!("خاصية '{}' غير معروفة للقوائم", prop)),
            },
            Value::Map(ref pairs) => match prop {
                "طول" | "حجم" => Ok(Value::Number(pairs.len() as f64)),
                _ => Err(format!("خاصية '{}' غير معروفة للقواميس", prop)),
            },
            _ => Err("الوصول للخصائص عبر '.' يتطلب كائنًا أو بنية بيانات".into()),
        }
    }

    #[inline(never)]
    fn eval_set_property(&mut self, target: &Expr, prop: &str, expr: &Expr) -> Result<Value, String> {
        let obj = self.eval(target)?;
        let new_val = self.eval(expr)?;
        match obj {
            Value::Instance { fields, .. } => {
                let mut borrowed = fields.borrow_mut();
                if let Some((_, val)) = borrowed.iter_mut().find(|(k, _)| k == prop) {
                    *val = new_val.clone();
                } else {
                    borrowed.push((prop.to_string(), new_val.clone()));
                }
                Ok(new_val)
            }
            _ => Err("إسناد الخصائص عبر '.' يتطلب كائنًا".into()),
        }
    }

    #[inline(never)]
    fn eval_method_call(&mut self, target: &Expr, method_name: &str, args: &[Expr]) -> Result<Value, String> {
        if let Expr::Variable(v) = target.unwrap_spanned() {
            if v == "أصل" || v == "الأصل" {
                let self_val = self.get_variable("هذا").map_err(|_| "لا يمكن استخدام 'أصل' خارج طريقة صنف".to_string())?;
                let owner_class = match self.get_variable("__class_owner") {
                    Ok(Value::Text(s)) => s,
                    _ => match &self_val {
                        Value::Instance { class_name, .. } => class_name.clone(),
                        _ => return Err("لا يمكن استخدام 'أصل' خارج كائن".into()),
                    },
                };
                let cdef = self.classes.get(&owner_class).ok_or_else(|| format!("تعريف الصنف '{}' غير موجود", owner_class))?;
                let parent_name = cdef.parent.clone().ok_or_else(|| format!("الصنف '{}' لا يرث من أي صنف أب", owner_class))?;
                let mut evaluated_args = Vec::with_capacity(args.len());
                for a in args {
                    evaluated_args.push(self.eval(a)?);
                }
                let (found_owner, method) = if method_name == "بناء" || method_name == "إنشاء" || method_name == "تهيئة" {
                    self.find_class_constructor(&parent_name)?.ok_or_else(|| format!("الصنف الأب '{}' لا يملك دالة بناء", parent_name))?
                } else {
                    self.find_class_method(&parent_name, method_name)?.ok_or_else(|| format!("الطريقة '{}' غير معرفة في الصنف الأب '{}'", method_name, parent_name))?
                };
                let span = self.current_span;
                self.call_stack.push((format!("{}.{}", found_owner, method.name), span));
                let res = self.invoke_method(self_val, &found_owner, &method, evaluated_args);
                if res.is_ok() {
                    self.call_stack.pop();
                }
                return res;
            }
        }

        let obj = self.eval(target)?;
        let mut evaluated_args = Vec::with_capacity(args.len());
        for a in args {
            evaluated_args.push(self.eval(a)?);
        }

        match obj {
            Value::Instance { class_name, fields } => {
                let (owner_class, method) = match self.find_class_method(&class_name, method_name)? {
                    Some(found) => found,
                    None => {
                        let all_names = self.collect_class_method_names(&class_name);
                        let method_names: Vec<&str> = all_names.iter().map(|s| s.as_str()).collect();
                        return Err(crate::diagnostics::suggest_function_error(method_name, &method_names));
                    }
                };
                let self_val = Value::Instance { class_name, fields };
                let span = self.current_span;
                self.call_stack.push((format!("{}.{}", owner_class, method.name), span));
                let res = self.invoke_method(self_val, &owner_class, &method, evaluated_args);
                if res.is_ok() {
                    self.call_stack.pop();
                }
                res
            }
            Value::Text(text) => {
                match method_name {
                    "طول" if evaluated_args.is_empty() => Ok(Value::Number(text.chars().count() as f64)),
                    "تحتوي" if evaluated_args.len() == 1 => match &evaluated_args[0] {
                        Value::Text(sub) => Ok(Value::Bool(text.contains(sub))),
                        _ => Err("طريقة تحتوي تتطلب نصاً للبحث".into()),
                    },
                    "تقسيم" if evaluated_args.len() == 1 => match &evaluated_args[0] {
                        Value::Text(delim) => {
                            let parts: Vec<Value> = text.split(delim).map(|s| Value::Text(s.to_string())).collect();
                            Ok(Value::List(parts))
                        }
                        _ => Err("طريقة تقسيم تتطلب فاصلاً نصياً".into()),
                    },
                    "عكس" if evaluated_args.is_empty() => Ok(Value::Text(text.chars().rev().collect())),
                    "أحرف_كبيرة" if evaluated_args.is_empty() => Ok(Value::Text(text.to_uppercase())),
                    "أحرف_صغيرة" if evaluated_args.is_empty() => Ok(Value::Text(text.to_lowercase())),
                    "استبدل" | "أعد" if evaluated_args.len() == 2 => match (&evaluated_args[0], &evaluated_args[1]) {
                        (Value::Text(from), Value::Text(to)) => Ok(Value::Text(text.replace(from, to))),
                        _ => Err("طريقة استبدل تتطلب نصين (القديم والجديد)".into()),
                    },
                    _ => Err(format!("الطريقة '{}' غير مدعومة للنصوص", method_name)),
                }
            }
            Value::List(items) => {
                match method_name {
                    "طول" if evaluated_args.is_empty() => Ok(Value::Number(items.len() as f64)),
                    "أضف" | "اضف" if evaluated_args.len() == 1 => {
                        let mut updated = items.clone();
                        updated.push(evaluated_args[0].clone());
                        Ok(Value::List(updated))
                    }
                    "عكس" if evaluated_args.is_empty() => {
                        let mut rev = items.clone();
                        rev.reverse();
                        Ok(Value::List(rev))
                    }
                    "فرز" if evaluated_args.is_empty() => {
                        let mut sorted = items.clone();
                        sorted.sort_by(|a, b| match (a, b) {
                            (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                            (Value::Text(x), Value::Text(y)) => x.cmp(y),
                            _ => std::cmp::Ordering::Equal,
                        });
                        Ok(Value::List(sorted))
                    }
                    "مجموع" if evaluated_args.is_empty() => {
                        let mut sum = 0.0;
                        for it in &items {
                            match it {
                                Value::Number(n) => sum += n,
                                _ => return Err("طريقة مجموع تتطلب قائمة أرقام".into()),
                            }
                        }
                        Ok(Value::Number(sum))
                    }
                    "طبّق" | "طبق" if evaluated_args.len() == 1 => match &evaluated_args[0] {
                        Value::Lambda { params, body, captured } => {
                            let mut mapped = Vec::with_capacity(items.len());
                            for item in items {
                                mapped.push(self.invoke_lambda(params, body, captured, vec![item])?);
                            }
                            Ok(Value::List(mapped))
                        }
                        _ => Err("طريقة طبّق تتطلب دالة كمعامل".into()),
                    },
                    "رشّح" | "رشح" if evaluated_args.len() == 1 => match &evaluated_args[0] {
                        Value::Lambda { params, body, captured } => {
                            let mut filtered = Vec::new();
                            for item in items {
                                match self.invoke_lambda(params, body, captured, vec![item.clone()])? {
                                    Value::Bool(true) => filtered.push(item),
                                    Value::Bool(false) => {}
                                    _ => return Err("دالة الترشيح في رشّح يجب أن ترجع قيمة منطقية".into()),
                                }
                            }
                            Ok(Value::List(filtered))
                        }
                        _ => Err("طريقة رشّح تتطلب دالة كمعامل".into()),
                    },
                    "اختزل" if evaluated_args.len() == 2 => match &evaluated_args[1] {
                        Value::Lambda { params, body, captured } => {
                            let mut acc = evaluated_args[0].clone();
                            for item in items {
                                acc = self.invoke_lambda(params, body, captured, vec![acc, item])?;
                            }
                            Ok(acc)
                        }
                        _ => Err("طريقة اختزل تتطلب قيمة ابتدائية ودالة ثنائية".into()),
                    },
                    _ => Err(format!("الطريقة '{}' غير مدعومة للقوائم", method_name)),
                }
            }
            Value::Map(pairs) => {
                match method_name {
                    "طول" | "حجم" if evaluated_args.is_empty() => Ok(Value::Number(pairs.len() as f64)),
                    "مفاتيح" if evaluated_args.is_empty() => {
                        Ok(Value::List(pairs.iter().map(|(k, _)| Value::Text(k.clone())).collect()))
                    }
                    "قيم" if evaluated_args.is_empty() => {
                        Ok(Value::List(pairs.iter().map(|(_, v)| v.clone()).collect()))
                    }
                    _ => Err(format!("الطريقة '{}' غير مدعومة للقواميس", method_name)),
                }
            }
            _ => Err(format!("لا يمكن استدعاء الطريقة '{}' على هذا النوع", method_name)),
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
                    (Value::List(mut a), Value::List(b)) if a.len() + b.len() <= max_list_size() => {
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
        if self.classes.contains_key(name) {
            let instance = Value::Instance {
                class_name: name.to_string(),
                fields: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            };
            if let Some((owner_class, ctor)) = self.find_class_constructor(name)? {
                if args.len() != ctor.params.len() {
                    return Err(format!("دالة بناء الصنف '{}' تتوقع {} معاملات، استلمت {}",
                        name, ctor.params.len(), args.len()));
                }
                let mut evaluated_args = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated_args.push(self.eval(arg)?);
                }
                let span = self.current_span;
                self.call_stack.push((format!("{}.بناء", owner_class), span));
                let res = self.invoke_method(instance.clone(), &owner_class, &ctor, evaluated_args);
                if res.is_ok() {
                    self.call_stack.pop();
                }
                res?;
            } else if !args.is_empty() {
                return Err(format!("الصنف '{}' ليس له دالة بناء تقبل معاملات", name));
            }
            return Ok(instance);
        }
        if let Some(Value::Lambda { params, body, captured }) = self.scopes.iter().rev().find_map(|s| s.get(name)).cloned() {
            let mut evaluated_args = Vec::with_capacity(args.len());
            for arg in args {
                evaluated_args.push(self.eval(arg)?);
            }
            let span = self.current_span;
            self.call_stack.push((name.to_string(), span));
            let res = self.invoke_lambda(&params, &body, &captured, evaluated_args);
            if res.is_ok() {
                self.call_stack.pop();
            }
            return res;
        }
        if !self.functions.contains_key(name) && is_builtin(name) {
            let mut values = Vec::with_capacity(args.len());
            for arg in args { values.push(self.eval(arg)?); }
            return call_builtin(name, values);
        }
        let function = match self.functions.get(name).cloned() {
            Some(f) => f,
            None => {
                let mut func_names: Vec<&str> = self.functions.keys().map(|s| s.as_str()).collect();
                for c in self.classes.keys() {
                    func_names.push(c.as_str());
                }
                return Err(crate::diagnostics::suggest_function_error(name, &func_names));
            }
        };
        let mut evaluated_args = Vec::with_capacity(args.len());
        for arg in args {
            evaluated_args.push(self.eval(arg)?);
        }
        let span = self.current_span;
        self.call_stack.push((name.to_string(), span));
        let res = self.invoke_lambda(&function.params, &function.body, &[], evaluated_args);
        if res.is_ok() {
            self.call_stack.pop();
        }
        res
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
        "ادخل" | "ملف_موجود" | "ألحق_ملف" | "الحق_ملف" | "الآن" | "الان" | "عدد_صحيح" |
        "عشوائي" | "عدد_عشوائي" | "عنصر_عشوائي" |
        "بيئة" | "بيئه" | "وسائط" |
        "تحليل_json" | "ترميز_json" | "تنسيق" |
        "مجلد_موجود" | "أنشئ_مجلد" | "انشئ_مجلد" | "قائمة_ملفات" | "احذف_ملف" | "مسار_حالي" |
        "اطلب_ويب" | "جلب_ويب" |
        "وضع_المنفذ" | "اكتب_رقمي" | "اقرأ_رقمي" | "اكتب_تماثلي" | "اقرأ_تماثلي" |
        "انتظر_ملي" | "تأخير" | "انتظر_ميكرو" | "وقت_التشغيل" | "نبضة_صوت" |
        "أوقف_صوت" | "تسلسلي_ابدأ" | "تسلسلي_اطبع" | "تسلسلي_متاح" |
        "تسلسلي_اقرأ" | "خريطة_قيمة" | "قيد_قيمة" | "ترجم_أردوينو" | "ترجم_اردوينو" |
        "أيقونة" | "ايقونة" | "قائمة_الأيقونات" | "قائمة_الايقونات" | "نسق_تصميم" |
        "عنصر_بطاقة" | "عنصر_زر" | "عنصر_إحصائية" | "عنصر_احصائية" |
        "عنصر_مفتاح" | "عنصر_تنبيه" | "عنصر_شارة" | "عنصر_حقل" | "عنصر_شريط_تقدم" |
        "بناء_تطبيق_هاتف" | "تصدير_تطبيق_هاتف" | "بناء_صفحة_ويب")
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

static RNG_STATE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn next_random_u64() -> u64 {
    use std::sync::atomic::Ordering;
    let mut current = RNG_STATE.load(Ordering::Relaxed);
    if current == 0 {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x123456789abcdef0);
        current = nanos ^ 0x9e3779b97f4a7c15;
    }
    let z = current.wrapping_add(0x9e3779b97f4a7c15);
    RNG_STATE.store(z, Ordering::Relaxed);
    let mut v = z;
    v = (v ^ (v >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    v = (v ^ (v >> 27)).wrapping_mul(0x94d049bb133111eb);
    v ^ (v >> 31)
}

fn random_f64() -> f64 {
    (next_random_u64() >> 11) as f64 / ((1u64 << 53) as f64)
}

fn checked_int(value: &Value, fn_name: &str) -> Result<i64, String> {
    match value {
        Value::Number(n) if n.is_finite() && n.fract() == 0.0
            && n.abs() <= 9_007_199_254_740_991.0 => Ok(*n as i64),
        _ => Err(format!("الدالة '{}' تتطلب أعدادًا صحيحة", fn_name)),
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
                Value::Instance { fields, .. } => Ok(Value::Number(fields.borrow().len() as f64)),
                _ => Err("طول يقبل قائمة أو نصًا أو قاموسًا أو كائنًا".into()),
            }
        }
        "نوع" => {
            require_args(name, &args, 1)?;
            let kind = match &args[0] {
                Value::Number(_) => "رقم".to_string(),
                Value::Text(_) => "نص".to_string(),
                Value::Bool(_) => "منطقي".to_string(),
                Value::Null => "لاشيء".to_string(),
                Value::List(_) => "قائمة".to_string(),
                Value::Map(_) => "قاموس".to_string(),
                Value::Instance { class_name, .. } => class_name.clone(),
                Value::Lambda { .. } => "دالة".to_string(),
            };
            Ok(Value::Text(kind))
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
            if items.len() >= max_list_size() { return Err("القائمة تجاوزت الحد الأقصى".into()); }
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
                    if items.len() > max_list_size() { return Err("القائمة تجاوزت الحد الأقصى".into()); }
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
            if !length.is_finite() || length > max_list_size() as f64 {
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
        "عشوائي" => {
            require_args(name, &args, 0)?;
            Ok(Value::Number(random_f64()))
        }
        "عدد_عشوائي" => {
            require_args(name, &args, 2)?;
            let min = checked_int(&args[0], "عدد_عشوائي")?;
            let max = checked_int(&args[1], "عدد_عشوائي")?;
            if min > max {
                return Err("الحد الأدنى يجب أن يكون أصغر من أو يساوي الحد الأقصى في عدد_عشوائي".into());
            }
            let range = (max - min + 1) as u64;
            let val = min + (next_random_u64() % range) as i64;
            finite_number(val as f64)
        }
        "عنصر_عشوائي" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::List(items) => {
                    if items.is_empty() {
                        return Err("لا يمكن اختيار عنصر عشوائي من قائمة فارغة".into());
                    }
                    let idx = (next_random_u64() as usize) % items.len();
                    Ok(items[idx].clone())
                }
                _ => Err("عنصر_عشوائي تتطلب قائمة".into()),
            }
        }
        "بيئة" | "بيئه" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(var_name) => match std::env::var(var_name) {
                    Ok(val) => Ok(Value::Text(val)),
                    Err(_) => Ok(Value::Null),
                },
                _ => Err("بيئة تتطلب اسم متغير نصي".into()),
            }
        }
        "وسائط" => {
            require_args(name, &args, 0)?;
            let env_args: Vec<Value> = std::env::args().map(Value::Text).collect();
            Ok(Value::List(env_args))
        }
        "تحليل_json" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(raw) => crate::json::parse_json(raw),
                _ => Err("تحليل_json تتطلب نصًا بتنسيق JSON".into()),
            }
        }
        "ترميز_json" => {
            require_args(name, &args, 1)?;
            let json_str = crate::json::to_json_string(&args[0])?;
            Ok(Value::Text(json_str))
        }
        "تنسيق" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Text(template), Value::List(values)) => {
                    let mut result = String::new();
                    let mut val_idx = 0;
                    let mut chars = template.chars().peekable();
                    while let Some(c) = chars.next() {
                        if c == '{' && chars.peek() == Some(&'}') {
                            chars.next(); // consume '}'
                            if val_idx < values.len() {
                                match &values[val_idx] {
                                    Value::Text(t) => result.push_str(t),
                                    other => result.push_str(&other.to_string()),
                                }
                                val_idx += 1;
                            } else {
                                return Err("عدد القيم في القائمة أقل من علامات التنسيق {}".into());
                            }
                        } else {
                            result.push(c);
                        }
                    }
                    Ok(Value::Text(result))
                }
                _ => Err("تنسيق تتطلب نص قالب وقائمة بالقيم".into()),
            }
        }
        "مجلد_موجود" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(path) => Ok(Value::Bool(std::path::Path::new(path).is_dir())),
                _ => Err("مجلد_موجود تتطلب مساراً نصياً".into()),
            }
        }
        "أنشئ_مجلد" | "انشئ_مجلد" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(path) => {
                    std::fs::create_dir_all(path)
                        .map_err(|e| format!("تعذر إنشاء المجلد '{}': {}", path, e))?;
                    Ok(Value::Bool(true))
                }
                _ => Err("أنشئ_مجلد تتطلب مساراً نصياً".into()),
            }
        }
        "قائمة_ملفات" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(path) => {
                    let entries = std::fs::read_dir(path)
                        .map_err(|e| format!("تعذر قراءة محتويات المجلد '{}': {}", path, e))?;
                    let mut names = Vec::new();
                    for entry in entries {
                        let entry = entry.map_err(|e| format!("خطأ أثناء قراءة مدخل المجلد: {}", e))?;
                        if let Some(name_str) = entry.file_name().to_str() {
                            names.push(name_str.to_string());
                        }
                    }
                    names.sort();
                    Ok(Value::List(names.into_iter().map(Value::Text).collect()))
                }
                _ => Err("قائمة_ملفات تتطلب مساراً نصياً".into()),
            }
        }
        "احذف_ملف" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(path) => {
                    std::fs::remove_file(path)
                        .map_err(|e| format!("تعذر حذف الملف '{}': {}", path, e))?;
                    Ok(Value::Bool(true))
                }
                _ => Err("احذف_ملف تتطلب مساراً نصياً".into()),
            }
        }
        "مسار_حالي" => {
            require_args(name, &args, 0)?;
            let dir = std::env::current_dir()
                .map_err(|e| format!("تعذر معرفة المسار الحالي: {}", e))?;
            Ok(Value::Text(dir.to_string_lossy().into_owned()))
        }
        "اطلب_ويب" | "جلب_ويب" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(url) => http_get(url),
                _ => Err("اطلب_ويب تتطلب رابطاً نصياً (URL)".into()),
            }
        }
        // ==================== دوال الأردوينو والمتحكمات الدقيقة ====================
        "وضع_المنفذ" | "اكتب_رقمي" | "اقرأ_رقمي" | "اكتب_تماثلي" | "اقرأ_تماثلي"
        | "انتظر_ملي" | "تأخير" | "انتظر_ميكرو" | "وقت_التشغيل" | "نبضة_صوت"
        | "أوقف_صوت" | "تسلسلي_ابدأ" | "تسلسلي_اطبع" | "تسلسلي_متاح"
        | "تسلسلي_اقرأ" | "خريطة_قيمة" | "قيد_قيمة" | "ترجم_أردوينو" | "ترجم_اردوينو" => {
            eval_arduino_builtin(name, args)
        }
        // ==================== دوال الأيقونات والتصميمات والهواتف والويب ====================
        "أيقونة" | "ايقونة" | "قائمة_الأيقونات" | "قائمة_الايقونات" | "نسق_تصميم"
        | "عنصر_بطاقة" | "عنصر_زر" | "عنصر_إحصائية" | "عنصر_احصائية"
        | "عنصر_مفتاح" | "عنصر_تنبيه" | "عنصر_شارة" | "عنصر_حقل" | "عنصر_شريط_تقدم"
        | "بناء_تطبيق_هاتف" | "تصدير_تطبيق_هاتف" | "بناء_صفحة_ويب" => {
            eval_ui_builtin(name, args)
        }
        _ => {
            let empty: Vec<&str> = Vec::new();
            Err(crate::diagnostics::suggest_function_error(name, &empty))
        }
    }
}

#[inline(never)]
fn eval_arduino_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "وضع_المنفذ" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Number(_pin), Value::Text(_mode)) => Ok(Value::Bool(true)),
                _ => Err("وضع_المنفذ تتطلب رقم المنفذ ونص الوضع ('مخرج' أو 'مدخل')".into()),
            }
        }
        "اكتب_رقمي" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Number(_pin), Value::Number(v)) => Ok(Value::Number(if *v != 0.0 { 1.0 } else { 0.0 })),
                (Value::Number(_pin), Value::Bool(b)) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
                (Value::Number(_pin), Value::Text(s)) => Ok(Value::Number(if s == "عالي" || s == "HIGH" { 1.0 } else { 0.0 })),
                _ => Err("اكتب_رقمي تتطلب رقم المنفذ وحالة المنفذ (1 أو 0 أو 'عالي' أو 'منخفض')".into()),
            }
        }
        "اقرأ_رقمي" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(_) => Ok(Value::Number(0.0)),
                _ => Err("اقرأ_رقمي تتطلب رقم المنفذ".into()),
            }
        }
        "اكتب_تماثلي" => {
            require_args(name, &args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Number(_pin), Value::Number(val)) => Ok(Value::Number(val.clamp(0.0, 255.0))),
                _ => Err("اكتب_تماثلي تتطلب رقم المنفذ وقيمة رقمية (0..255)".into()),
            }
        }
        "اقرأ_تماثلي" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(_) => Ok(Value::Number(512.0)),
                _ => Err("اقرأ_تماثلي تتطلب رقم المنفذ".into()),
            }
        }
        "انتظر_ملي" | "تأخير" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(ms) if *ms >= 0.0 => Ok(Value::Number(*ms)),
                _ => Err("انتظر_ملي تتطلب عدداً موجباً بالملي ثانية".into()),
            }
        }
        "انتظر_ميكرو" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Number(us) if *us >= 0.0 => Ok(Value::Number(*us)),
                _ => Err("انتظر_ميكرو تتطلب عدداً موجباً بالميكرو ثانية".into()),
            }
        }
        "وقت_التشغيل" => {
            require_args(name, &args, 0)?;
            Ok(Value::Number(1000.0))
        }
        "نبضة_صوت" => {
            if args.len() != 2 && args.len() != 3 {
                return Err("نبضة_صوت تتطلب منفذاً وتردداً (ومدة اختيارية)".into());
            }
            Ok(Value::Bool(true))
        }
        "أوقف_صوت" => {
            require_args(name, &args, 1)?;
            Ok(Value::Bool(true))
        }
        "تسلسلي_ابدأ" => {
            require_args(name, &args, 1)?;
            Ok(Value::Bool(true))
        }
        "تسلسلي_اطبع" => {
            require_args(name, &args, 1)?;
            Ok(Value::Text(args[0].to_string()))
        }
        "تسلسلي_متاح" => {
            require_args(name, &args, 0)?;
            Ok(Value::Bool(false))
        }
        "تسلسلي_اقرأ" => {
            require_args(name, &args, 0)?;
            Ok(Value::Text(String::new()))
        }
        "خريطة_قيمة" => {
            require_args(name, &args, 5)?;
            match (&args[0], &args[1], &args[2], &args[3], &args[4]) {
                (
                    Value::Number(x),
                    Value::Number(in_min),
                    Value::Number(in_max),
                    Value::Number(out_min),
                    Value::Number(out_max),
                ) => {
                    if (in_max - in_min).abs() < f64::EPSILON {
                        return Err("نطاق الإدخال في خريطة_قيمة لا يمكن أن يكون صفراً".into());
                    }
                    let mapped = (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
                    Ok(Value::Number(mapped))
                }
                _ => Err("خريطة_قيمة تتطلب 5 أرقام: (القيمة، أدنى_إدخال، أقصى_إدخال، أدنى_إخراج، أقصى_إخراج)".into()),
            }
        }
        "قيد_قيمة" => {
            require_args(name, &args, 3)?;
            match (&args[0], &args[1], &args[2]) {
                (Value::Number(x), Value::Number(low), Value::Number(high)) => {
                    Ok(Value::Number(x.clamp(*low, *high)))
                }
                _ => Err("قيد_قيمة تتطلب 3 أرقام: (القيمة، الحد_الأدنى، الحد_الأعلى)".into()),
            }
        }
        "ترجم_أردوينو" | "ترجم_اردوينو" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(src) => {
                    let ino = crate::arduino::transpile_arduino(src)?;
                    Ok(Value::Text(ino))
                }
                _ => Err("ترجم_أردوينو تتطلب نص شيفرة لسان".into()),
            }
        }
        _ => unreachable!(),
    }
}

#[inline(never)]
fn eval_ui_builtin(name: &str, args: Vec<Value>) -> Result<Value, String> {
    match name {
        "أيقونة" | "ايقونة" => {
            if args.is_empty() || args.len() > 3 {
                return Err("أيقونة تتطلب اسم الأيقونة (وحجمًا ولونًا اختياريين)".into());
            }
            let icon_name = match &args[0] {
                Value::Text(s) => s.as_str(),
                _ => return Err("اسم الأيقونة يجب أن يكون نصاً".into()),
            };
            let size = if args.len() >= 2 {
                match &args[1] {
                    Value::Number(n) if *n > 0.0 => *n as u32,
                    _ => 24,
                }
            } else {
                24
            };
            let color = if args.len() == 3 {
                match &args[2] {
                    Value::Text(c) => c.as_str(),
                    _ => "currentColor",
                }
            } else {
                "currentColor"
            };
            Ok(Value::Text(crate::ui::get_icon_svg(icon_name, size, color)))
        }
        "قائمة_الأيقونات" | "قائمة_الايقونات" => {
            require_args(name, &args, 0)?;
            let list = crate::ui::ICON_NAMES
                .iter()
                .map(|s| Value::Text((*s).to_string()))
                .collect();
            Ok(Value::List(list))
        }
        "نسق_تصميم" => {
            require_args(name, &args, 1)?;
            match &args[0] {
                Value::Text(theme) => Ok(Value::Text(crate::ui::get_theme_css(theme).to_string())),
                _ => Err("نسق_تصميم تتطلب اسم النسق نصاً (مثل 'داكن'، 'فاتح'، 'زمردي'، 'ملكي')".into()),
            }
        }
        "عنصر_بطاقة" => {
            if args.len() < 2 || args.len() > 3 {
                return Err("عنصر_بطاقة تتطلب (عنوان، وصف، [أيقونة])".into());
            }
            let title = args[0].to_string();
            let desc = args[1].to_string();
            let icon_name = if args.len() == 3 { args[2].to_string() } else { "نجمة".into() };
            let svg = crate::ui::get_icon_svg(&icon_name, 22, "currentColor");
            let html = format!(
                r#"<div class="lsn-card"><div class="lsn-card-icon">{svg}</div><div class="lsn-card-body"><h3>{title}</h3><p>{desc}</p></div></div>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_زر" => {
            if args.is_empty() || args.len() > 3 {
                return Err("عنصر_زر تتطلب (نص_الزر، [أيقونة]، [رسالة])".into());
            }
            let label = args[0].to_string();
            let icon_name = if args.len() >= 2 { args[1].to_string() } else { "تشغيل".into() };
            let msg = if args.len() == 3 { args[2].to_string() } else { label.clone() };
            let svg = crate::ui::get_icon_svg(&icon_name, 18, "#ffffff");
            let html = format!(
                r#"<button class="lsn-btn" onclick="if(window.showToast)showToast('{msg}')">{svg}<span>{label}</span></button>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_إحصائية" | "عنصر_احصائية" => {
            if args.len() < 2 || args.len() > 3 {
                return Err("عنصر_إحصائية تتطلب (عنوان، قيمة، [أيقونة])".into());
            }
            let title = args[0].to_string();
            let val = args[1].to_string();
            let icon_name = if args.len() == 3 { args[2].to_string() } else { "حرارة".into() };
            let svg = crate::ui::get_icon_svg(&icon_name, 18, "currentColor");
            let html = format!(
                r#"<div class="lsn-stat"><div class="lsn-stat-top"><span>{title}</span>{svg}</div><div class="lsn-stat-val">{val}</div></div>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_مفتاح" => {
            if args.len() < 2 || args.len() > 3 {
                return Err("عنصر_مفتاح تتطلب (عنوان، أيقونة، [مفعّل])".into());
            }
            let title = args[0].to_string();
            let icon_name = args[1].to_string();
            let is_on = if args.len() == 3 {
                matches!(args[2], Value::Bool(true) | Value::Number(1.0))
            } else {
                true
            };
            let on_cls = if is_on { "on" } else { "" };
            let svg = crate::ui::get_icon_svg(&icon_name, 20, "var(--primary)");
            let html = format!(
                r#"<div class="lsn-switch-row"><div class="lsn-switch-label">{svg}<span>{title}</span></div><button class="lsn-toggle {on_cls}" onclick="toggleSwitch(this, '{title}')"></button></div>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_تنبيه" => {
            require_args(name, &args, 2)?;
            let text = args[0].to_string();
            let kind = args[1].to_string();
            let svg = crate::ui::get_icon_svg(&kind, 20, "currentColor");
            let html = format!(
                r#"<div class="lsn-card" style="border-color:var(--primary)"><div class="lsn-card-icon">{svg}</div><div class="lsn-card-body"><p>{text}</p></div></div>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_شارة" => {
            require_args(name, &args, 1)?;
            let text = args[0].to_string();
            let html = format!(
                r#"<span style="display:inline-block;padding:4px 10px;border-radius:999px;background:var(--surface-2);color:var(--primary);font-size:12px;font-weight:700">{text}</span>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_حقل" => {
            require_args(name, &args, 2)?;
            let label = args[0].to_string();
            let placeholder = args[1].to_string();
            let html = format!(
                r#"<div style="display:flex;flex-direction:column;gap:6px"><label style="font-size:13px;color:var(--muted)">{label}</label><input placeholder="{placeholder}" style="padding:11px 14px;border-radius:12px;border:1px solid var(--border);background:var(--surface);color:var(--text)"/></div>"#
            );
            Ok(Value::Text(html))
        }
        "عنصر_شريط_تقدم" => {
            require_args(name, &args, 2)?;
            let label = args[0].to_string();
            let pct = match &args[1] {
                Value::Number(n) => n.clamp(0.0, 100.0),
                _ => 50.0,
            };
            let html = format!(
                r#"<div class="lsn-card" style="flex-direction:column;align-items:stretch;gap:8px"><div style="display:flex;justify-content:space-between;font-size:13px"><span>{label}</span><span>{pct}%</span></div><div style="height:8px;background:var(--surface-2);border-radius:4px;overflow:hidden"><div style="width:{pct}%;height:100%;background:var(--primary)"></div></div></div>"#
            );
            Ok(Value::Text(html))
        }
        "بناء_تطبيق_هاتف" => {
            if args.len() < 3 || args.len() > 4 {
                return Err("بناء_تطبيق_هاتف تتطلب (اسم_التطبيق، النسق، الأيقونة، [قائمة_العناصر])".into());
            }
            let app_name = args[0].to_string();
            let theme = args[1].to_string();
            let icon = args[2].to_string();
            let mut items = Vec::new();
            if args.len() == 4 {
                if let Value::List(list) = &args[3] {
                    for item in list {
                        items.push(item.to_string());
                    }
                }
            }
            let html = crate::ui::build_mobile_app_html(&app_name, &theme, &icon, &items, &[]);
            Ok(Value::Text(html))
        }
        "تصدير_تطبيق_هاتف" => {
            require_args(name, &args, 3)?;
            let app_name = args[0].to_string();
            let html = args[1].to_string();
            let out_dir = args[2].to_string();
            let path = crate::ui::export_mobile_pwa(&app_name, &html, std::path::Path::new(&out_dir))?;
            Ok(Value::Text(path.to_string_lossy().into_owned()))
        }
        "بناء_صفحة_ويب" => {
            require_args(name, &args, 3)?;
            let title = args[0].to_string();
            let theme = args[1].to_string();
            let mut sections = Vec::new();
            if let Value::List(list) = &args[2] {
                for item in list {
                    sections.push(item.to_string());
                }
            }
            Ok(Value::Text(crate::ui::build_web_page_html(&title, &theme, &sections)))
        }
        _ => unreachable!(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn http_get(url: &str) -> Result<Value, String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let stripped = if let Some(s) = url.strip_prefix("http://") {
        s
    } else if url.starts_with("https://") {
        return Err("اطلب_ويب المدمجة بدون مكتبات خارجية تدعم بروتوكول http:// المباشر؛ لبروتوكول https استخدم خادم وكيل محلي".into());
    } else {
        url
    };

    let (host_port, path) = match stripped.find('/') {
        Some(idx) => (&stripped[..idx], &stripped[idx..]),
        None => (stripped, "/"),
    };

    let addr = if host_port.contains(':') {
        host_port.to_string()
    } else {
        format!("{}:80", host_port)
    };

    let host_only = host_port.split(':').next().unwrap_or(host_port);

    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| format!("تعذر الاتصال بالخادم '{}': {}", addr, e))?;
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(10)));

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: Lisan/1.0\r\nConnection: close\r\n\r\n",
        path, host_only
    );
    stream.write_all(request.as_bytes())
        .map_err(|e| format!("فشل إرسال طلب الشبكة: {}", e))?;

    let mut response = Vec::new();
    stream.read_to_end(&mut response)
        .map_err(|e| format!("فشل قراءة استجابة الشبكة: {}", e))?;

    let text = String::from_utf8_lossy(&response);
    if let Some(idx) = text.find("\r\n\r\n") {
        Ok(Value::Text(text[idx + 4..].to_string()))
    } else if let Some(idx) = text.find("\n\n") {
        Ok(Value::Text(text[idx + 2..].to_string()))
    } else {
        Ok(Value::Text(text.into_owned()))
    }
}

#[cfg(target_arch = "wasm32")]
fn http_get(_url: &str) -> Result<Value, String> {
    Err("اطلب_ويب عبر المقابس المباشرة غير متاحة داخل WebAssembly".into())
}
