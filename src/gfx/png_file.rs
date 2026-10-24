//! The PNG decoder's file (pointer, length, memory list) (phase 5, `notes/conversion.md`): IDA 0x5BEC30 .. 0x5BEC48 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Png,
    STORE,
    0x5BEC30,
    0x5BEC48,
    crate::gfx::png_file_data::RUNS
);

pub const START: usize = 0x5BEC30;

pub type Slot = crate::gfx::block::Slot<Png>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Png>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
