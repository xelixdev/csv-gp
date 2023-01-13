use crate::{cell::Cell, error::CSVError, file::read_encoded_file};
use std::{io, path::Path};

struct CSVReader<R> {
    reader: R,
    delimiter: String,
}

impl<R: io::BufRead> CSVReader<R> {
    fn new(reader: R, delimiter: impl Into<String>) -> Self {
        Self {
            reader,
            delimiter: delimiter.into(),
        }
    }

    /// Returns a owned iterator of all the csv lines
    fn into_lines(self) -> CSVLineIntoIter<R> {
        CSVLineIntoIter::new(self)
    }
}

struct CSVLineIntoIter<B> {
    lines: io::Lines<B>,
    delimiter: String,
}

impl<B: io::BufRead> CSVLineIntoIter<B> {
    fn new(reader: CSVReader<B>) -> Self {
        Self {
            lines: reader.reader.lines(),
            delimiter: reader.delimiter,
        }
    }
}

impl<B: io::BufRead> Iterator for CSVLineIntoIter<B> {
    type Item = Result<Vec<Cell>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_selection = String::new();

        loop {
            match self.lines.next() {
                None => return None,
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(line)) => {
                    current_selection.push_str(&line);

                    // if there is an odd number of quotes in the current string, the newline is in quotes
                    if current_selection.matches('"').count() % 2 == 1 {
                        current_selection.push('\n');
                    } else {
                        return Some(parse_cells(&current_selection, &self.delimiter));
                    }
                }
            }
        }
    }
}

fn parse_cells(row: &str, delimiter: &str) -> io::Result<Vec<Cell>> {
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

    Ok(cells)
}

pub fn parse_file<'a>(
    filename: impl AsRef<Path> + 'a,
    delimiter: &'a str,
    encoding: &'a str,
) -> Result<impl Iterator<Item = io::Result<Vec<Cell>>> + 'a, CSVError> {
    let reader = read_encoded_file(filename, encoding)?;

    let parser = CSVReader::new(reader, delimiter.to_string());

    Ok(parser.into_lines())
}

#[cfg(test)]
mod parse_rows_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row\nnext,row\n".as_bytes();
        let result = CSVReader::new(input, ",")
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        )
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "test,row\nnext,row".as_bytes();
        let result = CSVReader::new(input, ",")
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        )
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"\nnext,row".as_bytes();
        let result = CSVReader::new(input, ",")
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("\"row\n\"")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        )
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"\nnext,row".as_bytes();
        let result = CSVReader::new(input, ",")
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("\"\"\"row\"\"\"")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        );
    }

    #[test]
    fn test_blank_row() {
        let input = "test,row\n\nnext,row".as_bytes();
        let result = CSVReader::new(input, ",")
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![Cell::new("")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        );
    }
}

#[cfg(test)]
mod parse_cells_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row";
        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("row")]
        )
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"";

        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("\"row\n\"")]
        )
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"";

        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("\"\"\"row\"\"\"")]
        )
    }

    #[test]
    fn test_quoted_delimiter() {
        let input = "test,\"row,\"";

        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("\"row,\"")]
        )
    }
}
