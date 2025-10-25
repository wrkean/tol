use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

use crate::{
    codegen::CodeGenerator, lexer::Lexer, parser::Parser, semantic_analyzer::SemanticAnalyzer,
};

mod codegen;
mod error;
mod lexer;
mod parser;
mod semantic_analyzer;
mod symbol;
mod toltype;

fn compile_c(c_code: &str) {
    let mut child = Command::new("gcc")
        .args(["-x", "c", "-", "-o", "exe"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped()) // capture errors
        .spawn()
        .expect("Failed to start gcc");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(c_code.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    if output.status.success() {
        println!("Binary compiled: ./exe");
    } else {
        eprintln!(
            "Compilation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// Returns the source string and the canonical path to it
pub fn get_source(args: &[String]) -> Result<(String, String), String> {
    if args.len() < 2 {
        return Err(format!("Paggamit: {} <pangalan_ng_source_file>", args[0]));
    }

    let path_to_source = args[1].clone();
    let source = fs::read_to_string(&path_to_source);

    if source.is_err() {
        return Err(format!("Nabigong makuha ang path {}", path_to_source));
    }

    Ok((path_to_source, source.unwrap()))
}

pub fn compile(source: &str, path_to_source: &str) {
    let mut lexer = Lexer::new(source, path_to_source);
    let tokens = lexer.lex();
    for tok in tokens {
        println!("{} <=> {:?}", tok.lexeme(), tok.kind());
    }

    let mut parser = Parser::new(tokens, path_to_source);
    let ast = parser.parse();

    let mut analyzer = SemanticAnalyzer::new(&ast, path_to_source);
    analyzer.analyze();

    if !analyzer.has_error() {
        let mut codegen = CodeGenerator::new(
            &ast,
            analyzer.inferred_types(),
            analyzer.get_declared_array_types(),
        );
        let c_code = codegen.generate();
        compile_c(c_code);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn command_line_args() {
        let dummy_many_args = vec!["tol".to_string(), "path_to_source".to_string()];

        assert_eq!(
            get_source(&dummy_many_args).unwrap_err(),
            "Nabigong makuha ang path path_to_source"
        );

        let dummy_args = vec!["tol".to_string()];

        assert_eq!(
            get_source(&dummy_args).unwrap_err(),
            "Paggamit: tol <pangalan_ng_source_file>"
        );
    }
}
