//! The tables of 0x100 colours the lighting picks from, and the two functions that fill one:
//! shared by their three functions.

use crate::session::Dword;

fn store(table: &[Dword; 0x100], entries: &[u32; 0x100]) {
    for (cell, &e) in table.iter().zip(entries) {
        cell.set(e);
    }
}

/// `scene::fill_light_ramp` (0x44F169): entry n = the first colour plus n over 255 of
/// the way to the second, each of three parts a byte, 0xFF above them. Converted:
/// `rc2k_core::scene::light::ramp`.
pub fn fill_light_ramp(c: [u32; 6], table: &[Dword; 0x100]) {
    let entries = rc2k_core::scene::light::ramp([c[0], c[1], c[2]], [c[3], c[4], c[5]]);
    store(table, &entries);
}

/// `scene::fill_light_scale` (0x44F1BC): entry n = the colour times n over 255.
/// Converted: `rc2k_core::scene::light::scale`.
pub fn fill_light_scale(c: [u32; 3], table: &[Dword; 0x100]) {
    store(table, &rc2k_core::scene::light::scale(c));
}
