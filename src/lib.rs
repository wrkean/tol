use std::{
    fs,
    io::{self},
    path::Path,
    process::Command,
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

fn compile_c(c_code: &str) -> io::Result<()> {
    let filename = "generated.c";

    fs::write(filename, c_code)?;
    println!("Nagsulat sa: {filename}");

    let clang_format_exists = Command::new("which")
        .arg("clang-format")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);

    if clang_format_exists {
        println!("Finoformat ang C code...");
        let status = Command::new("clang-format")
            .args(["-i", filename])
            .status()?;

        if !status.success() {
            eprintln!("Nabigo ang clang-format");
        }
    } else {
        println!("Hindi nahanap ang clang-format. Hindi na magfoformat.");
    }

    println!("Kinocompile ang {filename} gamit ang gcc");

    let output_binary = Path::new("generated").with_extension("out");

    let status = Command::new("gcc")
        .args([filename, "-o", output_binary.to_str().unwrap()])
        .status()?;

    if status.success() {
        println!("Na-compile: ./generated.out");
    } else {
        eprintln!("Nabigong mag-compile");
    }

    Ok(())
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
    let mut main_module = parser.parse();

    let mut analyzer = SemanticAnalyzer::new(&mut main_module);
    analyzer.analyze();

    if !analyzer.has_error() {
        let mut codegen = CodeGenerator::new(&main_module);
        let c_code = codegen.generate();
        compile_c(c_code).unwrap_or_else(|err| panic!("{err}"));
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
