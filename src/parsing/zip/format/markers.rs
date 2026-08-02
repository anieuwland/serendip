use std::collections::HashMap;

use log::warn;
use prost::Message;

use crate::markers::{self, Marker};

const CENTERBOX: &'static str = "Markers/Standard/Centerbox.gpbenc";
const CENTERPOINT: &'static str = "Markers/Standard/Centerpoint.gpbenc";
const HOTPOINT: &'static str = "Markers/Standard/Hotpoint.gpbenc";
const COLDPOINT: &'static str = "Markers/Standard/Coldpoint.gpbenc";

/// Extract measurement markers embedded in the Fluke thermogram.
///
/// Currently only the standard markers are supported, e.g. centerbox,
/// centerpoint, hotpoint and coldpoint. To support custom measurements
/// more sample data is needed.
pub fn extract_markers(files: &mut HashMap<String, Vec<u8>>) -> Vec<Marker> {
    let mut markers = Vec::with_capacity(4);
    markers.extend(extract_marker::<ZipBoxMarker, _>(files, CENTERBOX));
    markers.extend(extract_marker::<ZipPointMarker, _>(files, CENTERPOINT));
    markers.extend(extract_marker::<ZipPointMarker, _>(files, HOTPOINT));
    markers.extend(extract_marker::<ZipPointMarker, _>(files, COLDPOINT));
    markers
}

/// Decode one marker file and convert it to its domain form, warning and
/// returning `None` if the file is absent, undecodable, or incomplete.
fn extract_marker<R, M>(files: &mut HashMap<String, Vec<u8>>, path: &str) -> Option<M>
where
    R: Message + Default,
    M: TryFrom<R, Error = &'static str>,
{
    let bytes = files.remove(path)?;
    let raw = R::decode(bytes.as_slice())
        .inspect_err(|e| warn!("Could not decode {path}: {e}"))
        .ok()?;
    M::try_from(raw)
        .inspect_err(|e| warn!("Ignoring incomplete {path}: {e}"))
        .ok()
}

/// A point marker
#[derive(Clone, PartialEq, Message)]
struct ZipPointMarker {
    #[prost(message, tag = "1")]
    pub coords: Option<ZipPoint>,
    #[prost(message, tag = "2")]
    pub metadata: Option<ZipMetadata>,
}

/// A box marker
#[derive(Clone, PartialEq, Message)]
struct ZipBoxMarker {
    #[prost(message, tag = "1")]
    pub start: Option<ZipPoint>,
    #[prost(message, tag = "2")]
    pub end: Option<ZipPoint>,
    #[prost(message, tag = "3")]
    pub metadata: Option<ZipMetadata>,
}

/// An (x, y) location in any marker
#[derive(Clone, Copy, PartialEq, Message)]
pub struct ZipPoint {
    #[prost(uint32, tag = "1")]
    pub x: u32,
    #[prost(uint32, tag = "2")]
    pub y: u32,
}

/// Marker metadata
#[derive(Clone, PartialEq, Message)]
pub struct ZipMetadata {
    #[prost(string, tag = "1")]
    pub label1: String,
    #[prost(string, tag = "2")]
    pub label2: String,
}

impl From<ZipPoint> for markers::Point {
    fn from(p: ZipPoint) -> Self {
        markers::Point { x: p.x, y: p.y }
    }
}

impl From<ZipMetadata> for markers::Metadata {
    fn from(m: ZipMetadata) -> Self {
        markers::Metadata {
            label1: m.label1,
            label2: m.label2,
        }
    }
}

impl TryFrom<ZipPointMarker> for Marker {
    type Error = &'static str;

    fn try_from(raw: ZipPointMarker) -> Result<Self, Self::Error> {
        Ok(Marker::Point {
            coords: raw.coords.ok_or("missing coords")?.into(),
            metadata: raw.metadata.ok_or("missing metadata")?.into(),
        })
    }
}

impl TryFrom<ZipBoxMarker> for Marker {
    type Error = &'static str;

    fn try_from(raw: ZipBoxMarker) -> Result<Self, Self::Error> {
        Ok(Marker::Box {
            start: raw.start.ok_or("missing start")?.into(),
            end: raw.end.ok_or("missing end")?.into(),
            metadata: raw.metadata.ok_or("missing metadata")?.into(),
        })
    }
}
