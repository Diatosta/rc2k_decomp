//! The picture converters for 16-bit screens with MMX (`gfx::install_converters` puts them
//! in `dword_4F5288` for the surface format numbers 1 .. 4 when the processor has MMX, the
//! 3DNow ones when it has that as well; `gfx::convert_picture` calls the one of the
//! surface's format): four pixels of four 8-bit components a step turned into four 16-bit
//! pixels, as MMX does it:
//! - the components picked (`pand` with 0x00FF.., the second and fourth after `psrlw 8`)
//!   and packed (`packuswb`): mm0 = the first and third of each pixel, mm2 = the second and
//!   fourth;
//! - format 1 (5-6-5: 0x4070F9, 3DNow 0x4072FB; constants 0x4F51F8 ..): mm0 & 0xF800 | mm0
//!   >> 3 & 0x001F | mm2 << 3 & 0x07E0;
//! - format 2 (5-5-5: 0x407171, 0x407377; 0x4F5218 ..): mm0 >> 1 & 0x7C00 | mm0 >> 3 &
//!   0x001F | mm2 << 2 & 0x03E0;
//! - format 3 (1-5-5-5: 0x4071ED, 0x4073F7; 0x4F5238 .., the alpha 0x4F5258): as 2, and
//!   mm2's alpha bit (& 0x8000);
//! - format 4 (4-4-4-4: 0x407276, 0x407484; 0x4F5260 .., 0x4F5280): mm0 >> 4 & 0x0F00 | mm0
//!   >> 4 & 0x000F | mm2 & 0x00F0 | mm2 & 0xF000.
//!
//! The masks are the game's constants (read from it, so a change there would show).
//!
//! In:  esi = the pixels of four components, edi = where the 16-bit ones go, ecx = the
//!      pixels a row (a quarter of it, rounded down, is done: the rest of a row is not),
//!      edx = the rows, ebp = added to edi (format 1, MMX) or taken from it (the others)
//!      after each row
//! Out: esi, edi past what was done, edx 0, ecx a row's steps (ecx / 4); the MMX ones ebx
//!      a row's steps too, the 3DNow ones ebx the source past what was done and esi a
//!      row's steps; the zero flag set (`dec edx`)
//!
//! Callers: `gfx::convert_picture` (one row of width x height pixels, ebp 0). Refuses (before
//! anything) fewer than four pixels a row and no rows (the original's `dec` counts on through
//! 2^32).

use crate::errors::BadInput;
use crate::gfx::converters::Converters;

/// A converter: the format it makes and where its constants are (the number of the first
/// of its five masks in `gfx::converters`).
#[derive(Clone, Copy)]
pub struct Converter {
    pub format: u32,
    pub constants: usize,
    // The original's next-row step differs for the 3DNow ones; its one call (one row) never
    // takes it since phase 12.
    #[allow(dead_code)]
    pub three_dnow: bool,
}

fn constant(table: &Converters, n: usize) -> u64 {
    table.mask(n)
}

/// The converter's five masks.
fn masks(table: &Converters, c: &Converter) -> [u64; 5] {
    std::array::from_fn(|n| constant(table, c.constants + n))
}

/// The Rust version, in place on the picture's bytes: its one caller, `gfx::convert_picture`,
/// gives esi = edi = the buffer, one row (edx 1) of `pixels` pixels (ecx), ebp 0, and reads
/// nothing it leaves in the registers. Each step's 16 bytes are read before its 8 are
/// written, behind them.
pub fn convert_in_place(
    table: &Converters,
    bytes: &mut [u8],
    pixels: u32,
    c: Converter,
) -> Result<(), BadInput> {
    let steps = (pixels >> 2) as usize;
    if steps == 0 {
        return Err(BadInput(
            "fewer than four pixels a row (the original's dec counts on)",
        ));
    }
    if bytes.len() < steps * 16 {
        return Err(BadInput("a picture shorter than its pixels"));
    }
    // Converted: `rc2k_core::gfx::converters` (`step`).
    use rc2k_core::gfx::converters::step;
    let k = masks(table, &c);
    let quad =
        |b: &[u8], at: usize| u64::from_le_bytes(b[at..at + 8].try_into().expect("eight bytes"));
    for n in 0..steps {
        let (q0, q1) = (quad(bytes, 16 * n), quad(bytes, 16 * n + 8));
        bytes[8 * n..8 * n + 8].copy_from_slice(&step(c.format, &k, q0, q1).to_le_bytes());
    }
    Ok(())
}

/// The eight converters by their IDA address (`gfx::convert_picture`
/// finds the one the game's table names here).
pub const CONVERTERS: [(u32, Converter); 8] = [
    (
        0x4070F9,
        Converter {
            format: 1,
            constants: 1,
            three_dnow: false,
        },
    ), // gfx::rows_565_mmx
    (
        0x407171,
        Converter {
            format: 2,
            constants: 5,
            three_dnow: false,
        },
    ), // gfx::rows_555_mmx
    (
        0x4071ED,
        Converter {
            format: 3,
            constants: 9,
            three_dnow: false,
        },
    ), // gfx::rows_1555_mmx
    (
        0x407276,
        Converter {
            format: 4,
            constants: 14,
            three_dnow: false,
        },
    ), // gfx::rows_4444_mmx
    (
        0x4072FB,
        Converter {
            format: 1,
            constants: 1,
            three_dnow: true,
        },
    ), // gfx::rows_565_3dnow
    (
        0x407377,
        Converter {
            format: 2,
            constants: 5,
            three_dnow: true,
        },
    ), // gfx::rows_555_3dnow
    (
        0x4073F7,
        Converter {
            format: 3,
            constants: 9,
            three_dnow: true,
        },
    ), // gfx::rows_1555_3dnow
    (
        0x407484,
        Converter {
            format: 4,
            constants: 14,
            three_dnow: true,
        },
    ), // gfx::rows_4444_3dnow
];

/// The converter at `at` (an IDA address), if it is one of the eight.
pub fn converter_at(at: u32) -> Option<Converter> {
    CONVERTERS.iter().find(|(a, _)| *a == at).map(|(_, c)| *c)
}
