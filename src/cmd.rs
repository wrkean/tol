use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    /// Path to the source code to compile
    #[arg(help = "Path ng source code na ico-compile")]
    input_path: PathBuf,
}
