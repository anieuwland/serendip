

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

/// An (x, y) position in any marker
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

/// Marker metadata
#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    pub label1: String,
    pub label2: String,
}
