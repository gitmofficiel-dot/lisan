use crate::ast::{Expr, Op, Stmt};
use crate::bytecode::{Chunk, OpCode};
use crate::runtime::Value;
use std::collections::HashMap;

/// الآلة الافتراضية المكدسية لتشغيل بايت كود لسان (Lisan Stack VM)
pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    output: Vec<String>,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            output: Vec::new(),
        }
    }
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    /// تشغيل كود مصدري عبر محرك البايت كود VM
    pub fn run_source(&mut self, source: &str) -> Result<Vec<String>, String> {
        let tokens = crate::lexer::tokenize(source)?;
        let statements = crate::parser::parse(tokens)?;
        let chunk = Compiler::compile(&statements)?;
        self.interpret(&chunk)
    }

    /// تنفيذ كتلة بايت كود مجمعة
    pub fn interpret(&mut self, chunk: &Chunk) -> Result<Vec<String>, String> {
        self.output.clear();
        let mut ip = 0;
        let len = chunk.instructions.len();

        while ip < len {
            let op = &chunk.instructions[ip];
            match op {
                OpCode::Constant(idx) => {
                    self.stack.push(chunk.constants[*idx].clone());
                }
                OpCode::Null => self.stack.push(Value::Null),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),

                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    match (a, b) {
                        (Value::Number(x), Value::Number(y)) => self.stack.push(Value::Number(x + y)),
                        (Value::Text(x), Value::Text(y)) => self.stack.push(Value::Text(x + &y)),
                        (Value::Text(x), y) => self.stack.push(Value::Text(x + &y.to_string())),
                        (x, Value::Text(y)) => self.stack.push(Value::Text(x.to_string() + &y)),
                        _ => return Err("عملية الجمع تتطلب أرقاماً أو نصوصاً".into()),
                    }
                }
                OpCode::Subtract => {
                    let b = self.pop_number("الطرح")?;
                    let a = self.pop_number("الطرح")?;
                    self.stack.push(Value::Number(a - b));
                }
                OpCode::Multiply => {
                    let b = self.pop_number("الضرب")?;
                    let a = self.pop_number("الضرب")?;
                    self.stack.push(Value::Number(a * b));
                }
                OpCode::Divide => {
                    let b = self.pop_number("القسمة")?;
                    let a = self.pop_number("القسمة")?;
                    if b == 0.0 {
                        return Err("لا يمكن القسمة على الصفر!".into());
                    }
                    self.stack.push(Value::Number(a / b));
                }
                OpCode::Modulo => {
                    let b = self.pop_number("باقي القسمة")?;
                    let a = self.pop_number("باقي القسمة")?;
                    if b == 0.0 {
                        return Err("لا يمكن حساب باقي القسمة على الصفر!".into());
                    }
                    self.stack.push(Value::Number(a % b));
                }
                OpCode::Power => {
                    let b = self.pop_number("الأس")?;
                    let a = self.pop_number("الأس")?;
                    self.stack.push(Value::Number(a.powf(b)));
                }
                OpCode::Negate => {
                    let a = self.pop_number("السالب")?;
                    self.stack.push(Value::Number(-a));
                }
                OpCode::Not => {
                    let a = self.pop()?;
                    match a {
                        Value::Bool(b) => self.stack.push(Value::Bool(!b)),
                        _ => return Err("عملية 'ليس' تتطلب قيمة منطقية".into()),
                    }
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(Value::Bool(a != b));
                }
                OpCode::Greater => {
                    let b = self.pop_number("المقارنة أكبر من")?;
                    let a = self.pop_number("المقارنة أكبر من")?;
                    self.stack.push(Value::Bool(a > b));
                }
                OpCode::GreaterEqual => {
                    let b = self.pop_number("المقارنة أكبر من أو يساوي")?;
                    let a = self.pop_number("المقارنة أكبر من أو يساوي")?;
                    self.stack.push(Value::Bool(a >= b));
                }
                OpCode::Less => {
                    let b = self.pop_number("المقارنة أصغر من")?;
                    let a = self.pop_number("المقارنة أصغر من")?;
                    self.stack.push(Value::Bool(a < b));
                }
                OpCode::LessEqual => {
                    let b = self.pop_number("المقارنة أصغر من أو يساوي")?;
                    let a = self.pop_number("المقارنة أصغر من أو يساوي")?;
                    self.stack.push(Value::Bool(a <= b));
                }

                OpCode::DefineGlobal(name) => {
                    let val = self.pop()?;
                    self.globals.insert(name.clone(), val);
                }
                OpCode::GetGlobal(name) => {
                    if let Some(val) = self.globals.get(name) {
                        self.stack.push(val.clone());
                    } else {
                        let available: Vec<&str> = self.globals.keys().map(|s| s.as_str()).collect();
                        return Err(crate::diagnostics::suggest_variable_error(name, &available));
                    }
                }
                OpCode::SetGlobal(name) => {
                    let val = self.peek()?.clone();
                    if self.globals.contains_key(name) {
                        self.globals.insert(name.clone(), val);
                    } else {
                        return Err(format!("لا يمكن الإسناد لمتغير غير معرّف: '{}'", name));
                    }
                }

                OpCode::Jump(dest) => {
                    ip = *dest;
                    continue;
                }
                OpCode::JumpIfFalse(dest) => {
                    let cond = self.pop()?;
                    match cond {
                        Value::Bool(false) => {
                            ip = *dest;
                            continue;
                        }
                        Value::Bool(true) => {}
                        _ => return Err("الشرط في جملة التحكم يتطلب قيمة منطقية".into()),
                    }
                }
                OpCode::IterInit => {
                    let iter_val = self.pop()?;
                    match iter_val {
                        Value::List(items) => {
                            self.stack.push(Value::List(items));
                            self.stack.push(Value::Number(0.0));
                        }
                        _ => return Err("حلقة لكل تتطلب قائمة عناصر".into()),
                    }
                }
                OpCode::IterNext(var_name, end_dest) => {
                    let idx_val = self.stack.pop().ok_or("فقدان فهرس التكرار")?;
                    let list_val = self.stack.last().ok_or("فقدان قائمة التكرار")?;
                    match (list_val, &idx_val) {
                        (Value::List(items), Value::Number(n)) => {
                            let idx = *n as usize;
                            if idx < items.len() {
                                self.globals.insert(var_name.clone(), items[idx].clone());
                                self.stack.push(Value::Number((idx + 1) as f64));
                            } else {
                                self.stack.push(idx_val);
                                ip = *end_dest;
                                continue;
                            }
                        }
                        _ => return Err("بيانات تكرار غير صحيحة".into()),
                    }
                }
                OpCode::IterPop => {
                    let _ = self.stack.pop();
                    let _ = self.stack.pop();
                }

                OpCode::BuildList(count) => {
                    let mut items = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        items.push(self.pop()?);
                    }
                    items.reverse();
                    self.stack.push(Value::List(items));
                }
                OpCode::BuildMap(count) => {
                    let mut pairs = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        let val = self.pop()?;
                        let key = match self.pop()? {
                            Value::Text(k) => k,
                            _ => return Err("مفتاح القاموس يجب أن يكون نصاً".into()),
                        };
                        pairs.push((key, val));
                    }
                    pairs.reverse();
                    self.stack.push(Value::Map(pairs));
                }
                OpCode::GetIndex => {
                    let index = self.pop()?;
                    let target = self.pop()?;
                    match target {
                        Value::List(items) => match index {
                            Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => {
                                let idx = n as usize;
                                if let Some(item) = items.get(idx) {
                                    self.stack.push(item.clone());
                                } else {
                                    return Err(format!("الفهرس {} خارج حدود القائمة", idx));
                                }
                            }
                            _ => return Err("فهرس القائمة يجب أن يكون عدداً صحيحاً موجباً".into()),
                        },
                        Value::Map(pairs) => match index {
                            Value::Text(key) => {
                                if let Some((_, val)) = pairs.iter().find(|(k, _)| k == &key) {
                                    self.stack.push(val.clone());
                                } else {
                                    let available: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
                                    return Err(crate::diagnostics::suggest_dict_key_error(&key, &available));
                                }
                            }
                            _ => return Err("مفتاح القاموس يجب أن يكون نصاً".into()),
                        },
                        _ => return Err("الفهرسة مدعومة فقط للقوائم والقواميس".into()),
                    }
                }
                OpCode::SetIndex(var_name) => {
                    let val = self.pop()?;
                    let index = self.pop()?;
                    if let Some(target) = self.globals.get_mut(var_name) {
                        match target {
                            Value::List(items) => match index {
                                Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => {
                                    let idx = n as usize;
                                    if idx < items.len() {
                                        items[idx] = val.clone();
                                    } else {
                                        return Err(format!("الفهرس {} خارج حدود القائمة", idx));
                                    }
                                }
                                _ => return Err("فهرس القائمة يجب أن يكون عدداً صحيحاً".into()),
                            },
                            Value::Map(pairs) => match index {
                                Value::Text(key) => {
                                    if let Some((_, v)) = pairs.iter_mut().find(|(k, _)| k == &key) {
                                        *v = val.clone();
                                    } else {
                                        pairs.push((key, val.clone()));
                                    }
                                }
                                _ => return Err("مفتاح القاموس يجب أن يكون نصاً".into()),
                            },
                            _ => return Err("الفهرسة مدعومة للقوائم والقواميس فقط".into()),
                        }
                    } else {
                        return Err(format!("متغير غير معرّف: '{}'", var_name));
                    }
                }

                OpCode::CallBuiltin(name, arg_count) => {
                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();
                    let res = match name.as_str() {
                        "طول" if args.len() == 1 => match &args[0] {
                            Value::List(l) => Value::Number(l.len() as f64),
                            Value::Text(t) => Value::Number(t.chars().count() as f64),
                            Value::Map(m) => Value::Number(m.len() as f64),
                            Value::Instance { fields, .. } => Value::Number(fields.borrow().len() as f64),
                            _ => return Err("طول تتطلب قائمة أو نصاً أو قاموساً أو كائناً".into()),
                        },
                        "نوع" if args.len() == 1 => match &args[0] {
                            Value::Number(_) => Value::Text("رقم".into()),
                            Value::Text(_) => Value::Text("نص".into()),
                            Value::Bool(_) => Value::Text("منطقي".into()),
                            Value::List(_) => Value::Text("قائمة".into()),
                            Value::Map(_) => Value::Text("قاموس".into()),
                            Value::Null => Value::Text("لاشيء".into()),
                            Value::Instance { class_name, .. } => Value::Text(class_name.clone()),
                            Value::Lambda { .. } => Value::Text("دالة".into()),
                        },
                        "مجموع" if args.len() == 1 => match &args[0] {
                            Value::List(items) => {
                                let mut sum = 0.0;
                                for item in items {
                                    match item {
                                        Value::Number(n) => sum += *n,
                                        _ => return Err("مجموع تتطلب قائمة من الأرقام فقط".into()),
                                    }
                                }
                                Value::Number(sum)
                            }
                            _ => return Err("مجموع تتطلب قائمة".into()),
                        },
                        "متوسط" if args.len() == 1 => match &args[0] {
                            Value::List(items) if !items.is_empty() => {
                                let mut sum = 0.0;
                                for item in items {
                                    match item {
                                        Value::Number(n) => sum += *n,
                                        _ => return Err("متوسط تتطلب قائمة من الأرقام فقط".into()),
                                    }
                                }
                                Value::Number(sum / (items.len() as f64))
                            }
                            _ => return Err("متوسط تتطلب قائمة غير فارغة".into()),
                        },
                        "مفاتيح" if args.len() == 1 => match &args[0] {
                            Value::Map(m) => Value::List(m.iter().map(|(k, _)| Value::Text(k.clone())).collect()),
                            _ => return Err("مفاتيح تتطلب قاموساً".into()),
                        },
                        "قيم" if args.len() == 1 => match &args[0] {
                            Value::Map(m) => Value::List(m.iter().map(|(_, v)| v.clone()).collect()),
                            _ => return Err("قيم تتطلب قاموساً".into()),
                        },
                        "أعلى" | "اعلى" if args.len() == 2 => match (&args[0], &args[1]) {
                            (Value::Number(a), Value::Number(b)) => Value::Number(a.max(*b)),
                            _ => return Err("أعلى تتطلب رقمين".into()),
                        },
                        "أدنى" | "ادنى" if args.len() == 2 => match (&args[0], &args[1]) {
                            (Value::Number(a), Value::Number(b)) => Value::Number(a.min(*b)),
                            _ => return Err("أدنى تتطلب رقمين".into()),
                        },
                        "عكس" | "اعكس" if args.len() == 1 => match &args[0] {
                            Value::List(l) => {
                                let mut rev = l.clone();
                                rev.reverse();
                                Value::List(rev)
                            }
                            Value::Text(t) => Value::Text(t.chars().rev().collect()),
                            _ => return Err("عكس تتطلب قائمة أو نصاً".into()),
                        },
                        "فرز" if args.len() == 1 => match &args[0] {
                            Value::List(items) => {
                                let mut sorted = items.clone();
                                sorted.sort_by(|a, b| match (a, b) {
                                    (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                    (Value::Text(x), Value::Text(y)) => x.cmp(y),
                                    _ => std::cmp::Ordering::Equal,
                                });
                                Value::List(sorted)
                            }
                            _ => return Err("فرز تتطلب قائمة".into()),
                        },
                        "تحليل_json" if args.len() == 1 => match &args[0] {
                            Value::Text(raw) => crate::json::parse_json(raw)?,
                            _ => return Err("تحليل_json تتطلب نصاً".into()),
                        },
                        "ترميز_json" if args.len() == 1 => {
                            Value::Text(crate::json::to_json_string(&args[0])?)
                        }
                        other => {
                            let empty: Vec<&str> = Vec::new();
                            return Err(crate::diagnostics::suggest_function_error(other, &empty));
                        }
                    };
                    self.stack.push(res);
                }

                OpCode::Print => {
                    let val = self.pop()?;
                    self.output.push(val.to_string());
                }
                OpCode::Pop => {
                    let _ = self.pop()?;
                }
                OpCode::Return => {
                    break;
                }
            }
            ip += 1;
        }

        Ok(self.output.clone())
    }

    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "مكدس الآلة الافتراضية فارغ".to_string())
    }

    fn peek(&self) -> Result<&Value, String> {
        self.stack.last().ok_or_else(|| "مكدس الآلة الافتراضية فارغ".to_string())
    }

    fn pop_number(&mut self, op_name: &str) -> Result<f64, String> {
        match self.pop()? {
            Value::Number(n) => Ok(n),
            _ => Err(format!("عملية {} تتطلب قيمة عددية", op_name)),
        }
    }
}

/// مترجم يحول شجرة بناء التعليمات (AST) إلى بايت كود للآلة الافتراضية
pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn compile(statements: &[Stmt]) -> Result<Chunk, String> {
        let mut compiler = Self { chunk: Chunk::new() };
        for stmt in statements {
            compiler.compile_stmt(stmt)?;
        }
        compiler.chunk.emit(OpCode::Return, 0);
        Ok(compiler.chunk)
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Spanned(inner, _, _) => return self.compile_stmt(inner),
            Stmt::Let(name, expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::DefineGlobal(name.clone()), 0);
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::Print, 0);
            }
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::Pop, 0);
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.compile_stmt(s)?;
                }
            }
            Stmt::If(condition, yes_body, no_body) => {
                self.compile_expr(condition)?;
                let jump_false = self.chunk.emit(OpCode::JumpIfFalse(0), 0);

                for s in yes_body {
                    self.compile_stmt(s)?;
                }

                if let Some(no_stmts) = no_body {
                    let jump_end = self.chunk.emit(OpCode::Jump(0), 0);
                    let target_else = self.chunk.instructions.len();
                    self.chunk.patch_jump(jump_false, target_else);

                    for s in no_stmts {
                        self.compile_stmt(s)?;
                    }
                    let target_end = self.chunk.instructions.len();
                    self.chunk.patch_jump(jump_end, target_end);
                } else {
                    let target = self.chunk.instructions.len();
                    self.chunk.patch_jump(jump_false, target);
                }
            }
            Stmt::While(condition, body) => {
                let loop_start = self.chunk.instructions.len();
                self.compile_expr(condition)?;
                let jump_out = self.chunk.emit(OpCode::JumpIfFalse(0), 0);

                for s in body {
                    self.compile_stmt(s)?;
                }
                self.chunk.emit(OpCode::Jump(loop_start), 0);
                let loop_end = self.chunk.instructions.len();
                self.chunk.patch_jump(jump_out, loop_end);
            }
            Stmt::ForEach(name, iterable, body) => {
                self.compile_expr(iterable)?;
                self.chunk.emit(OpCode::IterInit, 0);
                let loop_start = self.chunk.instructions.len();
                let iter_next = self.chunk.emit(OpCode::IterNext(name.clone(), 0), 0);

                for s in body {
                    self.compile_stmt(s)?;
                }
                self.chunk.emit(OpCode::Jump(loop_start), 0);
                let loop_end = self.chunk.instructions.len();
                self.chunk.patch_jump(iter_next, loop_end);
                self.chunk.emit(OpCode::IterPop, 0);
            }
            _ => {
                // لبقية الجمل المتقدمة، تظل شجرة التفسير أو تضاف لاحقاً
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Spanned(inner, _, _) => return self.compile_expr(inner),
            Expr::Number(n) => {
                let idx = self.chunk.add_constant(Value::Number(*n));
                self.chunk.emit(OpCode::Constant(idx), 0);
            }
            Expr::Text(t) => {
                let idx = self.chunk.add_constant(Value::Text(t.clone()));
                self.chunk.emit(OpCode::Constant(idx), 0);
            }
            Expr::Bool(true) => { self.chunk.emit(OpCode::True, 0); }
            Expr::Bool(false) => { self.chunk.emit(OpCode::False, 0); }
            Expr::Null => { self.chunk.emit(OpCode::Null, 0); }
            Expr::Variable(name) => {
                self.chunk.emit(OpCode::GetGlobal(name.clone()), 0);
            }
            Expr::Assign(name, val) => {
                self.compile_expr(val)?;
                self.chunk.emit(OpCode::SetGlobal(name.clone()), 0);
            }
            Expr::Binary(left, op, right) => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    Op::Add => self.chunk.emit(OpCode::Add, 0),
                    Op::Subtract => self.chunk.emit(OpCode::Subtract, 0),
                    Op::Multiply => self.chunk.emit(OpCode::Multiply, 0),
                    Op::Divide => self.chunk.emit(OpCode::Divide, 0),
                    Op::Modulo => self.chunk.emit(OpCode::Modulo, 0),
                    Op::Power => self.chunk.emit(OpCode::Power, 0),
                    Op::Equal => self.chunk.emit(OpCode::Equal, 0),
                    Op::NotEqual => self.chunk.emit(OpCode::NotEqual, 0),
                    Op::Greater => self.chunk.emit(OpCode::Greater, 0),
                    Op::GreaterEqual => self.chunk.emit(OpCode::GreaterEqual, 0),
                    Op::Less => self.chunk.emit(OpCode::Less, 0),
                    Op::LessEqual => self.chunk.emit(OpCode::LessEqual, 0),
                    _ => return Err("العملية غير مدعومة بعد في محرك البايت كود".into()),
                };
            }
            Expr::Unary(op, val) => {
                self.compile_expr(val)?;
                match op {
                    Op::Negate => self.chunk.emit(OpCode::Negate, 0),
                    Op::Not => self.chunk.emit(OpCode::Not, 0),
                    _ => return Err("العملية الأحادية غير مدعومة بعد في البايت كود".into()),
                };
            }
            Expr::List(items) => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.chunk.emit(OpCode::BuildList(items.len()), 0);
            }
            Expr::Map(pairs) => {
                for (k, v) in pairs {
                    let k_idx = self.chunk.add_constant(Value::Text(k.clone()));
                    self.chunk.emit(OpCode::Constant(k_idx), 0);
                    self.compile_expr(v)?;
                }
                self.chunk.emit(OpCode::BuildMap(pairs.len()), 0);
            }
            Expr::Index(target, idx) => {
                self.compile_expr(target)?;
                self.compile_expr(idx)?;
                self.chunk.emit(OpCode::GetIndex, 0);
            }
            Expr::AssignIndex(name, idx, val) => {
                self.compile_expr(idx)?;
                self.compile_expr(val)?;
                self.chunk.emit(OpCode::SetIndex(name.clone()), 0);
            }
            Expr::Call(name, args) => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.chunk.emit(OpCode::CallBuiltin(name.clone(), args.len()), 0);
            }
            Expr::GetProperty(..) | Expr::SetProperty(..) | Expr::MethodCall(..) | Expr::Lambda(..) => {
                return Err("الخصائص والطرق الكائنية والدوال المجهولة تعمل حالياً عبر المفسّر".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_basic_arithmetic() {
        let mut vm = VM::new();
        let src = r#"
دع س = 15
دع ص = 3
اطبع(س * ص + 5)
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, ["50"]);
    }

    #[test]
    fn test_vm_collections_and_builtins() {
        let mut vm = VM::new();
        let src = r#"
دع درجات = [10, 20, 30]
اطبع(مجموع(درجات))
اطبع(متوسط(درجات))
اطبع(طول(درجات))
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, ["60", "20", "3"]);
    }
}
