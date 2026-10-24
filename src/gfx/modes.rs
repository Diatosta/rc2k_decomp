//! The screen mode tables (the thirds table, pixel kinds, sizes, the modes listed) (phase 5, `notes/conversion.md`): IDA 0x4F4B30 .. 0x4F503C out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Modes,
    STORE,
    0x4F4B30,
    0x4F503C,
    crate::gfx::modes_data::RUNS
);

pub const START: usize = 0x4F4B30;

pub type Slot = crate::gfx::block::Slot<Modes>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Modes>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
