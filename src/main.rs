use lisan::{check_file, Interpreter};
use std::env;

fn main() {
    // نشغّل البرنامج على خيط بمكدس كبير ليتسع لعمق استدعاء مثل بايثون (1000).
    lisan::runtime::run_with_large_stack(real_main);
}

fn real_main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("لسان 1.0.0 | لغة برمجة عربية عصرية\nالاستخدام:\n  lisan ملف.lsn  يترجم إلى ملف آلة أصلي (الخيار الافتراضي)\n  lisan --vm ملف.lsn   تشغيل عبر آلة البايت كود الافتراضية السريعة\n  lisan --interpret ملف.lsn  تشغيل مباشر عبر المفسّر (بلا حد للخطوات)\n  lisan --sandbox ملف.lsn  تشغيل آمن بحدود محدودة للشيفرة غير الموثوقة\n  lisan --arduino ملف.lsn [--output sketch.ino]  ترجمة إلى كود أردوينو ومتحكمات دقيقة C++\n  lisan --mobile ملف.lsn [--output مجلد]  تشغيل وتصدير تطبيق هاتف ذكي (PWA)\n  lisan --init [اسم_المشروع]  إنشاء مشروع لسان جديد\n  lisan --web [منفذ]   تشغيل بيئة التطوير التفاعلية في المتصفح\n  lisan --check ملف.lsn  لفحص قواعد الملف واستيراداته\n  lisan --test ملف.lsn   تشغيل تحققات البرنامج\n  lisan --native ملف.lsn --output البرنامج  ترجمة إلى ملف آلة أصلي عبر Clang\n  lisan --emit-llvm ملف.lsn --output الملف.ll  إخراج LLVM IR\n  lisan --build ملف.lsn [--output الناتج]  بناء حزمة تطبيق تنفيذي مستقل\n  lisan --tokens ملف.lsn  عرض الرموز\n  lisan --ast ملف.lsn  عرض شجرة بناء التعليمات\n  lisan -e 'اطبع(\"مرحبا\")'\n  lisan                 فتح الطرفية التفاعلية REPL");
        return;
    }
    if args.len() == 1 && (args[0] == "-V" || args[0] == "--version") {
        println!("لسان 1.0.0");
        return;
    }
    if !args.is_empty() && args[0] == "--arduino" {
        if args.len() != 2 && !(args.len() == 4 && args[2] == "--output") {
            eprintln!("الاستخدام: lisan --arduino ملف.lsn [--output sketch.ino]");
            std::process::exit(2);
        }
        let input = std::path::Path::new(&args[1]);
        let target = if args.len() == 4 { Some(std::path::Path::new(&args[3])) } else { None };
        match lisan::arduino::build_arduino_file(input, target) {
            Ok(path) => println!("تم توليد كود الأردوينو والمتحكمات بنجاح: {}", path.display()),
            Err(err) => { eprintln!("خطأ في ترجمة الأردوينو: {}", err); std::process::exit(1); }
        }
        return;
    }
    if !args.is_empty() && args[0] == "--mobile" {
        if args.len() != 2 && !(args.len() == 4 && args[2] == "--output") {
            eprintln!("الاستخدام: lisan --mobile ملف.lsn [--output مجلد_التطبيق]");
            std::process::exit(2);
        }
        match Interpreter::with_limits(lisan::runtime::Limits::full()).run_file(&args[1]) {
            Ok(lines) => {
                for line in &lines {
                    println!("{}", line);
                }
                if args.len() == 4 {
                    if let Some(last_html) = lines.iter().rev().find(|l| l.contains("<!DOCTYPE html>")) {
                        match lisan::ui::export_mobile_pwa("تطبيق لسان", last_html, std::path::Path::new(&args[3])) {
                            Ok(p) => println!("تم تصدير تطبيق الهاتف إلى: {}", p.display()),
                            Err(e) => { eprintln!("خطأ في تصدير تطبيق الهاتف: {}", e); std::process::exit(1); }
                        }
                    }
                }
            }
            Err(err) => { eprintln!("خطأ في تطبيق الهاتف: {}", err); std::process::exit(1); }
        }
        return;
    }
    if !args.is_empty() && args[0] == "--init" {
        let project_name = if args.len() >= 2 { &args[1] } else { "مشروع_لسان" };
        if let Err(err) = init_project(project_name) {
            eprintln!("خطأ في تهيئة المشروع: {}", err);
            std::process::exit(1);
        }
        return;
    }
    if !args.is_empty() && args[0] == "--web" {
        let port: u16 = if args.len() >= 2 {
            args[1].parse().unwrap_or(8080)
        } else {
            8080
        };
        if let Err(err) = serve_web_playground(port) {
            eprintln!("خطأ في تشغيل خادم الويب: {}", err);
            std::process::exit(1);
        }
        return;
    }
    if args.len() >= 2 && (args[0] == "--native" || args[0] == "--emit-llvm") {
        if args.len() != 2 && !(args.len() == 4 && args[2] == "--output") {
            eprintln!("الاستخدام: lisan --native ملف.lsn [--output الناتج]، أو --emit-llvm");
            std::process::exit(2);
        }
        let input = std::path::Path::new(&args[1]);
        let default_name = if args[0] == "--emit-llvm" { "program.ll" }
                           else if cfg!(windows) { "program.exe" } else { "program" };
        let target = std::path::Path::new(if args.len() == 4 { args[3].as_str() } else { default_name });
        let result = if args[0] == "--emit-llvm" {
            lisan::native::emit_file(input).and_then(|ir| {
                std::fs::write(target, ir).map_err(|e| format!("تعذر حفظ LLVM IR: {}",e))?;
                Ok(target.to_path_buf())
            })
        } else {
            lisan::native::build_native(input, target)
        };
        match result {
            Ok(path) => println!("نجحت الترجمة الأصلية: {}", path.display()),
            Err(error) => { eprintln!("خطأ في الترجمة الأصلية: {}",error); std::process::exit(1); }
        }
        return;
    }
    if !args.is_empty() && args[0] == "--build" {
        if args.len() != 2 && args.len() != 4 {
            eprintln!("الاستخدام: lisan --build ملف.lsn [--output الناتج]");
            std::process::exit(2);
        }
        if args.len() == 4 && args[2] != "--output" {
            eprintln!("الاستخدام: lisan --build ملف.lsn [--output الناتج]");
            std::process::exit(2);
        }
        let target = if args.len() == 4 { Some(std::path::Path::new(&args[3])) } else { None };
        match lisan::bundle::build_executable(std::path::Path::new(&args[1]), target) {
            Ok(path) => println!("أُنشئ البرنامج المستقل: {}", path.display()),
            Err(err) => { eprintln!("خطأ في البناء: {}", err); std::process::exit(1); }
        }
        return;
    }
    if args.len() == 2 && (args[0] == "--tokens" || args[0] == "--ast") {
        let source = match std::fs::read_to_string(&args[1]) {
            Ok(text) => text,
            Err(err) => { eprintln!("تعذّرت قراءة الملف: {}", err); std::process::exit(1); }
        };
        let tokens = match lisan::lexer::tokenize(&source) {
            Ok(tokens) => tokens,
            Err(err) => { eprintln!("خطأ: {}", err); std::process::exit(1); }
        };
        if args[0] == "--tokens" {
            for token in tokens { println!("{}:{} {:?}", token.line, token.column, token.kind); }
        } else {
            match lisan::parser::parse(tokens) {
                Ok(ast) => println!("{:#?}", ast),
                Err(err) => { eprintln!("خطأ: {}", err); std::process::exit(1); }
            }
        }
        return;
    }
    if args.len() == 2 && args[0] == "--test" {
        match Interpreter::for_cli().run_file(&args[1]) {
            Ok(_) => println!("نجحت اختبارات لسان"),
            Err(err) => { eprintln!("فشلت اختبارات لسان: {}", err); std::process::exit(1); }
        }
        return;
    }
    if args.len() == 2 && args[0] == "--check" {
        match check_file(&args[1]) {
            Ok(()) => println!("نجح فحص ملفات لسان"),
            Err(err) => { eprintln!("خطأ: {}", err); std::process::exit(1); }
        }
        return;
    }
    if args.len() == 2 && args[0] == "-e" {
        execute(&mut Interpreter::for_cli(), &args[1]);
        return;
    }
    if args.len() == 2 && args[0] == "--vm" {
        let source = match std::fs::read_to_string(&args[1]) {
            Ok(s) => s,
            Err(e) => { eprintln!("تعذرت قراءة الملف: {}", e); std::process::exit(1); }
        };
        let mut vm = lisan::vm::VM::new();
        match vm.run_source(&source) {
            Ok(lines) => for output in lines { println!("{}", output); },
            Err(err) => { eprintln!("خطأ VM: {}", err); std::process::exit(1); }
        }
        return;
    }
    if args.len() == 2 && args[0] == "--interpret" {
        if let Err(err) = Interpreter::for_cli().run_file(&args[1]) {
            eprintln!("خطأ: {}", err);
            std::process::exit(1);
        }
        return;
    }
    if args.len() == 2 && args[0] == "--sandbox" {
        // تشغيل آمن بحدود المتصفح: مفيد للشيفرة غير الموثوقة.
        match Interpreter::default().run_file(&args[1]) {
            Ok(lines) => for output in lines { println!("{}", output); },
            Err(err) => { eprintln!("خطأ: {}", err); std::process::exit(1); }
        }
        return;
    }
    if args.len() == 1 && args[0].ends_with(".lsn") {
        let source = std::path::Path::new(&args[0]);
        let stem = source.file_stem().unwrap().to_string_lossy();
        let name = if cfg!(windows) { format!("{}.exe", stem) } else { stem.to_string() };
        match lisan::native::build_native(source, std::path::Path::new(&name)) {
            Ok(path) => println!("أُنشئ الملف التنفيذي الأصلي: {}", path.display()),
            Err(error) => { eprintln!("خطأ في الترجمة الأصلية: {}", error); std::process::exit(1); }
        }
        return;
    }
    if !args.is_empty() {
        eprintln!("وسائط غير صحيحة. اكتب lisan --help");
        std::process::exit(2);
    }
    start_repl();
}

fn start_repl() {
    use std::io::{self, Write};
    println!("لسان 1.0.0 | الطرفية التفاعلية للغة لسان");
    println!("اكتب تعليماتك البرمجية بالعربية واضغط Enter للتنفيذ.");
    println!("اكتب 'خروج' أو 'exit' للإنهاء، أو 'مسح' لإعادة تهيئة المتغيرات.\n");

    let mut interpreter = Interpreter::for_cli();
    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print!("لسان > ");
        let _ = io::stdout().flush();
        line.clear();
        if stdin.read_line(&mut line).is_err() || line.is_empty() {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "خروج" | "انهاء" | "exit" | "quit" | "q") {
            println!("مع السلامة!");
            break;
        }
        if trimmed == "مسح" || trimmed == "clear" {
            interpreter = Interpreter::for_cli();
            println!("تمت إعادة ضبط المتغيرات بنجاح.");
            continue;
        }
        match interpreter.run_source(trimmed) {
            Ok(outputs) => {
                for out in outputs {
                    println!("{}", out);
                }
            }
            Err(err) => {
                eprintln!("خطأ: {}", err);
            }
        }
    }
}

fn init_project(name: &str) -> Result<(), String> {
    use std::fs;
    use std::path::Path;

    let root = Path::new(name);
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("تعذّر إنشاء مجلد المشروع: {}", e))?;

    let manifest = format!(
        "{{\n  \"الاسم\": \"{}\",\n  \"الإصدار\": \"1.0.0\",\n  \"المدخل\": \"src/الرئيسي.lsn\"\n}}\n",
        name
    );
    fs::write(root.join("lisan.json"), manifest)
        .map_err(|e| format!("تعذّر إنشاء ملف الإعدادات lisan.json: {}", e))?;

    let main_code = r#"# مشروع لسان جديد

صنف تطبيق {
    دالة بناء(الاسم) {
        هذا.الاسم = الاسم
    }

    دالة ابدأ() {
        اطبع(تنسيق("مرحباً بك في {}!", هذا.الاسم))
    }
}

دع ت = جديد تطبيق("مشروعي الأول بلغة لسان")
ت.ابدأ()
"#;
    fs::write(src_dir.join("الرئيسي.lsn"), main_code)
        .map_err(|e| format!("تعذّر إنشاء الملف الرئيسي: {}", e))?;

    println!("تم إنشاء مشروع لسان بنجاح في: {}", root.display());
    println!("لتشغيل المشروع:\n  lisan --interpret {}/src/الرئيسي.lsn", name);
    Ok(())
}

fn serve_web_playground(port: u16) -> Result<(), String> {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| format!("تعذّر حجز المنفذ {}: {}", addr, e))?;

    println!("بيئة لسان التفاعلية تعمل على: http://{}", addr);
    println!("اضغط Ctrl+C لإيقاف الخادم.");

    for stream in listener.incoming().flatten() {
        let mut stream = stream;
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        if n == 0 {
            continue;
        }
        let req = String::from_utf8_lossy(&buf[..n]);
        let path = req
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        let (content_type, body) = if path == "/lisan.wasm" {
            let wasm = std::fs::read("web/lisan.wasm").unwrap_or_default();
            ("application/wasm", wasm)
        } else {
            let html = std::fs::read("web/index.html").unwrap_or_else(|_| {
                b"<h1>Lisan 1.0.0 Web Playground</h1>".to_vec()
            });
            ("text/html; charset=utf-8", html)
        };

        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            content_type,
            body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(&body);
    }
    Ok(())
}

fn execute(interpreter: &mut Interpreter, source: &str) {
    match interpreter.run_source(source) {
        Ok(lines) => for output in lines { println!("{}", output); },
        Err(err) => { eprintln!("خطأ: {}", err); std::process::exit(1); },
    }
}
