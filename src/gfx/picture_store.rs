//! The picture loader's variables (the name, flags, size, rows and buffer of the picture, the list, the grades) (phase 5, `notes/conversion.md`): IDA 0x4F49C8 .. 0x4F4B30 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Picture,
    STORE,
    0x4F49C8,
    0x4F4B30,
    crate::gfx::picture_store_data::RUNS
);

pub const START: usize = 0x4F49C8;

pub type Slot = crate::gfx::block::Slot<Picture>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Picture>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
