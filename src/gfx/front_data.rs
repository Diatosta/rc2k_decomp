//! The DirectDraw front and back surface handles first values: the bytes of RAL.EXE's 0x4F32B4 .. 0x4F32BC that are not zero (generated from the image; `front::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
