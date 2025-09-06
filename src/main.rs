use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <name>", args[0]);
        process::exit(1);
    }

    let path_to_source = &args[1];
    let source = fs::read_to_string(path_to_source).unwrap_or_else(|e| {
        eprintln!("Nabigong basahin ang {path_to_source}: {e}");
        process::exit(1);
    });
}
