use crate::{csv_details::CSVDetails, error::CSVError};

use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
};

/// Saves a file containing only the valid rows according to the passed CSVDetails
pub(crate) fn save_valid_file(
    path: impl AsRef<Path>,
    csv_details: &CSVDetails,
    output_path: impl AsRef<Path>,
) -> Result<(), CSVError> {
    // Create intermediate directories
    if let Some(parent) = output_path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let mut reader = BufReader::new(File::open(path)?);
    let mut writer = BufWriter::new(File::create(output_path)?);

    let mut pos = 0u64;
    for valid_range in &csv_details.valid_byte_ranges {
        reader.seek_relative(valid_range.start - pos as i64)?;
        let mut bytes = reader.by_ref().take(valid_range.length);
        io::copy(&mut bytes, &mut writer)?;
        pos += valid_range.length;
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

    use crate::{checker::check_file, csv_details::ByteRange};

    use super::*;

    fn rows_with_valid(path: impl AsRef<Path>) -> (CSVDetails, impl AsRef<Path>) {
        fs::write(&path, "a,b\ninvalid\n\"\"\"quoted\"\"\",row").unwrap();
        let mut csv_details = CSVDetails::new();
        csv_details.valid_byte_ranges = vec![ByteRange::new(0, 3), ByteRange::new(12, 15)];

        (csv_details, path)
    }

    #[test]
    fn test_save_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let (csv_details, path) = rows_with_valid(dir.path().join("test_save_valid_file_base.csv"));
        let out_path = dir.path().join("test_save_valid_file.csv");

        save_valid_file(path, &csv_details, &out_path).unwrap();

        let file = fs::read_to_string(out_path).unwrap();

        assert_eq!(file, "a,b\n\"\"\"quoted\"\"\",row\n")
    }

    #[test]
    fn test_save_valid_file_create_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let (csv_details, path) = rows_with_valid(dir.path().join("create_parent_dir.csv"));
        let out_path = dir.path().join("parent").join("child.csv");

        save_valid_file(path, &csv_details, &out_path).unwrap();

        let file = fs::read_to_string(out_path).unwrap();

        assert_eq!(file, "a,b\n\"\"\"quoted\"\"\",row\n")
    }
}
