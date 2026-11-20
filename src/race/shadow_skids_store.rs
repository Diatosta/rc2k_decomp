//! The car shadow's and the skid marks' work: the shadow's points (records of 0x20 bytes), its
//! farthest length and record, its sun slant and hull count; the skid marks' wheel grip and
//! lift, a wheel's place, the ring of links (8 bytes each), the work values of the point being
//! laid, the next point, the cars' bit and the ring of points (0xC bytes each). `#[repr(C)]`,
//! laid over the race block's bytes. Plain cells: a drawing store (`session::DrawShared`),
//! written on the game's thread between frames and read by the drawing parts (the skid marks)
//! through their `scene::workers::DrawReads`. The shadow now keeps its own work
//! (`race::car_shadow::ShadowWork`), so the shadow fields here are unused and only keep the layout.

use crate::memory::cells::ZeroCells;

use crate::memory::session::{PlainBytes, PlainDword, PlainFloat};

/// One shadow point (0x20 bytes): the corner of the car's shadow, first as floats, then
/// as whole numbers (`race::car_shadow`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells, rc2k_derive::ByteView)]
pub(super) struct ShadowPoint {
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
    /// +0x18: the record's last two dwords; only moved with the record when points are swapped or
    /// sorted, never read for a value.
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
    /// +0x0: the shadow's points, 0x100 records of 0x20 bytes (`ShadowPoint`; 0x40 at most) and
    /// the hull's records after, held as bytes. Unused.
    pub shadow_points: PlainBytes<0x2000>,
    /// +0x2000: five dwords of the shadow's work. Unused.
    pub shadow_work: [PlainDword; 5],
    /// +0x2014: the length of the farthest point. Unused.
    pub farthest_length: PlainFloat,
    /// +0x2018: the farthest point's number. Unused.
    pub farthest: PlainDword,
    /// +0x201c: the first point's angle. Unused.
    pub first_angle: PlainFloat,
    /// +0x2020: cos(turn) x cot(sun). Unused.
    pub slant_x: PlainFloat,
    /// +0x2024: sin(turn) x cot(sun). Unused.
    pub slant_z: PlainFloat,
    /// +0x2028: the number of the shadow hull's points. Unused.
    pub hull_count: PlainDword,
    /// +0x202c: the least grip that lays a skid mark (`race::skid_laying`).
    pub skid_grip: PlainFloat,
    /// +0x2030: the lift of a skid point (`race::skidmarks`).
    pub skid_lift: PlainFloat,
    /// +0x2034: the wheel's place, three dwords (`race::skid_laying`).
    pub skid_place: [PlainDword; 3],
    /// +0x2040: the views that show the skid marks, as bits. Nothing reads or writes it; it only
    /// keeps the layout.
    pub unused_2040: PlainDword,
    /// +0x2044: 0x400 links of 8 bytes each (`race::skid_laying`, `race::skidmarks`,
    /// `race::stage_prefs`).
    pub skid_links: PlainBytes<0x2000>,
    /// +0x4044: the last point's turned work values (`race::skid_laying`).
    pub skid_turned: [PlainDword; 3],
    /// +0x4050: the last point's work values.
    pub skid_from: [PlainDword; 3],
    /// +0x405c: the new point's angle.
    pub skid_angle: PlainDword,
    /// +0x4060: the last point's turned angle.
    pub skid_turned_angle: PlainDword,
    /// +0x4064: the edge offset's x.
    pub skid_offset_x: PlainDword,
    /// +0x4068: the edge offset's z.
    pub skid_offset_z: PlainDword,
    /// +0x406c: the next point of the ring (`race::skid_laying`, `race::stage_prefs`).
    pub skid_next: PlainDword,
    /// +0x4070: the bit of the car being laid for.
    pub skid_cars: PlainDword,
    /// +0x4074: the points, 0xC bytes each (0x400 are used; the room is 0x800).
    pub skid_points: PlainBytes<0x6000>,
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ShadowSkids {
    pub const fn new() -> Self {
        Self {
            skid_grip: PlainFloat::new(8.0),
            skid_lift: PlainFloat::new(4096.0),
            ..ZeroCells::ZERO
        }
    }
}
