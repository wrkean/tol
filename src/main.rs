use std::{char::ParseCharError, env, fs, process};

use crate::{lexer::Lexer, parser::Parser};

mod error;
mod lexer;
mod parser;
mod tol;

// All variables are in English. The reason being
// is to make the code understandable to a wider audience.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Paggamit: {} <pangalan_ng_source_file>", args[0]);
        process::exit(1);
    }

    let path_to_source = &args[1];
    let source = fs::read_to_string(path_to_source).unwrap_or_else(|e| {
        eprintln!("Nabigong basahin ang {path_to_source}: {e}");
        process::exit(1);
    });

    let mut lexer = Lexer::new(&source, path_to_source);
    let tokens = lexer.lex();
    for tok in tokens {
        println!("{}", tok.lexeme());
    }

    let mut parser = Parser::new(tokens, path_to_source);
    let ast = parser.parse();
}
