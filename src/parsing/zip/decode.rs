use std::collections::HashMap;
use std::io::{Cursor, Read};

use log::debug;
use zip::ZipArchive;

use crate::markers::Marker;
use crate::parsing::zip::format::{
    CAL_TEMP_DATA_REX_FILE, CalibrationData, CameraInfo, IrData, IrImageInfo, Rex,
    extract_calibration_data, extract_camera_info, extract_ir_data, extract_ir_dimensions,
    extract_ir_image_info, extract_markers, extract_rex,
};

#[derive(Clone, Debug)]
pub struct SerendipZip {
    pub thermal: ThermalData,
    pub ir_image_info: IrImageInfo,
    pub camera_info: CameraInfo,
    pub calibration_data: CalibrationData,
    pub markers: Vec<Marker>,
}

/// The thermal data, whose storage can differ per model.
#[derive(Clone, Debug)]
pub enum ThermalData {
    /// Raw counts in `Images/Main/IR.data` (Ti400, Ti401P, ...)
    IrData(IrData),
    /// Rex family (TiS75+, ...): decicelsius in `CalTempDataRex.gpbenc`
    Rex(Rex),
}

impl SerendipZip {
    /// Get thermogram data in kelvin.
    pub fn kelvin(&self) -> Option<Vec<f32>> {
        debug!("Getting kelvin");
        match &self.thermal {
            ThermalData::IrData(ir_data) => {
                let params = &self.ir_image_info;
                let curve = self.calibration_data.curve.as_ref()?;
                ir_data.kelvin(params, curve)
            }
            ThermalData::Rex(rex) => Some(rex.kelvin()),
        }
    }

    pub fn width(&self) -> u16 {
        match &self.thermal {
            ThermalData::IrData(ir_data) => ir_data.width,
            ThermalData::Rex(rex) => rex.width,
        }
    }

    pub fn height(&self) -> u16 {
        match &self.thermal {
            ThermalData::IrData(ir_data) => ir_data.height,
            ThermalData::Rex(rex) => rex.height,
        }
    }
}

pub fn decode_zip_format(bytes: &[u8]) -> Option<SerendipZip> {
    let mut files = unzip(bytes).ok()?;
    let ir_image_info = extract_ir_image_info(&files)?;
    debug!("IRImageInfo: {ir_image_info:?}");
    let camera_info = extract_camera_info(&files)?;
    debug!("CameraInfo: {camera_info:?}");
    let calibration_data = extract_calibration_data(&files)?;
    debug!("CalibrationData: {calibration_data:?}");
    let thermal = extract_thermal_data(&mut files, &ir_image_info)?;
    let markers = extract_markers(&mut files);
    debug!("Markers: {markers:?}");
    Some(SerendipZip {
        thermal,
        ir_image_info,
        camera_info,
        calibration_data,
        markers,
    })
}

/// Extracts the thermal data, in whichever layout the file carries.
///
/// `CalTempDataRex.gpbenc` is preferred over `IR.data` as its values are
/// already temperatures, needing no calibration curves. No file has been
/// seen carrying both, however.
fn extract_thermal_data(
    files: &mut HashMap<String, Vec<u8>>,
    ir_image_info: &IrImageInfo,
) -> Option<ThermalData> {
    if files.contains_key(CAL_TEMP_DATA_REX_FILE) {
        return extract_rex(files, ir_image_info).map(ThermalData::Rex);
    }

    let (width, height) = extract_ir_dimensions(files, ir_image_info)?;
    debug!("Dimensions: {width} x {height}");
    extract_ir_data(files, width, height).map(ThermalData::IrData)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markers::Point;

    fn decode_sample(name: &str) -> SerendipZip {
        let path = format!("{}/thermograms/{name}.is2", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(path).expect("sample file present");
        decode_zip_format(&bytes).expect("decodes")
    }

    /// The kelvin value at a marker's position. Marker coordinates are in
    /// IRImageInfo's width/height space, twice the IR size on a TiS75+.
    fn kelvin_at(t: &SerendipZip, kelvin: &[f32], coords: &Point) -> f32 {
        let x = coords.x as usize / 2;
        let y = coords.y as usize / 2;
        kelvin[y * usize::from(t.width()) + x]
    }

    fn marker_coords<'a>(t: &'a SerendipZip, label: &str) -> &'a Point {
        t.markers
            .iter()
            .find_map(|m| match m {
                Marker::Point { coords, metadata } if metadata.label2 == label => Some(coords),
                _ => None,
            })
            .expect("marker present")
    }

    #[test]
    fn decodes_tis75_samples() {
        for name in [
            "fluke_tis75_1",
            "fluke_tis75_2",
            "fluke_tis75_3",
            "fluke_tis75_4",
        ] {
            let t = decode_sample(name);

            assert_eq!(t.width(), 384, "{name}");
            assert_eq!(t.height(), 288, "{name}");
            assert!(matches!(t.thermal, ThermalData::Rex(_)), "{name}");

            assert_eq!(t.camera_info.manufacturer, "Fluke", "{name}");

            let kelvin = t.kelvin().expect("kelvin");
            assert_eq!(kelvin.len(), 384 * 288, "{name}");

            // All temperatures earthly. The TiS75+ specs -30..155 °C but
            // reports beyond it: sample 4's coldest pixel is -37.9 °C.
            let min = kelvin.iter().copied().fold(f32::INFINITY, f32::min);
            let max = kelvin.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            assert!(min >= 173.15, "{name}: min {min}");
            assert!(max <= 473.15, "{name}: max {max}");
        }
    }

    /// The embedded hot/coldpoint markers are the camera's own scene
    /// extremes; the decoded thermal data must agree exactly at those
    /// positions.
    #[test]
    fn tis75_markers_land_on_thermal_extremes() {
        for name in [
            "fluke_tis75_1",
            "fluke_tis75_2",
            "fluke_tis75_3",
            "fluke_tis75_4",
        ] {
            let t = decode_sample(name);
            let kelvin = t.kelvin().expect("kelvin");

            let min = kelvin.iter().copied().fold(f32::INFINITY, f32::min);
            let max = kelvin.iter().copied().fold(f32::NEG_INFINITY, f32::max);

            let hot = kelvin_at(&t, &kelvin, marker_coords(&t, "Hotpoint"));
            let cold = kelvin_at(&t, &kelvin, marker_coords(&t, "Coldpoint"));
            assert_eq!(hot, max, "{name}");
            assert_eq!(cold, min, "{name}");
        }
    }

    #[test]
    fn tis75_sample_1_scene_temperatures() {
        let t = decode_sample("fluke_tis75_1");
        let kelvin = t.kelvin().expect("kelvin");

        // Scene extremes: -16.0 °C and 22.7 °C (from the camera's markers)
        let min = kelvin.iter().copied().fold(f32::INFINITY, f32::min);
        let max = kelvin.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!((min - 257.15).abs() < 1e-3, "min {min}");
        assert!((max - 295.85).abs() < 1e-3, "max {max}");
    }

    #[test]
    fn ti400_still_decodes_as_classic() {
        let t = decode_sample("fluke_ti400_1");

        assert_eq!(t.width(), 320);
        assert_eq!(t.height(), 240);
        assert!(matches!(t.thermal, ThermalData::IrData(_)));

        let kelvin = t.kelvin().expect("kelvin");
        assert_eq!(kelvin.len(), 320 * 240);
    }
}
