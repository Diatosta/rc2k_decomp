//! The progress bar's variables (position, width, glyphs, percent) (phase 5, `notes/conversion.md`): IDA 0x755224 .. 0x75523C out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Bar,
    STORE,
    0x755224,
    0x75523C,
    crate::gfx::bar_store_data::RUNS
);

pub const START: usize = 0x755224;

pub type Slot = crate::gfx::block::Slot<Bar>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Bar>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
