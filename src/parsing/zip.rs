use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::SerendipThermogram;

pub fn decode_zip_format(bytes: &[u8]) -> Option<SerendipThermogram> {
    let files = unzip(bytes).ok()?;
    let _ir_data = extract_ir_data(&files);
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

const IR_DATA_FILE: &'static str = "Images/Main/IR.data";

fn extract_ir_data(files: &HashMap<String, Vec<u8>>) -> Option<&Vec<u8>> {
    let ir_data_binary = files.get(IR_DATA_FILE);
    None
}
