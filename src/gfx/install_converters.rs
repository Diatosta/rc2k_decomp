//! `gfx::install_converters` (0x407062): fills the table of picture converters, one for each
//! pixel format number.
//!
//! The converter table has seven entries; a format without one is converted pixel by pixel
//! through `components_to_pixel`. Number 0 never has one. Numbers 5 and 6 get
//! `gfx::copy_pixel_rows`. Number 4 gets `gfx::components_to_4444_rows`, and numbers 1 to 3
//! nothing, unless the processor has MMX (`machine.has_mmx` is 1): then 1 to 4 get 0x4070F9,
//! 0x407171, 0x4071ED, 0x407276, or with 3DNow as well (`machine.has_3dnow` is 1)
//! 0x4072FB, 0x407377, 0x4073F7, 0x407484. Those eight are not ported. The name
//! is a reading. `start` calls it directly.

use crate::gfx::components_to_4444_rows::COMPONENTS_TO_4444_ROWS;
use crate::gfx::converters::Converters;
use crate::gfx::copy_pixel_rows::COPY_PIXEL_ROWS;

// ---- game data ----

pub const CONVERTER_COUNT: usize = rc2k_core::gfx::converters::CONVERTERS;

/// The converters for formats 1 to 4 with MMX, and with 3DNow as well.
const MMX: [u32; 4] = [0x4070F9, 0x407171, 0x4071ED, 0x407276];
const AMD: [u32; 4] = [0x4072FB, 0x407377, 0x4073F7, 0x407484];

/// The table for a processor with or without the two instruction sets, as IDA's addresses; 0 = no
/// converter.
pub fn converters(has_mmx: u32, has_3dnow: u32) -> [u32; CONVERTER_COUNT] {
    // Converted: `rc2k_core::gfx::converters::converters`.
    use rc2k_core::gfx::converters::Converter;
    rc2k_core::gfx::converters::converters(has_mmx, has_3dnow).map(|entry| match entry {
        Converter::None => 0,
        Converter::Components4444 => COMPONENTS_TO_4444_ROWS,
        Converter::CopyRows => COPY_PIXEL_ROWS,
        Converter::Mmx(n) => MMX[n as usize - 1],
        Converter::ThreeDNow(n) => AMD[n as usize - 1],
    })
}

pub fn install_converters(converters_store: &Converters, system: &crate::system::owner::System) {
    let table = converters(
        system.session.machine.has_mmx.get(),
        system.session.machine.has_3dnow.get(),
    );
    for (cell, entry) in converters_store.table.iter().zip(table) {
        cell.set(entry);
    }
}
