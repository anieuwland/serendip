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
