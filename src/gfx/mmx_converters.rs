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
//! 2^32). Compared in shadow mode. Self-test: the original against the Rust version on
//! pictures of the test's, all four formats; the 3DNow originals only where the processor has
//! 3DNow (a modern AMD or Intel one does not: then those tests skip).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::gfx::block::game_read;
use crate::gfx::converters::Slot as ConvSlot;

/// A converter: the format it makes and where its constants are (IDA addresses).
#[derive(Clone, Copy)]
pub struct Converter {
    pub format: u32,
    pub constants: ConvSlot,
    pub three_dnow: bool,
}

fn constant(at: ConvSlot) -> u64 {
    unsafe { game_read(at) }
}

/// The converter's five masks.
fn masks(c: &Converter) -> [u64; 5] {
    std::array::from_fn(|n| constant(c.constants + 8 * n))
}

/// The Rust version: `rows` rows of `pixels` pixels (the original's ecx, edx) from `from`
/// (esi) to `to` (edi), `gap` (ebp) bytes after each row made. Its one caller,
/// `gfx::convert_picture`, reads nothing it leaves in the registers.
pub fn convert_rows(
    from: u32,
    to: u32,
    pixels: u32,
    rows: u32,
    gap: u32,
    c: Converter,
) -> Result<(), Refused> {
    let steps = pixels >> 2;
    if steps == 0 || rows == 0 {
        return Err(BadInput(
            "fewer than four pixels a row or no rows (the original's dec counts on)",
        )
        .into());
    }
    // Converted: `rc2k_core::gfx::converters` (`step`, `next_row`).
    use rc2k_core::gfx::converters::{next_row, step};
    let k = masks(&c);
    let (mut from, mut to) = (from, to);
    for _ in 0..rows {
        for _ in 0..steps {
            unsafe {
                let q0 = (from as *const u64).read_unaligned();
                let q1 = ((from + 8) as *const u64).read_unaligned();
                (to as *mut u64).write_unaligned(step(c.format, &k, q0, q1));
            }
            from = from.wrapping_add(0x10);
            to = to.wrapping_add(8);
        }
        to = next_row(to, gap, c.format, c.three_dnow);
    }
    Ok(())
}

/// The eight converters by their IDA address (hooks removed 2026-10-23: `gfx::convert_picture`
/// finds the one the game's table names here).
pub const CONVERTERS: [(u32, Converter); 8] = [
    (
        0x4070F9,
        Converter {
            format: 1,
            constants: ConvSlot::at(0x4F51F8),
            three_dnow: false,
        },
    ), // gfx::rows_565_mmx
    (
        0x407171,
        Converter {
            format: 2,
            constants: ConvSlot::at(0x4F5218),
            three_dnow: false,
        },
    ), // gfx::rows_555_mmx
    (
        0x4071ED,
        Converter {
            format: 3,
            constants: ConvSlot::at(0x4F5238),
            three_dnow: false,
        },
    ), // gfx::rows_1555_mmx
    (
        0x407276,
        Converter {
            format: 4,
            constants: ConvSlot::at(0x4F5260),
            three_dnow: false,
        },
    ), // gfx::rows_4444_mmx
    (
        0x4072FB,
        Converter {
            format: 1,
            constants: ConvSlot::at(0x4F51F8),
            three_dnow: true,
        },
    ), // gfx::rows_565_3dnow
    (
        0x407377,
        Converter {
            format: 2,
            constants: ConvSlot::at(0x4F5218),
            three_dnow: true,
        },
    ), // gfx::rows_555_3dnow
    (
        0x4073F7,
        Converter {
            format: 3,
            constants: ConvSlot::at(0x4F5238),
            three_dnow: true,
        },
    ), // gfx::rows_1555_3dnow
    (
        0x407484,
        Converter {
            format: 4,
            constants: ConvSlot::at(0x4F5260),
            three_dnow: true,
        },
    ), // gfx::rows_4444_3dnow
];

/// The converter at `at` (an IDA address), if it is one of the eight.
pub fn converter_at(at: u32) -> Option<Converter> {
    CONVERTERS.iter().find(|(a, _)| *a == at).map(|(_, c)| *c)
}
