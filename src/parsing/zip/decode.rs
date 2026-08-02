use std::collections::HashMap;
use std::io::{Cursor, Read};

use log::debug;
use zip::ZipArchive;

use crate::parsing::zip::format::{
    CameraInfo, IrData, IrImageInfo, extract_camera_info, extract_ir_data, extract_ir_dimensions,
    extract_ir_image_info,
};

pub struct SerendipZip  {
    ir_data: IrData,
    ir_image_info: IrImageInfo,
    camera_info: CameraInfo,
}

impl SerendipZip {
    // pub fn kelvin(&self) -> Vec<f32> {
    //     let header = usize::from(self.ir_data.width);
    //     let size = usize::from(self.ir_data.width * self.ir_data.height);
    //     let _ir_data_u16 = bytemuck::cast_slice::<u8, u16>(&self.ir_data.data[header..size]);
    // }
}

pub fn decode_zip_format(bytes: &[u8]) -> Option<SerendipZip> {
    let mut files = unzip(bytes).ok()?;
    let (width, height) = extract_ir_dimensions(&files)?;
    debug!("Dimensions: {width} x {height}");
    let ir_image_info = extract_ir_image_info(&files)?;
    debug!("IRImageInfo: {ir_image_info:?}");
    let camera_info = extract_camera_info(&files)?;
    debug!("CameraInfo: {camera_info:?}");
    let ir_data = extract_ir_data(&mut files, width, height)?;
    Some(SerendipZip { ir_data, ir_image_info, camera_info })
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
