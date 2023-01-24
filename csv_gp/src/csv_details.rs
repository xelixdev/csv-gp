use std::{collections::HashSet, fmt::Display};

#[derive(Debug, Clone, Default)]
pub struct CSVDetails {
    /// Number of non-empty rows (including the header) in the file
    pub row_count: usize,
    /// Number of columns according to the header
    pub column_count: usize,
    /// Number of REPLACEMENT CHARACTERs (U+FFFD) in the file
    pub invalid_character_count: usize,
    /// List of line numbers that contain fewer columns than the header
    pub too_few_columns: Vec<usize>,
    /// List of line numbers that contain more columns than the header
    pub too_many_columns: Vec<usize>,
    /// Number of columns per line, the index corresponding to the line number
    pub column_count_per_line: Vec<usize>,
    /// List of line numbers that contain a correctly quoted delimiter
    pub quoted_delimiter: Vec<usize>,
    /// List of line numbers that contain a correctly quoted newline
    pub quoted_newline: Vec<usize>,
    /// List of line numbers that contain quoted-quotes ("")
    pub quoted_quote: Vec<usize>,
    /// List of line numbers that contain correctly quoted-quotes (only contained within quoted cells)
    pub quoted_quote_correctly: Vec<usize>,
    /// List of line numbers that have incorrectly quoted cells
    /// Incorrect meaning:
    ///     - Missing an opening or closing quote
    ///     - Containing unquoted quotes
    pub incorrect_cell_quote: Vec<usize>,
    /// List of line numbers that contain no data
    /// A row is considered empty if either:
    ///     - it contains zero cells
    ///     - all cells in the row are empty (either zero characters or just `""`)
    pub all_empty_rows: Vec<usize>,
    /// Set of all row numbers that are valid in the file
    pub valid_rows: HashSet<usize>,
}

impl CSVDetails {
    pub fn new() -> Self {
        Default::default()
    }

    /// The header is considered messed up when none of the rows have the same number of columns as the header
    pub fn header_messed_up(&self) -> bool {
        let bad_row_count = self.too_few_columns.len() + self.too_many_columns.len();
        self.row_count - 1 == bad_row_count
    }
}

impl Display for CSVDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut strings = vec![];

        if self.header_messed_up() {
            strings.push("The header is totally messed up, no rows have the same number of columns as the header.".to_string());
        }

        if self.row_count <= 1 {
            strings.push("There is only one row row in the file.".to_string());
            return write!(f, "{}", strings.join("\n"));
        }

        if self.column_count <= 1 {
            strings.push(format!(
                "There is {} columns in the file, so the delimiter is almost surely wrong.",
                self.column_count
            ));
            return write!(f, "{}", strings.join("\n"));
        }

        strings.push(format!(
            "There are {} rows in the file (including header), with {} columns (according to the header).",
            self.row_count, self.column_count
        ));

        if !self.too_few_columns.is_empty() || !self.too_many_columns.is_empty() {
            strings.push(format!(
                "There are {} rows with too many columns, and {} rows with too few columns.",
                self.too_many_columns.len(),
                self.too_few_columns.len()
            ));
        } else {
            strings.push("All rows have the same number of columns.".to_string());
        }

        if !self.quoted_delimiter.is_empty() {
            strings.push(format!(
                "There are {} lines with correctly quoted delimiter.",
                self.quoted_delimiter.len()
            ));
        } else {
            strings.push("There are no rows with correctly quoted delimiter.".to_string());
        }

        if !self.quoted_newline.is_empty() {
            strings.push(format!(
                "There are {} lines with correctly quoted newline.",
                self.quoted_newline.len()
            ));
        } else {
            strings.push("There are no rows with correctly quoted newline.".to_string());
        }

        if !self.quoted_quote.is_empty() {
            strings.push(format!(
                "There are {} lines with correctly quoted quote, out of that {} are absolutely correct.",
                self.quoted_quote.len(), self.quoted_quote_correctly.len()
            ));
        } else {
            strings.push("There are no rows with correctly quoted quote.".to_string())
        }

        if !self.incorrect_cell_quote.is_empty() {
            strings.push(format!(
                "There are {} lines with incorrect cell quotes.",
                self.incorrect_cell_quote.len()
            ));
        } else {
            strings.push("There are no rows with incorrect cell quotes.".to_string());
        }

        write!(f, "{}", strings.join("\n"))
    }
}
