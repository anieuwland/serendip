//! Decodes `CameraInfo.gpbenc`: a protobuf blob holding camera
//! identification (manufacturer, serial numbers) and resolutions.

use std::collections::HashMap;

use log::warn;
use prost::Message;

const CAMERA_INFO_FILE: &'static str = "CameraInfo.gpbenc";

/// Camera identification and capabilities.
#[derive(Clone, PartialEq, Message)]
pub struct CameraInfo {
    #[prost(uint32, tag = "12")]
    pub vl_width: u32,
    #[prost(uint32, tag = "13")]
    pub vl_height: u32,
    #[prost(string, tag = "22")]
    pub manufacturer: String,
    #[prost(string, tag = "23")]
    pub engine_serial: String,
    #[prost(string, tag = "24")]
    pub camera_serial: String,
}

/// Extracts and decodes CameraInfo.gpbenc from the unzipped files.
pub fn extract_camera_info(files: &HashMap<String, Vec<u8>>) -> Option<CameraInfo> {
    let bytes = files.get(CAMERA_INFO_FILE)?;
    CameraInfo::decode(bytes.as_slice())
        .inspect_err(|e| warn!("Could not decode {CAMERA_INFO_FILE}: {e}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ti400_sample() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/thermograms/fluke_ti400_1/CameraInfo.gpbenc"
        ))
        .expect("sample file present");
        let files = HashMap::from([(CAMERA_INFO_FILE.to_string(), bytes)]);

        let info = extract_camera_info(&files).expect("decodes");

        assert_eq!(info.vl_width, 2560);
        assert_eq!(info.vl_height, 1920);
        assert_eq!(info.manufacturer, "Fluke Thermography");
        assert_eq!(info.engine_serial, "G13080059");
        assert_eq!(info.camera_serial, "M13080110");
    }

    #[test]
    fn absent_file_yields_none() {
        assert_eq!(extract_camera_info(&HashMap::new()), None);
    }
}
