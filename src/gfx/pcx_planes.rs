//! The screenshot writer's plane buffer: the three planes of the row being written
//! (3 * width bytes, up to 1920 wide).

/// The block's length.
pub const LEN: usize = 0x3000;
