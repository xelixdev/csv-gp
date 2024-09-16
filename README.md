# CSV GP: Diagnose all your CSV issues

CSVs are a ubiquitous format for data transfer that are commonly [riddled with issues](https://donatstudios.com/Falsehoods-Programmers-Believe-About-CSVs). Most CSV libraries abort with an unhelpful error, CSV GP allows you to pinpoint these common issues with a CSV file, as well as export just the parsable lines from a file.

## Installation

CSV GP can be used in three ways.

### Standalone binary

1. [Install rust](https://www.rust-lang.org/tools/install)
2. Clone the repo and navigate into it
3. Run `cargo install --path csv_gp`
4. The `csv-gp` command will now be available to run, please see `csv-gp --help` for usage

### Rust library

Add the following to your `Cargo.toml`:

`csv-gp = { git = "https://github.com/xelixdev/csv-gp", rev = "<optional git tag>" }`

### Python library

### From package manager

The library is available on PyPI, at https://pypi.org/project/csv-gp/ so you can just run:

`pip install csv-gp`

### Compiling from source

1. [Install rust](https://www.rust-lang.org/tools/install)
2. Install (`pip install maturin`)
3. Clone the repo
4. Run `make all`
5. `cd csv_gp_python && maturin develop`

## Usage

## Rust standalone binary

After installing the binary, the default usage is running `csv-gp $FILE`. This will print a diagnosis of the file. The command provides options to change the delimiter and the encoding of the file. See `csv-gp -h` for details.

Another option provided is `--correct-rows-path` which will export only the correct rows to the provided path.

## Python library

The python library exposes two main functions, `check_file` and `get_rows`.

The check file function takes a path to file, the delimiter and the encoding (see https://github.com/xelixdev/csv-gp/blob/0f77c62841509c134a3bbe06ec178426e9c5aa10/csv_gp_python/csv_gp.pyi) and returns an instance of a class `CSVDetails` which provides details about the file. See the same file to see all the available attributes and their names/types.
If the `valid_rows_output_path` argument is provided to the function, only the correct rows will be exported to that path.

The get_rows once again takes a path to file, the delimiter and the encoding and additionally a list of row numbers. The function will then return the parsed cells for given rows. See the above file for the exact typing of the parameter and returned values.

## Releasing a new version of the Python lib

1. Update version numbers in `csv_gp_python/Cargo.toml` and `csv_gp/Cargo.toml`
2. Merge this change into main
3. Create a new release on GitHub, creating a tag in the form `vX.Y.Z`
4. The 'Publish' pipeline should begin running, and the new version will be published

## Running tests

### Running Rust tests

Run `cargo test`.

### Running Python tests

Follow the instructions on compiling from source. Then you can run `pytest`.
