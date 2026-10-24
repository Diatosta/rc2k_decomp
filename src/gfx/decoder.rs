//! The picture decoders' and glyph builder's variables (decoded picture, palette, sheet table, PCX header and stream) (phase 5, `notes/conversion.md`): IDA 0x519118 .. 0x51B9C8 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Decoder,
    STORE,
    0x519118,
    0x51B9C8,
    crate::gfx::decoder_data::RUNS
);

pub const START: usize = 0x519118;

pub type Slot = crate::gfx::block::Slot<Decoder>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Decoder>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
