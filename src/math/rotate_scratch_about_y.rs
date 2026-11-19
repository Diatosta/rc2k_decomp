//! `math::rotate_scratch_about_y`: the scratch point's x and z (floats) turned by an angle.
//!
//! x' = x cos + z sin, z' = z cos - x sin, on the x87 (`math::x87::turn_in_place`). Used by
//! `math::rotate_point_about_y` and its three siblings.

/// The scratch point, written and read back by each of the four that turn a point, so each
/// one's local.
#[derive(Clone, Copy, Default)]
pub struct Scratch {
    pub z: f32,
    pub x: f32,
    pub y: f32,
}

/// `math::rotate_scratch_about_y` on the x87 at whatever precision it is set to: what the
/// callers in modules not yet converted use, so that their own comparisons stay exact.
/// Also returns the angle table's entry.
pub fn rotate_scratch_about_y(
    tables: &crate::math::engine_tables::EngineTables,
    point: &mut Scratch,
    angle: u32,
) -> u32 {
    let entry = angle.wrapping_add(0x8_0000) >> 20;
    x87_turn(tables, point, entry);
    entry
}

fn x87_turn(tables: &crate::math::engine_tables::EngineTables, point: &mut Scratch, entry: u32) {
    use crate::math::engine_tables::{cosine, sine};
    let (mut x, mut z) = (point.x.to_bits(), point.z.to_bits());
    crate::math::x87::turn_in_place(
        sine(tables, entry as usize).get(),
        cosine(tables, entry as usize).get(),
        &mut x,
        &mut z,
    );
    point.x = f32::from_bits(x);
    point.z = f32::from_bits(z);
}

/// The converted version (`rc2k_core::math::geometry`, plain floats): the one its callers
/// use. Also returns the angle table's entry.
pub fn rotate_scratch_about_y_plain(
    tables: &crate::math::engine_tables::EngineTables,
    point: &mut Scratch,
    angle: u32,
) -> u32 {
    use rc2k_core::math::geometry;
    let entry = geometry::turn_entry(angle);
    {
        let (x, z) = geometry::turn_xz(
            point.x,
            point.z,
            crate::math::engine_tables::sine(tables, entry as usize).get(),
            crate::math::engine_tables::cosine(tables, entry as usize).get(),
        );
        point.x = x;
        point.z = z;

        entry
    }
}
