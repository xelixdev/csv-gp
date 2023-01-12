use char_reader::CharReader;

use crate::{cell::Cell, error::CSVError, file::read_encoded_file};
use std::{io, path::Path};

struct CSVLineReader<'a, R: io::Read> {
    reader: CharReader<R>,
    delimiter: &'a str,
}

impl<'a, R: io::Read> CSVLineReader<'a, R> {
    fn new(reader: R, delimiter: &'a str) -> Self {
        Self {
            reader: CharReader::new(reader),
            delimiter,
        }
    }
}

impl<R: io::Read> Iterator for CSVLineReader<'_, R> {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut opened_quotes = false;
        let mut current_selection = String::new();

        loop {
            match self.reader.next_char() {
                Err(e) => return Some(Err(e)),
                // Reached EOF
                Ok(None) => break,
                Ok(Some(current_char)) => {
                    // Reached a newline not in quotes, end of the line
                    if current_char == '\n' && !opened_quotes {
                        return Some(Ok(current_selection));
                    }

                    current_selection.push(current_char);

                    if current_char == '"' {
                        if let Ok(Some(next_char)) = self.reader.peek_char() {
                            if next_char != '"' {
                                // Matches either " or ,"
                                if current_selection == "\""
                                    || (current_selection
                                        .ends_with(&format!("{}{}", self.delimiter, "\""))
                                        && !opened_quotes)
                                {
                                    opened_quotes = true;
                                // Matches ", or "\n
                                } else if next_char.to_string() == self.delimiter
                                    || next_char == '\n'
                                {
                                    opened_quotes = false;
                                }
                            }
                        }
                    }
                }
            }
        }

        if !current_selection.is_empty() {
            Some(Ok(current_selection))
        } else {
            None
        }
    }
}

pub(crate) fn parse_file<'a>(
    filename: impl AsRef<Path> + 'a,
    delimiter: &'a str,
    encoding: &'a str,
) -> Result<impl Iterator<Item = io::Result<Vec<Cell>>> + 'a, CSVError> {
    let reader = read_encoded_file(filename, encoding)?;

    let parser = CSVLineReader::new(reader, delimiter);

    Ok(parser.map(|r| parse_cells(r, delimiter)))
}

fn parse_cells(row: io::Result<String>, delimiter: &str) -> io::Result<Vec<Cell>> {
    let chars = row?.chars().collect::<Vec<_>>();

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

#[cfg(test)]
mod parse_rows_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "test,row\nnext,row\n".as_bytes();
        let result = CSVLineReader::new(input, ",").collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec!["test,row".to_string(), "next,row".to_string()]
        )
    }

    #[test]
    fn test_no_trailing_newline() {
        let input = "test,row\nnext,row".as_bytes();
        let result = CSVLineReader::new(input, ",").collect::<Result<Vec<_>, _>>();

        assert_eq!(result.unwrap(), vec!["test,row", "next,row"])
    }

    #[test]
    fn test_quoted_newline() {
        let input = "test,\"row\n\"\nnext,row".as_bytes();
        let result = CSVLineReader::new(input, ",").collect::<Result<Vec<_>, _>>();

        assert_eq!(result.unwrap(), vec!["test,\"row\n\"", "next,row"])
    }

    #[test]
    fn test_quoted_quote() {
        let input = "test,\"\"\"row\"\"\"\nnext,row".as_bytes();
        let result = CSVLineReader::new(input, ",").collect::<Result<Vec<_>, _>>();

        assert_eq!(result.unwrap(), vec!["test,\"\"\"row\"\"\"", "next,row"])
    }

    #[test]
    fn test_blank_row() {
        let input = "test,row\n\nnext,row".as_bytes();
        let result = CSVLineReader::new(input, ",").collect::<Result<Vec<_>, _>>();

        assert_eq!(result.unwrap(), vec!["test,row", "", "next,row"])
    }
}

#[cfg(test)]
mod parse_cells_tests {
    use super::*;

    #[test]
    fn test_simple() {
        let input = Ok("test,row".to_string());
        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("row")]
        )
    }

    #[test]
    fn test_quoted_newline() {
        let input = Ok("test,\"row\n\"".to_string());

        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("\"row\n\"")]
        )
    }

    #[test]
    fn test_quoted_quote() {
        let input = Ok("test,\"\"\"row\"\"\"".to_string());

        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("\"\"\"row\"\"\"")]
        )
    }

    #[test]
    fn test_quoted_delimiter() {
        let input = Ok("test,\"row,\"".to_string());

        assert_eq!(
            parse_cells(input, ",").unwrap(),
            vec![Cell::new("test"), Cell::new("\"row,\"")]
        )
    }
}
