use std::fs;

use encoding_rs::Encoding;

use crate::error::{CSVError, UnknownEncoding};

pub(crate) fn read_encoded_file(filename: String, encoding: &str) -> Result<String, CSVError> {
    let bytes = fs::read(filename)?;

    if let Some(encoding) = Encoding::for_label(encoding.as_bytes()) {
        let (data, _encoding, _errors) = encoding.decode(&bytes);
        Ok(data.to_string())
    } else {
        Err(UnknownEncoding::new(encoding.into()).into())
    }
}
