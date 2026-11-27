//! خادم بروتوكول تصحيح الأخطاء (Debug Adapter Protocol - DAP) للغة لسان.
//!
//! يوفر تصحيحاً تفاعلياً حقيقياً لبرامج لسان (`.lsn`) عبر الإدخال والإخراج القياسي (stdio)
//! باستخدام تأطير `Content-Length:` القياسي، مع دعم نقاط التوقف والتتبع خطوة بخطوة
//! وفحص المتغيرات وتقييم التعابير العربية في بيئة التنفيذ الحية — بدون أي اعتماديات خارجية.

use crate::ast::Stmt;
use crate::json::{parse_json, to_json_string};
use crate::runtime::{Interpreter, Limits, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// خادم تصحيح أخطاء لسان (DAP Server)
pub struct DapServer {
    seq: u64,
    program_path: Option<String>,
    source_code: String,
    statements: Vec<Stmt>,
    current_index: usize,
    next_breakpoint_id: i64,
    requested_breakpoints: HashMap<String, Vec<StoredBreakpoint>>,
    hittable_lines: HashSet<usize>,
    active_breakpoints: HashSet<usize>,
    interpreter: Interpreter,
    stop_on_entry: bool,
    started: bool,
    terminated: bool,
    disconnected: bool,
    last_line: usize,
    last_column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StoredBreakpoint {
    id: i64,
    source_path: String,
    requested_line: Option<i64>,
    verified: bool,
    reason: &'static str,
    message: Option<String>,
    resolved_line: Option<usize>,
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DapServer {
    /// إنشاء مثيل جديد من خادم تصحيح الأخطاء
    pub fn new() -> Self {
        Self {
            seq: 1,
            program_path: None,
            source_code: String::new(),
            statements: Vec::new(),
            current_index: 0,
            next_breakpoint_id: 1,
            requested_breakpoints: HashMap::new(),
            hittable_lines: HashSet::new(),
            active_breakpoints: HashSet::new(),
            interpreter: Interpreter::with_limits(Limits::full()),
            stop_on_entry: false,
            started: false,
            terminated: false,
            disconnected: false,
            last_line: 1,
            last_column: 1,
        }
    }

    /// هل تم إنهاء الاتصال بالخادم؟
    pub fn is_disconnected(&self) -> bool {
        self.disconnected
    }

    fn next_seq(&mut self) -> u64 {
        let s = self.seq;
        self.seq += 1;
        s
    }

    fn make_response(
        &mut self,
        request_seq: i64,
        command: &str,
        success: bool,
        message: Option<&str>,
        body: Option<Value>,
    ) -> String {
        let seq = self.next_seq();
        let mut pairs = vec![
            ("seq".to_string(), Value::Number(seq as f64)),
            ("type".to_string(), Value::Text("response".into())),
            ("request_seq".to_string(), Value::Number(request_seq as f64)),
            ("success".to_string(), Value::Bool(success)),
            ("command".to_string(), Value::Text(command.into())),
        ];
        if let Some(msg) = message {
            pairs.push(("message".to_string(), Value::Text(msg.into())));
        }
        if let Some(b) = body {
            pairs.push(("body".to_string(), b));
        }
        to_json_string(&Value::map(pairs)).unwrap_or_else(|_| {
            format!(
                r#"{{"seq":{},"type":"response","request_seq":{},"success":{},"command":"{}"}}"#,
                seq, request_seq, success, command
            )
        })
    }

    fn make_event(&mut self, event_name: &str, body: Option<Value>) -> String {
        let seq = self.next_seq();
        let mut pairs = vec![
            ("seq".to_string(), Value::Number(seq as f64)),
            ("type".to_string(), Value::Text("event".into())),
            ("event".to_string(), Value::Text(event_name.into())),
        ];
        if let Some(b) = body {
            pairs.push(("body".to_string(), b));
        }
        to_json_string(&Value::map(pairs)).unwrap_or_else(|_| {
            format!(
                r#"{{"seq":{},"type":"event","event":"{}"}}"#,
                seq, event_name
            )
        })
    }

    /// معالجة رسالة DAP واحدة بتنسيق JSON وإرجاع قائمة الرسائل (ردود وأحداث) المراد إرسالها
    pub fn handle_message(&mut self, raw_json: &str) -> Vec<String> {
        let parsed = match parse_json(raw_json) {
            Ok(val) => val,
            Err(err) => {
                return vec![self.make_response(
                    0,
                    "unknown",
                    false,
                    Some(&format!("تعذّر تحليل رسالة DAP: {}", err)),
                    None,
                )];
            }
        };

        let request_seq = get_number(&parsed, "seq").unwrap_or(0.0) as i64;
        let command = get_string(&parsed, "command").unwrap_or_default();
        let arguments = get_field(&parsed, "arguments");

        match command.as_str() {
            "initialize" => self.handle_initialize(request_seq),
            "launch" => self.handle_launch(request_seq, arguments.as_ref()),
            "setBreakpoints" => self.handle_set_breakpoints(request_seq, arguments.as_ref()),
            "setExceptionBreakpoints" => {
                vec![self.make_response(request_seq, "setExceptionBreakpoints", true, None, Some(Value::map(vec![])))]
            }
            "configurationDone" => self.handle_configuration_done(request_seq),
            "threads" => self.handle_threads(request_seq),
            "stackTrace" => self.handle_stack_trace(request_seq),
            "scopes" => self.handle_scopes(request_seq),
            "variables" => self.handle_variables(request_seq, arguments.as_ref()),
            "next" => self.handle_step(request_seq, &command),
            "stepIn" | "stepOut" => self.handle_unsupported_step(request_seq, &command),
            "continue" => self.handle_continue(request_seq),
            "pause" => self.handle_pause(request_seq),
            "evaluate" => self.handle_evaluate(request_seq, arguments.as_ref()),
            "restart" => self.handle_restart(request_seq),
            "terminate" => self.handle_terminate(request_seq),
            "disconnect" => self.handle_disconnect(request_seq),
            other => vec![self.make_response(
                request_seq,
                other,
                true,
                None,
                Some(Value::map(vec![])),
            )],
        }
    }

    fn handle_initialize(&mut self, request_seq: i64) -> Vec<String> {
        let capabilities = Value::map(vec![
            ("supportsConfigurationDoneRequest".into(), Value::Bool(true)),
            ("supportsEvaluateForHovers".into(), Value::Bool(true)),
            ("supportsRestartRequest".into(), Value::Bool(true)),
            ("supportsTerminateRequest".into(), Value::Bool(true)),
            ("supportsStepBack".into(), Value::Bool(false)),
            ("supportsSetVariable".into(), Value::Bool(false)),
            ("supportsFunctionBreakpoints".into(), Value::Bool(false)),
        ]);

        let resp = self.make_response(request_seq, "initialize", true, None, Some(capabilities));
        let initialized_evt = self.make_event("initialized", None);
        vec![resp, initialized_evt]
    }

    fn handle_launch(&mut self, request_seq: i64, arguments: Option<&Value>) -> Vec<String> {
        let program = match arguments.and_then(|a| get_string(a, "program")) {
            Some(p) if !p.trim().is_empty() => p,
            _ => {
                return vec![self.make_response(
                    request_seq,
                    "launch",
                    false,
                    Some("لم يُحدَّد مسار ملف لسان في خاصية program"),
                    None,
                )];
            }
        };

        self.stop_on_entry = arguments
            .and_then(|a| get_bool(a, "stopOnEntry"))
            .unwrap_or(false);

        let mut changed_bps = Vec::new();
        if let Err(err) = self.load_program(&program, &mut changed_bps) {
            return vec![self.make_response(request_seq, "launch", false, Some(&err), None)];
        }

        let mut out = vec![self.make_response(request_seq, "launch", true, None, None)];
        self.emit_breakpoint_changed_events(&changed_bps, &mut out);
        out
    }

    fn load_program(&mut self, program: &str, changed_bps: &mut Vec<StoredBreakpoint>) -> Result<(), String> {
        let source = fs::read_to_string(program)
            .map_err(|e| format!("تعذّرت قراءة ملف لسان '{}': {}", program, e))?;
        let tokens = crate::lexer::tokenize(&source)?;
        let statements = crate::parser::parse(tokens)?;

        self.program_path = Some(program.to_string());
        self.source_code = source;
        self.statements = statements;
        self.current_index = 0;
        self.hittable_lines = collect_top_level_hittable_lines(&self.statements);
        self.interpreter = Interpreter::with_limits(Limits::full());
        self.started = false;
        self.terminated = false;

        if let Some(first) = self.statements.first() {
            let (l, c) = stmt_span(first);
            self.last_line = l;
            self.last_column = c;
        } else {
            self.last_line = 1;
            self.last_column = 1;
        }

        self.reconcile_breakpoints(changed_bps);
        Ok(())
    }

    fn alloc_breakpoint_id(&mut self) -> i64 {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        id
    }

    fn resolve_single_breakpoint(
        &self,
        source_path: &str,
        requested_line: Option<i64>,
    ) -> (bool, &'static str, Option<String>, Option<usize>) {
        let line_num = match requested_line {
            Some(n) if n >= 1 => n as usize,
            _ => {
                return (
                    false,
                    "failed",
                    Some("رقم السطر غير صالح؛ يجب أن يكون 1 أو أكبر".into()),
                    None,
                );
            }
        };

        if self.program_path.is_none() {
            return (
                false,
                "pending",
                Some("نقطة التوقف معلّقة بانتظار بدء تشغيل البرنامج".into()),
                Some(line_num),
            );
        }

        if source_path.trim().is_empty() || !self.path_matches_program(source_path) {
            return (
                false,
                "failed",
                Some("الملف المحدد لا يطابق الملف الرئيسي قيد التصحيح".into()),
                Some(line_num),
            );
        }

        if self.hittable_lines.contains(&line_num) {
            (true, "pending", None, Some(line_num))
        } else {
            (
                false,
                "failed",
                Some("السطر غير قابل للتوقف على المستوى الأعلى (تعليق أو فارغ أو متداخل أو مستورد)".into()),
                Some(line_num),
            )
        }
    }

    fn stored_to_dap_value(bp: &StoredBreakpoint) -> Value {
        let mut bp_obj = vec![
            ("id".into(), Value::Number(bp.id as f64)),
            ("verified".into(), Value::Bool(bp.verified)),
        ];
        if !bp.verified {
            bp_obj.push(("reason".into(), Value::Text(bp.reason.into())));
        }
        if let Some(msg) = &bp.message {
            bp_obj.push(("message".into(), Value::Text(msg.clone())));
        }
        if let Some(line) = bp.resolved_line {
            bp_obj.push(("line".into(), Value::Number(line as f64)));
        }
        if !bp.source_path.is_empty() {
            bp_obj.push((
                "source".into(),
                Value::map(vec![("path".into(), Value::Text(bp.source_path.clone()))]),
            ));
        }
        Value::map(bp_obj)
    }

    fn rebuild_active_breakpoints(&mut self) {
        self.active_breakpoints.clear();
        for entries in self.requested_breakpoints.values() {
            for bp in entries {
                if bp.verified {
                    if let Some(line) = bp.resolved_line {
                        self.active_breakpoints.insert(line);
                    }
                }
            }
        }
    }

    fn reconcile_breakpoints(&mut self, changed_bps: &mut Vec<StoredBreakpoint>) {
        let keys: Vec<String> = self.requested_breakpoints.keys().cloned().collect();

        for key in keys {
            let entries = match self.requested_breakpoints.get(&key) {
                Some(list) => list.clone(),
                None => continue,
            };
            let mut updated_entries = Vec::with_capacity(entries.len());
            for mut bp in entries {
                let (verified, reason, message, resolved_line) =
                    self.resolve_single_breakpoint(&bp.source_path, bp.requested_line);
                let changed = bp.verified != verified
                    || bp.reason != reason
                    || bp.message != message
                    || bp.resolved_line != resolved_line;
                bp.verified = verified;
                bp.reason = reason;
                bp.message = message;
                bp.resolved_line = resolved_line;
                if changed {
                    changed_bps.push(bp.clone());
                }
                updated_entries.push(bp);
            }
            self.requested_breakpoints.insert(key, updated_entries);
        }

        self.rebuild_active_breakpoints();
    }

    fn emit_breakpoint_changed_events(&mut self, changed_bps: &[StoredBreakpoint], out: &mut Vec<String>) {
        for bp in changed_bps {
            let body = Value::map(vec![
                ("reason".into(), Value::Text("changed".into())),
                ("breakpoint".into(), Self::stored_to_dap_value(bp)),
            ]);
            out.push(self.make_event("breakpoint", Some(body)));
        }
    }

    fn handle_set_breakpoints(&mut self, request_seq: i64, arguments: Option<&Value>) -> Vec<String> {
        let source_path = arguments
            .and_then(|a| get_field(a, "source"))
            .and_then(|s| get_string(&s, "path"))
            .unwrap_or_default();

        let mut requested_lines: Vec<Option<i64>> = Vec::new();
        if let Some(args) = arguments {
            if let Some(Value::List(bps)) = get_field(args, "breakpoints") {
                for bp in bps.borrow().iter() {
                    let line_opt = get_number(bp, "line").map(|n| n.trunc() as i64);
                    requested_lines.push(line_opt);
                }
            } else if let Some(Value::List(raw_lines)) = get_field(args, "lines") {
                for item in raw_lines.borrow().iter() {
                    let line_opt = match item {
                        Value::Number(line_num) => Some(line_num.trunc() as i64),
                        _ => None,
                    };
                    requested_lines.push(line_opt);
                }
            }
        }

        let mut stored_list = Vec::with_capacity(requested_lines.len());
        let mut bp_values = Vec::with_capacity(requested_lines.len());

        for req_line in requested_lines {
            let id = self.alloc_breakpoint_id();
            let (verified, reason, message, resolved_line) =
                self.resolve_single_breakpoint(&source_path, req_line);
            let stored = StoredBreakpoint {
                id,
                source_path: source_path.clone(),
                requested_line: req_line,
                verified,
                reason,
                message,
                resolved_line,
            };
            bp_values.push(Self::stored_to_dap_value(&stored));
            stored_list.push(stored);
        }

        let norm_key = normalize_path(&source_path);
        if !norm_key.is_empty() {
            self.requested_breakpoints.insert(norm_key, stored_list);
        }
        self.rebuild_active_breakpoints();

        let body = Value::map(vec![("breakpoints".into(), Value::list(bp_values))]);
        vec![self.make_response(request_seq, "setBreakpoints", true, None, Some(body))]
    }

    fn handle_configuration_done(&mut self, request_seq: i64) -> Vec<String> {
        let mut out = vec![self.make_response(request_seq, "configurationDone", true, None, None)];
        self.started = true;

        if self.statements.is_empty() {
            self.terminated = true;
            out.push(self.make_exited_event(0));
            out.push(self.make_event("terminated", None));
            return out;
        }

        if self.stop_on_entry {
            let (l, c) = stmt_span(&self.statements[self.current_index]);
            self.last_line = l;
            self.last_column = c;
            out.push(self.make_stopped_event("entry", None));
            return out;
        }

        self.run_until_stop(false, &mut out);
        out
    }

    fn handle_threads(&mut self, request_seq: i64) -> Vec<String> {
        let thread = Value::map(vec![
            ("id".into(), Value::Number(1.0)),
            ("name".into(), Value::Text("الخيط الرئيسي (لسان)".into())),
        ]);
        let body = Value::map(vec![("threads".into(), Value::list(vec![thread]))]);
        vec![self.make_response(request_seq, "threads", true, None, Some(body))]
    }

    fn handle_stack_trace(&mut self, request_seq: i64) -> Vec<String> {
        let path_str = self.program_path.clone().unwrap_or_default();
        let file_name = Path::new(&path_str)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("برنامج.lsn")
            .to_string();

        let source_obj = Value::map(vec![
            ("name".into(), Value::Text(file_name)),
            ("path".into(), Value::Text(path_str)),
        ]);

        let mut frames = Vec::new();

        // إذا كانت هناك إطارات استدعاء داخل المفسّر نعرضها من الأحدث للأقدم
        for (idx, (fn_name, span)) in self.interpreter.call_stack.iter().rev().enumerate() {
            let (l, c) = span.unwrap_or((self.last_line, self.last_column));
            frames.push(Value::map(vec![
                ("id".into(), Value::Number((idx + 1) as f64)),
                ("name".into(), Value::Text(fn_name.clone())),
                ("source".into(), source_obj.clone()),
                ("line".into(), Value::Number(l as f64)),
                ("column".into(), Value::Number(c as f64)),
            ]));
        }

        let main_id = frames.len() + 1;
        frames.push(Value::map(vec![
            ("id".into(), Value::Number(main_id as f64)),
            ("name".into(), Value::Text("<البرنامج الرئيسي>".into())),
            ("source".into(), source_obj),
            ("line".into(), Value::Number(self.last_line as f64)),
            ("column".into(), Value::Number(self.last_column as f64)),
        ]));

        let total = frames.len() as f64;
        let body = Value::map(vec![
            ("stackFrames".into(), Value::list(frames)),
            ("totalFrames".into(), Value::Number(total)),
        ]);
        vec![self.make_response(request_seq, "stackTrace", true, None, Some(body))]
    }

    fn handle_scopes(&mut self, request_seq: i64) -> Vec<String> {
        let locals_scope = Value::map(vec![
            ("name".into(), Value::Text("المتغيرات المحلية والعامة".into())),
            ("presentationHint".into(), Value::Text("locals".into())),
            ("variablesReference".into(), Value::Number(1.0)),
            ("expensive".into(), Value::Bool(false)),
        ]);
        let defs_scope = Value::map(vec![
            ("name".into(), Value::Text("الدوال والأصناف".into())),
            ("presentationHint".into(), Value::Text("registers".into())),
            ("variablesReference".into(), Value::Number(2.0)),
            ("expensive".into(), Value::Bool(false)),
        ]);
        let body = Value::map(vec![(
            "scopes".into(),
            Value::list(vec![locals_scope, defs_scope]),
        )]);
        vec![self.make_response(request_seq, "scopes", true, None, Some(body))]
    }

    fn handle_variables(&mut self, request_seq: i64, arguments: Option<&Value>) -> Vec<String> {
        let var_ref = arguments
            .and_then(|a| get_number(a, "variablesReference"))
            .unwrap_or(1.0) as i64;

        let mut vars: Vec<Value> = Vec::new();

        if var_ref == 1 {
            // جمع المتغيرات الحية من جميع النطاقات (النطاقات الأحدث تغطي الأقدم)
            let mut merged: BTreeMap<String, Value> = BTreeMap::new();
            for scope in &self.interpreter.scopes {
                for (name, val) in scope {
                    if !name.starts_with("__مصحح_") {
                        merged.insert(name.clone(), val.clone());
                    }
                }
            }

            if !merged.is_empty() {
                for (name, val) in merged {
                    vars.push(Value::map(vec![
                        ("name".into(), Value::Text(name)),
                        ("value".into(), Value::Text(format!("{}", val))),
                        ("type".into(), Value::Text(arabic_type_name(&val).into())),
                        ("variablesReference".into(), Value::Number(0.0)),
                    ]));
                }
            } else {
                // إذا توقفنا قبل تنفيذ أول سطر، نعرض التصريحات المكتشفة في الشيفرة
                let declared = collect_declared_variables(&self.statements);
                for name in declared {
                    vars.push(Value::map(vec![
                        ("name".into(), Value::Text(name)),
                        ("value".into(), Value::Text("<غير مهيّأ بعد>".into())),
                        ("type".into(), Value::Text("غير محدد".into())),
                        ("variablesReference".into(), Value::Number(0.0)),
                    ]));
                }
            }
        } else if var_ref == 2 {
            let mut fn_names: Vec<String> = self.interpreter.functions.keys().cloned().collect();
            fn_names.sort();
            for name in fn_names {
                if let Some(f) = self.interpreter.functions.get(&name) {
                    let sig = format!("دالة {}({})", f.name, f.params.join("، "));
                    vars.push(Value::map(vec![
                        ("name".into(), Value::Text(name)),
                        ("value".into(), Value::Text(sig)),
                        ("type".into(), Value::Text("دالة".into())),
                        ("variablesReference".into(), Value::Number(0.0)),
                    ]));
                }
            }

            let mut cls_names: Vec<String> = self.interpreter.classes.keys().cloned().collect();
            cls_names.sort();
            for name in cls_names {
                if let Some(c) = self.interpreter.classes.get(&name) {
                    let desc = match &c.parent {
                        Some(p) => format!("صنف {} يرث {}", c.name, p),
                        None => format!("صنف {}", c.name),
                    };
                    vars.push(Value::map(vec![
                        ("name".into(), Value::Text(name)),
                        ("value".into(), Value::Text(desc)),
                        ("type".into(), Value::Text("صنف".into())),
                        ("variablesReference".into(), Value::Number(0.0)),
                    ]));
                }
            }
        }

        let body = Value::map(vec![("variables".into(), Value::list(vars))]);
        vec![self.make_response(request_seq, "variables", true, None, Some(body))]
    }

    fn handle_unsupported_step(&mut self, request_seq: i64, command: &str) -> Vec<String> {
        let msg = format!(
            "الأمر '{}' غير مدعوم حالياً؛ المصحح يدعم التتبع على مستوى التعليمات العليا فقط",
            command
        );
        vec![self.make_response(request_seq, command, false, Some(&msg), None)]
    }

    fn handle_step(&mut self, request_seq: i64, command: &str) -> Vec<String> {
        let mut out = vec![self.make_response(request_seq, command, true, None, None)];

        if self.terminated || self.current_index >= self.statements.len() {
            self.terminated = true;
            out.push(self.make_exited_event(0));
            out.push(self.make_event("terminated", None));
            return out;
        }

        if !self.execute_current_statement(&mut out) {
            return out;
        }

        self.current_index += 1;
        if self.current_index < self.statements.len() {
            let (l, c) = stmt_span(&self.statements[self.current_index]);
            self.last_line = l;
            self.last_column = c;
            out.push(self.make_stopped_event("step", None));
        } else {
            self.terminated = true;
            out.push(self.make_exited_event(0));
            out.push(self.make_event("terminated", None));
        }

        out
    }

    fn handle_continue(&mut self, request_seq: i64) -> Vec<String> {
        let body = Value::map(vec![("allThreadsContinued".into(), Value::Bool(true))]);
        let mut out = vec![self.make_response(request_seq, "continue", true, None, Some(body))];

        if self.terminated || self.current_index >= self.statements.len() {
            self.terminated = true;
            out.push(self.make_exited_event(0));
            out.push(self.make_event("terminated", None));
            return out;
        }

        // نتجاوز نقطة التوقف الحالية التي توقفنا عندها بالفعل ثم نواصل حتى نقطة التوقف التالية أو النهاية
        self.run_until_stop(true, &mut out);
        out
    }

    fn handle_pause(&mut self, request_seq: i64) -> Vec<String> {
        let resp = self.make_response(request_seq, "pause", true, None, None);
        let stopped = self.make_stopped_event("pause", None);
        vec![resp, stopped]
    }

    fn handle_evaluate(&mut self, request_seq: i64, arguments: Option<&Value>) -> Vec<String> {
        let expr = match arguments.and_then(|a| get_string(a, "expression")) {
            Some(e) if !e.trim().is_empty() => e.trim().to_string(),
            _ => {
                return vec![self.make_response(
                    request_seq,
                    "evaluate",
                    false,
                    Some("التعبير المراد تقييمه فارغ"),
                    None,
                )];
            }
        };

        // 1) بحث مباشر في المتغيرات المعرّفة حالياً
        for scope in self.interpreter.scopes.iter().rev() {
            if let Some(val) = scope.get(&expr) {
                let body = Value::map(vec![
                    ("result".into(), Value::Text(format!("{}", val))),
                    ("type".into(), Value::Text(arabic_type_name(val).into())),
                    ("variablesReference".into(), Value::Number(0.0)),
                ]);
                return vec![self.make_response(request_seq, "evaluate", true, None, Some(body))];
            }
        }

        // 2) تقييم التعبير داخل المفسّر الحي بدون إبقاء متغير مؤقت
        let probe_var = "__مصحح_تقييم__";
        let probe_code = format!("دع {} = ({})", probe_var, expr);
        if self.interpreter.run_source(&probe_code).is_ok() {
            let val = self
                .interpreter
                .scopes
                .first_mut()
                .and_then(|s| s.remove(probe_var))
                .unwrap_or(Value::Null);
            let body = Value::map(vec![
                ("result".into(), Value::Text(format!("{}", val))),
                ("type".into(), Value::Text(arabic_type_name(&val).into())),
                ("variablesReference".into(), Value::Number(0.0)),
            ]);
            return vec![self.make_response(request_seq, "evaluate", true, None, Some(body))];
        }

        // 3) محاولة تنفيذ التعبير كجملة لسان كاملة
        match self.interpreter.run_source(&expr) {
            Ok(outputs) => {
                let result_text = if outputs.is_empty() {
                    "لاشيء".to_string()
                } else {
                    outputs.join("\n")
                };
                let body = Value::map(vec![
                    ("result".into(), Value::Text(result_text)),
                    ("type".into(), Value::Text("نص".into())),
                    ("variablesReference".into(), Value::Number(0.0)),
                ]);
                vec![self.make_response(request_seq, "evaluate", true, None, Some(body))]
            }
            Err(err) => vec![self.make_response(request_seq, "evaluate", false, Some(&err), None)],
        }
    }

    fn handle_restart(&mut self, request_seq: i64) -> Vec<String> {
        let Some(program) = self.program_path.clone() else {
            return vec![self.make_response(
                request_seq,
                "restart",
                false,
                Some("لا يوجد برنامج محمّل لإعادة تشغيله"),
                None,
            )];
        };

        let mut changed_bps = Vec::new();
        if let Err(err) = self.load_program(&program, &mut changed_bps) {
            return vec![self.make_response(request_seq, "restart", false, Some(&err), None)];
        }

        let mut out = vec![self.make_response(request_seq, "restart", true, None, None)];
        self.emit_breakpoint_changed_events(&changed_bps, &mut out);
        self.started = true;
        if self.stop_on_entry && !self.statements.is_empty() {
            let (l, c) = stmt_span(&self.statements[0]);
            self.last_line = l;
            self.last_column = c;
            out.push(self.make_stopped_event("entry", None));
        } else {
            self.run_until_stop(false, &mut out);
        }
        out
    }

    fn handle_terminate(&mut self, request_seq: i64) -> Vec<String> {
        self.terminated = true;
        let resp = self.make_response(request_seq, "terminate", true, None, None);
        let term = self.make_event("terminated", None);
        vec![resp, term]
    }

    fn handle_disconnect(&mut self, request_seq: i64) -> Vec<String> {
        self.disconnected = true;
        self.terminated = true;
        vec![self.make_response(request_seq, "disconnect", true, None, None)]
    }

    fn run_until_stop(&mut self, mut skip_first_breakpoint: bool, out: &mut Vec<String>) {
        while self.current_index < self.statements.len() {
            let (line, col) = stmt_span(&self.statements[self.current_index]);
            self.last_line = line;
            self.last_column = col;

            if !skip_first_breakpoint && self.has_breakpoint_at(line) {
                out.push(self.make_stopped_event("breakpoint", None));
                return;
            }
            skip_first_breakpoint = false;

            if !self.execute_current_statement(out) {
                return;
            }
            self.current_index += 1;
        }

        self.terminated = true;
        out.push(self.make_exited_event(0));
        out.push(self.make_event("terminated", None));
    }

    /// تنفيذ التعليمة الحالية؛ يُعيد `false` في حال حدوث خطأ تشغيل أدى لإنهاء الجلسة
    fn execute_current_statement(&mut self, out: &mut Vec<String>) -> bool {
        let stmt = self.statements[self.current_index].clone();
        let (line, col) = stmt_span(&stmt);
        self.last_line = line;
        self.last_column = col;

        let exec_result = match stmt.unwrap_spanned() {
            Stmt::Import(rel) if crate::modules::resolve_builtin_module(rel).is_none() => {
                // استيراد ملف نسبي مجاور للبرنامج
                let base_dir = self
                    .program_path
                    .as_deref()
                    .and_then(|p| Path::new(p).parent())
                    .unwrap_or_else(|| Path::new("."));
                let target = base_dir.join(rel);
                match crate::modules::load_file(&target) {
                    Ok(imported_stmts) => self.interpreter.run_program(&imported_stmts),
                    Err(e) => Err(e),
                }
            }
            _ => self.interpreter.run_program(&[stmt]),
        };

        match exec_result {
            Ok(printed_lines) => {
                for printed in printed_lines {
                    out.push(self.make_output_event("stdout", &format!("{}\n", printed)));
                }
                true
            }
            Err(err) => {
                out.push(self.make_output_event("stderr", &format!("{}\n", err)));
                self.terminated = true;
                out.push(self.make_exited_event(1));
                out.push(self.make_event("terminated", None));
                false
            }
        }
    }

    fn make_stopped_event(&mut self, reason: &str, description: Option<&str>) -> String {
        let mut pairs = vec![
            ("reason".into(), Value::Text(reason.into())),
            ("threadId".into(), Value::Number(1.0)),
            ("allThreadsStopped".into(), Value::Bool(true)),
        ];
        if let Some(desc) = description {
            pairs.push(("description".into(), Value::Text(desc.into())));
        }
        self.make_event("stopped", Some(Value::map(pairs)))
    }

    fn make_output_event(&mut self, category: &str, output: &str) -> String {
        let body = Value::map(vec![
            ("category".into(), Value::Text(category.into())),
            ("output".into(), Value::Text(output.into())),
        ]);
        self.make_event("output", Some(body))
    }

    fn make_exited_event(&mut self, exit_code: i64) -> String {
        let body = Value::map(vec![("exitCode".into(), Value::Number(exit_code as f64))]);
        self.make_event("exited", Some(body))
    }

    fn path_matches_program(&self, candidate: &str) -> bool {
        let Some(prog) = self.program_path.as_deref() else {
            return false;
        };
        if normalize_path(candidate) == normalize_path(prog) {
            return true;
        }
        if let (Ok(c1), Ok(c2)) = (fs::canonicalize(candidate), fs::canonicalize(prog)) {
            return c1 == c2;
        }
        false
    }

    fn has_breakpoint_at(&self, line: usize) -> bool {
        self.active_breakpoints.contains(&line)
    }
}

fn stmt_span(stmt: &Stmt) -> (usize, usize) {
    match stmt {
        Stmt::Spanned(_, line, col) => (*line, *col),
        _ => (1, 1),
    }
}

fn collect_top_level_hittable_lines(stmts: &[Stmt]) -> HashSet<usize> {
    let mut lines = HashSet::new();
    for stmt in stmts {
        let (line, _) = stmt_span(stmt);
        if line >= 1 {
            lines.insert(line);
        }
    }
    lines
}

fn normalize_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Ok(canon) = fs::canonicalize(trimmed) {
        return canon.to_string_lossy().to_string();
    }
    PathBuf::from(trimmed).to_string_lossy().replace('\\', "/")
}

fn collect_declared_variables(stmts: &[Stmt]) -> Vec<String> {
    let mut names = Vec::new();
    for s in stmts {
        match s.unwrap_spanned() {
            Stmt::Let(name, _) => {
                if !names.contains(name) {
                    names.push(name.clone());
                }
            }
            Stmt::LetUnpack(vars, _) => {
                for v in vars {
                    if !names.contains(v) {
                        names.push(v.clone());
                    }
                }
            }
            _ => {}
        }
    }
    names
}

fn arabic_type_name(val: &Value) -> &'static str {
    match val {
        Value::Number(_) => "عدد",
        Value::Text(_) => "نص",
        Value::Bool(_) => "منطقي",
        Value::Null => "فارغ",
        Value::List(_) => "قائمة",
        Value::Map(_) => "قاموس",
        Value::Tuple(_) => "صف",
        Value::Set(_) => "مجموعة",
        Value::Instance { .. } => "كائن",
        Value::Module { .. } => "وحدة",
        Value::Lambda { .. } => "دالة",
    }
}

fn get_field(value: &Value, key: &str) -> Option<Value> {
    if let Value::Map(pairs) = value {
        for (k, v) in pairs.borrow().iter() {
            if k == key {
                return Some(v.clone());
            }
        }
    }
    None
}

fn get_string(value: &Value, key: &str) -> Option<String> {
    match get_field(value, key) {
        Some(Value::Text(s)) => Some(s),
        _ => None,
    }
}

fn get_number(value: &Value, key: &str) -> Option<f64> {
    match get_field(value, key) {
        Some(Value::Number(n)) => Some(n),
        _ => None,
    }
}

fn get_bool(value: &Value, key: &str) -> Option<bool> {
    match get_field(value, key) {
        Some(Value::Bool(b)) => Some(b),
        _ => None,
    }
}

/// تشغيل خادم تصحيح الأخطاء (DAP) عبر الإدخال والإخراج القياسي (stdio)
#[cfg(not(target_arch = "wasm32"))]
pub fn run_dap_server() -> Result<(), String> {
    use std::io::{self, BufRead, Read, Write};

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    let mut server = DapServer::new();

    loop {
        let mut content_length: Option<usize> = None;

        // قراءة ترويسات DAP (مثل Content-Length)
        loop {
            let mut line = String::new();
            let bytes_read = stdin_lock
                .read_line(&mut line)
                .map_err(|e| format!("خطأ قراءة ترويسة DAP: {}", e))?;
            if bytes_read == 0 {
                return Ok(());
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            let lower = trimmed.to_ascii_lowercase();
            if let Some(rest) = lower.strip_prefix("content-length:") {
                if let Ok(len) = rest.trim().parse::<usize>() {
                    content_length = Some(len);
                }
            }
        }

        let length = match content_length {
            Some(l) if l > 0 => l,
            _ => continue,
        };

        let mut body_bytes = vec![0u8; length];
        stdin_lock
            .read_exact(&mut body_bytes)
            .map_err(|e| format!("خطأ قراءة محتوى رسالة DAP: {}", e))?;
        let body_str = String::from_utf8_lossy(&body_bytes);

        let responses = server.handle_message(&body_str);
        for resp in responses {
            let bytes = resp.as_bytes();
            write!(stdout_lock, "Content-Length: {}\r\n\r\n", bytes.len())
                .map_err(|e| format!("خطأ كتابة ترويسة DAP: {}", e))?;
            stdout_lock
                .write_all(bytes)
                .map_err(|e| format!("خطأ كتابة محتوى DAP: {}", e))?;
        }
        stdout_lock
            .flush()
            .map_err(|e| format!("خطأ تفريغ مخرجات DAP: {}", e))?;

        if server.is_disconnected() {
            return Ok(());
        }
    }
}

/// دالة بديلة على بيئة WebAssembly حيث لا يتوفر stdio التفاعلي
#[cfg(target_arch = "wasm32")]
pub fn run_dap_server() -> Result<(), String> {
    Err("خادم تصحيح الأخطاء (DAP) غير مدعوم في بيئة WebAssembly".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn write_temp_lisan_file(name: &str, content: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lisan_dap_{}_{}.lsn", name, nanos));
        fs::write(&path, content).expect("فشل إنشاء ملف اختبار لسان المؤقت");
        path
    }

    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    #[test]
    fn test_dap_full_lifecycle_with_breakpoint_step_and_continue() {
        let src = "دع س = 10\nدع ص = 20\nدع المجموع = س + ص\nاطبع(المجموع)\n";
        let file_path = write_temp_lisan_file("lifecycle", src);
        let path_str = file_path.to_string_lossy().to_string();
        let escaped_path = json_escape(&path_str);

        let mut server = DapServer::new();

        // 1. initialize
        let init_msgs = server.handle_message(r#"{"seq":1,"type":"request","command":"initialize","arguments":{}}"#);
        assert_eq!(init_msgs.len(), 2);
        let init_resp = parse_json(&init_msgs[0]).unwrap();
        assert_eq!(get_bool(&init_resp, "success"), Some(true));
        assert_eq!(get_string(&init_resp, "command").as_deref(), Some("initialize"));
        let body = get_field(&init_resp, "body").unwrap();
        assert_eq!(get_bool(&body, "supportsConfigurationDoneRequest"), Some(true));
        assert_eq!(get_bool(&body, "supportsEvaluateForHovers"), Some(true));
        assert_eq!(get_bool(&body, "supportsRestartRequest"), Some(true));
        assert_eq!(get_bool(&body, "supportsTerminateRequest"), Some(true));

        let init_evt = parse_json(&init_msgs[1]).unwrap();
        assert_eq!(get_string(&init_evt, "event").as_deref(), Some("initialized"));

        // 2. launch
        let launch_req = format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}"}}}}"#,
            escaped_path
        );
        let launch_msgs = server.handle_message(&launch_req);
        assert_eq!(launch_msgs.len(), 1);
        assert_eq!(get_bool(&parse_json(&launch_msgs[0]).unwrap(), "success"), Some(true));

        // 3. setBreakpoints (سطر 3: دع المجموع = س + ص)
        let bp_req = format!(
            r#"{{"seq":3,"type":"request","command":"setBreakpoints","arguments":{{"source":{{"path":"{}"}},"breakpoints":[{{"line":3}}]}}}}"#,
            escaped_path
        );
        let bp_msgs = server.handle_message(&bp_req);
        assert_eq!(bp_msgs.len(), 1);
        let bp_resp = parse_json(&bp_msgs[0]).unwrap();
        assert_eq!(get_bool(&bp_resp, "success"), Some(true));

        // 4. configurationDone -> يجب أن ينفذ السطرين 1 و 2 ثم يتوقف عند السطر 3
        let cfg_msgs = server.handle_message(r#"{"seq":4,"type":"request","command":"configurationDone"}"#);
        assert!(cfg_msgs.len() >= 2);
        let stopped_evt = parse_json(cfg_msgs.last().unwrap()).unwrap();
        assert_eq!(get_string(&stopped_evt, "event").as_deref(), Some("stopped"));
        let stopped_body = get_field(&stopped_evt, "body").unwrap();
        assert_eq!(get_string(&stopped_body, "reason").as_deref(), Some("breakpoint"));

        // 5. threads
        let th_msgs = server.handle_message(r#"{"seq":5,"type":"request","command":"threads"}"#);
        let th_resp = parse_json(&th_msgs[0]).unwrap();
        let th_body = get_field(&th_resp, "body").unwrap();
        if let Some(Value::List(threads)) = get_field(&th_body, "threads") {
            let list = threads.borrow();
            assert_eq!(list.len(), 1);
            assert_eq!(get_number(&list[0], "id"), Some(1.0));
            assert_eq!(get_string(&list[0], "name").as_deref(), Some("الخيط الرئيسي (لسان)"));
        } else {
            panic!("متوقع قائمة خيوط في استجابة threads");
        }

        // 6. stackTrace -> يجب أن يشير إلى السطر 3
        let st_msgs = server.handle_message(r#"{"seq":6,"type":"request","command":"stackTrace","arguments":{"threadId":1}}"#);
        let st_resp = parse_json(&st_msgs[0]).unwrap();
        let st_body = get_field(&st_resp, "body").unwrap();
        if let Some(Value::List(frames)) = get_field(&st_body, "stackFrames") {
            let list = frames.borrow();
            assert_eq!(get_number(&list[0], "line"), Some(3.0));
        } else {
            panic!("متوقع stackFrames");
        }

        // 7. scopes & variables -> يجب وجود س=10 و ص=20 قبل تنفيذ السطر 3
        let sc_msgs = server.handle_message(r#"{"seq":7,"type":"request","command":"scopes","arguments":{"frameId":1}}"#);
        assert_eq!(get_bool(&parse_json(&sc_msgs[0]).unwrap(), "success"), Some(true));

        let var_msgs = server.handle_message(r#"{"seq":8,"type":"request","command":"variables","arguments":{"variablesReference":1}}"#);
        let var_resp = parse_json(&var_msgs[0]).unwrap();
        let var_body = get_field(&var_resp, "body").unwrap();
        if let Some(Value::List(vars)) = get_field(&var_body, "variables") {
            let list = vars.borrow();
            assert!(list.iter().any(|v| get_string(v, "name").as_deref() == Some("س") && get_string(v, "value").as_deref() == Some("10")));
            assert!(list.iter().any(|v| get_string(v, "name").as_deref() == Some("ص") && get_string(v, "value").as_deref() == Some("20")));
        } else {
            panic!("متوقع قائمة متغيرات");
        }

        // 8. evaluate expression
        let eval_msgs = server.handle_message(r#"{"seq":9,"type":"request","command":"evaluate","arguments":{"expression":"س + ص"}}"#);
        let eval_resp = parse_json(&eval_msgs[0]).unwrap();
        assert_eq!(get_bool(&eval_resp, "success"), Some(true));
        let eval_body = get_field(&eval_resp, "body").unwrap();
        assert_eq!(get_string(&eval_body, "result").as_deref(), Some("30"));

        // 9. next -> ينفذ السطر 3 ويتوقف عند السطر 4
        let next_msgs = server.handle_message(r#"{"seq":10,"type":"request","command":"next","arguments":{"threadId":1}}"#);
        let next_stopped = parse_json(next_msgs.last().unwrap()).unwrap();
        assert_eq!(get_string(&next_stopped, "event").as_deref(), Some("stopped"));

        // 10. continue -> يطبع 30 وينتهي البرنامج
        let cont_msgs = server.handle_message(r#"{"seq":11,"type":"request","command":"continue","arguments":{"threadId":1}}"#);
        let has_output_30 = cont_msgs.iter().any(|m| {
            let v = parse_json(m).unwrap();
            get_string(&v, "event").as_deref() == Some("output")
                && get_field(&v, "body")
                    .and_then(|b| get_string(&b, "output"))
                    .map(|s| s.trim() == "30")
                    .unwrap_or(false)
        });
        let has_terminated = cont_msgs.iter().any(|m| {
            let v = parse_json(m).unwrap();
            get_string(&v, "event").as_deref() == Some("terminated")
        });
        assert!(has_output_30, "متوقع حدث output يحتوي على 30");
        assert!(has_terminated, "متوقع حدث terminated بعد إتمام البرنامج");

        // 11. disconnect
        let disc_msgs = server.handle_message(r#"{"seq":12,"type":"request","command":"disconnect"}"#);
        assert_eq!(get_bool(&parse_json(&disc_msgs[0]).unwrap(), "success"), Some(true));
        assert!(server.is_disconnected());

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_dap_run_to_completion_without_breakpoints() {
        let src = "اطبع(\"أهلاً\")\nاطبع(\"بلسان\")\n";
        let file_path = write_temp_lisan_file("run_all", src);
        let escaped_path = json_escape(&file_path.to_string_lossy());

        let mut server = DapServer::new();
        server.handle_message(r#"{"seq":1,"type":"request","command":"initialize"}"#);
        let launch_req = format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}"}}}}"#,
            escaped_path
        );
        server.handle_message(&launch_req);
        let msgs = server.handle_message(r#"{"seq":3,"type":"request","command":"configurationDone"}"#);

        let outputs: Vec<String> = msgs
            .iter()
            .filter_map(|m| {
                let v = parse_json(m).ok()?;
                if get_string(&v, "event").as_deref() == Some("output") {
                    get_field(&v, "body").and_then(|b| get_string(&b, "output"))
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(outputs, vec!["أهلاً\n".to_string(), "بلسان\n".to_string()]);
        assert!(msgs.iter().any(|m| {
            parse_json(m)
                .ok()
                .and_then(|v| get_string(&v, "event"))
                .as_deref()
                == Some("terminated")
        }));

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_dap_set_breakpoints_preserves_order_duplicates_invalid_lines_and_stable_ids() {
        let src = "// تعليق\nدع أ = 1\n\nدع ب = 2\nاطبع(أ + ب)\n";
        let file_path = write_temp_lisan_file("bp_order", src);
        let escaped_path = json_escape(&file_path.to_string_lossy());

        let mut server = DapServer::new();
        server.handle_message(r#"{"seq":1,"type":"request","command":"initialize"}"#);
        let launch_req = format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}"}}}}"#,
            escaped_path
        );
        let launch_msgs = server.handle_message(&launch_req);
        assert_eq!(get_bool(&parse_json(&launch_msgs[0]).unwrap(), "success"), Some(true));

        // السطور الصالحة العليا: 2، 4، 5. السطر 1 تعليق، 3 فارغ، 0 و -5 غير صالحين، 4 مكرر
        let bp_req = format!(
            r#"{{"seq":3,"type":"request","command":"setBreakpoints","arguments":{{"source":{{"path":"{}"}},"breakpoints":[{{"line":5}},{{"line":0}},{{"line":-5}},{{"line":4}},{{"line":4}},{{"line":1}},{{"line":3}}]}}}}"#,
            escaped_path
        );
        let bp_msgs = server.handle_message(&bp_req);
        assert_eq!(bp_msgs.len(), 1);
        let bp_resp = parse_json(&bp_msgs[0]).unwrap();
        let body = get_field(&bp_resp, "body").unwrap();
        let list = match get_field(&body, "breakpoints").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("متوقع مصفوفة breakpoints"),
        };

        assert_eq!(list.len(), 7);

        // 0: line 5 -> verified: true, id: 1
        assert_eq!(get_number(&list[0], "id"), Some(1.0));
        assert_eq!(get_bool(&list[0], "verified"), Some(true));
        assert_eq!(get_number(&list[0], "line"), Some(5.0));
        assert!(get_field(&list[0], "reason").is_none());

        // 1: line 0 -> verified: false, reason: "failed", line محذوف (بدون تحويل إلى 1)
        assert_eq!(get_number(&list[1], "id"), Some(2.0));
        assert_eq!(get_bool(&list[1], "verified"), Some(false));
        assert_eq!(get_string(&list[1], "reason").as_deref(), Some("failed"));
        assert!(get_field(&list[1], "line").is_none());
        assert!(get_string(&list[1], "message").is_some());

        // 2: line -5 -> verified: false, reason: "failed", line محذوف
        assert_eq!(get_number(&list[2], "id"), Some(3.0));
        assert_eq!(get_bool(&list[2], "verified"), Some(false));
        assert_eq!(get_string(&list[2], "reason").as_deref(), Some("failed"));
        assert!(get_field(&list[2], "line").is_none());
        assert!(get_string(&list[2], "message").is_some());

        // 3 & 4: duplicate line 4 -> محفوظان بالترتيب بمعرفين مختلفين
        assert_eq!(get_number(&list[3], "id"), Some(4.0));
        assert_eq!(get_bool(&list[3], "verified"), Some(true));
        assert_eq!(get_number(&list[3], "line"), Some(4.0));

        assert_eq!(get_number(&list[4], "id"), Some(5.0));
        assert_eq!(get_bool(&list[4], "verified"), Some(true));
        assert_eq!(get_number(&list[4], "line"), Some(4.0));

        // 5 & 6: comment line 1 & blank line 3 -> verified: false, reason: "failed"
        assert_eq!(get_number(&list[5], "id"), Some(6.0));
        assert_eq!(get_bool(&list[5], "verified"), Some(false));
        assert_eq!(get_string(&list[5], "reason").as_deref(), Some("failed"));
        assert_eq!(get_number(&list[5], "line"), Some(1.0));

        assert_eq!(get_number(&list[6], "id"), Some(7.0));
        assert_eq!(get_bool(&list[6], "verified"), Some(false));
        assert_eq!(get_string(&list[6], "reason").as_deref(), Some("failed"));
        assert_eq!(get_number(&list[6], "line"), Some(3.0));

        // إعادة إرسال setBreakpoints يجب ألا تعيد استخدام المعرفات السابقة (تبدأ من 8)
        let bp_req2 = format!(
            r#"{{"seq":4,"type":"request","command":"setBreakpoints","arguments":{{"source":{{"path":"{}"}},"breakpoints":[{{"line":2}}]}}}}"#,
            escaped_path
        );
        let bp_resp2 = parse_json(&server.handle_message(&bp_req2)[0]).unwrap();
        let list2 = match get_field(&get_field(&bp_resp2, "body").unwrap(), "breakpoints").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("متوقع مصفوفة breakpoints"),
        };
        assert_eq!(get_number(&list2[0], "id"), Some(8.0));
        assert_eq!(get_bool(&list2[0], "verified"), Some(true));

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_dap_nested_imported_and_foreign_breakpoints_unverified() {
        let helper_src = "دع قيمة_مستوردة = 99\n";
        let helper_path = write_temp_lisan_file("helper_mod", helper_src);
        let helper_name = helper_path.file_name().unwrap().to_string_lossy().to_string();

        // سطر 1: استيراد، سطر 2: تعريف دالة، سطر 3: سطر متداخل داخل الدالة، سطر 4: قوس الإغلاق، سطر 5: استدعاء أعلى
        let main_src = format!(
            "استورد \"{}\"\nدالة جمع(س، ص) {{\n    أرجع س + ص\n}}\nاطبع(جمع(قيمة_مستوردة، 1))\n",
            helper_name
        );
        let main_path = write_temp_lisan_file("main_nested", &main_src);
        let escaped_main = json_escape(&main_path.to_string_lossy());
        let escaped_helper = json_escape(&helper_path.to_string_lossy());

        let mut server = DapServer::new();
        server.handle_message(r#"{"seq":1,"type":"request","command":"initialize"}"#);
        let launch_msgs = server.handle_message(&format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}"}}}}"#,
            escaped_main
        ));
        assert_eq!(get_bool(&parse_json(&launch_msgs[0]).unwrap(), "success"), Some(true));

        // سطر 2 (بداية الدالة) وسطر 5 (الاستدعاء) أعلى مستوى، بينما سطر 3 متداخل داخل الدالة
        let bp_main = format!(
            r#"{{"seq":3,"type":"request","command":"setBreakpoints","arguments":{{"source":{{"path":"{}"}},"breakpoints":[{{"line":2}},{{"line":3}},{{"line":5}}]}}}}"#,
            escaped_main
        );
        let resp_main = parse_json(&server.handle_message(&bp_main)[0]).unwrap();
        let bps_main = match get_field(&get_field(&resp_main, "body").unwrap(), "breakpoints").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("متوقع breakpoints"),
        };
        assert_eq!(get_bool(&bps_main[0], "verified"), Some(true));
        assert_eq!(get_bool(&bps_main[1], "verified"), Some(false));
        assert_eq!(get_string(&bps_main[1], "reason").as_deref(), Some("failed"));
        assert_eq!(get_bool(&bps_main[2], "verified"), Some(true));

        // نقطة توقف في ملف خارجي/مستورد يجب أن تكون غير محققة
        let bp_foreign = format!(
            r#"{{"seq":4,"type":"request","command":"setBreakpoints","arguments":{{"source":{{"path":"{}"}},"breakpoints":[{{"line":1}}]}}}}"#,
            escaped_helper
        );
        let resp_foreign = parse_json(&server.handle_message(&bp_foreign)[0]).unwrap();
        let bps_foreign = match get_field(&get_field(&resp_foreign, "body").unwrap(), "breakpoints").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("متوقع breakpoints"),
        };
        assert_eq!(get_bool(&bps_foreign[0], "verified"), Some(false));
        assert_eq!(get_string(&bps_foreign[0], "reason").as_deref(), Some("failed"));

        let _ = fs::remove_file(main_path);
        let _ = fs::remove_file(helper_path);
    }

    #[test]
    fn test_dap_prelaunch_pending_reconciled_on_launch_and_restart() {
        let src_v1 = "// تعليق\nدع س = 1\nدع ص = 2\n";
        let file_path = write_temp_lisan_file("prelaunch_reconcile", src_v1);
        let escaped_path = json_escape(&file_path.to_string_lossy());

        let mut server = DapServer::new();
        server.handle_message(r#"{"seq":1,"type":"request","command":"initialize"}"#);

        // ضبط نقاط التوقف قبل launch: سطر 1 (تعليق) وسطر 3 (جملة) -> كلاهما pending قبل التحميل
        let bp_req = format!(
            r#"{{"seq":2,"type":"request","command":"setBreakpoints","arguments":{{"source":{{"path":"{}"}},"breakpoints":[{{"line":1}},{{"line":3}}]}}}}"#,
            escaped_path
        );
        let pre_resp = parse_json(&server.handle_message(&bp_req)[0]).unwrap();
        let pre_bps = match get_field(&get_field(&pre_resp, "body").unwrap(), "breakpoints").unwrap() {
            Value::List(l) => l.borrow().clone(),
            _ => panic!("متوقع breakpoints"),
        };
        assert_eq!(get_bool(&pre_bps[0], "verified"), Some(false));
        assert_eq!(get_string(&pre_bps[0], "reason").as_deref(), Some("pending"));
        assert_eq!(get_bool(&pre_bps[1], "verified"), Some(false));
        assert_eq!(get_string(&pre_bps[1], "reason").as_deref(), Some("pending"));
        let id_line1 = get_number(&pre_bps[0], "id");
        let id_line3 = get_number(&pre_bps[1], "id");

        // launch يجب أن يصالح نقاط التوقف ويُصدر أحداث breakpoint(changed) بالمعرفات نفسها
        let launch_req = format!(
            r#"{{"seq":3,"type":"request","command":"launch","arguments":{{"program":"{}","stopOnEntry":true}}}}"#,
            escaped_path
        );
        let launch_msgs = server.handle_message(&launch_req);
        let launch_seqs: Vec<f64> = launch_msgs
            .iter()
            .map(|m| get_number(&parse_json(m).unwrap(), "seq").unwrap())
            .collect();
        assert!(launch_seqs.windows(2).all(|w| w[0] < w[1]), "seq يجب أن يكون متزايداً بشكل صارم بعد launch: {:?}", launch_seqs);

        assert_eq!(get_bool(&parse_json(&launch_msgs[0]).unwrap(), "success"), Some(true));
        let bp_events: Vec<Value> = launch_msgs[1..]
            .iter()
            .map(|m| parse_json(m).unwrap())
            .filter(|v| get_string(v, "event").as_deref() == Some("breakpoint"))
            .collect();
        assert_eq!(bp_events.len(), 2);

        let ev0_bp = get_field(&get_field(&bp_events[0], "body").unwrap(), "breakpoint").unwrap();
        assert_eq!(get_number(&ev0_bp, "id"), id_line1);
        assert_eq!(get_bool(&ev0_bp, "verified"), Some(false));
        assert_eq!(get_string(&ev0_bp, "reason").as_deref(), Some("failed"));

        let ev1_bp = get_field(&get_field(&bp_events[1], "body").unwrap(), "breakpoint").unwrap();
        assert_eq!(get_number(&ev1_bp, "id"), id_line3);
        assert_eq!(get_bool(&ev1_bp, "verified"), Some(true));

        // تعديل الملف بحيث يصبح السطر 1 جملة تنفيذية والسطر 3 تعليقاً ثم تنفيذ restart
        let src_v2 = "دع س = 1\nدع ص = 2\n// تعليق الآن\n";
        fs::write(&file_path, src_v2).unwrap();

        let restart_msgs = server.handle_message(r#"{"seq":4,"type":"request","command":"restart"}"#);
        let restart_seqs: Vec<f64> = restart_msgs
            .iter()
            .map(|m| get_number(&parse_json(m).unwrap(), "seq").unwrap())
            .collect();
        assert!(restart_seqs.windows(2).all(|w| w[0] < w[1]), "seq يجب أن يكون متزايداً بشكل صارم بعد restart: {:?}", restart_seqs);

        assert_eq!(get_bool(&parse_json(&restart_msgs[0]).unwrap(), "success"), Some(true));
        let restart_bp_events: Vec<Value> = restart_msgs[1..]
            .iter()
            .map(|m| parse_json(m).unwrap())
            .filter(|v| get_string(v, "event").as_deref() == Some("breakpoint"))
            .collect();
        assert_eq!(restart_bp_events.len(), 2);

        let rev0_bp = get_field(&get_field(&restart_bp_events[0], "body").unwrap(), "breakpoint").unwrap();
        assert_eq!(get_number(&rev0_bp, "id"), id_line1);
        assert_eq!(get_bool(&rev0_bp, "verified"), Some(true));

        let rev1_bp = get_field(&get_field(&restart_bp_events[1], "body").unwrap(), "breakpoint").unwrap();
        assert_eq!(get_number(&rev1_bp, "id"), id_line3);
        assert_eq!(get_bool(&rev1_bp, "verified"), Some(false));
        assert_eq!(get_string(&rev1_bp, "reason").as_deref(), Some("failed"));

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_dap_rejects_step_in_and_step_out_without_advancing() {
        let src = "دع س = 1\nدع ص = 2\n";
        let file_path = write_temp_lisan_file("reject_step_in_out", src);
        let escaped_path = json_escape(&file_path.to_string_lossy());

        let mut server = DapServer::new();
        server.handle_message(r#"{"seq":1,"type":"request","command":"initialize"}"#);
        let launch_msgs = server.handle_message(&format!(
            r#"{{"seq":2,"type":"request","command":"launch","arguments":{{"program":"{}","stopOnEntry":true}}}}"#,
            escaped_path
        ));
        assert_eq!(get_bool(&parse_json(&launch_msgs[0]).unwrap(), "success"), Some(true));
        server.handle_message(r#"{"seq":3,"type":"request","command":"configurationDone"}"#);

        for (seq, cmd) in [(4, "stepIn"), (5, "stepOut")] {
            let req = format!(r#"{{"seq":{},"type":"request","command":"{}"}}"#, seq, cmd);
            let msgs = server.handle_message(&req);
            assert_eq!(msgs.len(), 1);
            let resp = parse_json(&msgs[0]).unwrap();
            assert_eq!(get_bool(&resp, "success"), Some(false));
            assert_eq!(get_string(&resp, "command").as_deref(), Some(cmd));
            assert!(get_string(&resp, "message").is_some());
        }

        // التحقق من بقاء المؤشر عند السطر 1 دون تقدم
        let st_msgs = server.handle_message(r#"{"seq":6,"type":"request","command":"stackTrace"}"#);
        let st_resp = parse_json(&st_msgs[0]).unwrap();
        if let Some(Value::List(frames)) = get_field(&get_field(&st_resp, "body").unwrap(), "stackFrames") {
            assert_eq!(get_number(&frames.borrow()[0], "line"), Some(1.0));
        } else {
            panic!("متوقع stackFrames");
        }

        let _ = fs::remove_file(file_path);
    }
}
