//! The blitter's and rescaler's variables first values: the bytes of RAL.EXE's 0x4F503C .. 0x4F50C0 that are not zero (generated from the image; `blit::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[(
    0x4,
    &[0x1f, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x57, 0x02],
)];
