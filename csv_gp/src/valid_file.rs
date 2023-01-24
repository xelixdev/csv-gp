use crate::{csv_details::CSVDetails, error::CSVError, parser::parse_file};

use std::{fs, io, path::Path};

/// Saves a file containing only the valid rows according to the passed CSVDetails
pub(crate) fn save_valid_file(
    path: impl AsRef<Path>,
    csv_details: &CSVDetails,
    delimiter: char,
    encoding: &str,
    output_path: impl AsRef<Path>,
) -> Result<(), CSVError> {
    // Create intermediate directories
    if let Some(parent) = output_path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter as u8)
        .quote_style(csv::QuoteStyle::Never) // quoting was untouched during parsing so set to avoid double quoting
        .from_path(output_path)
        .map_err(Into::<io::Error>::into)?;

    for (i, row_result) in parse_file(path, delimiter, encoding)?.enumerate() {
        if csv_details.valid_rows.contains(&i) {
            let row = row_result?;
            writer.write_record(row).map_err(Into::<io::Error>::into)?;
        }
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

    fn rows_with_valid(path: impl AsRef<Path>) -> (CSVDetails, impl AsRef<Path>) {
        fs::write(&path, "a,b\ninvalid\n\"\"\"quoted\"\"\",row").unwrap();
        let mut csv_details = CSVDetails::new();
        csv_details.valid_rows = HashSet::from_iter(vec![0, 2]);

        (csv_details, path)
    }

    #[test]
    fn test_save_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let (csv_details, path) = rows_with_valid(dir.path().join("test_save_valid_file_base.csv"));
        let out_path = dir.path().join("test_save_valid_file.csv");

        save_valid_file(path, &csv_details, ',', "utf-8", &out_path).unwrap();

        let file = fs::read_to_string(out_path).unwrap();

        assert_eq!(file, "a,b\n\"\"\"quoted\"\"\",row\n")
    }

    #[test]
    fn test_save_valid_file_create_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let (csv_details, path) = rows_with_valid(dir.path().join("create_parent_dir.csv"));
        let out_path = dir.path().join("parent").join("child.csv");

        save_valid_file(path, &csv_details, ',', "utf-8", &out_path).unwrap();

        let file = fs::read_to_string(out_path).unwrap();

        assert_eq!(file, "a,b\n\"\"\"quoted\"\"\",row\n")
    }
}
