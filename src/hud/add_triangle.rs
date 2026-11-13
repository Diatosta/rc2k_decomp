//! A polygon of three vertices: `hud::add_polygon`.

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

pub fn add_triangle(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    pen.polygons.polygon_vertex_count.set(3u32);
    crate::hud::add_polygon::add_polygon(frame, hud, pen)
}
