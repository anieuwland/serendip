use std::collections::HashMap;

use serde::Deserialize;

const IMAGE_PROPERTIES_FILE: &'static str = "ImageProperties.json";

/// Decodes a file's bytes to text: `ImageProperties.json` is typically UTF-16LE with a BOM.
fn decode_text(bytes: &[u8]) -> Option<String> {
    let utf16 = |chunks: &mut dyn Iterator<Item = &[u8]>, be: bool| {
        let units: Vec<u16> = chunks
            .map(|c| {
                let pair = [c[0], c[1]];
                if be {
                    u16::from_be_bytes(pair)
                } else {
                    u16::from_le_bytes(pair)
                }
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

/// Reads the IR frame dimensions from `ImageProperties.json`, if present.
pub fn extract_dimensions_from_properties(files: &HashMap<String, Vec<u8>>) -> Option<(u16, u16)> {
    let bytes = files.get(IMAGE_PROPERTIES_FILE)?;
    let text = decode_text(bytes)?;
    let properties: ImageProperties = serde_json::from_str(&text).ok()?;
    Some((properties.ir_sensor_width, properties.ir_sensor_height))
}
