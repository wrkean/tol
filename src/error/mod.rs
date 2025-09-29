use std::fs;

use colored::Colorize;

#[derive(Debug)]
pub struct CompilerError {
    kind: ErrorKind,
    message: String,
    notes: Vec<String>,
    helps: Vec<String>,
    line: usize,
    column: usize,
}

impl CompilerError {
    pub fn new(message: &str, kind: ErrorKind, line: usize, column: usize) -> Self {
        Self {
            message: message.to_string(),
            kind,
            notes: Vec::new(),
            helps: Vec::new(),
            line,
            column,
        }
    }

    pub fn add_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }

    pub fn add_help(mut self, help: &str) -> Self {
        self.helps.push(help.to_string());
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
                ErrorKind::Error => "ERROR".bold().red(),
                ErrorKind::Warning => "BABALA".bold().bright_yellow(),
                ErrorKind::Info => "INPORMASYON".bold().purple(),
            },
            self.line,
            self.column,
            self.message
        );

        for help in &self.helps {
            eprintln!("  {}: {}", "tulong".bold().bright_green(), help);
        }

        for note in &self.notes {
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
