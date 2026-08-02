use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::parsing::zip::format::{IrData, extract_ir_data, extract_ir_dimensions};

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
