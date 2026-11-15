use lisan::run;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
fn test_text_module_encoding_and_padding() {
    let code = r#"
    استورد نصوص كـ ن

    دع نص_أصلي = "لغة لسان العربية الفصيحة 2026"
    دع مشفر = ن.ترميز_قاعدة64(نص_أصلي)
    دع مفكوك = ن.فك_قاعدة64(مشفر)
    تحقق_يساوي(مفكوك, نص_أصلي)

    دع رابط = "https://example.com/بحث?س=لسان&صفحة=1"
    دع رابط_مشفر = ن.ترميز_url(رابط)
    دع رابط_مفكوك = ن.فك_url(رابط_مشفر)
    تحقق_يساوي(رابط_مفكوك, رابط)

    دع رقم = "42"
    دع رقم_محشو = ن.حشو_يمين(رقم, 5, "0")
    تحقق_يساوي(رقم_محشو, "00042")

    دع كلمة = "لسان"
    دع كلمة_محشوة = ن.حشو_يسار(كلمة, 7, "*")
    تحقق_يساوي(كلمة_محشوة, "لسان***")
    "#;
    let res = run(code);
    assert!(res.is_ok(), "Test failed: {:?}", res);
}

#[test]
fn test_from_text_import() {
    let code = r#"
    من نصوص استورد ترميز_قاعدة64، فك_قاعدة64

    دع سر = "بيانات_سرية"
    دع كود = ترميز_قاعدة64(سر)
    تحقق_يساوي(فك_قاعدة64(كود), سر)
    "#;
    assert!(run(code).is_ok());
}

#[test]
fn test_network_module_http_get() {
    // Start a mock HTTP server on an OS-assigned port
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
    let port = listener.local_addr().expect("local addr").port();
    let (tx, rx) = std::sync::mpsc::channel();

    let server_handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\nأهلا بكم في شبكة لسان";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
            drop(stream);
            let _ = tx.send(());
        }
    });

    let code = format!(
        r#"
        استورد شبكة كـ ش
        دع رابط = "http://127.0.0.1:{}/api/test"
        دع رد = ش.طلب_وب(رابط)
        تحقق_يساوي(رد["حالة"], 200)
        تحقق_صحيح(رد["نجاح"])
        تحقق_يساوي(رد["نص"], "أهلا بكم في شبكة لسان")
        "#,
        port
    );

    let res = run(&code);
    let _ = rx.recv_timeout(std::time::Duration::from_secs(3));
    let _ = server_handle.join();
    assert!(res.is_ok(), "Network request failed: {:?}", res);
}

#[test]
fn test_network_module_tcp_send() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind tcp server");
    let port = listener.local_addr().expect("local addr").port();
    let (tx, rx) = std::sync::mpsc::channel();

    let server_handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).unwrap_or(0);
            let msg = String::from_utf8_lossy(&buf[..n]);
            let reply = format!("تم الاستلام: {}", msg);
            let _ = stream.write_all(reply.as_bytes());
            let _ = stream.flush();
            drop(stream);
            let _ = tx.send(());
        }
    });

    let code = format!(
        r#"
        استورد شبكة
        دع رد = شبكة.أرسل_tcp("127.0.0.1", {}, "سلام يا لسان")
        تحقق_يساوي(رد, "تم الاستلام: سلام يا لسان")
        "#,
        port
    );

    let res = run(&code);
    let _ = rx.recv_timeout(std::time::Duration::from_secs(3));
    let _ = server_handle.join();
    assert!(res.is_ok(), "TCP send failed: {:?}", res);
}
