//! The screenshot writer's plane buffer, out of RAL.EXE's image: IDA 0x94F6F8 .. 0x9526F8.
//! `planes`: the three planes of the row being written (3 * width bytes, up to 1920 wide).

/// IDA's address of the block's first byte.
pub const START: usize = 0x94F6F8;
/// The block's length.
pub const LEN: usize = 0x9526F8 - START;

#[cfg(test)]
mod tests {
    use crate::gfx::owner::gfx;

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(&gfx().pictures.planes);
    }
}
