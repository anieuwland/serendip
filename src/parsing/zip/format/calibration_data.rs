//! Decodes `CalibrationData.gpbenc`: the radiometric calibration curve
//! used to map raw sensor values to temperatures.

use std::collections::HashMap;

use log::warn;
use prost::Message;

const CALIBRATION_DATA_FILE: &'static str = "CalibrationData.gpbenc";

/// Calibration data unique to this lens.
///
/// The curve is not expected to be absent (and may be assumed to be present)
/// but prost forces its optionality as protobuf can't guarantee its presence.
#[derive(Clone, PartialEq, Message)]
pub struct CalibrationData {
    #[prost(uint32, tag = "1")]
    pub date: u32, // Presumably YYYYMMDD
    #[prost(message, optional, tag = "3")]
    pub curve: Option<CalibrationCurve>,
}

/// A piecewise calibration curve with per-temperature-band coefficients.
///
/// A calibration curve has multiple segments, because 1 segment can't
/// accurately fit the sensor over all degrees. Each segment is a temperature
/// bands (from..to) with coefficients specific to that band.
#[derive(Clone, PartialEq, Message)]
pub struct CalibrationCurve {
    #[prost(float, tag = "3")]
    pub range_min: f32, // Presumed Celsius, like all fields here
    #[prost(float, tag = "4")]
    pub range_max: f32,
    #[prost(float, tag = "5")]
    pub extended_min: f32,
    #[prost(float, tag = "6")]
    pub extended_max: f32,
    #[prost(message, repeated, tag = "9")]
    pub bands: Vec<CalibrationBand>,
}

impl CalibrationCurve {
    pub fn get_raw_bands(&self) -> Vec<[f32; 5]> {
        self.bands.iter().map(|s| s.to_raw_band()).collect()
    }
}

/// One temperature band and its quadratic coefficients.
#[derive(Clone, PartialEq, Message)]
pub struct CalibrationBand {
    #[prost(float, tag = "1")]
    pub from: f32,
    #[prost(float, tag = "2")]
    pub to: f32,
    #[prost(float, tag = "3")]
    pub c: f32,
    #[prost(float, tag = "4")]
    pub b: f32,
    #[prost(float, tag = "5")]
    pub a: f32,
}

impl CalibrationBand {
    pub fn to_raw_band(&self) -> [f32; 5] {
        let from = self.a * self.from * self.from + self.b * self.from + self.c;
        let to =  self.a * self.to * self.to + self.b * self.to + self.c;
        [from, to, self.a, self.b, self.c]
    }
}

/// Extracts and decodes CalibrationData.gpbenc from the unzipped files.
pub fn extract_calibration_data(files: &HashMap<String, Vec<u8>>) -> Option<CalibrationData> {
    let bytes = files.get(CALIBRATION_DATA_FILE)?;
    CalibrationData::decode(bytes.as_slice())
        .inspect_err(|e| warn!("Could not decode {CALIBRATION_DATA_FILE}: {e}"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_ti400_sample() {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/thermograms/fluke_ti400_1/CalibrationData.gpbenc"
        ))
        .expect("sample file present");
        let files = HashMap::from([(CALIBRATION_DATA_FILE.to_string(), bytes)]);

        let data = extract_calibration_data(&files).expect("decodes");
        assert_eq!(data.date, 20130306);

        let curve = data.curve.expect("curve present");
        assert_eq!(curve.range_min, -10.0);
        assert_eq!(curve.range_max, 80.0);
        assert_eq!(curve.extended_min, -20.0);
        assert_eq!(curve.extended_max, 80.0);

        assert_eq!(curve.bands.len(), 6);
        let first = &curve.bands[0];
        assert_eq!(first.from, -180.0);
        assert_eq!(first.to, -120.0);
        assert!((first.c - 1016.96).abs() < 0.01);
        assert!((first.b - 11.77).abs() < 0.01);
        assert!((first.a - 0.0341).abs() < 0.0001);
        let last = &curve.bands[5];
        assert_eq!(last.from, 200.0);
        assert_eq!(last.to, 350.0);
    }

    #[test]
    fn absent_file_yields_none() {
        assert_eq!(extract_calibration_data(&HashMap::new()), None);
    }
}
