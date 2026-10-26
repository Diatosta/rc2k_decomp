//! The damage model's figures, out of RAL.EXE's image (IDA 0x75B340 .. 0x75B940; phase 7,
//! typed): shares, thresholds, the work floats, the tables of four (x0, y0, x1, y1)
//! `physics::flow::table_lerp` reads, and the rates. `#[repr(C)]` struct of atomic cells
//! with the image's layout, one static; `moved::MOVED` points the range at it. What each
//! figure is, is a reading at best (`physics/damage_model.rs`); the fields still named by
//! their IDA names (`flt_75b3a0`) are not worked out.

use crate::session::{Bytes, Dword, Float};

/// A gap's first bytes: `runs` are (offset, bytes) of the non-zero ones.
const fn sparse<const N: usize>(runs: &[(usize, &[u8])]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (at, bytes) = runs[r];
        let mut i = 0;
        while i < bytes.len() {
            out[at + i] = bytes[i];
            i += 1;
        }
        r += 1;
    }
    out
}

/// The damage model's figures.
#[repr(C)]
pub struct DamageModel {
    /// `flt_75B340`: a figure times it gives the failure chance (`physics::rate_damage`).
    pub chance_scale: Float,
    /// `flt_75B344`: not worked out.
    pub flt_75b344: Float,
    /// `flt_75B348`: the working grades are held up to it.
    pub grade_high: Float,
    /// `flt_75B34C`: the gauges and gears are held 0 ..= it; a gear equal to it is worn out.
    pub gauge_top: Float,
    /// `flt_75B350`: the gauges and grades are held down to it.
    pub gauge_low: Float,
    /// `flt_75B354`: not worked out.
    pub flt_75b354: Float,
    /// Unreferenced bytes (IDA 0X75B358 .. 0X75B35C).
    pub _gap_75b358: Bytes<4>,
    /// `flt_75B35C`: not worked out.
    pub flt_75b35c: Float,
    /// `flt_75B360`: not worked out.
    pub flt_75b360: Float,
    /// `flt_75B364`: not worked out.
    pub flt_75b364: Float,
    /// `flt_75B368`: not worked out.
    pub flt_75b368: Float,
    /// `flt_75B36C`: the work cell a `fistp` rounds into.
    pub scratch: Float,
    /// `flt_75B370`: not worked out.
    pub flt_75b370: Float,
    /// Unreferenced bytes (IDA 0X75B374 .. 0X75B378).
    pub _gap_75b374: Bytes<4>,
    /// `flt_75B378`: not worked out.
    pub flt_75b378: Float,
    /// `flt_75B37C`: not worked out.
    pub flt_75b37c: Float,
    /// `flt_75B380`: not worked out.
    pub flt_75b380: Float,
    /// Unreferenced bytes (IDA 0X75B384 .. 0X75B388).
    pub _gap_75b384: Bytes<4>,
    /// `flt_75B388`: the factor `physics::rate_damage` multiplies by (80 bits).
    pub grade_k: Float,
    /// `flt_75B38C`: not worked out.
    pub flt_75b38c: Float,
    /// `flt_75B390`: not worked out.
    pub flt_75b390: Float,
    /// `flt_75B394`: not worked out.
    pub flt_75b394: Float,
    /// `flt_75B398`: not worked out.
    pub flt_75b398: Float,
    /// `flt_75B39C`: not worked out.
    pub flt_75b39c: Float,
    /// `flt_75B3A0`: not worked out.
    pub flt_75b3a0: Float,
    /// `flt_75B3A4`: not worked out.
    pub flt_75b3a4: Float,
    /// `flt_75B3A8`: not worked out.
    pub flt_75b3a8: Float,
    /// `flt_75B3AC`: not worked out.
    pub flt_75b3ac: Float,
    /// `flt_75B3B0`: not worked out.
    pub flt_75b3b0: Float,
    /// `flt_75B3B4`: not worked out.
    pub flt_75b3b4: Float,
    /// `dword_75B3B8`: not worked out.
    pub dword_75b3b8: Dword,
    /// `dword_75B3BC`: not worked out.
    pub dword_75b3bc: Dword,
    /// `flt_75B3C0`: not worked out.
    pub flt_75b3c0: Float,
    /// `dword_75B3C4`: not worked out.
    pub dword_75b3c4: Dword,
    /// `flt_75B3C8`: not worked out.
    pub flt_75b3c8: Float,
    /// `dword_75B3CC` ..: the four wheels' loads, blended by `flt_75B360` (`physics::damage_factors`).
    pub wheel_loads: [Float; 4],
    /// `flt_75B3DC`: not worked out.
    pub flt_75b3dc: Float,
    /// `flt_75B3E0`: not worked out.
    pub flt_75b3e0: Float,
    /// `flt_75B3E4`: not worked out.
    pub flt_75b3e4: Float,
    /// `dword_75B3E8`: not worked out.
    pub dword_75b3e8: Dword,
    /// `dword_75B3EC`: not worked out.
    pub dword_75b3ec: Dword,
    /// `dword_75B3F0`: not worked out.
    pub dword_75b3f0: Dword,
    /// `flt_75B3F4`: not worked out.
    pub flt_75b3f4: Float,
    /// `flt_75B3F8`: not worked out.
    pub flt_75b3f8: Float,
    /// `flt_75B3FC`: not worked out.
    pub flt_75b3fc: Float,
    /// `flt_75B400`: not worked out.
    pub flt_75b400: Float,
    /// `flt_75B404`: not worked out.
    pub flt_75b404: Float,
    /// `flt_75B408`: not worked out.
    pub flt_75b408: Float,
    /// `flt_75B40C`: not worked out.
    pub flt_75b40c: Float,
    /// `flt_75B410`: not worked out.
    pub flt_75b410: Float,
    /// `flt_75B414`: not worked out.
    pub flt_75b414: Float,
    /// `flt_75B418`: not worked out.
    pub flt_75b418: Float,
    /// `flt_75B41C`: not worked out.
    pub flt_75b41c: Float,
    /// `flt_75B420`: not worked out.
    pub flt_75b420: Float,
    /// `flt_75B424`: not worked out.
    pub flt_75b424: Float,
    /// `flt_75B428`: not worked out.
    pub flt_75b428: Float,
    /// `flt_75B42C`: not worked out.
    pub flt_75b42c: Float,
    /// `flt_75B430`: not worked out.
    pub flt_75b430: Float,
    /// `flt_75B434`: not worked out.
    pub flt_75b434: Float,
    /// `flt_75B438`: not worked out.
    pub flt_75b438: Float,
    /// `flt_75B43C`: not worked out.
    pub flt_75b43c: Float,
    /// `flt_75B440`: not worked out.
    pub flt_75b440: Float,
    /// `flt_75B444`: not worked out.
    pub flt_75b444: Float,
    /// `flt_75B448`: not worked out.
    pub flt_75b448: Float,
    /// `flt_75B44C`: not worked out.
    pub flt_75b44c: Float,
    /// `flt_75B450`: not worked out.
    pub flt_75b450: Float,
    /// `flt_75B454`: not worked out.
    pub flt_75b454: Float,
    /// `flt_75B458`: not worked out.
    pub flt_75b458: Float,
    /// `flt_75B45C`: not worked out.
    pub flt_75b45c: Float,
    /// `flt_75B460`: not worked out.
    pub flt_75b460: Float,
    /// `flt_75B464`: not worked out.
    pub flt_75b464: Float,
    /// `flt_75B468`: not worked out.
    pub flt_75b468: Float,
    /// `flt_75B46C`: not worked out.
    pub flt_75b46c: Float,
    /// `flt_75B470`: not worked out.
    pub flt_75b470: Float,
    /// `flt_75B474`: not worked out.
    pub flt_75b474: Float,
    /// `flt_75B478`: not worked out.
    pub flt_75b478: Float,
    /// `flt_75B47C`: not worked out.
    pub flt_75b47c: Float,
    /// `flt_75B480`: not worked out.
    pub flt_75b480: Float,
    /// `flt_75B484`: not worked out.
    pub flt_75b484: Float,
    /// `flt_75B488`: not worked out.
    pub flt_75b488: Float,
    /// `flt_75B48C`: not worked out.
    pub flt_75b48c: Float,
    /// `flt_75B490`: not worked out.
    pub flt_75b490: Float,
    /// `flt_75B494`: not worked out.
    pub flt_75b494: Float,
    /// `flt_75B498`: not worked out.
    pub flt_75b498: Float,
    /// `flt_75B49C`: not worked out.
    pub flt_75b49c: Float,
    /// `dword_75B4A0`: the clock / random figure the failures by chance compare against.
    pub clock: Dword,
    /// Unreferenced bytes (IDA 0X75B4A4 .. 0X75B4A8).
    pub _gap_75b4a4: Bytes<4>,
    /// The tables of four (x0, y0, x1, y1): 0x75B4A8 .. 0x75B578 (`physics::damage_wear`,
    /// `rate_damage`, `damage_factors`).
    pub lerp_tables: [[Float; 4]; 13],
    /// `flt_75B578` ..: the wear rates (`flt_75B584` ..) `physics::damage_wear` raises and lowers the gauges by.
    pub wear_rates: [Float; 13],
    /// Unreferenced bytes (IDA 0X75B5AC .. 0X75B5B0).
    pub _gap_75b5ac: Bytes<4>,
    /// `flt_75B5B0`: not worked out.
    pub flt_75b5b0: Float,
    /// Unreferenced bytes (IDA 0X75B5B4 .. 0X75B5D0).
    pub _gap_75b5b4: Bytes<28>,
    /// `flt_75B5D0` ..: the gauges' rates (`physics::damage_gauges`), times the step's shares.
    pub gauge_rates: [Float; 24],
    /// `flt_75B630`: the first kind table (by `flt_75B3E4`'s bits).
    pub kind_table_a: [Float; 2],
    /// `flt_75B638`: the second kind table.
    pub kind_table_b: [Float; 2],
    /// Three more tables of four: 0x75B640, 0x75B650, 0x75B660.
    pub lerp_tables_b: [[Float; 4]; 3],
    /// `flt_75B670`: not worked out.
    pub flt_75b670: Float,
    /// Unreferenced bytes (IDA 0X75B674 .. 0X75B678).
    pub _gap_75b674: Bytes<4>,
    /// `flt_75B678` ..: more hit figures and thresholds (`physics::damage_gauges`, `rate_damage`).
    pub figures: [Float; 33],
    /// Unreferenced bytes (IDA 0X75B6FC .. 0X75B73C).
    pub _gap_75b6fc: Bytes<64>,
    /// `dword_75B73C`: the wear table (rows by `dword_75B3E8`, two columns, as the original
    /// indexes them), 0x180 bytes.
    pub wear_table: [Float; 96],
    /// `flt_75B8BC`: not worked out.
    pub flt_75b8bc: Float,
    /// `dword_75B8C0`: not worked out.
    pub dword_75b8c0: Dword,
    /// `dword_75B8C4`: not worked out.
    pub dword_75b8c4: Dword,
    /// Unreferenced bytes (IDA 0X75B8C8 .. 0X75B940).
    pub _gap_75b8c8: Bytes<120>,
}

/// First values the image's.
pub static DAMAGE: DamageModel = DamageModel {
    chance_scale: Float::from_bits(0x47800000),
    flt_75b344: Float::from_bits(0x40c00000),
    grade_high: Float::from_bits(0x3f800000),
    gauge_top: Float::from_bits(0x42c80000),
    gauge_low: Float::from_bits(0),
    flt_75b354: Float::from_bits(0x41000000),
    _gap_75b358: Bytes::new(sparse::<4>(&[(0x2, &[0x20, 0x41])])),
    flt_75b35c: Float::from_bits(0x43800000),
    flt_75b360: Float::from_bits(0x3f000000),
    flt_75b364: Float::from_bits(0x3e800000),
    flt_75b368: Float::from_bits(0x3dcccccd),
    scratch: Float::from_bits(0),
    flt_75b370: Float::from_bits(0x3c23d70a),
    _gap_75b374: Bytes::new(sparse::<4>(&[(0x2, &[0x7a, 0x44])])),
    flt_75b378: Float::from_bits(0x428c0000),
    flt_75b37c: Float::from_bits(0x3f000000),
    flt_75b380: Float::from_bits(0x44c92b10),
    _gap_75b384: Bytes::new(sparse::<4>(&[(0x0, &[0x6f, 0x12, 0x83, 0x3a])])),
    grade_k: Float::from_bits(0x3c23d70a),
    flt_75b38c: Float::from_bits(0x392f1cb9),
    flt_75b390: Float::from_bits(0x3c23d70a),
    flt_75b394: Float::from_bits(0x42340000),
    flt_75b398: Float::from_bits(0),
    flt_75b39c: Float::from_bits(0),
    flt_75b3a0: Float::from_bits(0),
    flt_75b3a4: Float::from_bits(0),
    flt_75b3a8: Float::from_bits(0),
    flt_75b3ac: Float::from_bits(0),
    flt_75b3b0: Float::from_bits(0),
    flt_75b3b4: Float::from_bits(0),
    dword_75b3b8: Dword::new(0),
    dword_75b3bc: Dword::new(0),
    flt_75b3c0: Float::from_bits(0),
    dword_75b3c4: Dword::new(0),
    flt_75b3c8: Float::from_bits(0),
    wheel_loads: [const { Float::from_bits(0) }; 4],
    flt_75b3dc: Float::from_bits(0),
    flt_75b3e0: Float::from_bits(0),
    flt_75b3e4: Float::from_bits(0),
    dword_75b3e8: Dword::new(0),
    dword_75b3ec: Dword::new(0),
    dword_75b3f0: Dword::new(0),
    flt_75b3f4: Float::from_bits(0),
    flt_75b3f8: Float::from_bits(0x42480000),
    flt_75b3fc: Float::from_bits(0x42480000),
    flt_75b400: Float::from_bits(0x42480000),
    flt_75b404: Float::from_bits(0),
    flt_75b408: Float::from_bits(0),
    flt_75b40c: Float::from_bits(0),
    flt_75b410: Float::from_bits(0),
    flt_75b414: Float::from_bits(0),
    flt_75b418: Float::from_bits(0),
    flt_75b41c: Float::from_bits(0),
    flt_75b420: Float::from_bits(0),
    flt_75b424: Float::from_bits(0),
    flt_75b428: Float::from_bits(0),
    flt_75b42c: Float::from_bits(0),
    flt_75b430: Float::from_bits(0),
    flt_75b434: Float::from_bits(0),
    flt_75b438: Float::from_bits(0),
    flt_75b43c: Float::from_bits(0),
    flt_75b440: Float::from_bits(0),
    flt_75b444: Float::from_bits(0),
    flt_75b448: Float::from_bits(0),
    flt_75b44c: Float::from_bits(0),
    flt_75b450: Float::from_bits(0),
    flt_75b454: Float::from_bits(0),
    flt_75b458: Float::from_bits(0),
    flt_75b45c: Float::from_bits(0),
    flt_75b460: Float::from_bits(0),
    flt_75b464: Float::from_bits(0),
    flt_75b468: Float::from_bits(0),
    flt_75b46c: Float::from_bits(0),
    flt_75b470: Float::from_bits(0),
    flt_75b474: Float::from_bits(0),
    flt_75b478: Float::from_bits(0),
    flt_75b47c: Float::from_bits(0),
    flt_75b480: Float::from_bits(0),
    flt_75b484: Float::from_bits(0),
    flt_75b488: Float::from_bits(0),
    flt_75b48c: Float::from_bits(0),
    flt_75b490: Float::from_bits(0),
    flt_75b494: Float::from_bits(0),
    flt_75b498: Float::from_bits(0),
    flt_75b49c: Float::from_bits(0),
    clock: Dword::new(0),
    _gap_75b4a4: Bytes::new(sparse::<4>(&[(0x0, &[0xcd, 0xcc, 0x4c, 0x3f])])),
    lerp_tables: [
        [
            Float::from_bits(0x42a00000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42c80000),
        ],
        [
            Float::from_bits(0x42480000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42480000),
        ],
        [
            Float::from_bits(0x42480000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42c80000),
        ],
        [
            Float::from_bits(0),
            Float::from_bits(0x3f800000),
            Float::from_bits(0x3e4ccccd),
            Float::from_bits(0x40400000),
        ],
        [
            Float::from_bits(0),
            Float::from_bits(0x42be0000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42c80000),
        ],
        [
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x42a00000),
        ],
        [
            Float::from_bits(0x42480000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3e99999a),
        ],
        [
            Float::from_bits(0x42340000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3c23d70a),
        ],
        [
            Float::from_bits(0x428c0000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3ba3d70a),
        ],
        [
            Float::from_bits(0x42480000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3b449ba6),
        ],
        [
            Float::from_bits(0x42aa0000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3ba3d70a),
        ],
        [
            Float::from_bits(0x420c0000),
            Float::from_bits(0),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3d4ccccd),
        ],
        [
            Float::from_bits(0x420c0000),
            Float::from_bits(0x3d4ccccd),
            Float::from_bits(0x42c80000),
            Float::from_bits(0),
        ],
    ],
    wear_rates: [
        Float::from_bits(0x3e4ccccd),
        Float::from_bits(0x3f800000),
        Float::from_bits(0x3ca3d70a),
        Float::from_bits(0x40000000),
        Float::from_bits(0x3f4ccccd),
        Float::from_bits(0x40a00000),
        Float::from_bits(0x3c23d70a),
        Float::from_bits(0x42c80000),
        Float::from_bits(0x40400000),
        Float::from_bits(0x40e00000),
        Float::from_bits(0x3f19999a),
        Float::from_bits(0x3c1374bc),
        Float::from_bits(0x3a83126f),
    ],
    _gap_75b5ac: Bytes::new(sparse::<4>(&[(0x2, &[0x80, 0x3f])])),
    flt_75b5b0: Float::from_bits(0),
    _gap_75b5b4: Bytes::new(sparse::<28>(&[(
        0x0,
        &[
            0x66, 0x66, 0xe6, 0x3f, 0x9a, 0x99, 0x99, 0x3f, 0x9a, 0x99, 0x19, 0x3f, 0x9a, 0x99,
            0x99, 0x3e, 0x9a, 0x99, 0x19, 0x3e, 0x0a, 0xd7, 0xa3, 0x3d, 0x8f, 0xc2, 0x75, 0x3d,
        ],
    )])),
    gauge_rates: [
        Float::from_bits(0x3d99999a),
        Float::from_bits(0x3e19999a),
        Float::from_bits(0x3d99999a),
        Float::from_bits(0x3d99999a),
        Float::from_bits(0x3e000000),
        Float::from_bits(0x3f000000),
        Float::from_bits(0x3e800000),
        Float::from_bits(0x3e800000),
        Float::from_bits(0x3dcccccd),
        Float::from_bits(0x420c0000),
        Float::from_bits(0x42480000),
        Float::from_bits(0x41c80000),
        Float::from_bits(0x399d4952),
        Float::from_bits(0x3951b717),
        Float::from_bits(0x3a03126f),
        Float::from_bits(0x3ba3d70a),
        Float::from_bits(0x3ca3d70a),
        Float::from_bits(0x3e4ccccd),
        Float::from_bits(0x3c03126f),
        Float::from_bits(0x3e19999a),
        Float::from_bits(0x40800000),
        Float::from_bits(0x3d4ccccd),
        Float::from_bits(0x3ca3d70a),
        Float::from_bits(0x3ba3d70a),
    ],
    kind_table_a: [Float::from_bits(0x3f99999a), Float::from_bits(0x3f800000)],
    kind_table_b: [Float::from_bits(0x3f800000), Float::from_bits(0x3f800000)],
    lerp_tables_b: [
        [
            Float::from_bits(0),
            Float::from_bits(0x3f000000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3f800000),
        ],
        [
            Float::from_bits(0),
            Float::from_bits(0x3f800000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3f000000),
        ],
        [
            Float::from_bits(0),
            Float::from_bits(0x3f800000),
            Float::from_bits(0x42c80000),
            Float::from_bits(0x3f000000),
        ],
    ],
    flt_75b670: Float::from_bits(0x3e19999a),
    _gap_75b674: Bytes::new(sparse::<4>(&[(0x0, &[0xcd, 0xcc, 0xcc, 0x3d])])),
    figures: [
        Float::from_bits(0x3e000000),
        Float::from_bits(0x3d4ccccd),
        Float::from_bits(0x3da3d70a),
        Float::from_bits(0x3d75c28f),
        Float::from_bits(0x3e99999a),
        Float::from_bits(0x3da3d70a),
        Float::from_bits(0x3ea3d70a),
        Float::from_bits(0x3f19999a),
        Float::from_bits(0x3e4ccccd),
        Float::from_bits(0x3e99999a),
        Float::from_bits(0x3f800000),
        Float::from_bits(0x3ecccccd),
        Float::from_bits(0x3e4ccccd),
        Float::from_bits(0x41000000),
        Float::from_bits(0x40c00000),
        Float::from_bits(0x40e00000),
        Float::from_bits(0x40400000),
        Float::from_bits(0x41400000),
        Float::from_bits(0x41700000),
        Float::from_bits(0x41200000),
        Float::from_bits(0x41200000),
        Float::from_bits(0x41c80000),
        Float::from_bits(0x41700000),
        Float::from_bits(0x41800000),
        Float::from_bits(0x42a00000),
        Float::from_bits(0x42200000),
        Float::from_bits(0x41f00000),
        Float::from_bits(0x42480000),
        Float::from_bits(0x42200000),
        Float::from_bits(0x41a00000),
        Float::from_bits(0x41200000),
        Float::from_bits(0x40a00000),
        Float::from_bits(0x3c75c28f),
    ],
    _gap_75b6fc: Bytes::new(sparse::<64>(&[(
        0x0,
        &[
            0x64, 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x14, 0x00,
            0x00, 0x00, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x1e, 0x00, 0x00, 0x00,
            0x3c, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64, 0x00,
            0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
            0x64, 0x00, 0x00, 0x00, 0x64,
        ],
    )])),
    wear_table: [
        Float::from_bits(0x64),
        Float::from_bits(0x14),
        Float::from_bits(0x28),
        Float::from_bits(0xa),
        Float::from_bits(0xf),
        Float::from_bits(0xf),
        Float::from_bits(0x1e),
        Float::from_bits(0x3c),
        Float::from_bits(5),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x14),
        Float::from_bits(0x23),
        Float::from_bits(8),
        Float::from_bits(0xd),
        Float::from_bits(0xc),
        Float::from_bits(0x19),
        Float::from_bits(0x32),
        Float::from_bits(4),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x12),
        Float::from_bits(0x28),
        Float::from_bits(6),
        Float::from_bits(0xa),
        Float::from_bits(0xa),
        Float::from_bits(0x14),
        Float::from_bits(0x28),
        Float::from_bits(3),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0xf),
        Float::from_bits(0x23),
        Float::from_bits(0x1c),
        Float::from_bits(0x23),
        Float::from_bits(0xf),
        Float::from_bits(0x1e),
        Float::from_bits(0x3c),
        Float::from_bits(8),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0xc),
        Float::from_bits(0x1e),
        Float::from_bits(0x16),
        Float::from_bits(0x1c),
        Float::from_bits(0xf),
        Float::from_bits(0x19),
        Float::from_bits(0x32),
        Float::from_bits(7),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0xc),
        Float::from_bits(0x12),
        Float::from_bits(0xa),
        Float::from_bits(0xc),
        Float::from_bits(0xa),
        Float::from_bits(0xf),
        Float::from_bits(0x28),
        Float::from_bits(3),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
        Float::from_bits(0x64),
    ],
    flt_75b8bc: Float::from_bits(0x40200000),
    dword_75b8c0: Dword::new(0),
    dword_75b8c4: Dword::new(0),
    _gap_75b8c8: Bytes::new(sparse::<120>(&[(
        0x0,
        &[
            0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x05, 0x00,
            0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            0x08, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07, 0x00,
            0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00,
            0x07, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x07,
        ],
    )])),
};

/// IDA 0X75B340, the first byte of the piece.
pub const START: usize = 0x75B340;
/// IDA 0X75B940, the end of the piece.
pub const END: usize = 0x75B940;

/// Where IDA 0X75B340 is now (`moved::MOVED`'s home).
pub fn damage_addr() -> u32 {
    crate::pointers::game_address(&DAMAGE as *const DamageModel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn damage_starts_as_the_games() {
        let size = size_of::<DamageModel>();
        assert!(END - START <= size && size <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts(&DAMAGE as *const DamageModel as *const u8, END - START)
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(DamageModel, lerp_tables), 0x75B4A8);
        at(offset_of!(DamageModel, wear_table), 0x75B73C);
    }
}
