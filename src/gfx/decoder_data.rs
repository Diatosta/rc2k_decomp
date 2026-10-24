//! The picture decoders' and glyph builder's variables (decoded picture, palette, sheet table, PCX header and stream) first values: the bytes of RAL.EXE's 0x519118 .. 0x51B9C8 that are not zero (generated from the image; `decoder::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[
    (0x43d, &[0x04]),
    (0x4d0, &[0xff, 0xff, 0xff, 0xff]),
    (
        0x285c,
        &[
            0x20, 0x03, 0x00, 0x00, 0x58, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5e, 0x01,
        ],
    ),
];
