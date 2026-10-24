//! The screenshot writer's plane buffer (three planes of a row, up to 1920 wide) first values: the bytes of RAL.EXE's 0x94F6F8 .. 0x9526F8 that are not zero (generated from the image; `pcx_planes::tests::first_values_are_the_games` checks them while it exists).

/// `(offset from the block's start, bytes)`, in order, apart; the rest is zero.
pub const RUNS: &[(usize, &[u8])] = &[];
