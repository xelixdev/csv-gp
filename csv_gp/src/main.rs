use std::time::Instant;

use clap::Parser;

use csv_gp::checker::check_file;

#[derive(Parser, Default, Debug)]
struct Arguments {
    file_path: String,
    #[clap(default_value = ",", short, long)]
    delimiter: String,
}

fn main() {
    let args = Arguments::parse();

    let start = Instant::now();

    let result = check_file(args.file_path, &args.delimiter).unwrap();

    println!("Checking took {}s.", start.elapsed().as_secs());

    if result.header_messed_up() {
        println!("The header is totally messed up, no rows have the same number of columns as the header.");
    }

    if result.row_count <= 1 {
        println!("There is only one row row in the file.");
        return;
    }

    if result.column_count <= 1 {
        println!(
            "There is {} columns in the file, so the delimiter is almost surely wrong.",
            result.column_count
        );
        return;
    }

    println!(
        "There are {} rows in the file (including header), with {} columns (according to the header).",
        result.row_count, result.column_count
    );

    if !result.too_few_columns.is_empty() || !result.too_many_columns.is_empty() {
        println!(
            "There are {} rows with too many columns, and {} rows with too few columns.",
            result.too_many_columns.len(),
            result.too_few_columns.len()
        );
    } else {
        println!("All rows have the same number of columns.")
    }

    if !result.quoted_delimiter.is_empty() {
        println!(
            "There are {} lines with correctly quoted delimiter.",
            result.quoted_delimiter.len()
        )
    } else {
        println!("There are no rows with correctly quoted delimiter.")
    }

    if !result.quoted_newline.is_empty() {
        println!(
            "There are {} lines with correctly quoted newline.",
            result.quoted_newline.len()
        )
    } else {
        println!("There are no rows with correctly quoted newline.")
    }

    if !result.quoted_quote.is_empty() {
        println!(
            "There are {} lines with correctly quoted quote, out of that {} are absolutely correct.",
            result.quoted_quote.len(), result.quoted_quote_correctly.len()
        )
    } else {
        println!("There are no rows with correctly quoted quote.")
    }

    if !result.incorrect_cell_quote.is_empty() {
        println!(
            "There are {} lines with incorrect cell quotes.",
            result.incorrect_cell_quote.len()
        )
    } else {
        println!("There are no rows with incorrect cell quotes.")
    }
}
