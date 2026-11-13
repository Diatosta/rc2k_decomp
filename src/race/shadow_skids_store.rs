//! The car shadow's and the skid marks' work (IDA 0x641AA4 .. 0x64BB18; phase 7, typed):
//! the shadow's points (records of 0x20 bytes), its farthest length and record, its sun
//! slant and hull count; the skid marks' wheel grip and lift, a wheel's place, the views,
//! the ring of links (8 bytes each), the work values of the point being laid, the next
//! point, the cars' bit and the ring of points (0xC bytes each). `#[repr(C)]` with the
//! image's layout, over the race block's bytes (`SHADOW_SKIDS`). Plain cells: a drawing store
//! (`session::DrawShared`), written on the game's thread between frames and read by the
//! drawing parts (the skid marks) through their `workers::DrawReads`; the shadow's points
//! (`ShadowPoint`) are the shadow's own work's (`race::car_shadow::ShadowWork`). Names are
//! readings unless IDA's.

use crate::cells::ZeroCells;

use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// One shadow point (0x20 bytes): the corner of the car's shadow, first as floats, then
/// as whole numbers (`race::car_shadow`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub struct ShadowPoint {
    /// +0x0: x.
    pub x: PlainDword,
    /// +0x4: y.
    pub y: PlainDword,
    /// +0x8: z.
    pub z: PlainDword,
    /// +0xC: the angle from the first point (`fpatan` of x and z), then whole numbers.
    pub angle: PlainDword,
    /// +0x10: the distance from the car.
    pub length: PlainDword,
    /// +0x14: 0 once the angles are laid (`race::car_shadow::shadow_angles`).
    pub zeroed: PlainDword,
    /// +0x18: the record's last two dwords; only moved with the record when points are swapped or sorted, never read for a value.
    pub carried_words: [PlainDword; 2],
}

const _: () = assert!(core::mem::size_of::<ShadowPoint>() == 0x20);

impl ShadowPoint {
    /// A record of zeros.
    pub const fn new() -> Self {
        Self {
            x: PlainDword::new(0),
            y: PlainDword::new(0),
            z: PlainDword::new(0),
            angle: PlainDword::new(0),
            length: PlainDword::new(0),
            zeroed: PlainDword::new(0),
            carried_words: [PlainDword::new(0), PlainDword::new(0)],
        }
    }

    /// The record's eight dwords, in order (it is swapped and sorted whole).
    pub fn words(&self) -> [&PlainDword; 8] {
        [
            &self.x,
            &self.y,
            &self.z,
            &self.angle,
            &self.length,
            &self.zeroed,
            &self.carried_words[0],
            &self.carried_words[1],
        ]
    }
}

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ShadowSkids {
    /// +0x0 (0x641AA4): `g_shadowPoints`: the shadow's points, 0x100 records of 0x20 bytes (`ShadowPoint`: x, y, z, angle, length, ...; 0x40 at most) and the hull's records after (`race::car_shadow`), held as bytes. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub shadow_points: PlainBytes<0x2000>,
    /// +0x2000 (0x643AA4): `flt_643AA4` .. `dword_643AB4`: unused since the shadow keeps its own work; kept for the layout.
    pub shadow_work: [PlainDword; 5],
    /// +0x2014 (0x643AB8): `flt_643AB8`: the length of the farthest point (`race::car_shadow::shadow_points`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub farthest_length: PlainFloat,
    /// +0x2018 (0x643ABC): `dword_643ABC`: the farthest point's number (`race::car_shadow`;
    /// the original kept its record's address). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub farthest: PlainDword,
    /// +0x201c (0x643AC0): `flt_643AC0`: the first point's angle. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub first_angle: PlainFloat,
    /// +0x2020 (0x643AC4): `flt_643AC4`: cos(turn) x cot(sun) (`race::car_shadow::sun_slant`). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub slant_x: PlainFloat,
    /// +0x2024 (0x643AC8): `flt_643AC8`: sin(turn) x cot(sun). Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub slant_z: PlainFloat,
    /// +0x2028 (0x643ACC): `g_shadowHullCount`: the shadow hull's points. Unused since the shadow keeps its own work (`race::car_shadow::ShadowWork`); kept for the layout.
    pub hull_count: PlainDword,
    /// +0x202c (0x643AD0): `flt_643AD0`: the least grip that lays a skid mark (`race::skid_laying`).
    pub skid_grip: PlainFloat,
    /// +0x2030 (0x643AD4): `flt_643AD4`: the lift of a skid point (`race::skidmarks`).
    pub skid_lift: PlainFloat,
    /// +0x2034 (0x643AD8): `dword_643AD8` .. `dword_643AE0`: the wheel's place (`race::skid_laying`).
    pub skid_place: [PlainDword; 3],
    /// +0x2040 (0x643AE4): the original kept here the views that show the skid marks, as bits (`race::skidmarks`); no code reads or writes it (the field's name nowhere in Rust), it only keeps the image's layout.
    pub unused_2040: PlainDword,
    /// +0x2044 (0x643AE8): `g_skidLinks`: 0x400 links of four words (before, ..., shown views; 8 bytes each; `race::skid_laying`, `race::skidmarks`, `race::stage_prefs`).
    pub skid_links: PlainBytes<0x2000>,
    /// +0x4044 (0x645AE8): `dword_645AE8` ..: the last point's turned work values (`race::skid_laying`).
    pub skid_turned: [PlainDword; 3],
    /// +0x4050 (0x645AF4): `dword_645AF4` ..: the last point's work values.
    pub skid_from: [PlainDword; 3],
    /// +0x405c (0x645B00): `dword_645B00`: the new point's angle.
    pub skid_angle: PlainDword,
    /// +0x4060 (0x645B04): `dword_645B04`: the last point's turned angle.
    pub skid_turned_angle: PlainDword,
    /// +0x4064 (0x645B08): `dword_645B08`: the edge offset's x.
    pub skid_offset_x: PlainDword,
    /// +0x4068 (0x645B0C): `dword_645B0C`: the edge offset's z.
    pub skid_offset_z: PlainDword,
    /// +0x406c (0x645B10): `dword_645B10`: the next point of the ring (`race::skid_laying`, `race::stage_prefs`).
    pub skid_next: PlainDword,
    /// +0x4070 (0x645B14): `dword_645B14`: the bit of the car being laid for.
    pub skid_cars: PlainDword,
    /// +0x4074 (0x645B18): `g_skidPoints`: the points, 0xC bytes each (0x400 are used; the room is 0x800).
    pub skid_points: PlainBytes<0x6000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl ShadowSkids {
    pub const fn new() -> Self {
        Self {
            skid_grip: PlainFloat::new(8.0),
            skid_lift: PlainFloat::new(4096.0),
            ..ZeroCells::ZERO
        }
    }
}
