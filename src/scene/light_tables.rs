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

/// A part of a colour times n over 255, as the original's `imul` and `idiv` give it.
fn scaled(part: u32, n: u32) -> u32 {
    ((part as i32 as i64 * n as i64) / 255) as u32
}

fn packed(parts: [u32; 3]) -> u32 {
    0xFF00_0000 | (parts[0] & 0xFF) << 16 | (parts[1] & 0xFF) << 8 | (parts[2] & 0xFF)
}

fn left(eax: u32, last: u32) -> Left {
    Left::nothing()
        .with(EAX, eax)
        .with(EBX, 0x100)
        .with(ECX, last)
        .with(EDX, 0)
        .with(EBP, 0xFF)
}

/// `scene::fill_light_ramp` (0x44F169): entry n = the first colour plus n over 255 of
/// the way to the second, each of three parts a byte, 0xFF above them.
pub unsafe fn fill_light_ramp(colours: *const [u32; 6], table: *mut [u32; 0x100]) -> Left {
    unsafe {
        let c = colours.read_unaligned();
        let mut last = 0;
        for n in 0..0x100u32 {
            last = packed(std::array::from_fn(|part| {
                scaled(c[part + 3].wrapping_sub(c[part]), n).wrapping_add(c[part])
            }));
            (table as *mut u32).add(n as usize).write_unaligned(last);
        }
        left(c[5], last)
    }
}

/// `scene::fill_light_scale` (0x44F1BC): entry n = the colour times n over 255.
pub unsafe fn fill_light_scale(colour: *const [u32; 3], table: *mut [u32; 0x100]) -> Left {
    unsafe {
        let c = colour.read_unaligned();
        let mut last = 0;
        for n in 0..0x100u32 {
            last = packed(c.map(|part| scaled(part, n)));
            (table as *mut u32).add(n as usize).write_unaligned(last);
        }
        left(c[2], last)
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
