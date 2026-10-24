//! The 3D reports switch (dword_4F52AC) first values: the bytes of RAL.EXE's 0x4F52AC .. 0x4F52B0 that are not zero (generated from the image; `reports::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[(0x0, &[0x01])];
