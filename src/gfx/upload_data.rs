//! The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch first values: the bytes of RAL.EXE's 0x5ABA18 .. 0x5ABA8C that are not zero (generated from the image; `upload::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
