//! The profile tables' first values.

/// One profile table: 0x100 whole numbers or floats, then their 0x100 differences.
pub type ProfileTable = [u32; 0x200];

/// A profile table with the given first values (index, value).
const fn table(first: &[(usize, u32)]) -> ProfileTable {
    let mut t = [const { 0 }; 0x200];
    let mut k = 0;
    while k < first.len() {
        t[first[k].0] = first[k].1;
        k += 1;
    }
    t
}

/// The 13 tables' first values, in this order: ramp 0, curves 0 and 1, ramps 1 and 2, sines 0 .. 7
/// (`RAMPS`, `CURVES`, `SINES`). `effects::make_profiles` works them out once.
pub const fn first_tables() -> [ProfileTable; 13] {
    [
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
    ]
}
