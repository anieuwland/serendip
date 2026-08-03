use std::collections::HashMap;

use log::{debug, warn};
use zerocopy::FromBytes;
use zerocopy::byteorder::little_endian::U16;

use crate::parsing::zip::format::IrImageInfo;
use crate::parsing::zip::format::calibration_data::CalibrationCurve;

#[derive(Clone, Debug)]
pub struct IrData {
    /// The raw bytes in the file. You probably want `as_u16`.
    pub raw: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

impl IrData {
    /// Returns the raw data as the little endian u16s it actually is.
    ///
    /// Includes header and body, which are also available through convenience
    /// functions.
    pub fn as_u16(&self) -> &[U16] {
        <[U16]>::ref_from_bytes(&self.raw).expect("length verified at construction")
    }

    #[allow(dead_code)]
    pub fn header(&self) -> &[U16] {
        let header = usize::from(self.width);
        &self.as_u16()[..header]
    }

    pub fn body(&self) -> &[U16] {
        let header = usize::from(self.width);
        debug!("IR data size: {:?} {:?}", self.width, self.height);
        let size = usize::from(self.width) * usize::from(self.height);
        &self.as_u16()[header..header + size]
    }

    /// Converts classic-family raw counts to kelvin: the calibration curve
    /// yields an apparent temperature, then emissivity, background
    /// temperature and transmission correct for real-world radiation.
    pub fn kelvin(&self, params: &IrImageInfo, curve: &CalibrationCurve) -> Option<Vec<f32>> {
        let raw_bands = curve.get_raw_bands();
        let raw_data = self.body();

        let kelvin = raw_data
            .iter()
            .map(|raw| raw_to_kelvin(f32::from(raw.get()), &raw_bands, params));
        Some(kelvin.collect())
    }

    /// The temperature in kelvin of the pixel at `(x, y)`, or `None` if
    /// out of bounds. NaN for raw counts outside all calibration bands,
    /// like `kelvin`.
    pub fn kelvin_at(
        &self,
        x: u16,
        y: u16,
        params: &IrImageInfo,
        curve: &CalibrationCurve,
    ) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = usize::from(y) * usize::from(self.width) + usize::from(x);
        let raw = f32::from(self.body()[index].get());
        Some(raw_to_kelvin(raw, &curve.get_raw_bands(), params))
    }
}

/// Converts one raw sensor count to kelvin (see `IrData::kelvin`), or NaN
/// if the count falls outside all calibration bands.
fn raw_to_kelvin(raw: f32, raw_bands: &[[f32; 5]], params: &IrImageInfo) -> f32 {
    let Some(band) = raw_bands.iter().find(|b| (b[0]..b[1]).contains(&raw)) else {
        return f32::NAN;
    };
    let a = band[2];
    let b = band[3];
    let c = band[4];

    let apparent_temp = (-b + f32::sqrt(b * b - 4.0 * a * (c - raw))) / (2.0 * a);
    let apparent_temp = apparent_temp + 273.15;

    let background_kelvin = params.background_temperature + 273.15; // C to K
    let term1 = apparent_temp.powi(4) - (1.0 - params.emissivity) * background_kelvin.powi(4);
    let term2 = params.transmission * params.emissivity;
    (term1 / term2).powf(0.25)
}

const IR_DATA_FILE: &'static str = "Images/Main/IR.data";

/// Extracts the IR frame dimensions, trying sources in decreasing order of
/// authority. Candidates are only accepted if they satisfy IR.data's length
/// equation (see `dimensions_fit_ir_data`).
///
/// 1. `IRImageInfo.gpbenc`'s stored dimensions (see
///    `IrImageInfo::stored_thermal_dimensions`), whose height includes
///    IR.data's one header row, subtracted here.
/// 2. `IR.data`'s own header: u32 LE fields at byte offsets 182 (width - 1)
///    and 186 (height - 1). Verified on Ti400 samples only.
pub fn extract_ir_dimensions(
    files: &HashMap<String, Vec<u8>>,
    ir_image_info: &IrImageInfo,
) -> Option<(u16, u16)> {
    let ir_data = files.get(IR_DATA_FILE)?;

    let without_header_row = |(w, h): (u16, u16)| Some((w, h.checked_sub(1)?));
    let candidates = [
        ir_image_info
            .stored_thermal_dimensions()
            .and_then(without_header_row),
        extract_dimensions_from_ir_data_header(ir_data),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|&(w, h)| dimensions_fit_ir_data(w, h, ir_data.len()))
}

/// Checks dimension candidates against IR.data's layout: a header of `width`
/// u16s followed by a `width * height` u16 payload, and nothing else.
fn dimensions_fit_ir_data(width: u16, height: u16, ir_data_len: usize) -> bool {
    let header = width as usize;
    let size = width as usize * height as usize;
    width > 0 && height > 0 && (header + size) * 2 == ir_data_len
}

/// Reads the IR frame dimensions from IR.data's header.
///
/// These hardcoded byte offsets were seen in Ti400 samples, but other
/// models probably have their own, so this will need to be augmented
/// in time.
///
/// Ti400: u32 LE at 182 and 186 holding width - 1 and height - 1
///
/// TODO: Record dimensions byte offsets for more camera models.
fn extract_dimensions_from_ir_data_header(ir_data: &[u8]) -> Option<(u16, u16)> {
    let field = |offset: usize| -> Option<u32> {
        let bytes = ir_data.get(offset..offset + 4)?;
        Some(u32::from_le_bytes(bytes.try_into().unwrap()))
    };
    let width = u16::try_from(field(182)? + 1).ok()?;
    let height = u16::try_from(field(186)? + 1).ok()?;
    Some((width, height))
}

/// Extracts the infrared data from the hash map.
///
/// The frame dimensions come from `IRImageInfo.gpbenc` (see
/// `extract_ir_dimensions`), since `IR.data` itself has no reliable header
/// fields for them. They are verified here against the file's length:
/// a header of `width` u16s followed by a `width * height` u16 payload.
///
/// Needs `mut` because it takes ownership of the IR.data file and removes
/// it from the hash map. This allows zero-copying.
///
/// # Arguments
/// - `files` - A hashmap which it expects to find `IR_DATA_FILE` in.
/// - `width` - The IR frame width from the file's metadata.
/// - `height` - The IR frame height from the file's metadata.
pub fn extract_ir_data(
    files: &mut HashMap<String, Vec<u8>>,
    width: u16,
    height: u16,
) -> Option<IrData> {
    let raw = files.remove(IR_DATA_FILE)?;

    let header = width as usize;
    let size = width as usize * height as usize;
    let length_u16 = raw.len() / 2;
    if header + size != length_u16 {
        warn!(
            "IR data of wrong size! Expected {:?} = {:?} + {:?} but got {:?}",
            header + size,
            header,
            size,
            length_u16
        );
        warn!("Width: {:?};  height: {:?}:", width, height);
        return None;
    };

    let ir_data = IrData { raw, width, height };
    Some(ir_data)
}
