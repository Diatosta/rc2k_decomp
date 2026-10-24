//! The texture upload's surface description (DDSURFACEDESC) and the mode-trial switch (phase 5, `notes/conversion.md`): IDA 0x5ABA18 .. 0x5ABA8C out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Upload,
    STORE,
    0x5ABA18,
    0x5ABA8C,
    crate::gfx::upload_data::RUNS
);

pub const START: usize = 0x5ABA18;

pub type Slot = crate::gfx::block::Slot<Upload>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Upload>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
