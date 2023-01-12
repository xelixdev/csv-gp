use std::{cmp::Ordering, io, path::Path};

use crate::{
    cell::Cell, csv_details::CSVDetails, error::CSVError, parser::parse_file,
    valid_file::save_valid_file,
};

/// Check the file located at `path`, interpreting the file with `delimiter` and `encoding`.
/// If `valid_rows_output_path` is passed, a file containing the valid rows will be written to the specified path.
pub fn check_file(
    path: impl AsRef<Path>,
    delimiter: &str,
    encoding: &str,
    valid_rows_output_path: Option<&str>,
) -> Result<CSVDetails, CSVError> {
    let rows = parse_file(&path, delimiter, encoding)?;

    let csv_details = check_rows(rows, delimiter)?;

    if let Some(valid_rows_path) = valid_rows_output_path {
        save_valid_file(&path, &csv_details, delimiter, encoding, valid_rows_path)?
    }

    Ok(csv_details)
}

fn check_rows(
    rows: impl Iterator<Item = io::Result<Vec<Cell>>>,
    delimiter: &str,
) -> Result<CSVDetails, CSVError> {
    let mut csv_details = CSVDetails::new();

    for (i, cells_result) in rows.enumerate() {
        let cells = cells_result?;
        csv_details.column_count_per_line.push(cells.len());
        if i == 0 {
            csv_details.column_count = cells.len()
        }

        check_row(&mut csv_details, &cells, delimiter, i);
    }

    Ok(csv_details)
}

fn check_row(csv_details: &mut CSVDetails, cells: &Vec<Cell>, delimiter: &str, row_number: usize) {
    // Cell checks
    let mut all_correctly_quoted = true;

    let mut has_quoted_quote = false;
    let mut has_quoted_newline = false;
    let mut has_quoted_delimiter = false;

    let mut empty = true;

    for cell in cells {
        all_correctly_quoted &= cell.correctly_quoted();

        has_quoted_quote |= !cell.is_empty() && cell.contains("\"\"");
        has_quoted_newline |= cell.contains("\n");
        has_quoted_delimiter |= cell.contains(delimiter);

        empty &= cell.is_empty();
        csv_details.invalid_character_count += cell.invalid_character_count();
    }

    // Length checks
    let mut too_many_columns = false;
    let mut too_few_columns = false;

    if !empty {
        match cells.len().cmp(&csv_details.column_count) {
            Ordering::Greater => too_many_columns = true,
            Ordering::Less => too_few_columns = true,
            Ordering::Equal => (),
        }
    }

    // Write results
    if has_quoted_quote {
        csv_details.quoted_quote.push(row_number);
        if all_correctly_quoted {
            csv_details.quoted_quote_correctly.push(row_number);
        }
    }

    if has_quoted_newline {
        csv_details.quoted_newline.push(row_number);
    }

    if has_quoted_delimiter {
        csv_details.quoted_delimiter.push(row_number);
    }

    if empty {
        csv_details.all_empty_rows.push(row_number);
    } else {
        csv_details.row_count += 1;
    }

    if !all_correctly_quoted {
        csv_details.incorrect_cell_quote.push(row_number);
    }

    if too_few_columns {
        csv_details.too_few_columns.push(row_number);
    }

    if too_many_columns {
        csv_details.too_many_columns.push(row_number);
    }

    if all_correctly_quoted && !too_few_columns && !too_many_columns && !empty {
        csv_details.valid_rows.insert(row_number);
    }
}

#[cfg(test)]
mod check_row_tests {
    use super::*;

    #[test]
    fn test_too_many_columns() {
        let mut csv_details = CSVDetails::new();
        csv_details.column_count = 2;

        check_row(
            &mut csv_details,
            &vec![Cell::new("test"), Cell::new("row")],
            ",",
            0,
        );
        check_row(
            &mut csv_details,
            &vec![Cell::new("test"), Cell::new("row"), Cell::new("extra")],
            ",",
            1,
        );

        assert_eq!(csv_details.too_many_columns, vec![1])
    }

    #[test]
    fn test_too_few_columns() {
        let mut csv_details = CSVDetails::new();
        csv_details.column_count = 2;

        check_row(
            &mut csv_details,
            &vec![Cell::new("test"), Cell::new("row")],
            ",",
            0,
        );
        check_row(&mut csv_details, &vec![Cell::new("test")], ",", 1);

        assert_eq!(csv_details.too_few_columns, vec![1])
    }

    #[test]
    fn test_all_correctly_quoted() {
        let mut csv_details = CSVDetails::new();

        check_row(&mut csv_details, &vec![Cell::new("test")], ",", 0);
        check_row(&mut csv_details, &vec![Cell::new("\"test")], ",", 1);

        assert_eq!(csv_details.incorrect_cell_quote, vec![1])
    }

    #[test]
    fn test_quoted_quote() {
        let mut csv_details = CSVDetails::new();

        check_row(&mut csv_details, &vec![Cell::new("test")], ",", 0);
        check_row(&mut csv_details, &vec![Cell::new("\"\"test")], ",", 1);
        check_row(&mut csv_details, &vec![Cell::new("\"\"\"test\"")], ",", 2);

        assert_eq!(csv_details.quoted_quote, vec![1, 2]);
        assert_eq!(csv_details.quoted_quote_correctly, vec![2]);
    }

    #[test]
    fn test_quoted_newline() {
        let mut csv_details = CSVDetails::new();

        check_row(&mut csv_details, &vec![Cell::new("test")], ",", 0);
        check_row(&mut csv_details, &vec![Cell::new("\"test\n\"")], ",", 1);

        assert_eq!(csv_details.quoted_newline, vec![1]);
    }

    #[test]
    fn test_quoted_delimiter() {
        let mut csv_details = CSVDetails::new();

        check_row(&mut csv_details, &vec![Cell::new("test")], ",", 0);
        check_row(&mut csv_details, &vec![Cell::new("\"test,\"")], ",", 1);

        assert_eq!(csv_details.quoted_delimiter, vec![1]);
    }

    #[test]
    fn test_all_empty() {
        let mut csv_details = CSVDetails::new();

        check_row(
            &mut csv_details,
            &vec![Cell::new("test"), Cell::new("")],
            ",",
            0,
        );
        check_row(
            &mut csv_details,
            &vec![Cell::new(""), Cell::new("\"\"")],
            ",",
            1,
        );

        assert_eq!(csv_details.all_empty_rows, vec![1]);
        assert_eq!(csv_details.row_count, 1);
        assert_eq!(csv_details.too_few_columns, vec![]);
    }
}
