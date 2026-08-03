#[derive(Clone, Debug, PartialEq)]
pub enum Marker {
    Point {
        coords: Point,
        metadata: Metadata,
    },
    Box {
        start: Point,
        end: Point,
        metadata: Metadata,
    },
}

impl Marker {
    /// This marker with its coordinates converted to index the thermal
    /// data directly.
    ///
    /// As stored in the file, marker coordinates are *not* thermal data
    /// positions: they are in `IRImageInfo`'s width/height space, which on
    /// all samples seen is exactly twice the thermal data size (see
    /// `IrImageInfo::stored_thermal_dimensions`). Halved coordinates are
    /// clamped to the thermal dimensions so no out-of-bounds positions are
    /// produced.
    pub fn to_thermal_space(&self, width: u16, height: u16) -> Marker {
        match self {
            Marker::Point { coords, metadata } => Marker::Point {
                coords: coords.to_thermal_space(width, height),
                metadata: metadata.clone(),
            },
            Marker::Box {
                start,
                end,
                metadata,
            } => Marker::Box {
                start: start.to_thermal_space(width, height),
                end: end.to_thermal_space(width, height),
                metadata: metadata.clone(),
            },
        }
    }
}

/// An (x, y) position in any marker
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    /// Halves the coordinates and clamps them to the given dimensions.
    /// See `Marker::to_thermal_space`.
    fn to_thermal_space(self, width: u16, height: u16) -> Point {
        Point {
            x: (self.x / 2).min(u32::from(width).saturating_sub(1)),
            y: (self.y / 2).min(u32::from(height).saturating_sub(1)),
        }
    }
}

/// Marker metadata
///
/// So far it is unclear what which label exactly means.
#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub label1: String,
    pub label2: String,
}
