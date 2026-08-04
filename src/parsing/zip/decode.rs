use std::collections::HashMap;
use std::io::{Cursor, Read};

use log::debug;
use zip::ZipArchive;

use crate::markers::Marker;
use crate::parsing::zip::format::{
    CAL_TEMP_DATA_REX_FILE, CalibrationData, CameraInfo, IrData, IrImageInfo, Rex,
    extract_calibration_data, extract_camera_info, extract_ir_data, extract_ir_dimensions,
    extract_ir_image_info, extract_markers, extract_rex, extract_visuals,
};

#[derive(Clone, Debug)]
pub struct SerendipZip {
    pub thermal: ThermalData,
    pub ir_image_info: IrImageInfo,
    pub camera_info: CameraInfo,
    pub calibration_data: CalibrationData,
    /// The measurement markers with their coordinates as stored in the
    /// file: in `IrImageInfo`'s width/height space, which is twice the
    /// thermal data size. They cannot index the thermal data directly;
    /// see `Marker::to_thermal_space` or `SerendipThermogram::markers`.
    pub markers: Vec<Marker>,
    pub visuals: HashMap<String, Vec<u8>>,
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

    /// The temperature in kelvin of the pixel at `(x, y)`, or `None` if
    /// out of bounds or the calibration curve is missing.
    pub fn kelvin_at(&self, x: u16, y: u16) -> Option<f32> {
        match &self.thermal {
            ThermalData::IrData(ir_data) => {
                let curve = self.calibration_data.curve.as_ref()?;
                ir_data.kelvin_at(x, y, &self.ir_image_info, curve)
            }
            ThermalData::Rex(rex) => rex.kelvin_at(x, y),
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

    /// Palette in ARGB.
    pub fn palette(&self) -> Option<&[[u8; 4]]> {
        match &self.thermal {
            ThermalData::IrData(_) => None,
            ThermalData::Rex(rex) => rex.palette.as_deref(),
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
    let visuals = extract_visuals(&mut files);
    Some(SerendipZip {
        thermal,
        ir_image_info,
        camera_info,
        calibration_data,
        markers,
        visuals,
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
        debug!("Extracting using CalTempDataRex file");
        return extract_rex(files, ir_image_info).map(ThermalData::Rex);
    }

    let (width, height) = extract_ir_dimensions(files, ir_image_info)?;
    debug!("Extracting using IrData");
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

    fn marker_coords(t: &SerendipZip, label: &str) -> Point {
        t.markers
            .iter()
            .find_map(|m| match m.to_thermal_space(t.width(), t.height()) {
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

            let at = |label| {
                let Point { x, y } = marker_coords(&t, label);
                t.kelvin_at(x as u16, y as u16).expect("in bounds")
            };
            assert_eq!(at("Hotpoint"), max, "{name}");
            assert_eq!(at("Coldpoint"), min, "{name}");
        }
    }

    /// `kelvin_at` must agree with the full `kelvin()` buffer on both
    /// thermal layouts, and reject out-of-bounds coordinates.
    #[test]
    fn kelvin_at_matches_full_decode() {
        // Ti400: IrData; TiS75+: Rex
        for name in ["fluke_ti400_1", "fluke_tis75_1"] {
            let t = decode_sample(name);
            let kelvin = t.kelvin().expect("kelvin");
            let (w, h) = (t.width(), t.height());

            for (x, y) in [(0, 0), (w / 2, h / 2), (w - 1, h - 1)] {
                let expected = kelvin[usize::from(y) * usize::from(w) + usize::from(x)];
                assert_eq!(t.kelvin_at(x, y), Some(expected), "{name} at ({x}, {y})");
            }

            assert_eq!(t.kelvin_at(w, 0), None, "{name}");
            assert_eq!(t.kelvin_at(0, h), None, "{name}");
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

    /// Reads a SmartView "all temperatures" export from `thermograms/
    /// reference/`: UTF-16 with BOM, tab-separated, decimal commas,
    /// temperatures in °F. Data rows start with a numeric row index,
    /// which is dropped, as are the title and column-header lines.
    fn read_reference_fahrenheit(name: &str) -> Vec<f32> {
        let path = format!(
            "{}/thermograms/reference/{name}.txt",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(path).expect("reference file present");
        let units: Vec<u16> = bytes[2..] // Skip the BOM
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let text = String::from_utf16(&units).expect("valid UTF-16");

        let mut temperatures = Vec::new();
        for line in text.lines() {
            let mut cells = line.split('\t');
            let is_data_row = cells.next().is_some_and(|c| c.parse::<u32>().is_ok());
            if !is_data_row {
                continue;
            }
            let row = cells
                .filter(|c| !c.trim().is_empty())
                .map(|c| c.replace(',', ".").parse::<f32>().expect("numeric cell"));
            temperatures.extend(row);
        }
        temperatures
    }

    /// Per-pixel regression against SmartView's own temperature exports,
    /// the ground truth for the classic decode pipeline.
    #[test]
    fn ti400_kelvin_matches_smartview_reference() {
        for name in ["fluke_ti400_1", "fluke_ti400_2", "fluke_ti400_3"] {
            let t = decode_sample(name);
            let kelvin = t.kelvin().expect("kelvin");
            let reference = read_reference_fahrenheit(name);
            assert_eq!(kelvin.len(), reference.len(), "{name}");

            let fahrenheit = kelvin.iter().map(|k| (k - 273.15) * 1.8 + 32.0);
            let diffs: Vec<f32> = fahrenheit
                .zip(&reference)
                .map(|(ours, theirs)| (ours - theirs).abs())
                .collect();

            let mean = diffs.iter().sum::<f32>() / diffs.len() as f32;
            let max = diffs.iter().copied().fold(0.0, f32::max);
            // Observed: mean ≤ 0.06 °F, max 3.24 °F (a few pixels at a
            // calibration band edge in sample 1)
            assert!(mean < 0.1, "{name}: mean diff {mean} °F");
            assert!(max < 4.0, "{name}: max diff {max} °F");
        }
    }

    #[test]
    fn decodes_ti401p_samples() {
        for i in 1..=9 {
            let name = format!("fluke_ti401p_{i}");
            let t = decode_sample(&name);

            assert_eq!(t.width(), 640, "{name}");
            assert_eq!(t.height(), 480, "{name}");
            assert!(matches!(t.thermal, ThermalData::IrData(_)), "{name}");

            let kelvin = t.kelvin().expect("kelvin");
            assert_eq!(kelvin.len(), 640 * 480, "{name}");
            let nans = kelvin.iter().filter(|k| k.is_nan()).count();
            assert_eq!(nans, 0, "{name}");
        }
    }

    /// The embedded temperature scale is the camera's own display range,
    /// which tracks the scene extremes when auto-scaled, as these portrait
    /// samples are. No SmartView reference exports exist for the Ti401P,
    /// so this is the best available check of the temperature computation.
    #[test]
    fn ti401p_kelvin_range_tracks_embedded_scale() {
        for i in 1..=9 {
            let name = format!("fluke_ti401p_{i}");
            let t = decode_sample(&name);
            let kelvin = t.kelvin().expect("kelvin");

            let min = kelvin.iter().copied().fold(f32::INFINITY, f32::min);
            let max = kelvin.iter().copied().fold(f32::NEG_INFINITY, f32::max);

            let scale = t.ir_image_info.scale.as_ref().expect("scale present");
            let scale_min = scale.min + 273.15;
            let scale_max = scale.max + 273.15;

            // Observed agreement is within 2 K; allow 2.5 K
            assert!(
                (min - scale_min).abs() < 2.5,
                "{name}: {min} vs {scale_min}"
            );
            assert!(
                (max - scale_max).abs() < 2.5,
                "{name}: {max} vs {scale_max}"
            );
        }
    }

    #[test]
    fn tis75_palette_present() {
        let t = decode_sample("fluke_tis75_1");
        let palette = t.palette().expect("palette present");
        assert_eq!(palette.len(), 256);
    }

    /// IrData-based files carry no palette; `palette()` must say so
    /// rather than fail.
    #[test]
    fn ti400_has_no_palette() {
        let t = decode_sample("fluke_ti400_1");
        assert!(matches!(t.thermal, ThermalData::IrData(_)));
        assert_eq!(t.palette(), None);
    }

    #[test]
    fn ti400_visuals_present() {
        let thermogram = decode_sample("fluke_ti400_1");
        assert_eq!(thermogram.visuals.len(), 2);
        assert!(thermogram.visuals.contains_key("Images/Main/028001E0.jpg"));
        assert!(thermogram.visuals.contains_key("Images/Main/050003C0.jpg"));
    }
}
