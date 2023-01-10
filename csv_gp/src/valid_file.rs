use std::{fs, io, path::Path};

use crate::{cell::Cell, csv_details::CSVDetails, error::DelimiterError};

pub fn get_delimiter_as_byte(delimiter: &str) -> Result<u8, DelimiterError> {
    let mut bytes = delimiter.bytes();

    let first = bytes.next();

    if bytes.next().is_some() {
        return Err(DelimiterError::MultiByteDelimiter);
    }

    if let Some(first) = first {
        Ok(first)
    } else {
        Err(DelimiterError::ZeroLengthDelimiter)
    }
}

/// Saves a file containing only the valid rows according to the passed CSVDetails
pub(crate) fn save_valid_file(
    rows: Vec<Vec<Cell>>,
    csv_details: &CSVDetails,
    delimiter: u8,
    output_path: impl AsRef<Path>,
) -> io::Result<()> {
    // Create intermediate directories
    if let Some(parent) = output_path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .quote_style(csv::QuoteStyle::Never) // quoting was untouched during parsing so set to avoid double quoting
        .from_path(output_path)?;

    let valid = rows
        .iter()
        .enumerate()
        .filter(|(i, _r)| csv_details.valid_rows.contains(i))
        .map(|x| x.1);

    for row in valid {
        writer.write_record(row)?;
    }

    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::{self},
    };

    use super::*;

    #[test]
    fn test_get_delimiter_as_byte_ok() {
        assert_eq!(get_delimiter_as_byte(","), Ok(b','))
    }

    #[test]
    fn test_get_delimiter_as_byte_zero_length() {
        assert_eq!(
            get_delimiter_as_byte(""),
            Err(DelimiterError::ZeroLengthDelimiter)
        )
    }

    #[test]
    fn test_get_delimiter_as_byte_multi_byte() {
        assert_eq!(
            get_delimiter_as_byte("test"),
            Err(DelimiterError::MultiByteDelimiter)
        )
    }

    fn rows_with_valid() -> (Vec<Vec<Cell>>, CSVDetails) {
        let rows = vec![
            vec![Cell::new("a"), Cell::new("b")],
            vec![Cell::new("invalid")],
            vec![Cell::new("\"\"\"quoted\"\"\""), Cell::new("row")],
        ];
        let mut csv_details = CSVDetails::new();
        csv_details.valid_rows = HashSet::from_iter(vec![0, 2]);

        (rows, csv_details)
    }

    #[test]
    fn test_save_valid_file() {
        let (rows, csv_details) = rows_with_valid();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_save_valid_file.csv");

        save_valid_file(rows, &csv_details, b',', &path).unwrap();

        let file = fs::read_to_string(path).unwrap();

        assert_eq!(file, "a,b\n\"\"\"quoted\"\"\",row\n")
    }

    #[test]
    fn test_save_valid_file_create_parent_dir() {
        let (rows, csv_details) = rows_with_valid();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("parent").join("child.csv");

        save_valid_file(rows, &csv_details, b',', &path).unwrap();

        let file = fs::read_to_string(path).unwrap();

        assert_eq!(file, "a,b\n\"\"\"quoted\"\"\",row\n")
    }
}
