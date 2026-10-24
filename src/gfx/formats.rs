//! The pixel formats' work variables (format numbers, row table, component format) (phase 5, `notes/conversion.md`): IDA 0x4F3F78 .. 0x4F492C out of RAL.EXE's image, one block (to the row table's end: its 602 dwords from 0x4F3FC4; 2026-10-24, the block once ended at 0x4F3FCC and the table ran past it)
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Formats,
    STORE,
    0x4F3F78,
    0x4F492C,
    crate::gfx::formats_data::RUNS
);

pub const START: usize = 0x4F3F78;

pub type Slot = crate::gfx::block::Slot<Formats>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Formats>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
