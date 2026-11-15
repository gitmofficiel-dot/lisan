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
