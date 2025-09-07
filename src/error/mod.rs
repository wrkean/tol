use std::{fs, path::PathBuf};

pub struct CompilerError<'a> {
    message: String,
    line: usize,
    column: usize,
    source_path: &'a str,
}

impl<'a> CompilerError<'a> {
    pub fn new(message: &str, line: usize, column: usize, source_path: &'a str) -> Self {
        Self {
            message: message.to_string(),
            line,
            column,
            source_path,
        }
    }

    pub fn display(&self) {
        let formatted = self.format_err();

        eprintln!("{formatted}");
    }

    fn format_err(&self) -> String {
        let err_msg_line = format!("nag-error: {}\n", self.message);
        let source_canonpath = fs::canonicalize(self.source_path).unwrap_or_else(|_| {
            eprintln!("Nabigong makuha ang canonical path ng {}", self.source_path);
            PathBuf::from(self.source_path)
        });
        let err_where_line = format!(
            "--> linyang {}, kolum {} sa {}\n",
            self.line,
            self.column,
            source_canonpath.to_str().unwrap()
        );

        format!("{}\n{}", err_msg_line, err_where_line)
    }
}
