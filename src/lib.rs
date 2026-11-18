pub mod arduino;
pub mod ast;
pub mod bundle;
pub mod bytecode;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
pub mod diagnostics;
pub mod fmt;
pub mod json;
pub mod lexer;
pub mod lsp;
pub mod modules;
pub mod native;
pub mod parser;
pub mod pkg;
pub mod raspberry;
pub mod runtime;
pub mod stdlib_ext;
pub mod tester;
pub mod ui;
pub mod vm;
pub mod wasm;

pub use runtime::{Interpreter, Limits, Value};

pub fn run_file(path: impl AsRef<std::path::Path>) -> Result<Vec<String>, String> {
    Interpreter::default().run_file(path)
}

pub fn check_file(path: impl AsRef<std::path::Path>) -> Result<(), String> {
    modules::load_file(path.as_ref()).map(|_| ())
}

pub fn run(source: &str) -> Result<Vec<String>, String> {
    Interpreter::default().run_source(source)
}
