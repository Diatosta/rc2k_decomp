//! The 3D primary and secondary card switches (phase 5, `notes/conversion.md`): IDA 0x5AB4AC .. 0x5AB4B4 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Cards,
    STORE,
    0x5AB4AC,
    0x5AB4B4,
    crate::gfx::cards_data::RUNS
);

pub const START: usize = 0x5AB4AC;

pub type Slot = crate::gfx::block::Slot<Cards>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Cards>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
