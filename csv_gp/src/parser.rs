use crate::{cell::Cell, error::CSVError, file::read_encoded_file};
use std::{io, path::Path};

struct CSVReader<R> {
    reader: R,
    delimiter: char,
}

impl<R: io::BufRead> CSVReader<R> {
    fn new(reader: R, delimiter: char) -> Self {
        Self { reader, delimiter }
    }

    /// Returns a owned iterator of all the csv lines
    fn into_lines(self) -> CSVLineIntoIter<R> {
        CSVLineIntoIter::new(self)
    }
}

struct CSVLineIntoIter<B> {
    lines: io::Lines<B>,
    delimiter: char,
}

impl<B: io::BufRead> CSVLineIntoIter<B> {
    fn new(reader: CSVReader<B>) -> Self {
        Self {
            lines: reader.reader.lines(),
            delimiter: reader.delimiter,
        }
    }
}

/// Determines if a passed string has fully closed quotes or not
fn has_open_quotes(s: &str, delimiter: char) -> bool {
    let mut is_open = false;
    let mut prev_char: Option<char> = None;

    let mut chars = s.chars().peekable();
    while let Some(current_char) = chars.next() {
        match (prev_char, current_char, chars.peek()) {
            // If there's a quoted-quote skip it
            (_, '"', Some('"')) => (),
            // Quote at beginning of line
            (None, '"', _) => is_open = true,
            // Quote at the end of the string
            (_, '"', None) => is_open = false,
            // Quote followed by the delimiter (`",`)
            (_, '"', Some(n)) if n == &delimiter => is_open = false,
            // Quote preceded by the delimiter (`,"`)
            (Some(c), '"', _) if c == delimiter => is_open = true,
            _ => (),
        }

        prev_char = Some(current_char);
    }

    is_open
}

impl<B: io::BufRead> Iterator for CSVLineIntoIter<B> {
    type Item = Result<Vec<Cell>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_selection = String::new();

        loop {
            match self.lines.next() {
                None => {
                    // in the case of a dangling quote current selection will be non-empty
                    if !current_selection.is_empty() {
                        return Some(parse_cells(&current_selection, self.delimiter));
                    } else {
                        return None;
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                Some(Ok(line)) => {
                    current_selection.push_str(&line);

                    if has_open_quotes(&current_selection, self.delimiter) {
                        current_selection.push('\n');
                    } else {
                        return Some(parse_cells(&current_selection, self.delimiter));
                    }
                }
            }
        }
    }
}

fn parse_cells(row: &str, delimiter: char) -> io::Result<Vec<Cell>> {
    if row.is_empty() {
        return Ok(Vec::new());
    }

    let chars = row.chars().collect::<Vec<_>>();

    let mut cells = Vec::new();
    let mut current_selection = String::new();
    let mut opened_quote = false;

    let mut i = 0;
    while i < chars.len() {
        let current_char = chars[i];
        let next_char = chars.get(i + 1);

        if current_char != delimiter && current_char != '"' {
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

        if current_char == delimiter {
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
    delimiter: char,
    encoding: &'a str,
) -> Result<impl Iterator<Item = io::Result<Vec<Cell>>> + 'a, CSVError> {
    let reader = read_encoded_file(filename, encoding)?;

    let parser = CSVReader::new(reader, delimiter);

    Ok(parser.into_lines())
}

#[cfg(test)]
mod parse_rows_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row\nnext,row\n".as_bytes();
        let result = CSVReader::new(input, ',')
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
        let result = CSVReader::new(input, ',')
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
        let input = "\"test\n\",\"broken\ncolumn\",\"another\ncolumn\"\nnext,row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![
                    Cell::new("\"test\n\""),
                    Cell::new("\"broken\ncolumn\""),
                    Cell::new("\"another\ncolumn\"")
                ],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        )
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"\nnext,row".as_bytes();
        let result = CSVReader::new(input, ',')
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
    fn test_incorrect_quoted_quote() {
        let input = "test,\"\"row\"\"\n\"\"next\"\",row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("\"\"row\"\"")],
                vec![Cell::new("\"\"next\"\""), Cell::new("row")],
            ]
        );
    }

    #[test]
    fn test_quoted_delimiter() {
        let input = "test,\"row,\"\nnext,row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("\"row,\"")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        );
    }

    #[test]
    fn test_blank_row() {
        let input = "test,row\n\nnext,row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        );
    }

    #[test]
    fn test_empty_row() {
        let input = "test,row\n,\nnext,row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![Cell::new(""), Cell::new("")],
                vec![Cell::new("next"), Cell::new("row")],
            ]
        );
    }

    #[test]
    fn test_dangling_quote() {
        let input = "test,row\n\"next,row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![Cell::new("\"next,row\n")],
            ]
        );
    }

    #[test]
    fn test_unescaped_cell_quote_does_not_consume_rest_of_rows() {
        let input = "test,row\n\"ne\"xt\",row\nfinal,row".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("row")],
                vec![Cell::new("\"ne\"xt\",row")],
                vec![Cell::new("final"), Cell::new("row")],
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
            parse_cells(input, ',').unwrap(),
            vec![Cell::new("test"), Cell::new("row")]
        )
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"";

        assert_eq!(
            parse_cells(input, ',').unwrap(),
            vec![Cell::new("test"), Cell::new("\"row\n\"")]
        )
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"";

        assert_eq!(
            parse_cells(input, ',').unwrap(),
            vec![Cell::new("test"), Cell::new("\"\"\"row\"\"\"")]
        )
    }

    #[test]
    fn test_quoted_delimiter() {
        let input = "test,\"row,\"";

        assert_eq!(
            parse_cells(input, ',').unwrap(),
            vec![Cell::new("test"), Cell::new("\"row,\"")]
        )
    }

    #[test]
    fn test_empty() {
        assert_eq!(parse_cells("", ',').unwrap(), vec![])
    }
}
