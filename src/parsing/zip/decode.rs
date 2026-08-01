use std::collections::HashMap;
use std::io::{Cursor, Read};

use log::debug;
use zip::ZipArchive;

use crate::parsing::zip::format::{
    CalibrationData, CameraInfo, IrData, IrImageInfo, extract_calibration_data,
    extract_camera_info, extract_ir_data, extract_ir_dimensions, extract_ir_image_info,
};

#[derive(Clone, Debug)]
pub struct SerendipZip  {
    pub ir_data: IrData,
    pub ir_image_info: IrImageInfo,
    pub camera_info: CameraInfo,
    pub calibration_data: CalibrationData,
}

impl SerendipZip {
    /// Get thermogram data in kelvin.
    pub fn kelvin(&self) -> Option<Vec<f32>> {
        debug!("Getting kelvin");
        let params = &self.ir_image_info;

        let curve = self.calibration_data.curve.as_ref()?;
        let raw_bands = curve.get_raw_bands();
        let raw_data = self.ir_data.body();

        let kelvin = raw_data.iter().map(|raw| {
            let raw = f32::from(raw.get());
            let maybe_band = raw_bands.iter().find(|b| (b[0]..b[1]).contains(&raw));
            match maybe_band {
                Some(band) => {
                    let a = band[2];
                    let b = band[3];
                    let c = band[4];

                    let apparent_temp = (-b + f32::sqrt(b * b - 4.0 * a * (c - raw))) / (2.0 * a);
                    let apparent_temp = apparent_temp + 273.15;

                    let term1 = apparent_temp.powi(4) - (1.0 - params.emissivity) * params.background_temperature.powi(4);
                    let term2 = params.transmission * params.emissivity;
                    (term1 / term2).powf(0.25)
                },
                None => f32::NAN,
            }
        });

        return Some(kelvin.collect());
    }
}

pub fn decode_zip_format(bytes: &[u8]) -> Option<SerendipZip> {
    let mut files = unzip(bytes).ok()?;
    let (width, height) = extract_ir_dimensions(&files)?;
    debug!("Dimensions: {width} x {height}");
    let ir_image_info = extract_ir_image_info(&files)?;
    debug!("IRImageInfo: {ir_image_info:?}");
    let camera_info = extract_camera_info(&files)?;
    debug!("CameraInfo: {camera_info:?}");
    let calibration_data = extract_calibration_data(&files)?;
    debug!("CalibrationData: {calibration_data:?}");
    let ir_data = extract_ir_data(&mut files, width, height)?;
    Some(SerendipZip { ir_data, ir_image_info, camera_info, calibration_data })
}

fn unzip(bytes: &[u8]) -> zip::result::ZipResult<HashMap<String, Vec<u8>>> {
    debug!("Extracting zip");
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let mut files = HashMap::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        debug!("    Entry: {:?}", entry.name());
        if entry.is_file() {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;
            files.insert(entry.name().to_string(), buf);
        }
    }
    Ok(files)
}
