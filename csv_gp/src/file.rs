use std::{fs::File, io, path::Path};

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::error::{CSVError, UnknownEncoding};

pub(crate) fn read_encoded_file(
    filename: impl AsRef<Path>,
    encoding: &str,
) -> Result<impl io::Read, CSVError> {
    let file = File::open(filename)?;

    if let Some(encoding) = Encoding::for_label(encoding.as_bytes()) {
        Ok(DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding))
            .build(file))
    } else {
        Err(UnknownEncoding::new(encoding.into()).into())
    }
}
