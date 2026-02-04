use std::io;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum CompilerError {
    #[error(transparent)]
    IO(#[from] io::Error),
}
