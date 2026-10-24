//! The screenshot writer's plane buffer (3 * width bytes, up to 1920 wide) (phase 5, `notes/conversion.md`): IDA 0x94F6F8 .. 0x9526F8 out of RAL.EXE's image, one block
//! with the image's layout (`gfx::block`): an address `X` of IDA is at `base() + (X - START)`;
//! a `Slot` is such an address, checked against the block.

crate::region!(
    Planes,
    STORE,
    0x94F6F8,
    0x9526F8,
    crate::gfx::pcx_planes_data::RUNS
);

pub const START: usize = 0x94F6F8;

/// `byte_94F6F8`: the three planes of the row being written (3 * width bytes).
pub const PLANES: Slot = Slot::at(0x94F6F8);

pub type Slot = crate::gfx::block::Slot<Planes>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::block::first_values_are_the_games::<Planes>(&STORE);
    }

    #[test]
    fn moved_knows_the_block() {
        assert_eq!(crate::moved::place(START), STORE.addr());
        assert_eq!(crate::moved::place(START + LEN - 1), STORE.addr() + LEN - 1);
    }
}
