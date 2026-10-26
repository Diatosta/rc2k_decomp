//! The tables of 0x100 colours the lighting picks from, and the two functions that fill one:
//! shared by their three functions.

use crate::errors::BadInput;
use crate::pointers::GamePtr;

fn store(table: u32, entries: &[u32; 0x100]) -> Result<(), BadInput> {
    GamePtr::new(table).write(*entries, "a table of light colours")
}

/// `scene::fill_light_ramp` (0x44F169): entry n = the first colour plus n over 255 of
/// the way to the second, each of three parts a byte, 0xFF above them. Converted:
/// `rc2k_core::scene::light::ramp`.
pub fn fill_light_ramp(colours: u32, table: u32) -> Result<(), BadInput> {
    let c: [u32; 6] = GamePtr::new(colours).read("two light colours")?;
    let entries = rc2k_core::scene::light::ramp([c[0], c[1], c[2]], [c[3], c[4], c[5]]);
    store(table, &entries)
}

/// `scene::fill_light_scale` (0x44F1BC): entry n = the colour times n over 255.
/// Converted: `rc2k_core::scene::light::scale`.
pub fn fill_light_scale(colour: u32, table: u32) -> Result<(), BadInput> {
    let c: [u32; 3] = GamePtr::new(colour).read("a light colour")?;
    let entries = rc2k_core::scene::light::scale(c);
    store(table, &entries)
}
