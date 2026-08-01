use std::io;
use std::path::Path;

mod parsing;

use parsing::zip::SerendipZip;
use parsing::{decode_blob_format, decode_zip_format};

use crate::SerendipThermogram::Zip;

#[derive(Clone, Debug)]
pub enum SerendipThermogram {
    Zip(SerendipZip),
    // Blob(SerendipBlob),
}

impl SerendipThermogram {
    pub fn new_from_path(file_path: &Path) -> Result<SerendipThermogram, io::Error> {
        let bytes = std::fs::read(file_path)?;
        SerendipThermogram::new_from_bytes(&bytes)
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Result<SerendipThermogram, io::Error> {
        let t = match bytes.starts_with(b"PK\x03\x04") {
            true => decode_zip_format(bytes).map(|t| SerendipThermogram::Zip(t)),
            false => decode_blob_format(),
        };

        t.ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Could not decode thermogram",
        ))
    }

    pub fn kelvin(&self) -> Option<Vec<f32>> {
        match (self) {
            Zip(t) => t.kelvin(),
        }
    }

    pub fn width(&self) -> u16 {
        match self {
            Zip(t) => t.ir_data.width,
        }
    }

    pub fn height(&self) -> u16 {
        match self {
            Zip(t) => t.ir_data.height,
        }
    }
}
