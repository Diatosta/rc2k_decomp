//! `gfx::install_converters` (0x407062): fills the table of picture converters, one for each
//! pixel format number.
//!
//! In:  nothing
//! Out: every register preserved
//! The table `dword_4F5288` has seven entries; a format without one is converted pixel by pixel
//! through `components_to_pixel`. Number 0 never has one. Numbers 5 and 6 get
//! `gfx::copy_pixel_rows`. Number 4 gets `gfx::components_to_4444_rows`, and numbers 1 to 3
//! nothing, unless the processor has MMX (`dword_5183A8` is 1): then 1 to 4 get `sub_4070F9`,
//! `sub_407171`, `sub_4071ED`, `sub_407276`, or with 3DNow as well (`dword_5183B8` is 1)
//! `sub_4072FB`, `sub_407377`, `sub_4073F7`, `sub_407484`. Those eight are not ported. The name
//! is a reading. `start` calls it directly (2026-10-23); the registers above are the
//! original's, for reference.

use crate::framework::rva;
use crate::gfx::components_to_4444_rows::COMPONENTS_TO_4444_ROWS;
use crate::gfx::converters::Slot as ConvSlot;
use crate::gfx::copy_pixel_rows::COPY_PIXEL_ROWS;
use crate::session::MACHINE;
use crate::store::game_addr;

// ---- game data ----

pub const CONVERTERS_RVA: ConvSlot = ConvSlot::at(0x4F5288); // dword_4F5288
pub const CONVERTER_COUNT: usize = rc2k_core::gfx::converters::CONVERTERS;

/// The converters for formats 1 to 4 with MMX, and with 3DNow as well.
const MMX: [usize; 4] = [rva(0x4070F9), rva(0x407171), rva(0x4071ED), rva(0x407276)];
const AMD: [usize; 4] = [rva(0x4072FB), rva(0x407377), rva(0x4073F7), rva(0x407484)];

// ---- Rust version ----

/// The table for a processor with or without the two instruction sets, as addresses in the
/// game; 0 = no converter.
pub fn converters(has_mmx: u32, has_3dnow: u32) -> [u32; CONVERTER_COUNT] {
    // Converted: `rc2k_core::gfx::converters::converters`.
    use rc2k_core::gfx::converters::Converter;
    rc2k_core::gfx::converters::converters(has_mmx, has_3dnow).map(|entry| {
        let at = match entry {
            Converter::None => return 0,
            Converter::Components4444 => COMPONENTS_TO_4444_ROWS,
            Converter::CopyRows => COPY_PIXEL_ROWS,
            Converter::Mmx(n) => MMX[n as usize - 1],
            Converter::ThreeDNow(n) => AMD[n as usize - 1],
        };
        game_addr(at) as u32
    })
}

pub unsafe fn install_converters() {
    let table = converters(MACHINE.has_mmx.get(), MACHINE.has_3dnow.get());
    unsafe { (game_addr(CONVERTERS_RVA) as *mut [u32; CONVERTER_COUNT]).write_unaligned(table) };
}
