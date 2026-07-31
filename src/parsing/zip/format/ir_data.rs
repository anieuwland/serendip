use std::collections::HashMap;

use crate::parsing::zip::format::image_properties::extract_dimensions_from_properties;

pub struct IrData {
    pub data: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

const IR_DATA_FILE: &'static str = "Images/Main/IR.data";

/// Extracts the IR frame dimensions, trying sources in decreasing order of
/// authority. Candidates are only accepted if they satisfy IR.data's length
/// equation (see `dimensions_fit_ir_data`).
///
/// 1. `ImageProperties.json` (`IRPROP_IR_SENSOR_WIDTH` / `..HEIGHT`).
///    Not always present.
/// 2. `IR.data`'s own header: u32 LE fields at byte offsets 182 (width - 1)
///    and 186 (height - 1). Verified on Ti400 samples only.
pub fn extract_ir_dimensions(files: &HashMap<String, Vec<u8>>) -> Option<(u16, u16)> {
    let ir_data = files.get(IR_DATA_FILE)?;

    let candidates = [
        extract_dimensions_from_properties(files),
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
/// The frame dimensions come from `ImageProperties.json` (see
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
    let data = files.remove(IR_DATA_FILE)?;

    let header = width as usize;
    let size = width as usize * height as usize;
    let length_u16 = data.len() / 2;
    if header + size != length_u16 {
        println!(
            "IR data of wrong size! Expected {:?} = {:?} + {:?} but got {:?}",
            header + size,
            header,
            size,
            length_u16
        );
        println!("Width: {:?};  height: {:?}:", width, height);
        return None;
    };

    let ir_data = IrData {
        data,
        width,
        height,
    };
    Some(ir_data)
}
