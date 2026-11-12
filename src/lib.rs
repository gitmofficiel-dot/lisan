pub mod ast;
pub mod bundle;
pub mod bytecode;
pub mod diagnostics;
pub mod json;
pub mod lexer;
pub mod modules;
pub mod native;
pub mod parser;
pub mod runtime;
pub mod vm;
pub mod wasm;

pub use runtime::{Interpreter, Value};

pub fn run_file(path: impl AsRef<std::path::Path>) -> Result<Vec<String>, String> {
    Interpreter::default().run_file(path)
}

pub fn check_file(path: impl AsRef<std::path::Path>) -> Result<(), String> {
    modules::load_file(path.as_ref()).map(|_| ())
}

pub fn run(source: &str) -> Result<Vec<String>, String> {
    Interpreter::default().run_source(source)
}
