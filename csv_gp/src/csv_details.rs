use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct CSVDetails {
    /// Number of non-blank rows (including the header) in the file
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
    /// List of line numbers where all cells in the row are empty (either zero characters or just `""`)
    pub all_empty_rows: Vec<usize>,
    /// List of line numbers that are completely blank
    pub blank_rows: Vec<usize>,
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

    pub fn report(&self) -> String {
        let mut results = String::new();

        if self.header_messed_up() {
            results += "The header is totally messed up, no rows have the same number of columns as the header.\n";
        }

        if self.row_count <= 1 {
            results += "There is only one row in the file.";
            return results;
        }

        if self.column_count <= 1 {
            results += &format!(
                "There is {} columns in the file, so the delimiter is almost surely wrong.",
                self.column_count
            );
            return results;
        }

        results += &format!(
            "There are {} ({} of which are valid) rows in the file (including header), with {} columns (according to the header).\n",
            self.row_count, self.valid_rows.len(), self.column_count
        );

        if !self.too_few_columns.is_empty() || !self.too_many_columns.is_empty() {
            results += &format!(
                "There are {} rows with too many columns, and {} rows with too few columns.\n",
                self.too_many_columns.len(),
                self.too_few_columns.len()
            );
        } else {
            results += "All rows have the same number of columns.\n";
        }

        if !self.blank_rows.is_empty() {
            results += &format!("There are {} blank rows.\n", self.blank_rows.len());
        }

        if !self.all_empty_rows.is_empty() {
            results += &format!(
                "There are {} rows where all the cells are empty.\n",
                self.all_empty_rows.len()
            );
        }

        if !self.quoted_delimiter.is_empty() {
            results += &format!(
                "There are {} lines with correctly quoted delimiter.\n",
                self.quoted_delimiter.len()
            );
        } else {
            results += "There are no rows with correctly quoted delimiter.\n";
        }

        if !self.quoted_newline.is_empty() {
            results += &format!(
                "There are {} lines with correctly quoted newline.\n",
                self.quoted_newline.len()
            );
        } else {
            results += "There are no rows with correctly quoted newline.\n";
        }

        if !self.quoted_quote.is_empty() {
            results += &format!(
                "There are {} lines with correctly quoted quote, out of that {} are absolutely correct.\n",
                self.quoted_quote.len(), self.quoted_quote_correctly.len()
            );
        } else {
            results += "There are no rows with correctly quoted quote.\n";
        }

        if !self.incorrect_cell_quote.is_empty() {
            results += &format!(
                "There are {} lines with incorrect cell quotes.\n",
                self.incorrect_cell_quote.len()
            );
        } else {
            results += "There are no rows with incorrect cell quotes.\n";
        }

        results
    }
}
