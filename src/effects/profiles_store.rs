//! The profiles' rings and tables.

use crate::memory::session::PlainDword;

/// The profiles' rings of indexes.
#[derive(rc2k_derive::Cells)]
pub struct ProfileRings {
    /// Each of the 0x100 indexes' one before it (`effects::make_profiles`).
    pub before: [PlainDword; 256],
    /// Each index's one after it.
    pub after: [PlainDword; 256],
    /// 0 (`effects::make_profiles` writes it).
    pub zero: PlainDword,
}

impl ProfileRings {
    /// The values the game starts with.
    pub const fn new() -> Self {
        ProfileRings {
            before: [const { PlainDword::new(0x0) }; 256],
            after: [const { PlainDword::new(0x0) }; 256],
            zero: PlainDword::new(0x0),
        }
    }
}

/// One profile table: 0x100 whole numbers and then their 0x100 differences (the first
/// dwords hold the table's inputs until `effects::make_profiles` makes it).
pub type ProfileTable = [PlainDword; 0x200];

/// A profile table with the given first values (index, value).
const fn table(first: &[(usize, u32)]) -> ProfileTable {
    let mut t = [const { PlainDword::new(0) }; 0x200];
    let mut k = 0;
    while k < first.len() {
        t[first[k].0] = PlainDword::new(first[k].1);
        k += 1;
    }
    t
}

/// The profile tables (13 of 0x800 bytes, in this order of memory: ramp 0, curves 0 and 1, ramps 1
/// and 2, sines 0 .. 7).
#[derive(rc2k_derive::Cells)]
pub struct ProfileTables {
    /// Ramp 0, curve 0, curve 1, ramp 1, ramp 2, then sines 0 .. 7 (`RAMPS`, `CURVES`, `SINES`).
    pub tables: [ProfileTable; 13],
}

impl ProfileTables {
    /// The values the game starts with.
    pub const fn new() -> Self {
        ProfileTables {
            tables: [
                table(&[]),
                table(&[(0, 0xC0000000), (2, 0x469C4000), (3, 0x48435000)]),
                table(&[(1, 0x3F000000), (2, 0xC87DE800)]),
                table(&[(0, 0xFF)]),
                table(&[(0, 0x7F)]),
                table(&[(2, 0x466A6000), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48212200), (3, 0x3BC90FDB)]),
                table(&[(2, 0x4851F600), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48816500), (3, 0x3BC90FDB)]),
                table(&[(2, 0x4899CF00), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48B23900), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48CAA300), (3, 0x3BC90FDB)]),
                table(&[(2, 0x48E30D00), (3, 0x3BC90FDB)]),
            ],
        }
    }
}
