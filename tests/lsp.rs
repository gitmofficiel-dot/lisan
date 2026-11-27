use lisan::lsp::{check_source_for_diagnostics, get_completion_items, get_hover_documentation, handle_json_rpc};

#[test]
fn test_lsp_initialize() {
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let resp = handle_json_rpc(req).expect("LSP should respond to initialize");
    assert!(resp.contains("\"capabilities\""));
    assert!(resp.contains("\"hoverProvider\":true"));
    assert!(resp.contains("\"completionProvider\""));
}

#[test]
fn test_lsp_diagnostics_on_syntax_error() {
    let code_with_err = "دع س = \nاطبع(س)";
    let diags = check_source_for_diagnostics(code_with_err);
    assert!(!diags.is_empty(), "Expected syntax error diagnostic");
    assert!(diags[0].message.contains("السطر"));
    assert_eq!(diags[0].severity, 1);
}

#[test]
fn test_lsp_did_open_publishes_diagnostics() {
    let req = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///test.lsn","languageId":"lisan","version":1,"text":"دع س = \nاطبع(س)"}}}"#;
    let resp = handle_json_rpc(req).expect("LSP should publish diagnostics on open error");
    assert!(resp.contains("textDocument/publishDiagnostics"));
    assert!(resp.contains("file:///test.lsn"));
    assert!(resp.contains("السطر"));
}

#[test]
fn test_lsp_hover_documentation() {
    let doc_let = get_hover_documentation("دع").expect("Hover for دع should exist");
    assert!(doc_let.contains("تعريف متغير"));

    let doc_with = get_hover_documentation("مع").expect("Hover for مع should exist");
    assert!(doc_with.contains("مدير السياق"));

    let doc_sin = get_hover_documentation("جا").expect("Hover for جا should exist");
    assert!(doc_sin.contains("الجيب"));

    let doc_pi = get_hover_documentation("ط").expect("Hover for ط should exist");
    assert!(doc_pi.contains("3.14159"));

    let doc_none = get_hover_documentation("متغير_غير_معروف");
    assert!(doc_none.is_none());
}

#[test]
fn test_lsp_completion_items() {
    let items = get_completion_items();
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "دع"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "دالة"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "مع"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "استورد"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "رياضيات"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "جا"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "سحاب"));
    assert!(items.iter().any(|(lbl, _, _)| *lbl == "مرقم"));

    let req = r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{}}"#;
    let resp = handle_json_rpc(req).expect("LSP should respond to completion request");
    assert!(resp.contains("items"));
    assert!(resp.contains("\"label\":\"دع\""));
    assert!(resp.contains("\"label\":\"استورد\""));
}

fn lsp_frame(json_body: &str) -> Vec<u8> {
    let bytes = json_body.as_bytes();
    let mut framed = format!("Content-Length: {}\r\n\r\n", bytes.len()).into_bytes();
    framed.extend_from_slice(bytes);
    framed
}

fn wait_for_exit_while_stdin_open(
    child: &mut std::process::Child,
    timeout: std::time::Duration,
) -> Option<std::process::ExitStatus> {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("try_wait failed") {
            return Some(status);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    None
}

#[test]
fn test_lsp_subprocess_exits_zero_after_shutdown_with_stdin_open() {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let mut child = Command::new(env!("CARGO_BIN_EXE_lisan"))
        .arg("--lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn lisan --lsp");

    let mut stdin = child.stdin.take().expect("stdin pipe missing");
    stdin
        .write_all(&lsp_frame(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        ))
        .expect("failed to write initialize");
    stdin
        .write_all(&lsp_frame(
            r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#,
        ))
        .expect("failed to write shutdown");
    stdin
        .write_all(&lsp_frame(
            r#"{"jsonrpc":"2.0","method":"exit","params":null}"#,
        ))
        .expect("failed to write exit");
    stdin.flush().expect("failed to flush stdin");

    // Keep stdin intentionally open until after the child process exits.
    let status = wait_for_exit_while_stdin_open(&mut child, Duration::from_secs(3));
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("lisan --lsp hung after shutdown+exit while stdin remained open");
    }
    drop(stdin);

    let exit_status = status.unwrap();
    assert_eq!(
        exit_status.code(),
        Some(0),
        "Expected exit code 0 after shutdown+exit, got {:?}",
        exit_status.code()
    );

    let mut stdout_buf = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout_buf)
        .expect("failed reading stdout");
    assert!(
        stdout_buf.contains(r#""id":2,"result":null"#),
        "Expected shutdown response on stdout, got: {}",
        stdout_buf
    );
}

#[test]
fn test_lsp_subprocess_exits_one_without_shutdown_with_stdin_open() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let mut child = Command::new(env!("CARGO_BIN_EXE_lisan"))
        .arg("--lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn lisan --lsp");

    let mut stdin = child.stdin.take().expect("stdin pipe missing");
    stdin
        .write_all(&lsp_frame(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        ))
        .expect("failed to write initialize");
    stdin
        .write_all(&lsp_frame(
            r#"{"jsonrpc":"2.0","method":"exit","params":null}"#,
        ))
        .expect("failed to write exit");
    stdin.flush().expect("failed to flush stdin");

    // Keep stdin intentionally open until after the child process exits.
    let status = wait_for_exit_while_stdin_open(&mut child, Duration::from_secs(3));
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("lisan --lsp hung after exit-without-shutdown while stdin remained open");
    }
    drop(stdin);

    let exit_status = status.unwrap();
    assert_eq!(
        exit_status.code(),
        Some(1),
        "Expected exit code 1 for exit without prior shutdown, got {:?}",
        exit_status.code()
    );
}
