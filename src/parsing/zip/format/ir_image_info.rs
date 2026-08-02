//! Decodes `Images/Main/IRImageInfo.gpbenc`: a protobuf blob holding the
//! thermal parameters (emissivity, background temperature, transmission)
//! and the display temperature scale.

use std::collections::HashMap;

use log::warn;
use prost::Message;

const IR_IMAGE_INFO_FILE: &'static str = "Images/Main/IRImageInfo.gpbenc";

/// Thermal parameters of the IR capture.
#[derive(Clone, PartialEq, Message)]
pub struct IrImageInfo {
    #[prost(uint32, tag = "4")]
    pub width: u32, // Unclear what coordinate space; not 1-to-1 IR data size
    #[prost(uint32, tag = "5")]
    pub height: u32, // Unclear what coordinate space; not 1-to-1 IR data size
    #[prost(float, tag = "10")]
    pub emissivity: f32,
    #[prost(float, tag = "11")]
    pub background_temperature: f32, // Presumed in Celsius
    #[prost(float, tag = "12")]
    pub transmission: f32,
    #[prost(message, optional, tag = "14")]
    pub scale: Option<TemperatureScale>,
}

impl IrImageInfo {
    /// The dimensions of the thermal data as stored: half this struct's
    /// width/height fields.
    ///
    /// On all samples seen those fields are exactly twice the stored
    /// thermal data size. For `IR.data` that includes its one header row
    /// (Ti400: 640 × 482 → 320 × 241 = 320 × (240 + 1)); for
    /// `CalTempDataRex.gpbenc` it is the pixel data itself (TiS75+:
    /// 768 × 576 → 384 × 288). Callers must verify the result against
    /// their file's actual length.
    pub fn stored_thermal_dimensions(&self) -> Option<(u16, u16)> {
        let width = u16::try_from(self.width / 2).ok()?;
        let height = u16::try_from(self.height / 2).ok()?;
        Some((width, height))
    }
}

/// The temperature range of the display palette, presumably Celsius.
#[derive(Clone, PartialEq, Message)]
pub struct TemperatureScale {
    #[prost(float, tag = "6")]
    pub min: f32,
    #[prost(float, tag = "7")]
    pub max: f32,
}

/// Extracts and decodes IRImageInfo.gpbenc from the unzipped files.
pub fn extract_ir_image_info(files: &HashMap<String, Vec<u8>>) -> Option<IrImageInfo> {
    let bytes = files.get(IR_IMAGE_INFO_FILE)?;
    IrImageInfo::decode(bytes.as_slice())
        .inspect_err(|e| warn!("Could not decode {IR_IMAGE_INFO_FILE}: {e}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ti400_sample() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/thermograms/fluke_ti400_1/Images/Main/IRImageInfo.gpbenc"
        ))
        .expect("sample file present");
        let files = HashMap::from([(IR_IMAGE_INFO_FILE.to_string(), bytes)]);

        let info = extract_ir_image_info(&files).expect("decodes");

        assert_eq!(info.width, 640);
        assert_eq!(info.height, 482);
        assert!((info.emissivity - 0.95).abs() < 1e-6);
        assert!((info.background_temperature - 22.0).abs() < 1e-6);
        assert!((info.transmission - 1.0).abs() < 1e-6);

        let scale = info.scale.expect("scale present");
        assert!((scale.min - 16.4).abs() < 0.01);
        assert!((scale.max - 79.46).abs() < 0.01);
    }

    #[test]
    fn absent_file_yields_none() {
        assert_eq!(extract_ir_image_info(&HashMap::new()), None);
    }
}
