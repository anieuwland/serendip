use std::path::Path;
use std::io;

mod parsing;

use parsing::{decode_blob_format, decode_zip_format};

pub struct SerendipThermogram {
    raw_data: Vec<u8>
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
            true => decode_zip_format(bytes),
            false => decode_blob_format(),
        };

        t.ok_or(io::Error::new(io::ErrorKind::InvalidData, "Could not decode thermogram"))
    }

    pub fn kelvin(&self) -> Vec<u8> {
        return self.raw_data.clone();
    }
}
