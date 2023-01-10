// (rob) This is a transliteration of the code in csv.py - this should be carcinized 🦀

use std::path::Path;

use crate::{cell::Cell, error::CSVError, file::read_encoded_file};

pub(crate) fn parse_file(
    filename: impl AsRef<Path>,
    delimiter: &str,
    encoding: &str,
) -> Result<Vec<Vec<Cell>>, CSVError> {
    let data = read_encoded_file(filename, encoding)?;

    let rows = parse_rows(&data, delimiter);

    Ok(rows.iter().map(|r| parse_cells(r, delimiter)).collect())
}

fn parse_rows(text: &str, delimiter: &str) -> Vec<String> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut rows = Vec::new();

    let mut opened_quotes = false;
    let mut current_selection = String::new();

    let mut i = 0usize;
    while i < chars.len() {
        let current_char = chars[i];

        // Reached a newline not in quotes
        if current_char == '\n' && !opened_quotes {
            rows.push(current_selection.clone());
            current_selection = String::new();
        } else if current_char == '"' {
            current_selection.push(current_char);
            if let Some(next_char) = chars.get(i + 1) {
                if next_char == &'"' {
                    current_selection.push(*next_char);
                    i += 1;
                } else if current_selection == "\""
                    || (current_selection.ends_with(&format!("{}{}", delimiter, "\""))
                        && !opened_quotes)
                {
                    opened_quotes = true;
                } else if next_char.to_string() == delimiter || next_char == &'\n' {
                    opened_quotes = false;
                }
            }
        } else {
            current_selection.push(current_char);
        }

        i += 1;
    }

    // There was no trailing newline, append the rest of the data
    if !current_selection.is_empty() {
        rows.push(current_selection);
    }

    rows
}

fn parse_cells(row: &str, delimiter: &str) -> Vec<Cell> {
    let chars = row.chars().collect::<Vec<_>>();

    let mut cells = Vec::new();
    let mut current_selection = String::new();
    let mut opened_quote = false;

    let mut i = 0;
    while i < chars.len() {
        let current_char = chars[i];
        let next_char = chars.get(i + 1);

        if current_char.to_string() != delimiter && current_char != '"' {
            current_selection.push(current_char);
            i += 1;
            continue;
        }

        if let Some(next_char) = next_char {
            if current_char == '"' && next_char == &'"' {
                current_selection.push_str("\"\"");
                i += 2;
                continue;
            }
        }

        if current_char == '"' && (next_char.is_none() || next_char != Some(&'"')) {
            current_selection.push('"');
            opened_quote = !opened_quote;
            i += 1;
            continue;
        }

        if current_char.to_string() == delimiter {
            if opened_quote {
                current_selection.push(current_char);
            } else {
                cells.push(Cell::new(current_selection.clone()));
                current_selection = String::new();
            }

            i += 1;
            continue;
        }
    }

    cells.push(Cell::new(current_selection));

    cells
}

#[cfg(test)]
mod parse_rows_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row\nnext,row\n";

        assert_eq!(parse_rows(input, ","), vec!["test,row", "next,row"])
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "test,row\nnext,row";

        assert_eq!(parse_rows(input, ","), vec!["test,row", "next,row"])
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"\nnext,row";

        assert_eq!(parse_rows(input, ","), vec!["test,\"row\n\"", "next,row"])
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"\nnext,row";

        assert_eq!(
            parse_rows(input, ","),
            vec!["test,\"\"\"row\"\"\"", "next,row"]
        )
    }
}

#[cfg(test)]
mod parse_cells_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row";
        assert_eq!(
            parse_cells(input, ","),
            vec![Cell::new("test"), Cell::new("row")]
        )
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"";

        assert_eq!(
            parse_cells(input, ","),
            vec![Cell::new("test"), Cell::new("\"row\n\"")]
        )
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"";

        assert_eq!(
            parse_cells(input, ","),
            vec![Cell::new("test"), Cell::new("\"\"\"row\"\"\"")]
        )
    }

    #[test]
    fn test_quoted_delimiter() {
        let input = "test,\"row,\"";

        assert_eq!(
            parse_cells(input, ","),
            vec![Cell::new("test"), Cell::new("\"row,\"")]
        )
    }
}
