//! The car shadow's and the skid marks' work (IDA 0x641AA4 .. 0x64BB18; phase 7, typed):
//! the shadow's points (records of 0x20 bytes), its farthest length and record, its sun
//! slant and hull count; the skid marks' wheel grip and lift, a wheel's place, the views,
//! the ring of links (8 bytes each), the work values of the point being laid, the next
//! point, the cars' bit and the ring of points (0xC bytes each). `#[repr(C)]` with the
//! image's layout, over the race block's bytes (`SHADOW_SKIDS`). Names are readings unless IDA's.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, Dword, Float};

/// One shadow point (0x20 bytes): the corner of the car's shadow, first as floats, then
/// as whole numbers (`race::car_shadow`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShadowPoint {
    /// +0x0: x.
    pub x: Dword,
    /// +0x4: y.
    pub y: Dword,
    /// +0x8: z.
    pub z: Dword,
    /// +0xC: the angle from the first point (`fpatan` of x and z), then whole numbers.
    pub angle: Dword,
    /// +0x10: the distance from the car.
    pub length: Dword,
    /// +0x14: 0 once the angles are laid (`race::car_shadow::shadow_angles`).
    pub zeroed: Dword,
    /// +0x18: not worked out (swapped with the farthest point's, sorted with the record).
    pub unknown_18: [Dword; 2],
}

const _: () = assert!(core::mem::size_of::<ShadowPoint>() == 0x20);

impl ShadowPoint {
    /// A record of zeros.
    pub const fn new() -> Self {
        Self {
            x: Dword::new(0),
            y: Dword::new(0),
            z: Dword::new(0),
            angle: Dword::new(0),
            length: Dword::new(0),
            zeroed: Dword::new(0),
            unknown_18: [Dword::new(0), Dword::new(0)],
        }
    }

    /// The record's eight dwords, in order (it is swapped and sorted whole).
    pub fn words(&self) -> [&Dword; 8] {
        [
            &self.x,
            &self.y,
            &self.z,
            &self.angle,
            &self.length,
            &self.zeroed,
            &self.unknown_18[0],
            &self.unknown_18[1],
        ]
    }
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ShadowSkids {
    /// +0x0 (0x641AA4): `g_shadowPoints`: the shadow's points, records of 0x20 bytes (x, y, z, angle, length, ...; 0x40 at most) and the hull's records after (`race::car_shadow`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_points: [ShadowPoint; 0x100],
    /// +0x2000 (0x643AA4): `flt_643AA4` .. `dword_643AB4`: not worked out.
    pub shadow_work: [Dword; 5],
    /// +0x2014 (0x643AB8): `flt_643AB8`: the length of the farthest point (`race::car_shadow::shadow_points`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub farthest_length: Float,
    /// +0x2018 (0x643ABC): `dword_643ABC`: the farthest point's number (`race::car_shadow`;
    /// the original kept its record's address). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub farthest: Dword,
    /// +0x201c (0x643AC0): `flt_643AC0`: the first point's angle. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub first_angle: Float,
    /// +0x2020 (0x643AC4): `flt_643AC4`: cos(turn) x cot(sun) (`race::car_shadow::sun_slant`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub slant_x: Float,
    /// +0x2024 (0x643AC8): `flt_643AC8`: sin(turn) x cot(sun). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub slant_z: Float,
    /// +0x2028 (0x643ACC): `g_shadowHullCount`: the shadow hull's points. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub hull_count: Dword,
    /// +0x202c (0x643AD0): `flt_643AD0`: the least grip that lays a skid mark (`race::skid_laying`).
    pub skid_grip: Float,
    /// +0x2030 (0x643AD4): `flt_643AD4`: the lift of a skid point (`race::skidmarks`).
    pub skid_lift: Float,
    /// +0x2034 (0x643AD8): `dword_643AD8` .. `dword_643AE0`: the wheel's place (`race::skid_laying`).
    pub skid_place: [Dword; 3],
    /// +0x2040 (0x643AE4): `dword_643AE4`: the views that show the skid marks, as bits (`race::skidmarks`).
    pub skid_views: Dword,
    /// +0x2044 (0x643AE8): `g_skidLinks`: 0x400 links of four words (before, ..., shown views; 8 bytes each; `race::skid_laying`, `race::skidmarks`, `race::stage_prefs`).
    pub skid_links: Bytes<0x2000>,
    /// +0x4044 (0x645AE8): `dword_645AE8` ..: the last point's turned work values (`race::skid_laying`).
    pub skid_turned: [Dword; 3],
    /// +0x4050 (0x645AF4): `dword_645AF4` ..: the last point's work values.
    pub skid_from: [Dword; 3],
    /// +0x405c (0x645B00): `dword_645B00`: the new point's angle.
    pub skid_angle: Dword,
    /// +0x4060 (0x645B04): `dword_645B04`: the last point's turned angle.
    pub skid_turned_angle: Dword,
    /// +0x4064 (0x645B08): `dword_645B08`: the edge offset's x.
    pub skid_offset_x: Dword,
    /// +0x4068 (0x645B0C): `dword_645B0C`: the edge offset's z.
    pub skid_offset_z: Dword,
    /// +0x406c (0x645B10): `dword_645B10`: the next point of the ring (`race::skid_laying`, `race::stage_prefs`).
    pub skid_next: Dword,
    /// +0x4070 (0x645B14): `dword_645B14`: the bit of the car being laid for.
    pub skid_cars: Dword,
    /// +0x4074 (0x645B18): `g_skidPoints`: the points, 0xC bytes each (0x400 are used; the room is 0x800).
    pub skid_points: Bytes<0x6000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
impl ShadowSkids {
    pub const fn new() -> Self {
        crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START))
    }
}

/// IDA 0x641AA4, the first byte of the piece.
pub const START: usize = 0x641AA4;
/// IDA 0x64BB18, the end of the piece.
pub const END: usize = 0x64BB18;

const _: () = assert!(END - START == core::mem::size_of::<ShadowSkids>());

/// Where IDA 0x641AA4 is now.
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&race().frame.shadow_skids))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(race().frame.shadow_skids.first(), START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(ShadowSkids, shadow_work), 0x643AA4 - START);
        assert_eq!(offset_of!(ShadowSkids, farthest_length), 0x643AB8 - START);
        assert_eq!(offset_of!(ShadowSkids, hull_count), 0x643ACC - START);
        assert_eq!(offset_of!(ShadowSkids, skid_grip), 0x643AD0 - START);
        assert_eq!(offset_of!(ShadowSkids, skid_links), 0x643AE8 - START);
        assert_eq!(offset_of!(ShadowSkids, skid_turned), 0x645AE8 - START);
        assert_eq!(offset_of!(ShadowSkids, skid_next), 0x645B10 - START);
        assert_eq!(offset_of!(ShadowSkids, skid_points), 0x645B18 - START);
    }
}
