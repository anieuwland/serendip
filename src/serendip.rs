use std::collections::HashMap;
use std::io;
use std::path::Path;

pub mod markers;
pub mod parsing;

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
        match self {
            Zip(t) => t.kelvin(),
        }
    }

    /// Returns the set of encoded images present in this is2.
    pub fn visuals(&self) -> &HashMap<String, Vec<u8>> {
        match self {
            Zip(t) => &t.visuals
        }
    }

    /// Deterministically returns a single encoded image from the set
    /// of images in this is2.
    ///
    /// To get the decoded variant, see `visual_decoded`.
    pub fn visual(&self) -> Option<&[u8]> {
        let (_, bytes) = self.visuals().iter().min_by_key(|(path, _)| *path)?;
        Some(bytes)
    }

    pub fn width(&self) -> u16 {
        match self {
            Zip(t) => t.width(),
        }
    }

    pub fn height(&self) -> u16 {
        match self {
            Zip(t) => t.height(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_sample(name: &str) -> SerendipThermogram {
        let path = format!("{}/thermograms/{name}.is2", env!("CARGO_MANIFEST_DIR"));
        SerendipThermogram::new_from_path(Path::new(&path)).expect("decodes")
    }

    /// `visual()` must pick the lexicographically first path so the choice
    /// is deterministic regardless of HashMap iteration order.
    #[test]
    fn ti400_visual_picks_first_path_deterministically() {
        let t = decode_sample("fluke_ti400_1");

        let visual = t.visual().expect("visual present");
        let expected = t.visuals().get("Images/Main/028001E0.jpg").expect("main jpg present");
        assert_eq!(visual, expected);
        assert!(visual.starts_with(&[0xFF, 0xD8])); // JPEG SOI marker
    }
}
