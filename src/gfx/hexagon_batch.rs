//! The marks' hexagon batch drawn by d3d (the index and vertex counts, the fans, the records) (phase 5, `notes/conversion.md`): IDA 0x62423C .. 0x628A44 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Hexagons,
    STORE,
    0x62423C,
    0x628A44,
    crate::gfx::hexagon_batch_data::RUNS
);

pub const START: usize = 0x62423C;

pub type Slot = crate::gfx::block::Slot<Hexagons>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Hexagons>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
