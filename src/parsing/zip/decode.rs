use std::collections::HashMap;
use std::io::{Cursor, Read};
use serde::Deserialize;
use zip::ZipArchive;

pub struct SerendipZip  {
    ir_data: IrData
}

impl SerendipZip {
    pub fn kelvin(&self) {
        let header = usize::from(self.ir_data.width);
        let size = usize::from(self.ir_data.width * self.ir_data.height);
        // TODO Check length
        // if header_length + size > self.ir_data.ir_data.len() { return };
        let _ir_data_u16 = bytemuck::cast_slice::<u8, u16>(&self.ir_data.data[header..size]);
    }
}

pub fn decode_zip_format(bytes: &[u8]) -> Option<SerendipZip> {
    let mut files = unzip(bytes).ok()?;
    let (width, height) = extract_ir_dimensions(&files)?;
    println!("Dimensions: {width} x {height}");
    let _ir_data = extract_ir_data(&mut files, width, height);
    None
}

fn unzip(bytes: &[u8]) -> zip::result::ZipResult<HashMap<String, Vec<u8>>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))?;
    let mut files = HashMap::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        println!("{:?}", entry.name());
        if entry.is_file() {
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf)?;
            files.insert(entry.name().to_string(), buf);
        }
    }
    Ok(files)
}

pub struct IrData  {
    data: Vec<u8>,
    width: u16,
    height: u16,
}

const IR_DATA_FILE: &'static str = "Images/Main/IR.data";
const IMAGE_PROPERTIES_FILE: &'static str = "ImageProperties.json";

/// Decodes a file's bytes to text: `ImageProperties.json` is typically UTF-16LE with a BOM.
fn decode_text(bytes: &[u8]) -> Option<String> {
    let utf16 = |chunks: &mut dyn Iterator<Item = &[u8]>, be: bool| {
        let units: Vec<u16> = chunks
            .map(|c| {
                let pair = [c[0], c[1]];
                if be { u16::from_be_bytes(pair) } else { u16::from_le_bytes(pair) }
            })
            .collect();
        String::from_utf16(&units).ok()
    };

    match bytes {
        [0xFF, 0xFE, rest @ ..] => utf16(&mut rest.chunks_exact(2), false),
        [0xFE, 0xFF, rest @ ..] => utf16(&mut rest.chunks_exact(2), true),
        _ => String::from_utf8(bytes.to_vec()).ok(),
    }
}

/// The subset of `ImageProperties.json` that serendip reads. All values in
/// the file are JSON strings, including numeric ones (e.g. `"320"`), hence
/// the string-parsing deserializer on the numeric fields.
#[derive(Deserialize)]
struct ImageProperties {
    #[serde(rename = "IRPROP_IR_SENSOR_WIDTH", deserialize_with = "from_string")]
    ir_sensor_width: u16,
    #[serde(rename = "IRPROP_IR_SENSOR_HEIGHT", deserialize_with = "from_string")]
    ir_sensor_height: u16,
}

/// Deserializes a numeric value stored as a JSON string (`"320"` -> 320).
fn from_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

/// Extracts the IR frame dimensions, trying sources in decreasing order of
/// authority. Candidates are only accepted if they satisfy IR.data's length
/// equation (see `dimensions_fit_ir_data`).
///
/// 1. `ImageProperties.json` (`IRPROP_IR_SENSOR_WIDTH` / `..HEIGHT`).
///    Not always present.
/// 2. `IR.data`'s own header: u32 LE fields at byte offsets 182 (width - 1)
///    and 186 (height - 1). Verified on Ti400 samples only.
fn extract_ir_dimensions(files: &HashMap<String, Vec<u8>>) -> Option<(u16, u16)> {
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

/// Reads the IR frame dimensions from `ImageProperties.json`, if present.
fn extract_dimensions_from_properties(files: &HashMap<String, Vec<u8>>) -> Option<(u16, u16)> {
    let bytes = files.get(IMAGE_PROPERTIES_FILE)?;
    let text = decode_text(bytes)?;
    let properties: ImageProperties = serde_json::from_str(&text).ok()?;
    Some((properties.ir_sensor_width, properties.ir_sensor_height))
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
fn extract_ir_data(files: &mut HashMap<String, Vec<u8>>, width: u16, height: u16) -> Option<IrData> {
    let data = files.remove(IR_DATA_FILE)?;

    let header = width as usize;
    let size = width as usize * height as usize;
    let length_u16 = data.len() / 2;
    if header + size != length_u16 {
        println!("IR data of wrong size! Expected {:?} = {:?} + {:?} but got {:?}", header + size, header, size, length_u16);
        println!("Width: {:?};  height: {:?}:", width, height);
        return None;
    };

    let ir_data = IrData { data, width, height, };
    Some(ir_data)
}
