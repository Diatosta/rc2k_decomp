//! The PNG header's fields (width, height, compression, colour type) (phase 5, `notes/conversion.md`): IDA 0xA8C190 .. 0xA8C1A4 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    PngHeader,
    STORE,
    0xA8C190,
    0xA8C1A4,
    crate::gfx::png_header_data::RUNS
);

pub const START: usize = 0xA8C190;

pub type Slot = crate::gfx::block::Slot<PngHeader>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<PngHeader>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
