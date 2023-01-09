# CSV GP: Diagnose all your CSV issues

CSVs are a ubiquitous format for data transfer that are commonly [riddled with issues](https://donatstudios.com/Falsehoods-Programmers-Believe-About-CSVs). Most CSV libraries abort with an unhelpful error, CSV GP allows you to pinpoint these common issues with a CSV file, as well as export just the parseable lines from a file.

## Installation

CSV GP can be used in three ways.

### Standalone binary

1. [Install rust](https://www.rust-lang.org/tools/install)
2. Clone the repo
3. Run `cargo install --path csv_gp`
4. The `csv-gp` command will now be available to run, please see `csv-gp --help` for usage

### Rust library

Add the following to your `Cargo.toml`:

`csv-gp = { git = "https://github.com/xelixdev/csv-gp", rev = "<optional git tag>" }`

### Python library

### From package manager

The libary is uploaded to the `xelix` codeartifact repository, once you are authenticated to use that you can install with:

`pip install --index-url <codeartifact url> csv-gp`

### Compiling from source

1. [Install rust](https://www.rust-lang.org/tools/install)
2. Install (`pip install maturin`)
3. Clone the repo
4. `cd csv_gp_python && maturin develop`
