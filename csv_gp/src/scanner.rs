use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader},
};

pub struct Scanner<R> {
    source: BufReader<R>,
    remainder: VecDeque<Token>,
    state: State,
    delimiter: u8,
}

impl<R: io::Read> Scanner<R> {
    pub fn new(source: R, delimiter: u8) -> Self {
        Self {
            source: io::BufReader::new(source),
            delimiter,
            state: State::Start,
            remainder: VecDeque::new(),
        }
    }
}

enum State {
    Start,
    DataStart,
    InData,
    InDelimiter,
    InQuote,
    NewlineStart,
    InNewline,
}

fn transition(state: &State, c: u8, delimiter: u8) -> State {
    use State::*;

    match state {
        Start => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => NewlineStart,
            _ => DataStart,
        },
        DataStart => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => NewlineStart,
            _ => InData,
        },
        InData => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => NewlineStart,
            _ => InData,
        },
        InDelimiter => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => NewlineStart,
            _ => DataStart,
        },
        InQuote => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => NewlineStart,
            _ => DataStart,
        },
        NewlineStart => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => InNewline,
            _ => DataStart,
        },
        InNewline => match c {
            c if c == delimiter => InDelimiter,
            b'"' => InQuote,
            b'\r' | b'\n' => InNewline,
            _ => DataStart,
        },
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
            for c in input {
                let next_state = transition(&self.state, *c, self.delimiter);
                match next_state {
                    State::Start => unreachable!(), // Start is the bootstraping state, should never be transitioned to
                    State::DataStart => tokens.push_back(Token::Data),
                    State::InData => (),
                    State::InDelimiter => tokens.push_back(Token::Delimiter),
                    State::InQuote => tokens.push_back(Token::Quote),
                    State::NewlineStart => tokens.push_back(Token::Newline),
                    State::InNewline => (),
                }
                self.state = next_state;
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
pub enum Token {
    Delimiter,
    Quote,
    Newline,
    Data,
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    fn check(input: &str, expected: Vec<Token>) {
        let s = Scanner::new(input.as_bytes(), b',');
        let actual = s.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn scanner_scans() {
        check(
            "some,line\n\"with\",quotes",
            vec![
                Data, Delimiter, Data, Newline, Quote, Data, Quote, Delimiter, Data,
            ],
        );
    }
}
