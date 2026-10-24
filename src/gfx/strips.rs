//! The HUD's view strips' places (sizes, tops, lefts) (phase 5, `notes/conversion.md`): IDA 0x6D1870 .. 0x6D18C8 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Strips,
    STORE,
    0x6D1870,
    0x6D18C8,
    crate::gfx::strips_data::RUNS
);

pub const START: usize = 0x6D1870;

pub type Slot = crate::gfx::block::Slot<Strips>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Strips>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
