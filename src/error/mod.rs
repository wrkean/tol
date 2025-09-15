use std::fs;

use colored::Colorize;

#[derive(Debug)]
pub struct CompilerError {
    kind: ErrorKind,
    message: String,
    note: Option<String>,
    help: Option<String>,
    line: usize,
    column: usize,
}

impl CompilerError {
    pub fn new(message: &str, kind: ErrorKind, line: usize, column: usize) -> Self {
        Self {
            message: message.to_string(),
            kind,
            note: None,
            help: None,
            line,
            column,
        }
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn display(&self, source_path: &str) {
        let canon_source_path = if let Ok(pathbuf) = fs::canonicalize(source_path) {
            pathbuf.to_string_lossy().into_owned()
        } else {
            source_path.to_owned()
        };

        eprintln!(
            "  [ {} || {} ]",
            source_path.bold(),
            canon_source_path.bold()
        );
        eprintln!(
            "  {}[{}:{}]: {}",
            match self.kind {
                ErrorKind::Error => "error".bold().red(),
                ErrorKind::Warning => "babala".bold().bright_yellow(),
                ErrorKind::Info => "inpormasyon".bold().purple(),
            },
            self.line,
            self.column,
            self.message
        );

        if let Some(help) = &self.help {
            eprintln!("  {}: {}", "tulong".bold().bright_green(), help)
        }

        if let Some(note) = &self.note {
            eprintln!("  {}: {}", "tala".bold().cyan(), note);
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Error,
    Warning,
    Info,
}
