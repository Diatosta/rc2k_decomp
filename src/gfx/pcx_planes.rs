//! The screenshot writer's plane buffer, out of RAL.EXE's image: IDA 0x94F6F8 .. 0x9526F8.
//! `byte_94F6F8`: the three planes of the row being written (3 * width bytes, up to 1920 wide).

/// IDA's address of the block's first byte.
pub const START: usize = 0x94F6F8;
/// The block's length.
pub const LEN: usize = 0x9526F8 - START;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::owner::gfx;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&gfx().pictures.planes, START, START + LEN);
    }

    #[test]
    fn moved_knows_the_block() {
        let pictures = &gfx().pictures;
        assert_eq!(crate::moved::place(START), pictures.planes.addr() as usize);
        assert_eq!(
            crate::moved::place(START + LEN - 1),
            (pictures.planes.addr() + LEN as u32 - 1) as usize
        );
    }
}
