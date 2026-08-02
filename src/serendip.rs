use std::path::Path;
use std::io;

mod parsing;

use parsing::{decode_blob_format, decode_zip_format};
use parsing::zip::SerendipZip;

pub enum SerendipThermogram {
    Zip(SerendipZip),
    // Blob(SerendipBlob),
}

impl SerendipThermogram {
    pub fn new_from_path(
        file_path: &Path,
    ) -> Result<SerendipThermogram, io::Error> {
        let bytes = std::fs::read(file_path)?;
        SerendipThermogram::new_from_bytes(&bytes)
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Result<SerendipThermogram, io::Error> {
        let t = match bytes.starts_with(b"PK\x03\x04") {
            true => decode_zip_format(bytes).map(|t| SerendipThermogram::Zip(t)),
            false => decode_blob_format(),
        };

        t.ok_or(io::Error::new(io::ErrorKind::InvalidData, "Could not decode thermogram"))
    }
}


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
