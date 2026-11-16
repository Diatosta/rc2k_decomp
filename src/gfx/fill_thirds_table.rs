//! The table of n / 3 for n = 0 .. 0x2FF: 0x300 bytes at `thirds`, the mean of three components by
//! their sum.

pub const TABLE_LEN: usize = 0x300;

pub fn fill_thirds_table(gfx: &crate::gfx::owner::Gfx) {
    let table: [u8; TABLE_LEN] = rc2k_core::gfx::pixels::thirds_table();
    gfx.drawing.thirds.write(&table);
}
