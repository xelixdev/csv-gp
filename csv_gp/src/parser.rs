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

    // Remove quoted quotes - Basically any two consecutive quotes can be ignored for the purposes of this
    // function.
    // After removing quoted quotes, we can also remove any case of "," if delimiter is ,; as those don't count
    // as open cell
    let s2 = s
        .replace("\"\"", "")
        .replace(&format!("\"{}\"", delimiter), "");

    let mut chars = s2.chars().peekable();
    while let Some(current_char) = chars.next() {
        match (prev_char, current_char, chars.peek()) {
            // Quote at beginning of line
            (None, '"', _) => is_open = true,
            // Quote at the end of the string preceeded by the delimiter (`,"`), unless already open
            (Some(c), '"', None) if c == delimiter && !is_open => is_open = true,
            // Quote at the end of the string
            (_, '"', None) => is_open = false,
            // Quote followed by the delimiter (`",`), if already open
            (_, '"', Some(n)) if n == &delimiter && is_open => is_open = false,
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
                // we have reached the end of the file
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
                    // special case for strange CRLF files with \r\r\n as line break, not ideal as it will alter valid quoted sequences also but ¯\_(ツ)_/¯
                    let line = line.trim_end_matches('\r');

                    current_selection.push_str(line);

                    if has_open_quotes(&current_selection, self.delimiter) {
                        // this newline is escaped, add back to text and continue loop
                        current_selection.push('\n');
                    } else {
                        // we have a full csv line, parse and return
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

    let mut cells = Vec::new();
    let mut current_selection = String::new();
    let mut opened_quote = false;

    for char in row.chars() {
        if char == delimiter && !opened_quote {
            // we are at the end of a cell, reset stack
            cells.push(Cell::new(current_selection.clone()));
            current_selection = String::new();
        } else {
            // ... otherwise add to the stack
            current_selection.push(char);
            // If we're on a quote, add to stack and flip the opened quote flag
            if char == '"' {
                opened_quote = !opened_quote;
            }
        }
    }

    // add final cell to cells
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
mod has_open_quotes_tests {
    use super::*;

    #[test]
    fn test_empty() {
        let input = "";

        assert!(!has_open_quotes(input, ','))
    }

    #[test]
    fn test_no_quotes() {
        let input = "asdfasdf";

        assert!(!has_open_quotes(input, ','))
    }

    #[test]
    fn test_with_opened_quote() {
        let input = "\"asdfasdf";

        assert!(has_open_quotes(input, ','))
    }

    #[test]
    fn test_with_closed_quote() {
        let input = "\"\"asdfasdf";

        assert!(!has_open_quotes(input, ','))
    }

    #[test]
    fn test_two_quotes_middle() {
        let input = "\"asdf\"\"asdf";

        assert!(has_open_quotes(input, ','))
    }

    #[test]
    fn test_two_quotes_end() {
        let input = "\"asdfasdf\"\"";

        assert!(has_open_quotes(input, ','))
    }

    #[test]
    fn test_three_quotes_end() {
        let input = "\"asdfasdf\"\"\"";

        assert!(!has_open_quotes(input, ','))
    }

    #[test]
    fn test_three_quotes_start() {
        let input = "\"\"\"asdfasdf";

        assert!(has_open_quotes(input, ','))
    }

    #[test]
    fn test_only_three_quotes_start() {
        let input = "\"\"\"";

        assert!(has_open_quotes(input, ','))
    }

    #[test]
    fn test_three_quotes_end_of_line() {
        let input = "X,\"\"\"";

        assert!(has_open_quotes(input, ','))
    }

    #[test]
    fn test_just_delimiter_quotes() {
        let input = "d,e,\",\"";

        assert!(!has_open_quotes(input, ','));
    }

    #[test]
    fn test_just_delimiter_open() {
        let input = "a,,\",";

        assert!(has_open_quotes(input, ','));
    }
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
    fn test_strange_crlf() {
        let input = "test,\"row\"\r\r\nnext,row\r\r\n".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("test"), Cell::new("\"row\"")],
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

    #[test]
    fn test_newline_and_quotes() {
        let input = "A,B,C\nA,X,\"\"\"28-35, GIDC Industrial\nEstate, Nan\"\nY,Z,Q\nX,\"\"\"\nVillege Poicha\"\"\",Q\nX,\"\"\"Villege Poicha\"\"\n\",Q\nN,Y,C".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("A"), Cell::new("B"), Cell::new("C")],
                vec![
                    Cell::new("A"),
                    Cell::new("X"),
                    Cell::new("\"\"\"28-35, GIDC Industrial\nEstate, Nan\"")
                ],
                vec![Cell::new("Y"), Cell::new("Z"), Cell::new("Q")],
                vec![
                    Cell::new("X"),
                    Cell::new("\"\"\"\nVillege Poicha\"\"\""),
                    Cell::new("Q"),
                ],
                vec![
                    Cell::new("X"),
                    Cell::new("\"\"\"Villege Poicha\"\"\n\""),
                    Cell::new("Q"),
                ],
                vec![Cell::new("N"), Cell::new("Y"), Cell::new("C")],
            ]
        )
    }

    #[test]
    fn test_quotes_just_delimiter() {
        let input = "c1,c2,c3\nd,e,\",\"\na,b,c\nd,e,\",\"".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("c1"), Cell::new("c2"), Cell::new("c3")],
                vec![Cell::new("d"), Cell::new("e"), Cell::new("\",\"")],
                vec![Cell::new("a"), Cell::new("b"), Cell::new("c")],
                vec![Cell::new("d"), Cell::new("e"), Cell::new("\",\"")],
            ]
        )
    }

    #[test]
    fn test_false_positive_delimiter_removal() {
        let input = "a,b,c\n\"lll\",\"\"\"\",\"\"\",\n\"".as_bytes();
        let result = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        assert_eq!(
            result.unwrap(),
            vec![
                vec![Cell::new("a"), Cell::new("b"), Cell::new("c")],
                vec![
                    Cell::new("\"lll\""),
                    Cell::new("\"\"\"\""),
                    Cell::new("\"\"\",\n\"")
                ],
            ]
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
