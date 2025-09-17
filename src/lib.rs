use std::fs;

use crate::{lexer::Lexer, parser::Parser};

mod error;
mod lexer;
mod parser;
mod toltype;

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
        println!("{}", tok.lexeme());
    }

    let mut parser = Parser::new(tokens, path_to_source);
    let ast = parser.parse();
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
