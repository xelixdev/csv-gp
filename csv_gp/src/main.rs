use std::{process::exit, time::Instant};

use clap::Parser;

use csv_gp::checker::check_file;

/// CSV GP: Diagnose all your CSV issues
#[derive(Parser, Default, Debug)]
struct Arguments {
    /// Path to file to check
    file_path: String,
    /// Path to output the correct rows in the file to
    #[clap(short, long)]
    correct_rows_path: Option<String>,
    #[clap(default_value = ",", short, long)]
    delimiter: char,
    #[clap(default_value = "utf-8", short, long)]
    encoding: String,
}

fn main() {
    let args = Arguments::parse();

    let start = Instant::now();

    let result = check_file(
        args.file_path,
        args.delimiter,
        &args.encoding,
        args.correct_rows_path.as_deref(),
    );

    println!("Checking took {}s.", start.elapsed().as_secs());

    match result {
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
        Ok(r) => {
            println!("{}", r);
            if let Some(path) = args.correct_rows_path {
                println!("Correct rows were saved to {}", path)
            }
        }
    }
}
