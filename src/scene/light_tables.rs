//! The tables of 0x100 colours the lighting picks from, and the two functions that
//! fill one: shared by their three hooks.

use crate::framework::{EAX, EBP, EBX, ECX, EDX, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::Region;

pub const LIGHT_TABLE_RVA: usize = rva(0x635118); // g_lightTable
pub const LAMP_TABLE_A_RVA: usize = rva(0x635518); // dword_635518
pub const LAMP_TABLE_B_RVA: usize = rva(0x635918); // dword_635918
pub const CAR_LIGHT_TABLE_RVA: usize = rva(0x635D18); // g_carLightTable
pub const LAMP_TABLE_C_RVA: usize = rva(0x636118); // unk_636118
pub const TABLE_LEN: usize = 0x100 * 4;

pub fn table(rva: usize) -> Region {
    (game_addr(rva) as u32, TABLE_LEN)
}

fn left(eax: u32, last: u32) -> Left {
    Left::nothing()
        .with(EAX, eax)
        .with(EBX, 0x100)
        .with(ECX, last)
        .with(EDX, 0)
        .with(EBP, 0xFF)
}

unsafe fn store(table: *mut [u32; 0x100], entries: &[u32; 0x100]) {
    unsafe { table.write_unaligned(*entries) };
}

/// `scene::fill_light_ramp` (0x44F169): entry n = the first colour plus n over 255 of
/// the way to the second, each of three parts a byte, 0xFF above them. Converted:
/// `rc2k_core::scene::light::ramp`.
pub unsafe fn fill_light_ramp(colours: *const [u32; 6], table: *mut [u32; 0x100]) -> Left {
    unsafe {
        let c = colours.read_unaligned();
        let entries = rc2k_core::scene::light::ramp([c[0], c[1], c[2]], [c[3], c[4], c[5]]);
        store(table, &entries);
        left(c[5], entries[0xFF])
    }
}

/// `scene::fill_light_scale` (0x44F1BC): entry n = the colour times n over 255.
/// Converted: `rc2k_core::scene::light::scale`.
pub unsafe fn fill_light_scale(colour: *const [u32; 3], table: *mut [u32; 0x100]) -> Left {
    unsafe {
        let c = colour.read_unaligned();
        let entries = rc2k_core::scene::light::scale(c);
        store(table, &entries);
        left(c[2], entries[0xFF])
    }
}

/// Colours for a test, by lot: bytes, and values no colour has.
pub fn test_colours(seed: &mut u32) -> [u32; 6] {
    std::array::from_fn(|_| {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let lot = *seed >> 8;
        match lot % 7 {
            0 => 0,
            1 => 0xFF,
            2 => (lot / 7) % 0x1_0000,
            3 => ((lot / 7) % 0x200).wrapping_neg(),
            _ => (lot / 7) % 0x100,
        }
    })
}
