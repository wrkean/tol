use std::{env, process};

// All variables are in English. The reason being
// is to make the code understandable to a wider audience.
fn main() {
    let args: Vec<String> = env::args().collect();

    let (path_to_source, source) = tol::get_source(&args).unwrap_or_else(|err_msg| {
        eprintln!("{}", err_msg);
        process::exit(1);
    });

    tol::compile(source, path_to_source);
}
