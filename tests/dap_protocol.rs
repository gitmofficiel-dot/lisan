use lisan::json::{parse_json, to_json_string};
use lisan::runtime::Value;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

static NEXT: AtomicU64 = AtomicU64::new(0);
const TIMEOUT: Duration = Duration::from_secs(10);

fn field(v: &Value, key: &str) -> Value {
    match v {
        Value::Map(m) => m
            .borrow()
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::Null),
        _ => Value::Null,
    }
}
fn number(v: &Value) -> i64 {
    if let Value::Number(n) = v {
        *n as i64
    } else {
        panic!("expected number: {:?}", v)
    }
}
fn text(s: &str) -> String {
    to_json_string(&Value::Text(s.into())).unwrap()
}
fn list(v: Value) -> Vec<Value> {
    if let Value::List(v) = v {
        v.borrow().clone()
    } else {
        panic!("expected list: {:?}", v)
    }
}

struct Client {
    child: Child,
    input: Option<ChildStdin>,
    messages: mpsc::Receiver<Result<String, String>>,
    reader: Option<thread::JoinHandle<()>>,
    seq: i64,
    received: i64,
    dir: PathBuf,
}
impl Client {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "lisan-dap-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_lisan"))
            .arg("--dap")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take();
        let stdout = child.stdout.take().unwrap();
        let (tx, messages) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut r = BufReader::new(stdout);
            loop {
                let frame = (|| -> Result<Option<String>, String> {
                    let mut length = None;
                    let mut header_bytes = 0;
                    loop {
                        let mut line = Vec::new();
                        let n = r
                            .by_ref()
                            .take(8193)
                            .read_until(b'\n', &mut line)
                            .map_err(|e| e.to_string())?;
                        if n == 0 {
                            return if header_bytes == 0 {
                                Ok(None)
                            } else {
                                Err("truncated header".into())
                            };
                        }
                        header_bytes += n;
                        if header_bytes > 8192 {
                            return Err("oversized header".into());
                        }
                        if line == b"\r\n" || line == b"\n" {
                            break;
                        }
                        let line = String::from_utf8(line).map_err(|e| e.to_string())?;
                        if let Some((_key, value)) = line
                            .split_once(':')
                            .filter(|(key, _)| key.trim().eq_ignore_ascii_case("content-length"))
                        {
                            if length.is_some() {
                                return Err("duplicate Content-Length".into());
                            }
                            length =
                                Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?);
                        }
                    }
                    let n = length.ok_or("missing Content-Length")?;
                    if n > 1024 * 1024 {
                        return Err("oversized body".into());
                    }
                    let mut body = vec![0; n];
                    r.read_exact(&mut body).map_err(|e| e.to_string())?;
                    Ok(Some(String::from_utf8(body).map_err(|e| e.to_string())?))
                })();
                match frame {
                    Ok(Some(body)) => {
                        if tx.send(Ok(body)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        break;
                    }
                }
            }
        });
        Self {
            child,
            input,
            messages,
            reader: Some(reader),
            seq: 0,
            received: 0,
            dir,
        }
    }
    fn source(&self, name: &str, source: &str) -> String {
        let p = self.dir.join(name);
        fs::write(&p, source).unwrap();
        p.to_str().unwrap().into()
    }
    fn recv(&mut self, deadline: Instant) -> Value {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let body = self
            .messages
            .recv_timeout(remaining)
            .expect("DAP frame timeout or EOF")
            .expect("invalid DAP frame");
        let value = parse_json(&body).expect("invalid DAP JSON");
        let seq = number(&field(&value, "seq"));
        assert!(
            seq > self.received,
            "outbound sequence did not increase: {} after {}",
            seq,
            self.received
        );
        self.received = seq;
        value
    }
    fn request(&mut self, command: &str, args: &str) -> Value {
        self.seq += 1;
        let body = format!(
            "{{\"seq\":{},\"type\":\"request\",\"command\":{},\"arguments\":{}}}",
            self.seq,
            text(command),
            args
        );
        let input = self.input.as_mut().unwrap();
        write!(input, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        input.flush().unwrap();
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let value = self.recv(deadline);
            if field(&value, "type") == Value::Text("response".into()) {
                assert_eq!(number(&field(&value, "request_seq")), self.seq);
                assert_eq!(field(&value, "command"), Value::Text(command.into()));
                return value;
            }
            assert_eq!(field(&value, "type"), Value::Text("event".into()));
        }
    }
    fn event(&mut self, name: &str) -> Value {
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let value = self.recv(deadline);
            assert_eq!(field(&value, "type"), Value::Text("event".into()));
            if field(&value, "event") == Value::Text(name.into()) {
                return field(&value, "body");
            }
        }
    }
    fn launch(&mut self, p: &str, entry: bool) {
        assert_eq!(
            field(&self.request("initialize", "{}"), "success"),
            Value::Bool(true)
        );
        let args = format!("{{\"program\":{},\"stopOnEntry\":{}}}", text(p), entry);
        assert_eq!(
            field(&self.request("launch", &args), "success"),
            Value::Bool(true)
        );
    }
    fn breakpoints(&mut self, p: &str, lines: &[i64]) -> Vec<Value> {
        let items = lines
            .iter()
            .map(|n| format!("{{\"line\":{}}}", n))
            .collect::<Vec<_>>()
            .join(",");
        let args = format!(
            "{{\"source\":{{\"path\":{}}},\"breakpoints\":[{}]}}",
            text(p),
            items
        );
        let response = self.request("setBreakpoints", &args);
        assert_eq!(field(&response, "success"), Value::Bool(true));
        list(field(&field(&response, "body"), "breakpoints"))
    }
    fn line(&mut self) -> i64 {
        let response = self.request("stackTrace", "{}");
        number(&field(
            &list(field(&field(&response, "body"), "stackFrames"))[0],
            "line",
        ))
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn framed_breakpoints_preserve_order_duplicates_and_invalid_locations() {
    let mut c = Client::new();
    let p = c.source(
        "main.lsn",
        "// comment\nدع س = 1\n\nدع ص = 2\nاطبع(س + ص)\n",
    );
    c.launch(&p, true);
    let points = c.breakpoints(&p, &[5, 0, -1, 4, 4, 1, 3]);
    assert_eq!(points.len(), 7);
    for (point, expected) in points
        .iter()
        .zip([true, false, false, true, true, false, false])
    {
        assert_eq!(field(point, "verified"), Value::Bool(expected));
    }
    for (point, line) in points.iter().zip([5, 0, -1, 4, 4, 1, 3]) {
        if line > 0 {
            assert_eq!(number(&field(point, "line")), line);
        }
    }
    let mut ids: Vec<_> = points.iter().map(|p| number(&field(p, "id"))).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 7);
    assert_eq!(field(&points[1], "line"), Value::Null);
    assert_eq!(field(&points[2], "line"), Value::Null);
}

#[test]
fn framed_prelaunch_reconciliation_keeps_ids_and_hits_real_breakpoint() {
    let mut c = Client::new();
    let p = c.source("main.lsn", "// comment\nدع س = 1\nدع ص = 2\n");
    c.request("initialize", "{}");
    let points = c.breakpoints(&p, &[3, 1]);
    assert!(points
        .iter()
        .all(|p| field(p, "verified") == Value::Bool(false)));
    let args = format!("{{\"program\":{}}}", text(&p));
    assert_eq!(
        field(&c.request("launch", &args), "success"),
        Value::Bool(true)
    );
    for (old, verified) in points.iter().zip([true, false]) {
        let event = c.event("breakpoint");
        let point = field(&event, "breakpoint");
        assert_eq!(field(&point, "id"), field(old, "id"));
        assert_eq!(field(&point, "verified"), Value::Bool(verified));
    }
    c.request("configurationDone", "{}");
    assert_eq!(
        field(&c.event("stopped"), "reason"),
        Value::Text("breakpoint".into())
    );
    assert_eq!(c.line(), 3);
}

#[test]
fn framed_unsupported_steps_do_not_move_and_next_keeps_variables() {
    let mut c = Client::new();
    let p = c.source("main.lsn", "دع س = 1\nدع ص = 2\nاطبع(س + ص)\n");
    c.launch(&p, true);
    c.request("configurationDone", "{}");
    c.event("stopped");
    assert_eq!(c.line(), 1);
    for command in ["stepIn", "stepOut"] {
        assert_eq!(
            field(&c.request(command, "{}"), "success"),
            Value::Bool(false)
        );
        assert_eq!(c.line(), 1);
    }
    c.request("next", "{}");
    c.event("stopped");
    assert_eq!(c.line(), 2);
    let response = c.request("evaluate", "{\"expression\":\"س\"}");
    assert_eq!(
        field(&field(&response, "body"), "result"),
        Value::Text("1".into())
    );
}

#[test]
fn framed_nested_and_foreign_breakpoints_are_not_armed() {
    let mut c = Client::new();
    let p = c.source(
        "main.lsn",
        "دالة جمع(س، ص) {\nأرجع س + ص\n}\nاطبع(جمع(3، 4))\n",
    );
    let other = c.source("other.lsn", "دع ع = 5\n");
    c.launch(&p, false);
    let points = c.breakpoints(&p, &[1, 2, 3, 4]);
    for (point, expected) in points.iter().zip([true, false, false, true]) {
        assert_eq!(field(point, "verified"), Value::Bool(expected));
    }
    assert_eq!(
        field(&c.breakpoints(&other, &[1])[0], "verified"),
        Value::Bool(false)
    );
    c.breakpoints(&p, &[]);
    c.request("configurationDone", "{}");
    c.event("terminated");
}

#[test]
fn framed_restart_reparses_source_and_preserves_breakpoint_ids() {
    let mut c = Client::new();
    let p = c.source("main.lsn", "// comment\nدع س = 1\nدع ص = 2\n");
    c.launch(&p, true);
    let old = c.breakpoints(&p, &[1, 3]);
    fs::write(&p, "دع س = 1\nدع ص = 2\n// comment\n").unwrap();
    assert_eq!(
        field(&c.request("restart", "{}"), "success"),
        Value::Bool(true)
    );
    for (old, expected) in old.iter().zip([true, false]) {
        let point = field(&c.event("breakpoint"), "breakpoint");
        assert_eq!(field(&point, "id"), field(old, "id"));
        assert_eq!(field(&point, "verified"), Value::Bool(expected));
    }
}

#[test]
fn framed_canonical_alias_can_clear_main_breakpoints() {
    let mut c = Client::new();
    let p = c.source("main.lsn", "دع س = 1\nدع ص = 2\n");
    fs::create_dir(c.dir.join("sub")).unwrap();
    let alias = c.dir.join("sub").join("..").join("main.lsn");
    c.launch(&p, false);
    assert_eq!(
        field(&c.breakpoints(alias.to_str().unwrap(), &[2])[0], "verified"),
        Value::Bool(true)
    );
    assert!(c.breakpoints(&p, &[]).is_empty());
    c.request("configurationDone", "{}");
    c.event("terminated");
}

#[test]
fn framed_disconnect_exits_cleanly_while_stdin_remains_open() {
    let mut c = Client::new();
    c.request("initialize", "{}");
    assert_eq!(
        field(&c.request("disconnect", "{}"), "success"),
        Value::Bool(true)
    );
    assert!(
        matches!(
            c.messages.recv_timeout(TIMEOUT),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ),
        "expected stdout EOF after disconnect"
    );
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(status) = c.child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        assert!(Instant::now() < deadline, "disconnect did not exit");
        thread::sleep(Duration::from_millis(10));
    }
}
