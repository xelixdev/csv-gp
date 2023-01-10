use std::{process::exit, time::Instant};

use clap::Parser;

use csv_gp::checker::check_file;

#[derive(Parser, Default, Debug)]
struct Arguments {
    file_path: String,
    #[clap(short, long)]
    correct_rows: Option<String>,
    #[clap(default_value = ",", short, long)]
    delimiter: String,
    #[clap(default_value = "utf-8", short, long)]
    encoding: String,
}

fn main() {
    let args = Arguments::parse();

    let start = Instant::now();

    let result = check_file(
        args.file_path,
        &args.delimiter,
        &args.encoding,
        args.correct_rows.as_deref(),
    );

    println!("Checking took {}s.", start.elapsed().as_secs());

    match result {
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
        Ok(result) => {
            println!("{}", result);
            if let Some(path) = args.correct_rows {
                println!("Correct rows were saved to {}", path)
            }
        }
    }
}
