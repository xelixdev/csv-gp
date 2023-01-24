use crate::error::{CSVError, UnknownEncoding};

use std::{fs::File, io, path::Path};

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

pub(crate) fn read_encoded_file(
    filename: impl AsRef<Path>,
    encoding: &str,
) -> Result<impl io::BufRead, CSVError> {
    let file = File::open(filename)?;

    if let Some(encoding) = Encoding::for_label(encoding.as_bytes()) {
        Ok(io::BufReader::new(
            DecodeReaderBytesBuilder::new()
                .encoding(Some(encoding))
                .build(file),
        ))
    } else {
        Err(CSVError::UnknownEncoding(UnknownEncoding::Encoding(
            encoding.into(),
        )))
    }
}
