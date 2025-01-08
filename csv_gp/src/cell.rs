use crate::{
    csv_details::ByteRange,
    scanner::{Token, TokenType},
};

/// Short-hand macro for creating cells
#[macro_export]
macro_rules! cell {
    () => (Cell::new(Vec::new()));
    ($($x:expr),+ $(,)?) => (Cell::new(vec![$($x),*]));
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cell {
    tokens: Vec<Token>,
    correctly_quoted: bool,
    contains_double_quote: bool,
}

impl Cell {
    pub fn new(v: Vec<Token>) -> Self {
        let (correctly_quoted, contains_double_quote) = Cell::determine_quotes(
            v.iter()
                .map(|t| &t.token_type)
                .collect::<Vec<_>>()
                .as_slice(),
        );
        Self {
            tokens: v,
            correctly_quoted,
            contains_double_quote,
        }
    }

    /// Returns if the cell has correct quoting, and if a double quote was found in the cell
    fn determine_quotes(tokens: &[&TokenType]) -> (bool, bool) {
        // This looks an awful lot like parsing, maybe move there?

        let mut opening_quote = false;
        let mut closing_quote = false;

        let mut stripped = tokens;
        if let Some(s) = stripped.strip_prefix(&[&TokenType::Quote]) {
            stripped = s;
            opening_quote = true;
        }
        if let Some(s) = stripped.strip_suffix(&[&TokenType::Quote]) {
            stripped = s;
            closing_quote = true;
        }

        let mut tokens = stripped.into_iter().peekable();
        let mut single_quote_found = false;
        let mut double_quote_found = false;
        loop {
            match tokens.next() {
                Some(t) if t == &&TokenType::Quote => {
                    let has_paired_quote = tokens.next_if(|t| t == &&&TokenType::Quote).is_some();
                    if !has_paired_quote {
                        single_quote_found = true;
                    } else {
                        double_quote_found = true;
                    }
                }
                Some(_) => (),
                None => break,
            }
        }

        let unmatched_surronding_quotes = opening_quote != closing_quote;
        let surrounding_quotes = opening_quote && closing_quote;
        let correctly_quoted = !unmatched_surronding_quotes
            && ((surrounding_quotes && !single_quote_found)
                || (!surrounding_quotes && !double_quote_found));

        (correctly_quoted, double_quote_found)
    }

    pub fn correctly_quoted(&self) -> bool {
        self.correctly_quoted
    }

    pub fn contains_double_quote(&self) -> bool {
        self.contains_double_quote
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() || self.tokens.iter().all(|t| t.token_type == TokenType::Quote)
    }

    pub fn contains(&self, token_type: &TokenType) -> bool {
        self.tokens.iter().any(|t| &t.token_type == token_type)
    }

    pub fn invalid_character_count(&self) -> usize {
        0
        // self.0.matches('\u{FFFD}').count()
    }

    pub fn byte_range(&self) -> Option<ByteRange> {
        let start = self.tokens.first()?.start;
        let last = self.tokens.last()?;
        let length = last.start + last.length - start;
        Some(ByteRange::new(start as i64, length))
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    mod correctly_quoted {
        use super::*;
        use Token::*;

        #[test]
        fn incorrect() {
            assert!(!cell!(
                Quote, Data, Quote, Data, Quote, Delimiter, Data, Delimiter, Data, Quote
            )
            .correctly_quoted())
        }

        #[test]
        fn incorrect_2() {
            assert!(
                !cell!(Quote, Data, Quote, Data, Quote, Delimiter, Quote, Data, Quote)
                    .correctly_quoted()
            )
        }

        #[test]
        fn correct() {
            assert!(cell!(
                Quote, Data, Quote, Quote, Data, Quote, Quote, Delimiter, Data, Delimiter, Data,
                Quote
            )
            .correctly_quoted())
        }

        #[test]
        fn correct_2() {
            assert!(cell!(
                Quote, Data, Quote, Quote, Data, Quote, Quote, Delimiter, Quote, Quote, Data, Quote
            )
            .correctly_quoted())
        }

        #[test]
        fn all_quotes() {
            assert!(cell!(Quote, Quote, Quote, Quote).correctly_quoted())
        }

        #[test]
        fn no_quotes() {
            assert!(cell!(Data).correctly_quoted())
        }

        #[test]
        fn quoted_cell() {
            assert!(cell!(Quote, Data, Quote).correctly_quoted())
        }

        #[test]
        fn no_opening() {
            assert!(!cell!(Data, Quote).correctly_quoted())
        }

        #[test]
        fn no_closing() {
            assert!(!cell!(Quote, Data).correctly_quoted())
        }
    }
}*/
