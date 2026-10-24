//! The screenshot switch, its lock and the file name's template first values: the bytes of RAL.EXE's 0x4F52B4 .. 0x4F52C4 that are not zero (generated from the image; `shots::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[(0x8, &[0x30, 0x30, 0x30, 0x2e, 0x70, 0x63, 0x78])];
