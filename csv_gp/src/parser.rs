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

    /// Returns an owned iterator of all the csv lines
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

/// Quote parse state across line boundaries.
#[derive(Clone, Copy)]
struct QuoteState {
    is_open: bool,
    prev_char: Option<char>,
    prev_prev_char: Option<char>,
}

impl QuoteState {
    fn initial() -> Self {
        Self {
            is_open: false,
            prev_char: None,
            prev_prev_char: None,
        }
    }
}

/// Returns quote state after processing `s` with the given initial state.
fn quote_state_after(s: &str, delimiter: char, initial: QuoteState) -> QuoteState {
    let mut is_open = initial.is_open;
    let mut prev_char = initial.prev_char;
    let mut prev_prev_char = initial.prev_prev_char;
    let mut chars = s.chars().peekable();

    while let Some(current_char) = chars.next() {
        if current_char == '"' {
            if chars.peek() == Some(&'"') {
                chars.next();
                prev_prev_char = prev_char;
                prev_char = Some('"');
                continue;
            }

            // Handle consecutive delimiter-only cells pattern: ,"","," or ,",","
            // When closing a cell that contains just the delimiter, and next is delimiter,
            // and we just came from pattern that started with ,"
            // This handles the specific case where prev content was just the delimiter char
            if is_open
                && prev_char == Some(delimiter)
                && prev_prev_char == Some('"')
                && chars.peek() == Some(&delimiter)
            {
                // This quote closes a "," cell; consume the closing quote and following delimiter
                chars.next();
                is_open = false;
                prev_prev_char = Some('"');
                prev_char = Some(delimiter);
                continue;
            }

            let at_start = prev_char.is_none();
            let after_delimiter = prev_char == Some(delimiter);
            let at_end = chars.peek().is_none();
            let before_delimiter = chars.peek() == Some(&delimiter);
            let single_quote_after_escaped = at_end && prev_char == Some('"');

            match (
                at_start,
                after_delimiter,
                at_end,
                before_delimiter,
                is_open,
                single_quote_after_escaped,
            ) {
                // closing quote: inside quoted field, next not delimiter, prev was "
                (_, _, _, false, true, _) if prev_char == Some('"') => is_open = false,
                // at start of cell or after delimiter when not open → opening quote
                // This must come before EOL close to handle ," at line end
                (true, _, _, _, false, _) | (_, true, _, _, false, _) => is_open = true,
                // end of string when open (and not escaped single-quote) or delimiter after open → close
                (_, _, true, _, true, false) | (_, _, _, true, true, _) => is_open = false,
                // at EOL, was open, single-quote-after-escaped → close (empty quoted at EOL)
                (_, _, true, _, true, true) => is_open = false,
                // at EOL, was not open, single-quote-after-escaped → open (dangling quote)
                (_, _, true, _, false, true) => is_open = true,
                // inside content, prev was " (e.g. after escaped quote) → stay in quoted field
                (_, false, false, _, _, _) if prev_char == Some('"') => is_open = true,
                _ => (),
            }
        }
        prev_prev_char = prev_char;
        prev_char = Some(current_char);
    }

    QuoteState {
        is_open,
        prev_char,
        prev_prev_char,
    }
}

impl<B: io::BufRead> Iterator for CSVLineIntoIter<B> {
    type Item = Result<Vec<Cell>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut current_selection = String::new();
        let mut state = QuoteState::initial();

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
                    state = quote_state_after(line, self.delimiter, state);

                    if state.is_open {
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

        assert!(!quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_no_quotes() {
        let input = "asdfasdf";

        assert!(!quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_with_opened_quote() {
        let input = "\"asdfasdf";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_with_closed_quote() {
        let input = "\"\"asdfasdf";

        assert!(!quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_two_quotes_middle() {
        let input = "\"asdf\"\"asdf";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_two_quotes_end() {
        let input = "\"asdfasdf\"\"";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_three_quotes_end() {
        let input = "\"asdfasdf\"\"\"";

        assert!(!quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_three_quotes_start() {
        let input = "\"\"\"asdfasdf";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_only_three_quotes_start() {
        let input = "\"\"\"";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_three_quotes_end_of_line() {
        let input = "X,\"\"\"";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open)
    }

    #[test]
    fn test_just_delimiter_quotes() {
        let input = "d,e,\",\"";

        assert!(!quote_state_after(input, ',', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_just_delimiter_open() {
        let input = "a,,\",";

        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_unclosed_quote_mid_field() {
        let input = "a|b|\"text|d|e";
        assert!(quote_state_after(input, '|', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_unclosed_quote_with_pipe_delimiter() {
        let input = "a|b||c|d|e|f|g|h|i|j|k|\"unclosed|m|n|o";
        assert!(quote_state_after(input, '|', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_pipe_delimiter_basic() {
        let input = "a|b|c";
        assert!(!quote_state_after(input, '|', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_pipe_delimiter_quoted_cell() {
        let input = "a|\"b|c\"|d";
        assert!(!quote_state_after(input, '|', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_pipe_delimiter_only_cell() {
        let input = "a|\"|\"";
        assert!(!quote_state_after(input, '|', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_consecutive_delimiter_only_cells() {
        // Two consecutive "," cells: ,",",",", (empty, ",", ",", empty)
        let input = ",\",\",\",\",";
        assert!(!quote_state_after(input, ',', QuoteState::initial()).is_open);
    }

    #[test]
    fn test_unclosed_delimiter_only_cell() {
        // One complete "," cell followed by incomplete: ,",","," (missing closing quote)
        let input = ",\",\",\",";
        assert!(quote_state_after(input, ',', QuoteState::initial()).is_open);
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

    #[test]
    fn test_unclosed_quote_accumulates_following_lines() {
        let input = "a|b|c\nd|\"unclosed|f\ng|h|i\nj|k|l".as_bytes();
        let result = CSVReader::new(input, '|')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        let rows = result.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].len(), 3);
        let row1_str: String = rows[1]
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("|");
        assert!(row1_str.contains("unclosed"));
    }

    #[test]
    fn test_escaped_quote_at_line_boundary_stays_open() {
        let input = "a|b|c\nd|\"b\n\"\"|c|d\ne|f|g".as_bytes();
        let result = CSVReader::new(input, '|')
            .into_lines()
            .collect::<Result<Vec<_>, _>>();

        let rows = result.unwrap();
        assert_eq!(rows.len(), 2);
        let row1_str: String = rows[1]
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("|");
        assert!(row1_str.contains("|c|d"));
        assert!(row1_str.contains("e|f|g"));
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

#[cfg(test)]
mod equivalence_tests {
    use super::*;
    use std::io::BufRead;

    /// Full-buffer reference for equivalence tests.
    fn rows_via_full_buffer(
        input: &[u8],
        delimiter: char,
    ) -> Result<Vec<Vec<Cell>>, std::io::Error> {
        let lines = std::io::BufReader::new(input).lines();
        let mut current_selection = String::new();
        let mut rows = Vec::new();

        for line in lines {
            let line: String = line?.trim_end_matches('\r').to_string();
            current_selection.push_str(&line);
            if quote_state_after(&current_selection, delimiter, QuoteState::initial()).is_open {
                current_selection.push('\n');
            } else {
                rows.push(parse_cells(&current_selection, delimiter)?);
                current_selection.clear();
            }
        }
        if !current_selection.is_empty() {
            rows.push(parse_cells(&current_selection, delimiter)?);
        }
        Ok(rows)
    }

    #[test]
    fn test_incremental_matches_full_buffer_simple() {
        let input = "a|b|c\nd|e|f\ng|h|i".as_bytes();
        let iter_rows = CSVReader::new(input, '|')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ref_rows = rows_via_full_buffer(input, '|').unwrap();
        assert_eq!(iter_rows, ref_rows);
    }

    #[test]
    fn test_incremental_matches_full_buffer_unclosed_quote() {
        let input = "a|b|c\nd|\"unclosed|f\ng|h|i\nj|k|l".as_bytes();
        let iter_rows = CSVReader::new(input, '|')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ref_rows = rows_via_full_buffer(input, '|').unwrap();
        assert_eq!(iter_rows, ref_rows);
    }

    #[test]
    fn test_incremental_matches_full_buffer_escaped_at_boundary() {
        let input = "a|b|c\nd|\"b\n\"\"|c|d\ne|f|g".as_bytes();
        let iter_rows = CSVReader::new(input, '|')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ref_rows = rows_via_full_buffer(input, '|').unwrap();
        assert_eq!(iter_rows, ref_rows);
    }

    #[test]
    fn test_incremental_matches_full_buffer_quoted_newline() {
        let input = "\"test\n\",\"broken\ncolumn\"\nnext,row".as_bytes();
        let iter_rows = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ref_rows = rows_via_full_buffer(input, ',').unwrap();
        assert_eq!(iter_rows, ref_rows);
    }

    #[test]
    fn test_incremental_matches_full_buffer_delimiter_only_cell() {
        let input = "a,\",\",b\nc,d,e".as_bytes();
        let iter_rows = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ref_rows = rows_via_full_buffer(input, ',').unwrap();
        assert_eq!(iter_rows, ref_rows);
    }

    #[test]
    fn test_incremental_matches_full_buffer_delimiter_only_at_boundary() {
        let input = "a,\",\"\n\"b\",c".as_bytes();
        let iter_rows = CSVReader::new(input, ',')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let ref_rows = rows_via_full_buffer(input, ',').unwrap();
        assert_eq!(iter_rows, ref_rows);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_accumulated_string_performance() {
        let line = "a|b||c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z|1|2|3\n";
        let mut accumulated = String::with_capacity(line.len() * 10_000);
        for _ in 0..10_000 {
            accumulated.push_str(line);
        }

        let start = Instant::now();
        let _ = quote_state_after(&accumulated, '|', QuoteState::initial()).is_open;
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 100,
            "quote_state_after took {}ms for 10k lines, expected <100ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_iterator_on_accumulated_lines_fast() {
        let line = "a|\"unclosed|b|c\n";
        let mut input = String::with_capacity(line.len() * 5000);
        for _ in 0..5000 {
            input.push_str(line);
        }
        let start = Instant::now();
        let rows = CSVReader::new(input.as_bytes(), '|')
            .into_lines()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let elapsed = start.elapsed();
        assert_eq!(rows.len(), 1);
        assert!(
            elapsed.as_millis() < 500,
            "iterator took {}ms",
            elapsed.as_millis()
        );
    }
}
