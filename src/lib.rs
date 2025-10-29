use std::{
    fs,
    io::{self},
    path::Path,
    process::Command,
};

use crate::{
    codegen::CodeGenerator,
    lexer::Lexer,
    parser::{Parser, module::Module},
    semantic_analyzer::SemanticAnalyzer,
};

mod codegen;
mod error;
mod lexer;
mod parser;
mod semantic_analyzer;
mod symbol;
mod toltype;

fn compile_c(c_code: &str) -> io::Result<()> {
    let build_dir = Path::new("build");
    if !build_dir.exists()
        && let Err(e) = fs::create_dir(build_dir)
    {
        eprintln!("Nabigong gumawa ng `build` folder");
        eprintln!("Error: {e}");
        return Err(e);
    }

    let filename = build_dir.join("generated.c");

    if let Err(e) = fs::write(&filename, c_code) {
        eprintln!("Nabigong gawin ang filename na {}", filename.display());
        return Err(e);
    }
    println!("Nagsulat sa: {}", filename.to_str().unwrap());

    let clang_format_exists = Command::new("which")
        .arg("clang-format")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);

    if clang_format_exists {
        println!("Finoformat ang C code...");
        let status = Command::new("clang-format")
            .args(["-i", filename.to_str().unwrap()])
            .status()?;

        if !status.success() {
            eprintln!("Nabigo ang clang-format");
        }
    } else {
        println!("Hindi nahanap ang clang-format. Hindi na magfoformat.");
    }

    println!("Kinocompile ang {} gamit ang gcc", filename.display());

    let output_binary = filename.with_extension("out");

    let status = Command::new("gcc")
        .args([
            "-w", // Supress english warnings
            filename.to_str().unwrap(),
            "-o",
            output_binary.to_str().unwrap(),
        ])
        .status()?;

    if status.success() {
        println!("Na-compile: {}", output_binary.display());
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

pub fn compile(source: String, path_to_source: String) {
    let mut main_module = Module::new(source, path_to_source);
    let mut should_compile = false;

    let mut lexer = Lexer::new(&mut main_module);
    lexer.lex();
    should_compile |= lexer.has_error();
    let tokens = &main_module.tokens;
    for tok in tokens {
        println!("{} <=> {:?}", tok.lexeme(), tok.kind());
    }

    let mut parser = Parser::new(&mut main_module);
    parser.parse();
    should_compile |= parser.has_error();

    let mut analyzer = SemanticAnalyzer::new(&mut main_module);
    analyzer.analyze();
    should_compile |= analyzer.has_error();

    if !should_compile {
        let mut codegen = CodeGenerator::new(&main_module);
        let c_code = codegen.generate();
        compile_c(c_code).unwrap_or_else(|err| panic!("{err}"));
    }
}
