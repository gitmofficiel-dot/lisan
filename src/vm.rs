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
                OpCode::In => {
                    let container = self.pop()?;
                    let item = self.pop()?;
                    let res = match &container {
                        Value::List(l) => l.borrow().contains(&item),
                        Value::Tuple(t) => t.contains(&item),
                        Value::Set(s) => s.borrow().contains(&item),
                        Value::Map(m) => {
                            if let Value::Text(ref k) = item {
                                m.borrow().iter().any(|(mk, _)| mk == k)
                            } else {
                                false
                            }
                        }
                        Value::Text(t) => {
                            if let Value::Text(ref needle) = item {
                                t.contains(needle.as_str())
                            } else {
                                false
                            }
                        }
                        _ => return Err("معامل 'في' يتطلب قائمة أو صفاً أو مجموعة أو قاموساً أو نصاً".into()),
                    };
                    self.stack.push(Value::Bool(res));
                }
                OpCode::NotIn => {
                    let container = self.pop()?;
                    let item = self.pop()?;
                    let res = match &container {
                        Value::List(l) => !l.borrow().contains(&item),
                        Value::Tuple(t) => !t.contains(&item),
                        Value::Set(s) => !s.borrow().contains(&item),
                        Value::Map(m) => {
                            if let Value::Text(ref k) = item {
                                !m.borrow().iter().any(|(mk, _)| mk == k)
                            } else {
                                true
                            }
                        }
                        Value::Text(t) => {
                            if let Value::Text(ref needle) = item {
                                !t.contains(needle.as_str())
                            } else {
                                true
                            }
                        }
                        _ => return Err("معامل 'ليس في' يتطلب قائمة أو صفاً أو مجموعة أو قاموساً أو نصاً".into()),
                    };
                    self.stack.push(Value::Bool(res));
                }

                OpCode::DefineGlobal(name) => {
                    let val = self.pop()?;
                    self.globals.insert(name.clone(), val);
                }
                OpCode::UnpackDefine(names) => {
                    let container = self.pop()?;
                    let items = match container {
                        Value::List(l) => l.borrow().clone(),
                        Value::Tuple(t) => t.clone(),
                        _ => return Err("فك التعبئة في البايت كود يتطلب قائمة أو صفاً".into()),
                    };
                    if items.len() != names.len() {
                        return Err(format!("فشل فك التعبئة في البايت كود: عدد المتغيرات ({}) لا يطابق عدد العناصر ({})", names.len(), items.len()));
                    }
                    for (name, item) in names.iter().zip(items) {
                        self.globals.insert(name.clone(), item);
                    }
                }
                OpCode::UnpackAssign(names) => {
                    let container = self.pop()?;
                    let items = match container {
                        Value::List(l) => l.borrow().clone(),
                        Value::Tuple(t) => t.clone(),
                        _ => return Err("فك التعبئة في البايت كود يتطلب قائمة أو صفاً".into()),
                    };
                    if items.len() != names.len() {
                        return Err(format!("فشل فك التعبئة في البايت كود: عدد المتغيرات ({}) لا يطابق عدد العناصر ({})", names.len(), items.len()));
                    }
                    for (name, item) in names.iter().zip(items) {
                        if !self.globals.contains_key(name) {
                            return Err(format!("لا يمكن الإسناد لمتغير غير معرّف: '{}'", name));
                        }
                        self.globals.insert(name.clone(), item);
                    }
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
                    let items_list = match iter_val {
                        Value::List(items) => Value::List(items),
                        Value::Tuple(items) => Value::list(items),
                        Value::Set(items) => Value::list(items.borrow().clone()),
                        Value::Text(t) => {
                            let chars = t.chars().map(|c| Value::Text(c.to_string())).collect();
                            Value::list(chars)
                        }
                        _ => return Err("حلقة لكل أو التوليد المختصر تتطلب قائمة أو صفاً أو مجموعة أو نصاً".into()),
                    };
                    self.stack.push(items_list);
                    self.stack.push(Value::Number(0.0));
                }
                OpCode::IterNext(var_name, end_dest) => {
                    let idx_val = self.stack.pop().ok_or("فقدان فهرس التكرار")?;
                    let list_val = self.stack.last().cloned().ok_or("فقدان قائمة التكرار")?;
                    match (&list_val, &idx_val) {
                        (Value::List(items), Value::Number(n)) => {
                            let idx = *n as usize;
                            let borrowed = items.borrow();
                            if idx < borrowed.len() {
                                self.globals.insert(var_name.clone(), borrowed[idx].clone());
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
                    self.stack.push(Value::list(items));
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
                    self.stack.push(Value::map(pairs));
                }
                OpCode::GetIndex => {
                    let index = self.pop()?;
                    let target = self.pop()?;
                    match target {
                        Value::List(items) => match index {
                            Value::Number(n) if n.fract() == 0.0 && (n >= 0.0 || (-n) as usize <= items.borrow().len()) => {
                                let idx = if n >= 0.0 { n as usize } else { items.borrow().len() - (-n) as usize };
                                let borrowed = items.borrow();
                                if let Some(item) = borrowed.get(idx) {
                                    self.stack.push(item.clone());
                                } else {
                                    return Err(format!("الفهرس {} خارج حدود القائمة", idx));
                                }
                            }
                            _ => return Err("فهرس القائمة يجب أن يكون عدداً صحيحاً موجباً".into()),
                        },
                        Value::Map(pairs) => match index {
                            Value::Text(key) => {
                                let borrowed = pairs.borrow();
                                if let Some((_, val)) = borrowed.iter().find(|(k, _)| k == &key) {
                                    self.stack.push(val.clone());
                                } else {
                                    let available: Vec<&str> = borrowed.iter().map(|(k, _)| k.as_str()).collect();
                                    return Err(crate::diagnostics::suggest_dict_key_error(&key, &available));
                                }
                            }
                            _ => return Err("مفتاح القاموس يجب أن يكون نصاً".into()),
                        },
                        Value::Tuple(items) => match index {
                            Value::Number(n) if n >= 0.0 && n.fract() == 0.0 => {
                                let idx = n as usize;
                                if let Some(item) = items.get(idx) {
                                    self.stack.push(item.clone());
                                } else {
                                    return Err(format!("الفهرس {} خارج حدود الصف", idx));
                                }
                            }
                            _ => return Err("فهرس الصف يجب أن يكون عدداً صحيحاً موجباً".into()),
                        },
                        _ => return Err("الفهرسة مدعومة فقط للقوائم والقواميس والصفوف".into()),
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
                                    let mut borrowed = items.borrow_mut();
                                    if idx < borrowed.len() {
                                        borrowed[idx] = val.clone();
                                    } else {
                                        return Err(format!("الفهرس {} خارج حدود القائمة", idx));
                                    }
                                }
                                _ => return Err("فهرس القائمة يجب أن يكون عدداً صحيحاً".into()),
                            },
                            Value::Map(pairs) => match index {
                                Value::Text(key) => {
                                    let mut borrowed = pairs.borrow_mut();
                                    if let Some((_, v)) = borrowed.iter_mut().find(|(k, _)| k == &key) {
                                        *v = val.clone();
                                    } else {
                                        borrowed.push((key, val.clone()));
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

                OpCode::Slice => {
                    let step_val = match self.pop()? {
                        Value::Number(n) => Some(n as i64),
                        Value::Null => None,
                        _ => return Err("خطوة التقطيع يجب أن تكون عدداً".into()),
                    };
                    let end_val = match self.pop()? {
                        Value::Number(n) => Some(n as i64),
                        Value::Null => None,
                        _ => return Err("نهاية التقطيع يجب أن تكون عدداً".into()),
                    };
                    let start_val = match self.pop()? {
                        Value::Number(n) => Some(n as i64),
                        Value::Null => None,
                        _ => return Err("بداية التقطيع يجب أن تكون عدداً".into()),
                    };
                    let target = self.pop()?;
                    let res = match target {
                        Value::List(items) => {
                            let borrowed = items.borrow();
                            let indices = crate::runtime::compute_slice_indices(borrowed.len(), start_val, end_val, step_val)?;
                            let sliced: Vec<Value> = indices.iter().map(|&i| borrowed[i].clone()).collect();
                            Value::list(sliced)
                        }
                        Value::Tuple(items) => {
                            let indices = crate::runtime::compute_slice_indices(items.len(), start_val, end_val, step_val)?;
                            let sliced: Vec<Value> = indices.iter().map(|&i| items[i].clone()).collect();
                            Value::Tuple(sliced)
                        }
                        Value::Text(text) => {
                            let chars: Vec<char> = text.chars().collect();
                            let indices = crate::runtime::compute_slice_indices(chars.len(), start_val, end_val, step_val)?;
                            let sliced: String = indices.iter().map(|&i| chars[i]).collect();
                            Value::Text(sliced)
                        }
                        _ => return Err("التقطيع متاح فقط للقوائم والصفوف والنصوص".into()),
                    };
                    self.stack.push(res);
                }

                OpCode::ListAppend(depth) => {
                    let val = self.pop()?;
                    let stack_len = self.stack.len();
                    if stack_len < *depth {
                        return Err("مكدس الآلة الافتراضية غير كافٍ لإضافة عنصر التوليد المختصر".into());
                    }
                    let target_idx = stack_len - *depth;
                    match &self.stack[target_idx] {
                        Value::List(l) => {
                            l.borrow_mut().push(val);
                        }
                        _ => return Err("الهدف في التوليد المختصر ليس قائمة".into()),
                    }
                }

                OpCode::GetProperty(name) => {
                    let target = self.pop()?;
                    match target {
                        Value::Map(m) => {
                            let borrowed = m.borrow();
                            if let Some((_, v)) = borrowed.iter().find(|(k, _)| k == name) {
                                self.stack.push(v.clone());
                            } else {
                                self.stack.push(Value::Null);
                            }
                        }
                        Value::Instance { fields, .. } => {
                            let borrowed = fields.borrow();
                            if let Some((_, v)) = borrowed.iter().find(|(k, _)| k == name) {
                                self.stack.push(v.clone());
                            } else {
                                self.stack.push(Value::Null);
                            }
                        }
                        Value::Module { exports, .. } => {
                            let borrowed = exports.borrow();
                            if let Some((_, v)) = borrowed.iter().find(|(k, _)| k == name) {
                                self.stack.push(v.clone());
                            } else {
                                self.stack.push(Value::Null);
                            }
                        }
                        _ => return Err(format!("لا يمكن قراءة الخاصية '{}' من هذا النوع في البايت كود", name)),
                    }
                }

                OpCode::SetProperty(name) => {
                    let val = self.pop()?;
                    let target = self.pop()?;
                    match target {
                        Value::Map(m) => {
                            let mut borrowed = m.borrow_mut();
                            if let Some((_, v)) = borrowed.iter_mut().find(|(k, _)| k == name) {
                                *v = val.clone();
                            } else {
                                borrowed.push((name.clone(), val.clone()));
                            }
                            self.stack.push(val);
                        }
                        Value::Instance { fields, .. } => {
                            let mut borrowed = fields.borrow_mut();
                            if let Some((_, v)) = borrowed.iter_mut().find(|(k, _)| k == name) {
                                *v = val.clone();
                            } else {
                                borrowed.push((name.clone(), val.clone()));
                            }
                            self.stack.push(val);
                        }
                        _ => return Err(format!("لا يمكن تعديل الخاصية '{}' في هذا النوع في البايت كود", name)),
                    }
                }

                OpCode::CallMethod(name, arg_count) => {
                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();
                    let target = self.pop()?;
                    let res = match target {
                        Value::List(items) => {
                            match name.as_str() {
                                "طول" if args.is_empty() => Value::Number(items.borrow().len() as f64),
                                "أضف" | "اضف" | "ألحق" | "الحق" if args.len() == 1 => {
                                    items.borrow_mut().push(args[0].clone());
                                    Value::List(items.clone())
                                }
                                "احذف" if args.len() == 1 => {
                                    let idx = match &args[0] {
                                        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => *n as usize,
                                        _ => return Err("فهرس الحذف يجب أن يكون عدداً صحيحاً".into()),
                                    };
                                    let mut borrowed = items.borrow_mut();
                                    if idx >= borrowed.len() {
                                        return Err(format!("فهرس الحذف {} خارج حدود القائمة", idx));
                                    }
                                    borrowed.remove(idx)
                                }
                                "أدرج" | "ادرج" if args.len() == 2 => {
                                    let idx = match &args[0] {
                                        Value::Number(n) if *n >= 0.0 && n.fract() == 0.0 => *n as usize,
                                        _ => return Err("فهرس الإدراج يجب أن يكون عدداً صحيحاً".into()),
                                    };
                                    let mut borrowed = items.borrow_mut();
                                    if idx > borrowed.len() {
                                        return Err(format!("فهرس الإدراج {} خارج حدود القائمة", idx));
                                    }
                                    borrowed.insert(idx, args[1].clone());
                                    drop(borrowed);
                                    Value::List(items.clone())
                                }
                                "تمديد" | "مدد" if args.len() == 1 => {
                                    let to_add: Vec<Value> = match &args[0] {
                                        Value::List(other) => other.borrow().clone(),
                                        Value::Tuple(other) => other.clone(),
                                        _ => return Err("طريقة تمديد تتطلب قائمة أو صفاً".into()),
                                    };
                                    items.borrow_mut().extend(to_add);
                                    Value::List(items.clone())
                                }
                                "مسح" if args.is_empty() => {
                                    items.borrow_mut().clear();
                                    Value::List(items.clone())
                                }
                                "نسخ" if args.is_empty() => {
                                    let copy = items.borrow().clone();
                                    Value::list(copy)
                                }
                                "عكس" if args.is_empty() => {
                                    let mut rev = items.borrow().clone();
                                    rev.reverse();
                                    Value::list(rev)
                                }
                                "فرز" if args.is_empty() => {
                                    let mut sorted = items.borrow().clone();
                                    sorted.sort_by(|a, b| match (a, b) {
                                        (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                        (Value::Text(x), Value::Text(y)) => x.cmp(y),
                                        _ => std::cmp::Ordering::Equal,
                                    });
                                    Value::list(sorted)
                                }
                                "مجموع" if args.is_empty() => {
                                    let mut sum = 0.0;
                                    for it in items.borrow().iter() {
                                        match it {
                                            Value::Number(n) => sum += n,
                                            _ => return Err("طريقة مجموع تتطلب قائمة أرقام".into()),
                                        }
                                    }
                                    Value::Number(sum)
                                }
                                other => return Err(format!("الطريقة '{}' غير مدعومة للقوائم في البايت كود", other)),
                            }
                        }
                        Value::Map(pairs) => {
                            match name.as_str() {
                                "طول" | "حجم" if args.is_empty() => Value::Number(pairs.borrow().len() as f64),
                                "مفاتيح" if args.is_empty() => {
                                    let keys: Vec<Value> = pairs.borrow().iter().map(|(k, _)| Value::Text(k.clone())).collect();
                                    Value::list(keys)
                                }
                                "قيم" if args.is_empty() => {
                                    let vals: Vec<Value> = pairs.borrow().iter().map(|(_, v)| v.clone()).collect();
                                    Value::list(vals)
                                }
                                "عناصر" if args.is_empty() => {
                                    let entries: Vec<Value> = pairs.borrow().iter().map(|(k, v)| Value::Tuple(vec![Value::Text(k.clone()), v.clone()])).collect();
                                    Value::list(entries)
                                }
                                "يحتوي" if args.len() == 1 => {
                                    let key = match &args[0] {
                                        Value::Text(s) => s.as_str(),
                                        _ => return Err("يحتوي تتطلب نصاً كمفتاح".into()),
                                    };
                                    Value::Bool(pairs.borrow().iter().any(|(k, _)| k == key))
                                }
                                "مسح" if args.is_empty() => {
                                    pairs.borrow_mut().clear();
                                    Value::Map(pairs.clone())
                                }
                                other => return Err(format!("الطريقة '{}' غير مدعومة للقواميس في البايت كود", other)),
                            }
                        }
                        Value::Set(items) => {
                            match name.as_str() {
                                "طول" | "حجم" if args.is_empty() => Value::Number(items.borrow().len() as f64),
                                "أضف" | "اضف" if args.len() == 1 => {
                                    let mut borrowed = items.borrow_mut();
                                    if !borrowed.contains(&args[0]) {
                                        borrowed.push(args[0].clone());
                                    }
                                    Value::Set(items.clone())
                                }
                                "احذف" if args.len() == 1 => {
                                    let mut borrowed = items.borrow_mut();
                                    borrowed.retain(|x| x != &args[0]);
                                    Value::Set(items.clone())
                                }
                                "يحتوي" if args.len() == 1 => {
                                    Value::Bool(items.borrow().contains(&args[0]))
                                }
                                "قائمة" if args.is_empty() => {
                                    Value::list(items.borrow().clone())
                                }
                                "مسح" if args.is_empty() => {
                                    items.borrow_mut().clear();
                                    Value::Set(items.clone())
                                }
                                other => return Err(format!("الطريقة '{}' غير مدعومة للمجموعات في البايت كود", other)),
                            }
                        }
                        Value::Text(text) => {
                            match name.as_str() {
                                "طول" if args.is_empty() => Value::Number(text.chars().count() as f64),
                                "تحتوي" if args.len() == 1 => match &args[0] {
                                    Value::Text(sub) => Value::Bool(text.contains(sub)),
                                    _ => return Err("طريقة تحتوي تتطلب نصاً للبحث".into()),
                                },
                                "تقسيم" if args.len() == 1 => match &args[0] {
                                    Value::Text(delim) => {
                                        let parts: Vec<Value> = text.split(delim).map(|s| Value::Text(s.to_string())).collect();
                                        Value::list(parts)
                                    }
                                    _ => return Err("طريقة تقسيم تتطلب فاصلاً نصياً".into()),
                                },
                                "عكس" if args.is_empty() => Value::Text(text.chars().rev().collect()),
                                "أحرف_كبيرة" if args.is_empty() => Value::Text(text.to_uppercase()),
                                "أحرف_صغيرة" if args.is_empty() => Value::Text(text.to_lowercase()),
                                "استبدل" if args.len() == 2 => match (&args[0], &args[1]) {
                                    (Value::Text(from), Value::Text(to)) => Value::Text(text.replace(from, to)),
                                    _ => return Err("طريقة استبدل تتطلب نصين".into()),
                                },
                                other => return Err(format!("الطريقة '{}' غير مدعومة للنصوص في البايت كود", other)),
                            }
                        }
                        Value::Tuple(items) => {
                            match name.as_str() {
                                "طول" if args.is_empty() => Value::Number(items.len() as f64),
                                "قائمة" if args.is_empty() => Value::list(items.clone()),
                                other => return Err(format!("الطريقة '{}' غير مدعومة للصفوف في البايت كود", other)),
                            }
                        }
                        Value::Module { name: ref mod_name, .. } => {
                            crate::runtime::call_builtin_for_module(mod_name, name, args)?
                        }
                        _ => return Err("استدعاء الطرق في البايت كود غير مدعوم لهذا النوع".into()),
                    };
                    self.stack.push(res);
                }

                OpCode::CallBuiltin(name, arg_count) => {
                    let mut args = Vec::with_capacity(*arg_count);
                    for _ in 0..*arg_count {
                        args.push(self.pop()?);
                    }
                    args.reverse();
                    let res = match name.as_str() {
                        "طول" if args.len() == 1 => match &args[0] {
                            Value::List(l) => Value::Number(l.borrow().len() as f64),
                            Value::Text(t) => Value::Number(t.chars().count() as f64),
                            Value::Map(m) => Value::Number(m.borrow().len() as f64),
                            Value::Tuple(t) => Value::Number(t.len() as f64),
                            Value::Set(s) => Value::Number(s.borrow().len() as f64),
                            Value::Instance { fields, .. } => Value::Number(fields.borrow().len() as f64),
                            _ => return Err("طول تتطلب قائمة أو نصاً أو قاموساً أو كائناً".into()),
                        },
                        "نوع" if args.len() == 1 => match &args[0] {
                            Value::Number(_) => Value::Text("رقم".into()),
                            Value::Text(_) => Value::Text("نص".into()),
                            Value::Bool(_) => Value::Text("منطقي".into()),
                            Value::List(_) => Value::Text("قائمة".into()),
                            Value::Map(_) => Value::Text("قاموس".into()),
                            Value::Tuple(_) => Value::Text("صف".into()),
                            Value::Set(_) => Value::Text("مجموعة".into()),
                            Value::Null => Value::Text("لاشيء".into()),
                            Value::Instance { class_name, .. } => Value::Text(class_name.clone()),
                            Value::Lambda { .. } => Value::Text("دالة".into()),
                            Value::Module { .. } => Value::Text("وحدة".into()),
                        },
                        "مجموع" if args.len() == 1 => match &args[0] {
                            Value::List(items) => {
                                let mut sum = 0.0;
                                for item in items.borrow().iter() {
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
                            Value::List(items) => {
                                let borrowed = items.borrow();
                                if borrowed.is_empty() {
                                    return Err("متوسط تتطلب قائمة غير فارغة".into());
                                }
                                let mut sum = 0.0;
                                for item in borrowed.iter() {
                                    match item {
                                        Value::Number(n) => sum += *n,
                                        _ => return Err("متوسط تتطلب قائمة من الأرقام فقط".into()),
                                    }
                                }
                                Value::Number(sum / (borrowed.len() as f64))
                            }
                            _ => return Err("متوسط تتطلب قائمة".into()),
                        },
                        "مفاتيح" if args.len() == 1 => match &args[0] {
                            Value::Map(m) => Value::list(m.borrow().iter().map(|(k, _)| Value::Text(k.clone())).collect()),
                            _ => return Err("مفاتيح تتطلب قاموساً".into()),
                        },
                        "قيم" if args.len() == 1 => match &args[0] {
                            Value::Map(m) => Value::list(m.borrow().iter().map(|(_, v)| v.clone()).collect()),
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
                                let mut rev = l.borrow().clone();
                                rev.reverse();
                                Value::list(rev)
                            }
                            Value::Text(t) => Value::Text(t.chars().rev().collect()),
                            _ => return Err("عكس تتطلب قائمة أو نصاً".into()),
                        },
                        "فرز" if args.len() == 1 => match &args[0] {
                            Value::List(items) => {
                                let mut sorted = items.borrow().clone();
                                sorted.sort_by(|a, b| match (a, b) {
                                    (Value::Number(x), Value::Number(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                                    (Value::Text(x), Value::Text(y)) => x.cmp(y),
                                    _ => std::cmp::Ordering::Equal,
                                });
                                Value::list(sorted)
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
                            if let Ok(val) = crate::runtime::call_builtin_for_module("", other, args) {
                                val
                            } else {
                                let empty: Vec<&str> = Vec::new();
                                return Err(crate::diagnostics::suggest_function_error(other, &empty));
                            }
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

struct LoopInfo {
    start_ip: usize,
    break_jumps: Vec<usize>,
}

/// مترجم يحول شجرة بناء التعليمات (AST) إلى بايت كود للآلة الافتراضية
pub struct Compiler {
    chunk: Chunk,
    loop_stack: Vec<LoopInfo>,
}

impl Compiler {
    pub fn compile(statements: &[Stmt]) -> Result<Chunk, String> {
        let mut compiler = Self {
            chunk: Chunk::new(),
            loop_stack: Vec::new(),
        };
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
            Stmt::LetUnpack(names, expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::UnpackDefine(names.clone()), 0);
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr)?;
                self.chunk.emit(OpCode::Print, 0);
            }
            Stmt::Expr(expr) => {
                if let Expr::AssignUnpack(names, target_expr) = expr.unwrap_spanned() {
                    self.compile_expr(target_expr)?;
                    self.chunk.emit(OpCode::UnpackAssign(names.clone()), 0);
                } else {
                    self.compile_expr(expr)?;
                    self.chunk.emit(OpCode::Pop, 0);
                }
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

                self.loop_stack.push(LoopInfo { start_ip: loop_start, break_jumps: Vec::new() });
                for s in body {
                    self.compile_stmt(s)?;
                }
                let loop_info = self.loop_stack.pop().unwrap();

                self.chunk.emit(OpCode::Jump(loop_start), 0);
                let loop_end = self.chunk.instructions.len();
                self.chunk.patch_jump(jump_out, loop_end);
                for brk in loop_info.break_jumps {
                    self.chunk.patch_jump(brk, loop_end);
                }
            }
            Stmt::ForEach(name, iterable, body) => {
                self.compile_expr(iterable)?;
                self.chunk.emit(OpCode::IterInit, 0);
                let loop_start = self.chunk.instructions.len();
                let iter_next = self.chunk.emit(OpCode::IterNext(name.clone(), 0), 0);

                self.loop_stack.push(LoopInfo { start_ip: loop_start, break_jumps: Vec::new() });
                for s in body {
                    self.compile_stmt(s)?;
                }
                let loop_info = self.loop_stack.pop().unwrap();

                self.chunk.emit(OpCode::Jump(loop_start), 0);
                let loop_end = self.chunk.instructions.len();
                self.chunk.patch_jump(iter_next, loop_end);
                self.chunk.emit(OpCode::IterPop, 0);
                for brk in loop_info.break_jumps {
                    self.chunk.patch_jump(brk, loop_end);
                }
            }
            Stmt::ForEachUnpack(names, iterable, body) => {
                self.compile_expr(iterable)?;
                self.chunk.emit(OpCode::IterInit, 0);
                let loop_start = self.chunk.instructions.len();
                let temp_var = format!("__تفكيك_تكرار_{}__", self.chunk.instructions.len());
                let iter_next = self.chunk.emit(OpCode::IterNext(temp_var.clone(), 0), 0);
                self.chunk.emit(OpCode::GetGlobal(temp_var), 0);
                self.chunk.emit(OpCode::UnpackDefine(names.clone()), 0);

                self.loop_stack.push(LoopInfo { start_ip: loop_start, break_jumps: Vec::new() });
                for s in body {
                    self.compile_stmt(s)?;
                }
                let loop_info = self.loop_stack.pop().unwrap();

                self.chunk.emit(OpCode::Jump(loop_start), 0);
                let loop_end = self.chunk.instructions.len();
                self.chunk.patch_jump(iter_next, loop_end);
                self.chunk.emit(OpCode::IterPop, 0);
                for brk in loop_info.break_jumps {
                    self.chunk.patch_jump(brk, loop_end);
                }
            }
            Stmt::Break => {
                if let Some(info) = self.loop_stack.last_mut() {
                    let jmp = self.chunk.emit(OpCode::Jump(0), 0);
                    info.break_jumps.push(jmp);
                } else {
                    return Err("أمر 'توقف' (break) مسموح به فقط داخل حلقات التكرار".into());
                }
            }
            Stmt::Continue => {
                if let Some(info) = self.loop_stack.last() {
                    self.chunk.emit(OpCode::Jump(info.start_ip), 0);
                } else {
                    return Err("أمر 'تابع' (continue) مسموح به فقط داخل حلقات التكرار".into());
                }
            }
            Stmt::Return(maybe_expr) => {
                if let Some(e) = maybe_expr {
                    self.compile_expr(e)?;
                } else {
                    self.chunk.emit(OpCode::Null, 0);
                }
                self.chunk.emit(OpCode::Return, 0);
            }
            Stmt::ImportModule { module, alias } => {
                let target_name = alias.clone().unwrap_or_else(|| {
                    if crate::modules::is_source_name(module) {
                        std::path::Path::new(module)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(module)
                            .to_string()
                    } else {
                        module.clone()
                    }
                });
                let mut initial_exports = Vec::new();
                let norm = crate::diagnostics::normalize_arabic(module);
                if matches!(norm.as_str(), "رياضيات" | "math") {
                    initial_exports.push(("ط".to_string(), Value::Number(std::f64::consts::PI)));
                    initial_exports.push(("قيمة_ط".to_string(), Value::Number(std::f64::consts::PI)));
                    initial_exports.push(("هـ".to_string(), Value::Number(std::f64::consts::E)));
                    initial_exports.push(("قيمة_هـ".to_string(), Value::Number(std::f64::consts::E)));
                }
                let mod_val = Value::Module {
                    name: module.clone(),
                    exports: std::rc::Rc::new(std::cell::RefCell::new(initial_exports)),
                };
                let idx = self.chunk.add_constant(mod_val);
                self.chunk.emit(OpCode::Constant(idx), 0);
                self.chunk.emit(OpCode::DefineGlobal(target_name), 0);
            }
            Stmt::FromImport { module, items } => {
                let norm = crate::diagnostics::normalize_arabic(module);
                if matches!(norm.as_str(), "رياضيات" | "math") {
                    for item in items {
                        if item == "ط" || item == "قيمة_ط" {
                            let idx = self.chunk.add_constant(Value::Number(std::f64::consts::PI));
                            self.chunk.emit(OpCode::Constant(idx), 0);
                            self.chunk.emit(OpCode::DefineGlobal(item.clone()), 0);
                        } else if item == "هـ" || item == "قيمة_هـ" {
                            let idx = self.chunk.add_constant(Value::Number(std::f64::consts::E));
                            self.chunk.emit(OpCode::Constant(idx), 0);
                            self.chunk.emit(OpCode::DefineGlobal(item.clone()), 0);
                        }
                    }
                }
            }
            _ => return Err(format!("الجملة غير مدعومة بعد في محرك البايت كود: {:?}", stmt)),
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
                    Op::In => self.chunk.emit(OpCode::In, 0),
                    Op::NotIn => self.chunk.emit(OpCode::NotIn, 0),
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
            Expr::FString(segments) => {
                let empty_idx = self.chunk.add_constant(Value::Text(String::new()));
                self.chunk.emit(OpCode::Constant(empty_idx), 0);
                for seg in segments {
                    self.compile_expr(seg)?;
                    self.chunk.emit(OpCode::Add, 0);
                }
            }
            Expr::Slice(target, start, end, step) => {
                self.compile_expr(target)?;
                if let Some(s) = start {
                    self.compile_expr(s)?;
                } else {
                    let idx = self.chunk.add_constant(Value::Null);
                    self.chunk.emit(OpCode::Constant(idx), 0);
                }
                if let Some(e) = end {
                    self.compile_expr(e)?;
                } else {
                    let idx = self.chunk.add_constant(Value::Null);
                    self.chunk.emit(OpCode::Constant(idx), 0);
                }
                if let Some(st) = step {
                    self.compile_expr(st)?;
                } else {
                    let idx = self.chunk.add_constant(Value::Null);
                    self.chunk.emit(OpCode::Constant(idx), 0);
                }
                self.chunk.emit(OpCode::Slice, 0);
            }
            Expr::ListComprehension { expr, var, iterable, condition } => {
                self.chunk.emit(OpCode::BuildList(0), 0);
                self.compile_expr(iterable)?;
                self.chunk.emit(OpCode::IterInit, 0);
                let loop_start = self.chunk.instructions.len();
                let iter_next = self.chunk.emit(OpCode::IterNext(var.clone(), 0), 0);

                if let Some(cond) = condition {
                    self.compile_expr(cond)?;
                    let jump_skip = self.chunk.emit(OpCode::JumpIfFalse(0), 0);
                    self.compile_expr(expr)?;
                    self.chunk.emit(OpCode::ListAppend(3), 0);
                    let after_append = self.chunk.instructions.len();
                    self.chunk.patch_jump(jump_skip, after_append);
                } else {
                    self.compile_expr(expr)?;
                    self.chunk.emit(OpCode::ListAppend(3), 0);
                }

                self.chunk.emit(OpCode::Jump(loop_start), 0);
                let loop_end = self.chunk.instructions.len();
                self.chunk.patch_jump(iter_next, loop_end);
                self.chunk.emit(OpCode::IterPop, 0);
            }
            Expr::MethodCall(target, method_name, args) => {
                self.compile_expr(target)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.chunk.emit(OpCode::CallMethod(method_name.clone(), args.len()), 0);
            }
            Expr::GetProperty(target, prop) => {
                self.compile_expr(target)?;
                self.chunk.emit(OpCode::GetProperty(prop.clone()), 0);
            }
            Expr::SetProperty(target, prop, val) => {
                self.compile_expr(target)?;
                self.compile_expr(val)?;
                self.chunk.emit(OpCode::SetProperty(prop.clone()), 0);
            }
            Expr::Lambda(..) => {
                return Err("الدوال المجهولة (Lambda) تعمل حالياً عبر المفسّر".into());
            }
            Expr::AssignUnpack(..) => {
                return Err("فك التعبئة غير مسموح كتعبير مستقل داخل تعبير آخر".into());
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

    #[test]
    fn test_vm_rejects_unsupported_statements_instead_of_silent_drop() {
        let mut vm = VM::new();
        let err = vm.run_source("دالة اجمع(أ، ب) { أرجع أ + ب } اطبع(اجمع(2، 3))").unwrap_err();
        assert!(err.contains("الجملة غير مدعومة بعد في محرك البايت كود"));
    }

    #[test]
    fn test_vm_in_and_not_in() {
        let mut vm = VM::new();
        let src = r#"
دع قائمة = [10, 20, 30]
اطبع(20 في قائمة)
اطبع(99 ليس في قائمة)
اطبع(50 في قائمة)
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, ["صحيح", "صحيح", "خطأ"]);
    }

    #[test]
    fn test_vm_unpacking() {
        let mut vm = VM::new();
        let src = r#"
دع أ، ب = [100, 200]
اطبع(أ)
اطبع(ب)
أ، ب = [ب، أ]
اطبع(أ)
اطبع(ب)
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, ["100", "200", "200", "100"]);
    }

    #[test]
    fn test_vm_fstring() {
        let mut vm = VM::new();
        let src = r#"
دع اسم = "لسان"
دع عدد = 42
اطبع(ن"لغة {اسم} رقم {عدد}")
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, ["لغة لسان رقم 42"]);
    }

    #[test]
    fn test_vm_slicing() {
        let mut vm = VM::new();
        let src = r#"
دع أرقام = [10, 20, 30, 40, 50]
اطبع(أرقام[1:4])
اطبع(أرقام[:2])
اطبع(أرقام[3:])
اطبع(أرقام[::-1])
دع نص = "العربية"
اطبع(نص[:6])
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, [
            "[20، 30، 40]",
            "[10، 20]",
            "[40، 50]",
            "[50، 40، 30، 20، 10]",
            "العربي"
        ]);
    }

    #[test]
    fn test_vm_list_comprehension() {
        let mut vm = VM::new();
        let src = r#"
دع مربعات = [س * س لكل س في [1, 2, 3, 4, 5]]
اطبع(مربعات)
دع زوجي = [س لكل س في [1, 2, 3, 4, 5, 6] إذا س % 2 == 0]
اطبع(زوجي)
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, [
            "[1، 4، 9، 16، 25]",
            "[2، 4، 6]"
        ]);
    }

    #[test]
    fn test_vm_method_calls() {
        let mut vm = VM::new();
        let src = r#"
دع قائمة = [30, 10, 20]
قائمة.أضف(40)
اطبع(قائمة.طول())
اطبع(قائمة.فرز())
اطبع(قائمة.عكس())
اطبع(قائمة.مجموع())
دع نص = "أ،ب،ج"
اطبع(نص.تقسيم("،"))
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, [
            "4",
            "[10، 20، 30، 40]",
            "[40، 20، 10، 30]",
            "100",
            "[\"أ\"، \"ب\"، \"ج\"]"
        ]);
    }

    #[test]
    fn test_vm_property_access() {
        let mut vm = VM::new();
        let src = r#"
دع كائن = {"اسم": "أحمد", "عمر": 30}
اطبع(كائن.اسم)
كائن.اسم = "سعيد"
اطبع(كائن.اسم)
"#;
        let out = vm.run_source(src).unwrap();
        assert_eq!(out, [
            "أحمد",
            "سعيد"
        ]);
    }
}
