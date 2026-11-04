//! The screenshot writer's plane buffer, out of RAL.EXE's image (phase 5; typed in phase 7,
//! `notes/conversion.md`): IDA 0x94F6F8 .. 0x9526F8. `byte_94F6F8`: the three planes of the
//! row being written (3 * width bytes, up to 1920 wide).

use crate::session::Bytes;

/// IDA's address of the block's first byte.
pub const START: usize = 0x94F6F8;
/// The block's length.
pub const LEN: usize = 0x9526F8 - START;

/// The planes of the row being written (a buffer, zero at the start).
pub static PLANES: Bytes<LEN> = Bytes::new([0; LEN]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&PLANES, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), PLANES.addr() as usize);
        assert_eq!(
            crate::moved::place(START + LEN - 1),
            (PLANES.addr() + LEN as u32 - 1) as usize
        );
    }
}
