//! `math::rotate_point_about_y_ecx`: `math::rotate_point_about_y` returning only x', z' and
//! the angle table's entry (whole numbers, through the scratch point).

use crate::math::rotate_point_about_y::rotate_point_about_y;

/// x', z' and the angle table's entry.
pub fn rotate_point_about_y_ecx(
    tables: crate::math::engine_tables::EngineTablesRef,
    x: i32,
    y: i32,
    z: i32,
    angle: u32,
) -> [u32; 3] {
    let [x, _, z, entry] = rotate_point_about_y(tables, x, y, z, angle);
    [x, z, entry]
}
