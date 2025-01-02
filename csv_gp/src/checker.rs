use std::{cmp::Ordering, path::Path};

use crate::{
    cell::Cell, csv_details::CSVDetails, error::CSVError, parser::parse_file, scanner::Token,
    valid_file::save_valid_file,
};

/// Check the file located at `path`, interpreting the file with `delimiter` and `encoding`.
/// If `valid_rows_output_path` is passed, a file containing the valid rows will be written to the specified path.
pub fn check_file(
    path: impl AsRef<Path>,
    delimiter: char,
    encoding: &str,
    valid_rows_output_path: Option<impl AsRef<Path>>,
) -> Result<CSVDetails, CSVError> {
    let rows = parse_file(&path, delimiter, encoding)?;

    let csv_details = check_rows(rows)?;

    if let Some(valid_rows_path) = valid_rows_output_path {
        save_valid_file(&path, &csv_details, delimiter, encoding, valid_rows_path)?
    }

    Ok(csv_details)
}

fn check_rows(
    rows: impl Iterator<Item = Result<Vec<Cell>, CSVError>>,
) -> Result<CSVDetails, CSVError> {
    let mut csv_details = CSVDetails::new();

    for (i, cells_result) in rows.enumerate() {
        let cells = cells_result?;
        csv_details.column_count_per_line.push(cells.len());
        if i == 0 {
            csv_details.column_count = cells.len()
        }

        check_row(&mut csv_details, &cells, i);
    }

    Ok(csv_details)
}

fn check_row(csv_details: &mut CSVDetails, cells: &Vec<Cell>, row_number: usize) {
    let blank_row = cells.is_empty();

    // Cell checks
    let mut all_correctly_quoted = true;

    let mut has_quoted_quote = false;
    let mut has_quoted_newline = false;
    let mut has_quoted_delimiter = false;

    let mut all_empty = true;

    for cell in cells {
        all_correctly_quoted &= cell.correctly_quoted();

        has_quoted_quote |= !cell.is_empty() && cell.contains_double_quote();
        has_quoted_newline |= cell.contains(&Token::Newline);
        has_quoted_delimiter |= cell.contains(&Token::Delimiter);

        all_empty &= cell.is_empty();
        csv_details.invalid_character_count += cell.invalid_character_count();
    }

    // Length checks
    let mut too_many_columns = false;
    let mut too_few_columns = false;

    if !blank_row {
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

    if all_empty && !blank_row {
        csv_details.all_empty_rows.push(row_number);
    }

    if blank_row {
        csv_details.blank_rows.push(row_number);
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

    if all_correctly_quoted && !too_few_columns && !too_many_columns && !blank_row {
        csv_details.valid_rows.insert(row_number);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use pretty_assertions::assert_eq;

    use crate::cell;
    use Token::*;

    use super::*;

    fn check(rows: Vec<Vec<Cell>>, expected: CSVDetails) {
        let res = check_rows(rows.into_iter().map(Ok)).unwrap();
        assert_eq!(res, expected);
    }

    #[test]
    fn too_many_columns() {
        check(
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data), cell!(Data)],
            ],
            CSVDetails {
                too_many_columns: vec![1],
                row_count: 2,
                column_count: 2,
                column_count_per_line: vec![2, 3],
                valid_rows: HashSet::from([0]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn too_few_columns() {
        check(
            vec![vec![cell!(Data), cell!(Data)], vec![cell!(Data)]],
            CSVDetails {
                row_count: 2,
                column_count: 2,
                too_few_columns: vec![1],
                column_count_per_line: vec![2, 1],
                valid_rows: HashSet::from([0]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn all_correctly_quoted() {
        check(
            vec![vec![cell!(Data)], vec![cell!(Quote, Data)]],
            CSVDetails {
                incorrect_cell_quote: vec![1],
                row_count: 2,
                column_count: 1,
                column_count_per_line: vec![1, 1],
                valid_rows: HashSet::from([0]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_quoted_quote() {
        check(
            vec![
                vec![cell!(Data)],
                vec![cell!(Quote, Quote, Data)],
                vec![cell!(Quote, Quote, Quote, Data, Quote)],
            ],
            CSVDetails {
                quoted_quote: vec![1, 2],
                quoted_quote_correctly: vec![2],
                incorrect_cell_quote: vec![1],
                valid_rows: HashSet::from([0, 2]),
                row_count: 3,
                column_count: 1,
                column_count_per_line: vec![1, 1, 1],
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_quoted_newline() {
        check(
            vec![vec![cell!(Data)], vec![cell!(Quote, Data, Newline, Quote)]],
            CSVDetails {
                quoted_newline: vec![1],
                row_count: 2,
                column_count: 1,
                column_count_per_line: vec![1, 1],
                valid_rows: HashSet::from([0, 1]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_quoted_delimiter() {
        check(
            vec![
                vec![cell!(Data)],
                vec![cell!(Quote, Data, Delimiter, Quote)],
            ],
            CSVDetails {
                quoted_delimiter: vec![1],
                row_count: 2,
                column_count: 1,
                column_count_per_line: vec![1, 1],
                valid_rows: HashSet::from([0, 1]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_all_empty() {
        check(
            vec![
                vec![cell!(Data), cell!()],
                vec![cell!(), cell!(Quote, Quote)],
            ],
            CSVDetails {
                all_empty_rows: vec![1],
                row_count: 2,
                column_count: 2,
                column_count_per_line: vec![2, 2],
                valid_rows: HashSet::from([0, 1]),
                ..Default::default()
            },
        );
    }

    #[test]
    fn test_blank_row() {
        check(
            vec![vec![cell!(Data), cell!()], vec![]],
            CSVDetails {
                blank_rows: vec![1],
                row_count: 1,
                column_count: 2,
                column_count_per_line: vec![2, 0],
                valid_rows: HashSet::from([0]),
                ..Default::default()
            },
        );
    }
}
