use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader},
};

pub struct Scanner<R> {
    source: BufReader<R>,
    byte_pos: u64,
    remainder: VecDeque<Token>,
    state: State,
    delimiter: u8,
}

impl<R> Scanner<R> {
    fn new(source: BufReader<R>, delimiter: u8) -> Self {
        Self {
            source,
            delimiter,
            state: State::Start,
            remainder: VecDeque::new(),
            byte_pos: 0,
        }
    }
}

impl<R: io::Read> Scanner<R> {
    pub fn from_reader(source: R, delimiter: u8) -> Self {
        Self::new(io::BufReader::new(source), delimiter)
    }
}

enum State {
    Start,
    NonQuoted,
    Quoted,
    StartDoubleQuote,
    EndQuotedQuote,
}

fn transition(state: &State, t: &TokenType) -> State {
    use State::*;

    match state {
        Start => match t {
            TokenType::Quote => Quoted,
            _ => NonQuoted,
        },
        NonQuoted => match t {
            TokenType::Quote => Quoted,
            _ => NonQuoted,
        },
        Quoted => match t {
            TokenType::Quote => StartDoubleQuote,
            _ => NonQuoted,
        },
        StartDoubleQuote => match t {
            TokenType::Quote => EndQuotedQuote,
            _ => NonQuoted,
        },
        EndQuotedQuote => match t {
            TokenType::Quote => Quoted,
            _ => NonQuoted,
        },
    }
}

fn match_token(c: &u8, delimiter: u8) -> TokenType {
    match c {
        c if c == &delimiter => TokenType::Delimiter,
        b'\n' | b'\r' => TokenType::Newline,
        b'"' => TokenType::Quote,
        _ => TokenType::Data,
    }
}

impl<R: io::Read> Iterator for Scanner<R> {
    type Item = io::Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(token) = self.remainder.pop_front() {
            return Some(Ok(token));
        } else {
            let (input, len) = match self.source.fill_buf() {
                Err(e) => {
                    return Some(Err(e));
                }
                Ok(input) => (input, input.len()),
            };

            if len == 0 {
                return None;
            }

            let mut tokens = VecDeque::with_capacity(len);
            let mut iter = input
                .iter()
                .map(|c| match_token(c, self.delimiter))
                .peekable();
            loop {
                match iter.next() {
                    None => break,
                    Some(t) => {
                        self.state = transition(&self.state, &t);
                        match t {
                            TokenType::Delimiter => {
                                tokens.push_back(Token::delimiter(self.byte_pos));
                            }
                            TokenType::Quote => {
                                tokens.push_back(Token::quote(self.byte_pos));
                            }
                            TokenType::Newline => {
                                let start = self.byte_pos;
                                // Consume all the newline characters
                                while iter.peek().is_some_and(|t| t == &TokenType::Newline) {
                                    iter.next();
                                    self.byte_pos += 1;
                                }
                                tokens.push_back(Token::newline(start, self.byte_pos - start + 1));
                            }
                            // FIXME: if a data token continues after the end of the buffer it will produce two tokens
                            TokenType::Data => {
                                let start = self.byte_pos;
                                while iter.peek().is_some_and(|t| t == &TokenType::Data) {
                                    iter.next();
                                    self.byte_pos += 1;
                                }
                                tokens.push_back(Token::data(start, self.byte_pos - start + 1));
                            }
                        }
                    }
                }
                self.byte_pos += 1;
            }
            self.source.consume(len);
            let token = tokens
                .pop_front()
                .expect("tokens to have at least one element");
            self.remainder = tokens;
            Some(Ok(token))
        }
    }
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum TokenType {
    Delimiter,
    Quote,
    Newline,
    Data,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
pub struct Token {
    pub token_type: TokenType,
    /// The byte position the token starts at
    pub start: u64,
    /// The length in bytes of the token
    pub length: u64,
}

impl Token {
    fn new(token_type: TokenType, start: u64, length: u64) -> Self {
        Self {
            token_type,
            start,
            length,
        }
    }

    fn data(start: u64, length: u64) -> Self {
        Self {
            token_type: TokenType::Data,
            start,
            length,
        }
    }

    fn newline(start: u64, length: u64) -> Self {
        Self {
            token_type: TokenType::Newline,
            start,
            length,
        }
    }

    fn delimiter(start: u64) -> Self {
        Self {
            token_type: TokenType::Delimiter,
            start,
            length: 1,
        }
    }

    fn quote(start: u64) -> Self {
        Self {
            token_type: TokenType::Quote,
            start,
            length: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn check(input: &str, expected: Vec<Token>) {
        let s = Scanner::from_reader(input.as_bytes(), b',');
        let actual = s.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn scanner_scans() {
        check(
            "some,line\n\"with\",quotes",
            vec![
                Token::data(0, 4),
                Token::delimiter(4),
                Token::data(5, 4),
                Token::newline(9, 1),
                Token::quote(10),
                Token::data(11, 4),
                Token::quote(15),
                Token::delimiter(16),
                Token::data(17, 6),
            ],
        );
    }

    #[test]
    fn handles_buffer_boundry() {
        let input = "some,line that extends over buffer,boundry";
        let s = Scanner::new(BufReader::with_capacity(10, input.as_bytes()), b',');
        assert_eq!(
            s.into_iter().flatten().collect::<Vec<_>>(),
            vec![
                Token::data(0, 4),
                Token::delimiter(4),
                Token::data(5, 29),
                Token::delimiter(34),
                Token::data(35, 7),
            ]
        );
    }
}
