use encoding_rs::Encoding;

use crate::{
    cell::Cell,
    error::{CSVError, UnknownEncoding},
    scanner::{Scanner, TokenType},
};
use std::{fs::File, io, path::Path};

pub struct CSVReader<R> {
    scanner: Scanner<R>,
    state: State,
}

#[derive(Debug, Clone)]
enum State {
    QuotedField,
    UnquotedField,
    AfterQuoteWithinQuotedField,
    AfterRecordEnd,
    AfterFieldEnd,
}

impl<R> CSVReader<R> {
    pub fn from_scanner(scanner: Scanner<R>) -> Self {
        Self {
            scanner,
            state: State::AfterRecordEnd,
        }
    }
}

impl CSVReader<File> {
    pub fn from_path(path: impl AsRef<Path>, delimiter: u8) -> Result<Self, CSVError> {
        let scanner = Scanner::from_reader(File::open(path)?, delimiter);
        Ok(Self::from_scanner(scanner))
    }
}

/// Compute the next parser state from the current state and the current token.
fn transition(current: &State, t: &TokenType) -> State {
    use State::*;
    use TokenType::*;

    match current {
        QuotedField => match t {
            Delimiter => QuotedField,
            Quote => AfterQuoteWithinQuotedField,
            Newline => QuotedField,
            Data => QuotedField,
        },
        UnquotedField => match t {
            Delimiter => AfterFieldEnd,
            Newline => AfterRecordEnd,
            Quote => UnquotedField,
            Data => UnquotedField,
        },
        AfterQuoteWithinQuotedField => match t {
            Delimiter => AfterFieldEnd,
            Quote => QuotedField,
            Newline => AfterRecordEnd,
            Data => UnquotedField,
        },
        AfterFieldEnd => match t {
            Delimiter => AfterFieldEnd,
            Quote => QuotedField,
            Newline => AfterRecordEnd,
            Data => UnquotedField,
        },
        AfterRecordEnd => match t {
            Delimiter => AfterFieldEnd,
            Quote => QuotedField,
            Newline => AfterRecordEnd,
            Data => UnquotedField,
        },
    }
}

impl<R: io::Read> Iterator for CSVReader<R> {
    type Item = Result<Vec<Cell>, CSVError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut row = vec![];
        let mut current_cell = vec![];

        loop {
            match self.scanner.next() {
                // we have reached the end of the file
                None => {
                    if !current_cell.is_empty() {
                        row.push(Cell::new(current_cell));
                        return Some(Ok(row));
                    } else {
                        return None;
                    }
                }
                Some(token) => {
                    let token = match token {
                        Err(e) => return Some(Err(e.into())),
                        Ok(t) => t,
                    };
                    self.state = transition(&self.state, &token.token_type);
                    match self.state {
                        State::QuotedField
                        | State::UnquotedField
                        | State::AfterQuoteWithinQuotedField => {
                            current_cell.push(token);
                        }
                        State::AfterRecordEnd => {
                            row.push(Cell::new(current_cell));
                            return Some(Ok(row));
                        }
                        State::AfterFieldEnd => {
                            row.push(Cell::new(current_cell));
                            current_cell = Vec::new();
                        }
                    }
                }
            }
        }
    }
}

pub fn parse_file(
    filename: impl AsRef<Path>,
    delimiter: char,
    encoding: &str,
) -> Result<impl Iterator<Item = Result<Vec<Cell>, CSVError>>, CSVError> {
    let encoding = Encoding::for_label(encoding.as_bytes())
        .ok_or_else(|| CSVError::UnknownEncoding(UnknownEncoding::Encoding(encoding.into())))?;
    let mut buf = [0; 4];
    let delimiter = delimiter.encode_utf8(&mut buf);
    let (encoded_delimiter, _, _) = encoding.encode(delimiter);
    let encoded_delimiter = encoded_delimiter[0];

    Ok(CSVReader::from_path(filename, encoded_delimiter)?.into_iter())
}

/*
#[cfg(test)]
mod tests {
    use crate::cell;

    use super::*;
    use crate::scanner::TokenType::*;

    use pretty_assertions::assert_eq;

    fn check(input: &'static str, expected: Vec<Vec<Cell>>) {
        let bytes = input.as_bytes();
        let scanner = Scanner::from_reader(bytes, b',');
        let reader = CSVReader::from_scanner(scanner);

        let actual = reader.into_iter().flatten().collect::<Vec<_>>();

        assert_eq!(expected, actual);
    }

    #[test]
    fn simple() {
        check(
            "test,row\nnext,row\n",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data)],
            ],
        )
    }

    #[test]
    fn crlf() {
        check(
            "test,row\r\nnext,row\r\n",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data)],
            ],
        )
    }

    #[test]
    fn test_strange_crlf() {
        check(
            "test,\"row\"\r\r\nnext,row\r\r\n",
            vec![
                vec![cell!(Data), cell!(Quote, Data, Quote)],
                vec![cell!(Data), cell!(Data)],
            ],
        )
    }

    #[test]
    fn no_trailing_newline() {
        check(
            "test,row\nnext,row",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data)],
            ],
        )
    }

    #[test]
    fn quoted_newline() {
        check(
            "\"test\n\",\"broken\ncolumn\",\"another\ncolumn\"\nnext,row",
            vec![
                vec![
                    cell!(Quote, Data, Newline, Quote),
                    cell!(Quote, Data, Newline, Data, Quote),
                    cell!(Quote, Data, Newline, Data, Quote),
                ],
                vec![cell!(Data), cell!(Data)],
            ],
        )
    }

    #[test]
    fn quoted_quote() {
        check(
            "test,\"\"\"row\"\"\"\nnext,row",
            vec![
                vec![
                    Cell::new(vec![Data]),
                    cell!(Quote, Quote, Quote, Data, Quote, Quote, Quote),
                ],
                vec![cell!(Data), cell!(Data)],
            ],
        )
    }

    #[test]
    fn incorrect_quoted_quote() {
        check(
            "test,\"\"row\"\"\n\"\"next\"\",row",
            vec![
                vec![cell!(Data), cell!(Quote, Quote, Data, Quote, Quote)],
                vec![cell!(Quote, Quote, Data, Quote, Quote), cell!(Data)],
            ],
        );
    }

    #[test]
    fn quoted_delimiter() {
        check(
            "test,\"row,\"\nnext,row",
            vec![
                vec![cell!(Data), cell!(Quote, Data, Delimiter, Quote)],
                vec![cell!(Data), cell!(Data)],
            ],
        );
    }

    #[test]
    fn blank_row() {
        check(
            "test,row\n\nnext,row",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data)],
            ],
        );
    }

    #[test]
    fn empty_row() {
        check(
            "test,row\n,\nnext,row",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(), cell!()],
                vec![cell!(Data), cell!(Data)],
            ],
        );
    }

    #[test]
    fn dangling_quote() {
        check(
            "test,row\n\"next,row",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Quote, Data, Delimiter, Data)],
            ],
        );
    }

    #[test]
    fn unescaped_cell_quote_does_not_consume_rest_of_rows() {
        check(
            "test,row\n\"ne\"xt\",row\nfinal,row",
            vec![
                vec![cell!(Data), cell!(Data)],
                vec![cell!(Quote, Data, Quote, Data, Quote), cell!(Data)],
                vec![cell!(Data), cell!(Data)],
            ],
        );
    }

    #[test]
    fn test_newline_and_quotes() {
        check("A,B,C\nA,X,\"\"\"28-35, GIDC Industrial\nEstate, Nan\"\nY,Z,Q\nX,\"\"\"\nVillege Poicha\"\"\",Q\nX,\"\"\"Villege Poicha\"\"\n\",Q\nN,Y,C"
        ,
            vec![
                vec![cell!(Data), cell!(Data), cell!(Data)],
                vec![
                    cell!(Data),
                    cell!(Data),
                    cell!(Quote, Quote, Quote, Data, Delimiter, Data, Newline, Data, Delimiter, Data, Quote)
                ],
                vec![cell!(Data), cell!(Data), cell!(Data)],
                vec![
                    cell!(Data),
                    cell!(Quote, Quote, Quote, Newline, Data, Quote, Quote, Quote),
                    cell!(Data),
                ],
                vec![
                    cell!(Data),
                    cell!(Quote, Quote, Quote, Data, Quote, Quote, Newline, Quote),
                    cell!(Data),
                ],
                vec![cell!(Data), cell!(Data), cell!(Data)],
            ]
        )
    }

    #[test]
    fn test_quotes_just_delimiter() {
        check(
            "c1,c2,c3\nd,e,\",\"\na,b,c\nd,e,\",\"",
            vec![
                vec![cell!(Data), cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data), cell!(Quote, Delimiter, Quote)],
                vec![cell!(Data), cell!(Data), cell!(Data)],
                vec![cell!(Data), cell!(Data), cell!(Quote, Delimiter, Quote)],
            ],
        )
    }

    #[test]
    fn test_false_positive_delimiter_removal() {
        check(
            "a,b,c\n\"lll\",\"\"\"\",\"\"\",\n\"",
            vec![
                vec![cell!(Data), cell!(Data), cell!(Data)],
                vec![
                    cell!(Quote, Data, Quote),
                    cell!(Quote, Quote, Quote, Quote),
                    cell!(Quote, Quote, Quote, Delimiter, Newline, Quote),
                ],
            ],
        )
    }
} */
