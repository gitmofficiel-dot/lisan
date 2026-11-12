use lisan::{check_file, Interpreter};
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("لسان 0.9.2 | لغة برمجة عربية عصرية\nالاستخدام:\n  lisan ملف.lsn  يترجم إلى ملف آلة أصلي (الخيار الافتراضي)\n  lisan --vm ملف.lsn   تشغيل عبر آلة البايت كود الافتراضية السريعة\n  lisan --interpret ملف.lsn  تشغيل مباشر عبر المفسّر\n  lisan --check ملف.lsn  لفحص قواعد الملف واستيراداته\n  lisan --test ملف.lsn   تشغيل تحققات البرنامج\n  lisan --native ملف.lsn --output البرنامج  ترجمة إلى ملف آلة أصلي عبر Clang\n  lisan --emit-llvm ملف.lsn --output الملف.ll  إخراج LLVM IR\n  lisan --build ملف.lsn [--output الناتج]  بناء حزمة تطبيق تنفيذي مستقل\n  lisan --tokens ملف.lsn  عرض الرموز\n  lisan --ast ملف.lsn  عرض شجرة بناء التعليمات\n  lisan -e 'اطبع(\"مرحبا\")'\n  lisan                 فتح الطرفية التفاعلية REPL");
        return;
    }
    if args.len() == 1 && (args[0] == "-V" || args[0] == "--version") {
        println!("لسان 0.9.2");
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
        match Interpreter::default().run_file(&args[1]) {
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
        execute(&mut Interpreter::default(), &args[1]);
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
    println!("لسان 0.9.1 (نوفمبر 2026) | الطرفية التفاعلية للغة لسان");
    println!("اكتب تعليماتك البرمجية بالعربية واضغط Enter للتنفيذ.");
    println!("اكتب 'خروج' أو 'exit' للإنهاء، أو 'مسح' لإعادة تهيئة المتغيرات.\n");

    let mut interpreter = Interpreter::default();
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
            interpreter = Interpreter::default();
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

fn execute(interpreter: &mut Interpreter, source: &str) {
    match interpreter.run_source(source) {
        Ok(lines) => for output in lines { println!("{}", output); },
        Err(err) => { eprintln!("خطأ: {}", err); std::process::exit(1); },
    }
}
