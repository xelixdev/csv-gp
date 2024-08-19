use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell(String);

impl Cell {
    pub fn new(v: impl Into<String>) -> Self {
        Self(v.into())
    }

    pub fn correctly_quoted(&self) -> bool {
        if !self.0.contains('"') {
            return true;
        }

        let mut starts = false;
        let mut ends = false;
        let mut stripped: &str = &self.0.trim();

        if let Some(s) = stripped.strip_prefix('"') {
            stripped = s;
            starts = true;
        }

        if let Some(s) = stripped.strip_suffix('"') {
            stripped = s;
            ends = true;
        }

        if !starts || !ends {
            return false;
        }

        if !stripped.contains('"') {
            return true;
        }

        stripped.matches("\"\"").count() * 2 == stripped.matches('\"').count()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() || self.0 == "\"\""
    }

    pub fn contains(&self, pat: &str) -> bool {
        self.0.contains(pat)
    }

    pub fn invalid_character_count(&self) -> usize {
        self.0.matches('\u{FFFD}').count()
    }
}

impl AsRef<[u8]> for Cell {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incorrect() {
        assert!(!Cell::new("\"Anlagestiftung der UBS für \"Immobilien Schweiz\", Zürich, c/o UBS Fund Management AG\"").correctly_quoted())
    }

    #[test]
    fn test_incorrect_2() {
        assert!(!Cell::new("\"5\"379'319'026\",\"SINV-00110094\"").correctly_quoted())
    }

    #[test]
    fn test_correct() {
        assert!(Cell::new("\"Anlagestiftung der UBS für \"\"Immobilien Schweiz\"\", Zürich, c/o UBS Fund Management AG\"").correctly_quoted())
    }

    #[test]
    fn test_correct_2() {
        assert!(Cell::new("\"5\"\"379'319'026\"\",\"\"SINV-00110094\"").correctly_quoted())
    }

    #[test]
    fn test_quotes_strip_whitespace() {
        assert!(Cell::new("\"cameron\" ").correctly_quoted());
        assert!(Cell::new(" \"james\"").correctly_quoted());
        assert!(Cell::new(" \"matt\" ").correctly_quoted());
    }

    #[test]
    fn test_no_quotes() {
        assert!(Cell::new("test").correctly_quoted())
    }

    #[test]
    fn test_no_quotes_when_stripped() {
        assert!(Cell::new("\"test\"").correctly_quoted())
    }

    #[test]
    fn test_does_not_start() {
        assert!(!Cell::new("test\"").correctly_quoted())
    }

    #[test]
    fn test_does_not_end() {
        assert!(!Cell::new("\"test").correctly_quoted())
    }
}
