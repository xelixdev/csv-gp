use std::collections::HashSet;

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
