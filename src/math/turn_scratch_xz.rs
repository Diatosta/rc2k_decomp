//! `math::turn_scratch_xz` (0x41FEF8): the scratch point's x and z (floats at
//! `dword_5C5F50` and `dword_5C5F4C`) turned by an angle.
//!
//! In:  edx = the angle (a dword's turn is 2^32: `g_sines`/`g_cosines` entry (edx +
//!      0x80000) >> 20)
//! Out: edx = the entry; the rest preserved
//!
//! x' = x cos + z sin, z' = z cos - x sin, on the x87 as the original (`math::x87::
//! turn_in_place`). Six callers, `math::turn_point_xz` and its three siblings.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

/// The scratch point (`g_scratchZ`, `X`, `Y`: 0x5C5F4C .. in the image until 2026-10-24):
/// written and read back by each of the four that turn a point (their other users,
/// 0x41FEA2 .. 0x41FEF0, are unreached), so now each one's local.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Scratch {
    pub z: u32,
    pub x: u32,
    pub y: u32,
}

/// `math::turn_scratch_xz` exactly as the original (the x87, at whatever precision it is
/// set to): what the Rust callers in modules not yet converted use, so that their own
/// comparisons stay exact. The entry left in edx.
pub fn turn_scratch_xz(point: &mut Scratch, angle: u32) -> u32 {
    let entry = angle.wrapping_add(0x8_0000) >> 20;
    x87_turn(point, entry);
    entry
}

fn x87_turn(point: &mut Scratch, entry: u32) {
    let a = |table: &crate::session::Float| table.addr() + 4 * entry as usize;
    crate::math::x87::turn_in_place(
        crate::pointers::host_at(a(crate::scene::engine_tables_store::sine(0))) as *const u32,
        crate::pointers::host_at(a(crate::scene::engine_tables_store::cosine(0))) as *const u32,
        &raw mut point.x,
        &raw mut point.z,
    );
}

/// The converted version (`rc2k_core::math::geometry`, plain floats): the hook's, and what
/// converted callers use. The entry left in edx.
pub fn turn_scratch_xz_plain(point: &mut Scratch, angle: u32) -> u32 {
    use rc2k_core::math::geometry;
    let entry = geometry::turn_entry(angle);
    {
        let (x, z) = geometry::turn_xz(
            f32::from_bits(point.x),
            f32::from_bits(point.z),
            crate::scene::engine_tables_store::sine(entry as usize).get(),
            crate::scene::engine_tables_store::cosine(entry as usize).get(),
        );
        point.x = x.to_bits();
        point.z = z.to_bits();

        entry
    }
}
