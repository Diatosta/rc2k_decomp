//! The skid marks' work: the least grip that lays a mark, the lift of a point, a wheel's place,
//! the ring of links (8 bytes each), the next point, the cars' bit and the ring of points (0xC
//! bytes each). A plain record held as a `session::DrawArc`: written on the game's thread
//! between frames (short borrows) and read by the drawing parts (the skid marks) as a shared
//! copy of the pointer, through their `scene::workers::DrawReads` (`ShadowSkidsRef`). The tables
//! are 40 KB: lending the pointer costs nothing, and a write copies them only if a drawing scope
//! still holds the old ones.

use crate::memory::session::{ArcRead, DrawArc};

/// The links' table: 0x400 links of 8 bytes each.
pub const LINKS_LEN: usize = 0x2000;
/// The points' table: 0xC bytes each (0x400 are used; the room is 0x800).
pub const POINTS_LEN: usize = 0x6000;

#[derive(Clone)]
pub struct ShadowSkids {
    /// The least grip that lays a skid mark (`race::skid_laying`).
    pub skid_grip: f32,
    /// The lift of a skid point (`race::skidmarks`).
    pub skid_lift: f32,
    /// The wheel's place, three dwords (`race::skid_laying`).
    pub skid_place: [u32; 3],
    /// 0x400 links of 8 bytes each (`race::skid_laying`, `race::skidmarks`,
    /// `race::stage_prefs`).
    pub skid_links: [u8; LINKS_LEN],
    /// The next point of the ring (`race::skid_laying`, `race::stage_prefs`).
    pub skid_next: u32,
    /// The bit of the car being laid for.
    pub skid_cars: u32,
    /// The points, 0xC bytes each.
    pub skid_points: [u8; POINTS_LEN],
}

impl ShadowSkids {
    pub const fn new() -> Self {
        Self {
            skid_grip: 8.0,
            skid_lift: 4096.0,
            skid_place: [0; 3],
            skid_links: [0; LINKS_LEN],
            skid_next: 0,
            skid_cars: 0,
            skid_points: [0; POINTS_LEN],
        }
    }

    /// The little-endian dword at byte `offset` of the points' table.
    pub fn point_word(&self, offset: usize) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| self.skid_points[offset + i]))
    }

    /// The dword `value` written at byte `offset` of the points' table.
    pub fn set_point_word(&mut self, offset: usize, value: u32) {
        self.skid_points[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// The dword `value` written at byte `offset` of the links' table.
    pub fn set_link_word(&mut self, offset: usize, value: u32) {
        self.skid_links[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

/// The record as one reader finds it: the game's thread's own (read through the store when
/// used), or a drawing part's share of it.
#[derive(Clone, Copy)]
pub enum ShadowSkidsRef<'a> {
    Store(&'a DrawArc<ShadowSkids>),
    Copy(&'a ShadowSkids),
}

impl ShadowSkidsRef<'_> {
    /// `f` on the record as it stands (a short read: nothing in `f` may write the store).
    pub fn read<R>(self, f: impl FnOnce(&ShadowSkids) -> R) -> R {
        match self {
            Self::Store(store) => f(&store.borrow()),
            Self::Copy(record) => f(record),
        }
    }
}

impl<'a> From<&'a ArcRead<ShadowSkids>> for ShadowSkidsRef<'a> {
    fn from(share: &'a ArcRead<ShadowSkids>) -> Self {
        Self::Copy(share)
    }
}
