use std::cmp::Ordering;

use crate::{
    error::CSVError,
    file::read_encoded_file,
    parser::{parse_cells, parse_rows},
};

#[derive(Debug, Clone)]
pub struct CSVDetails {
    pub row_count: usize,
    pub column_count: usize,
    pub too_few_columns: Vec<usize>,
    pub too_many_columns: Vec<usize>,
    pub column_count_per_line: Vec<usize>,
    pub quoted_delimiter: Vec<usize>,
    pub quoted_newline: Vec<usize>,
    pub quoted_quote: Vec<usize>,
    pub quoted_quote_correctly: Vec<usize>,
    pub incorrect_cell_quote: Vec<usize>,
    pub all_empty_rows: Vec<usize>,
}

impl CSVDetails {
    fn new(row_count: usize) -> Self {
        Self {
            row_count,
            column_count: 0,
            column_count_per_line: vec![0; row_count],
            too_few_columns: Vec::new(),
            too_many_columns: Vec::new(),
            quoted_delimiter: Vec::new(),
            quoted_newline: Vec::new(),
            quoted_quote: Vec::new(),
            quoted_quote_correctly: Vec::new(),
            incorrect_cell_quote: Vec::new(),
            all_empty_rows: Vec::new(),
        }
    }

    pub fn header_messed_up(&self) -> bool {
        let bad_row_count = self.too_few_columns.len() + self.too_many_columns.len();
        self.row_count - 1 == bad_row_count
    }
}

pub fn check_file(
    filename: String,
    delimiter: &str,
    encoding: &str,
) -> Result<CSVDetails, CSVError> {
    let data = read_encoded_file(filename, encoding)?;

    let rows = parse_rows(&data, delimiter);
    let mut csv_details = CSVDetails::new(rows.len());

    for (i, row) in rows.iter().enumerate() {
        let cells = parse_cells(row, delimiter);

        csv_details.column_count_per_line[i] = cells.len();
        if i == 0 {
            csv_details.column_count = cells.len();
        }
        check_row(&mut csv_details, cells, delimiter, i)
    }

    Ok(csv_details)
}

fn check_row(csv_details: &mut CSVDetails, cells: Vec<String>, delimiter: &str, row_number: usize) {
    // Length checks
    match cells.len().cmp(&csv_details.column_count) {
        Ordering::Greater => csv_details.too_many_columns.push(row_number),
        Ordering::Less => csv_details.too_few_columns.push(row_number),
        Ordering::Equal => (),
    }

    // Cell checks
    let mut all_correctly_quoted = true;

    let mut has_quoted_quote = false;
    let mut has_quoted_newline = false;
    let mut has_quoted_delimiter = false;

    let mut empty = true;

    for cell in cells {
        all_correctly_quoted &= cell_correctly_quoted(&cell);

        has_quoted_quote |= cell != "\"\"" && cell.contains("\"\"");
        has_quoted_newline |= cell.contains('\n');
        has_quoted_delimiter |= cell.contains(delimiter);

        empty &= cell.is_empty() || cell == "\"\""
    }

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
    }

    if !all_correctly_quoted {
        csv_details.incorrect_cell_quote.push(row_number);
    }
}

fn cell_correctly_quoted(cell: &str) -> bool {
    if !cell.contains('"') {
        return true;
    }

    let mut starts = false;
    let mut ends = false;
    let mut stripped = cell;

    if let Some(s) = stripped.strip_prefix('"') {
        stripped = s;
        starts = true;
    }

    if let Some(s) = stripped.strip_suffix('"') {
        stripped = s;
        ends = true;
    }

    if !starts || !ends {
        return false;
    }

    if !stripped.contains('"') {
        return true;
    }

    !stripped.replace("\"\"", "").contains('"')
}

#[cfg(test)]
mod check_row_tests {
    use super::*;

    #[test]
    fn test_too_many_columns() {
        let mut csv_details = CSVDetails::new(1);
        csv_details.column_count = 2;

        check_row(&mut csv_details, vec!["test".into(), "row".into()], ",", 0);
        check_row(
            &mut csv_details,
            vec!["test".into(), "row".into(), "extra".into()],
            ",",
            1,
        );

        assert_eq!(csv_details.too_many_columns, vec![1])
    }

    #[test]
    fn test_too_few_columns() {
        let mut csv_details = CSVDetails::new(1);
        csv_details.column_count = 2;

        check_row(&mut csv_details, vec!["test".into(), "row".into()], ",", 0);
        check_row(&mut csv_details, vec!["test".into()], ",", 1);

        assert_eq!(csv_details.too_few_columns, vec![1])
    }

    #[test]
    fn test_all_correctly_quoted() {
        let mut csv_details = CSVDetails::new(1);

        check_row(&mut csv_details, vec!["test".into()], ",", 0);
        check_row(&mut csv_details, vec!["\"test".into()], ",", 1);

        assert_eq!(csv_details.incorrect_cell_quote, vec![1])
    }

    #[test]
    fn test_quoted_quote() {
        let mut csv_details = CSVDetails::new(1);

        check_row(&mut csv_details, vec!["test".into()], ",", 0);
        check_row(&mut csv_details, vec!["\"\"test".into()], ",", 1);
        check_row(&mut csv_details, vec!["\"\"\"test\"".into()], ",", 2);

        assert_eq!(csv_details.quoted_quote, vec![1, 2]);
        assert_eq!(csv_details.quoted_quote_correctly, vec![2]);
    }

    #[test]
    fn test_quoted_newline() {
        let mut csv_details = CSVDetails::new(1);

        check_row(&mut csv_details, vec!["test".into()], ",", 0);
        check_row(&mut csv_details, vec!["\"test\n\"".into()], ",", 1);

        assert_eq!(csv_details.quoted_newline, vec![1]);
    }

    #[test]
    fn test_quoted_delimiter() {
        let mut csv_details = CSVDetails::new(1);

        check_row(&mut csv_details, vec!["test".into()], ",", 0);
        check_row(&mut csv_details, vec!["\"test,\"".into()], ",", 1);

        assert_eq!(csv_details.quoted_delimiter, vec![1]);
    }

    #[test]
    fn test_all_empty() {
        let mut csv_details = CSVDetails::new(1);

        check_row(&mut csv_details, vec!["test".into(), "".into()], ",", 0);
        check_row(&mut csv_details, vec!["".into(), "\"\"".into()], ",", 1);

        assert_eq!(csv_details.all_empty_rows, vec![1]);
    }
}

#[cfg(test)]
mod cell_correctly_quoted_tests {
    use super::*;

    #[test]
    fn test_incorrect() {
        assert!(!cell_correctly_quoted("\"Anlagestiftung der UBS für \"Immobilien Schweiz\", Zürich, c/o UBS Fund Management AG\""))
    }

    #[test]
    fn test_incorrect_2() {
        assert!(!cell_correctly_quoted(
            "\"5\"379'319'026\",\"SINV-00110094\""
        ))
    }

    #[test]
    fn test_correct() {
        assert!(cell_correctly_quoted("\"Anlagestiftung der UBS für \"\"Immobilien Schweiz\"\", Zürich, c/o UBS Fund Management AG\""))
    }

    #[test]
    fn test_correct_2() {
        assert!(cell_correctly_quoted(
            "\"5\"\"379'319'026\"\",\"\"SINV-00110094\""
        ))
    }

    #[test]
    fn test_no_quotes() {
        assert!(cell_correctly_quoted("test"))
    }

    #[test]
    fn test_no_quotes_when_stripped() {
        assert!(cell_correctly_quoted("\"test\""))
    }

    #[test]
    fn test_does_not_start() {
        assert!(!cell_correctly_quoted("test\""))
    }

    #[test]
    fn test_does_not_end() {
        assert!(!cell_correctly_quoted("\"test"))
    }
}
