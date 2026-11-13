//! `math::rotate_scratch_about_y` (0x41FEF8): the scratch point's x and z (floats at
//! `FrameScratch.scratch_x` and `FrameScratch.scratch_z`) turned by an angle.
//!
//! Returns: the entry
//!
//! x' = x cos + z sin, z' = z cos - x sin, on the x87 as the original (`math::x87::
//! turn_in_place`). Six callers, `math::rotate_point_about_y` and its three siblings.

/// The scratch point (`g_scratchZ`, `X`, `Y`: 0x5C5F4C .. in the image until 2026-10-24):
/// written and read back by each of the four that turn a point (their other users,
/// 0x41FEA2 .. 0x41FEF0, are unreached), so now each one's local.
#[derive(Clone, Copy, Default)]
pub struct Scratch {
    pub z: f32,
    pub x: f32,
    pub y: f32,
}

/// `math::rotate_scratch_about_y` exactly as the original (the x87, at whatever precision it is
/// set to): what the Rust callers in modules not yet converted use, so that their own
/// comparisons stay exact. The entry left in edx.
pub fn rotate_scratch_about_y(
    tables: &crate::scene::engine_tables_store::EngineTables,
    point: &mut Scratch,
    angle: u32,
) -> u32 {
    let entry = angle.wrapping_add(0x8_0000) >> 20;
    x87_turn(tables, point, entry);
    entry
}

fn x87_turn(
    tables: &crate::scene::engine_tables_store::EngineTables,
    point: &mut Scratch,
    entry: u32,
) {
    use crate::scene::engine_tables_store::{cosine, sine};
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

/// The converted version (`rc2k_core::math::geometry`, plain floats): the one
/// its callers use. The entry left in edx.
pub fn rotate_scratch_about_y_plain(
    tables: &crate::scene::engine_tables_store::EngineTables,
    point: &mut Scratch,
    angle: u32,
) -> u32 {
    use rc2k_core::math::geometry;
    let entry = geometry::turn_entry(angle);
    {
        let (x, z) = geometry::turn_xz(
            point.x,
            point.z,
            crate::scene::engine_tables_store::sine(tables, entry as usize).get(),
            crate::scene::engine_tables_store::cosine(tables, entry as usize).get(),
        );
        point.x = x;
        point.z = z;

        entry
    }
}
