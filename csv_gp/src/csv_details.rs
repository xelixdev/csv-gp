use std::{collections::HashSet, fmt::Display};

#[derive(Debug, Clone, Default)]
pub struct CSVDetails {
    pub row_count: usize,
    pub column_count: usize,
    pub invalid_character_count: usize,
    pub too_few_columns: Vec<usize>,
    pub too_many_columns: Vec<usize>,
    pub column_count_per_line: Vec<usize>,
    pub quoted_delimiter: Vec<usize>,
    pub quoted_newline: Vec<usize>,
    pub quoted_quote: Vec<usize>,
    pub quoted_quote_correctly: Vec<usize>,
    pub incorrect_cell_quote: Vec<usize>,
    pub all_empty_rows: Vec<usize>,
    pub valid_rows: HashSet<usize>,
}

impl CSVDetails {
    pub fn new() -> Self {
        Self {
            row_count: 0,
            column_count: 0,
            invalid_character_count: 0,
            column_count_per_line: Vec::new(),
            too_few_columns: Vec::new(),
            too_many_columns: Vec::new(),
            quoted_delimiter: Vec::new(),
            quoted_newline: Vec::new(),
            quoted_quote: Vec::new(),
            quoted_quote_correctly: Vec::new(),
            incorrect_cell_quote: Vec::new(),
            all_empty_rows: Vec::new(),
            valid_rows: HashSet::new(),
        }
    }

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
