//! Decodes `CalTempDataRex.gpbenc`: the thermal data of "Rex" family
//! models (TiS75+, PTi120), which have no `Images/Main/IR.data`.
//!
//! The file is a protobuf whose submessage 4 holds three repeated fields:
//!
//! - tag 4: a 16384-entry monotone lookup table, presumed the camera's
//!   counts-to-temperature curve, in decicelsius.
//! - tag 11: the thermal data, width × height signed varints. These are
//!   **already temperatures in decicelsius**, not raw counts: verified on
//!   four TiS75+ samples, where the embedded hotpoint and coldpoint
//!   markers land exactly on the thermal data's maximum and minimum.
//! - tag 12: a 256-entry ARGB display palette (not decoded here).
//!
//! The IR dimensions appear nowhere in Rex files. They come from
//! `IRImageInfo.gpbenc` (see `IrImageInfo::stored_thermal_dimensions`),
//! verified against the pixel count. Confirmed on TiS75+ samples only.

use std::collections::HashMap;

use log::warn;
use prost::Message;

use crate::parsing::zip::format::IrImageInfo;

pub(crate) const CAL_TEMP_DATA_REX_FILE: &'static str = "CalTempDataRex.gpbenc";

/// Wire form of CalTempDataRex.gpbenc. Only the fields serendip needs.
#[derive(Clone, PartialEq, Message)]
struct CalTempDataRex {
    #[prost(message, optional, tag = "4")]
    thermal: Option<RexThermalData>,
}

/// Wire form of the thermal data submessage.
#[derive(Clone, PartialEq, Message)]
struct RexThermalData {
    /// Counts-to-decicelsius lookup table. Not needed to get temperatures
    /// (the thermal data is already converted) but kept as format
    /// documentation.
    #[prost(int64, repeated, tag = "4")]
    lut: Vec<i64>,
    /// The thermal data, row-major, in decicelsius.
    #[prost(int64, repeated, tag = "11")]
    decicelsius: Vec<i64>,
}

/// The thermal data of a Rex-family thermogram.
#[derive(Clone, Debug, PartialEq)]
pub struct Rex {
    /// Row-major temperatures in decicelsius (227 = 22.7 °C).
    ///
    /// Format notes suggest −1 may be an invalid-pixel sentinel on some
    /// models, but it is indistinguishable from a legitimate −0.1 °C and
    /// is therefore treated as a temperature.
    pub decicelsius: Vec<i64>,
    pub width: u16,
    pub height: u16,
}

impl Rex {
    /// Returns the data as temperatures in kelvin.
    ///
    /// The data is already stored in decicelsius so fetching in kelvin
    /// requires a simple conversion.
    pub fn kelvin(&self) -> Vec<f32> {
        self.decicelsius
            .iter()
            .map(|d| decicelsius_to_kelvin(*d))
            .collect()
    }

    /// The temperature in kelvin of the pixel at `(x, y)`, or `None` if
    /// out of bounds.
    pub fn kelvin_at(&self, x: u16, y: u16) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = usize::from(y) * usize::from(self.width) + usize::from(x);
        Some(decicelsius_to_kelvin(self.decicelsius[index]))
    }
}

fn decicelsius_to_kelvin(decicelsius: i64) -> f32 {
    decicelsius as f32 / 10.0 + 273.15
}

/// Extracts and decodes CalTempDataRex.gpbenc from the unzipped files.
///
/// Needs `mut` because it takes ownership of the file and removes it from
/// the hash map, so the (sizeable) raw bytes are dropped after decoding.
///
/// # Arguments
/// - `files` - A hashmap which it expects to find `CAL_TEMP_DATA_REX_FILE` in.
/// - `ir_image_info` - Provides the dimensions, which Rex files store nowhere.
pub fn extract_rex(
    files: &mut HashMap<String, Vec<u8>>,
    ir_image_info: &IrImageInfo,
) -> Option<Rex> {
    let bytes = files.remove(CAL_TEMP_DATA_REX_FILE)?;
    let rex = CalTempDataRex::decode(bytes.as_slice())
        .inspect_err(|e| warn!("Could not decode {CAL_TEMP_DATA_REX_FILE}: {e}"))
        .ok()?;

    let thermal = rex.thermal?;
    let num_pixels = thermal.decicelsius.len();
    let (width, height) = ir_image_info.stored_thermal_dimensions()?;
    if usize::from(width) * usize::from(height) != num_pixels {
        warn!(
            "IRImageInfo's stored dimensions ({width} x {height}) don't fit \
             {CAL_TEMP_DATA_REX_FILE}'s {num_pixels} pixels"
        );
        return None;
    }

    Some(Rex {
        decicelsius: thermal.decicelsius,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tis75_files() -> HashMap<String, Vec<u8>> {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/thermograms/fluke_tis75_1/CalTempDataRex.gpbenc"
        ))
        .expect("sample file present");
        HashMap::from([(CAL_TEMP_DATA_REX_FILE.to_string(), bytes)])
    }

    fn tis75_ir_image_info() -> IrImageInfo {
        IrImageInfo {
            width: 768,
            height: 576,
            ..Default::default()
        }
    }

    #[test]
    fn decodes_tis75_sample() {
        let rex = extract_rex(&mut tis75_files(), &tis75_ir_image_info()).expect("decodes");

        assert_eq!(rex.width, 384);
        assert_eq!(rex.height, 288);
        assert_eq!(rex.decicelsius.len(), 384 * 288);

        // Scene extremes known from the sample's hot/coldpoint markers
        let min = *rex.decicelsius.iter().min().unwrap();
        let max = *rex.decicelsius.iter().max().unwrap();
        assert_eq!(min, -160); // -16.0 °C
        assert_eq!(max, 227); // 22.7 °C
    }

    #[test]
    fn kelvin_converts_decicelsius() {
        let rex = Rex {
            decicelsius: vec![0, 227, -160],
            width: 3,
            height: 1,
        };
        let kelvin = rex.kelvin();
        assert!((kelvin[0] - 273.15).abs() < 1e-3);
        assert!((kelvin[1] - 295.85).abs() < 1e-3);
        assert!((kelvin[2] - 257.15).abs() < 1e-3);
    }

    #[test]
    fn wrong_dimensions_yield_none() {
        let info = IrImageInfo {
            width: 640,
            height: 480,
            ..Default::default()
        };
        assert_eq!(extract_rex(&mut tis75_files(), &info), None);
    }

    #[test]
    fn absent_file_yields_none() {
        assert_eq!(
            extract_rex(&mut HashMap::new(), &tis75_ir_image_info()),
            None
        );
    }
}
