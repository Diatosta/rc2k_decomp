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

use crate::errors::{BadInput, Refused};
use crate::framework::{EBX, ECX, EDI, EDX, ESI, Regs, SelfTest, game_read, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};

// ---- MMX's operations on a quadword ----

fn words(x: u64) -> [u16; 4] {
    [
        x as u16,
        (x >> 16) as u16,
        (x >> 32) as u16,
        (x >> 48) as u16,
    ]
}
fn quad(w: [u16; 4]) -> u64 {
    w[0] as u64 | (w[1] as u64) << 16 | (w[2] as u64) << 32 | (w[3] as u64) << 48
}
fn psrlw(x: u64, n: u32) -> u64 {
    quad(words(x).map(|w| w >> n))
}
fn psllw(x: u64, n: u32) -> u64 {
    quad(words(x).map(|w| w << n))
}
/// `packuswb a, b`: the eight signed words to unsigned bytes, saturated; a's four first.
fn packuswb(a: u64, b: u64) -> u64 {
    let sat = |w: u16| (w as i16).clamp(0, 255) as u64;
    let (a, b) = (words(a), words(b));
    let mut out = 0u64;
    for (n, &w) in a.iter().chain(b.iter()).enumerate() {
        out |= sat(w) << (8 * n);
    }
    out
}

/// A converter: the format it makes and where its constants are (IDA addresses).
#[derive(Clone, Copy)]
pub struct Converter {
    pub format: u32,
    pub constants: usize,
    pub three_dnow: bool,
}

fn constant(at: usize) -> u64 {
    unsafe { game_read(rva(at)) }
}

/// Four pixels (two quadwords at `from`) converted, by the format.
fn step(c: &Converter, q0: u64, q1: u64) -> u64 {
    let k = |n: usize| constant(c.constants + 8 * n);
    let (m4, m5, m6, m7) = (k(0), k(1), k(2), k(3));
    let low = packuswb(q0 & m4, q1 & m4);
    let high = packuswb(psrlw(q0, 8) & m4, psrlw(q1, 8) & m4);
    match c.format {
        1 => (low & m5) | (psrlw(low, 3) & m7) | (psllw(high, 3) & m6),
        2 => (psrlw(low, 1) & m5) | (psrlw(low, 3) & m7) | (psllw(high, 2) & m6),
        3 => (psrlw(low, 1) & m5) | (high & k(4)) | (psrlw(low, 3) & m7) | (psllw(high, 2) & m6),
        _ => (psrlw(low, 4) & m5) | (high & k(4)) | (psrlw(low, 4) & m7) | (high & m6),
    }
}

/// The Rust version: what it leaves (see the header).
pub fn convert_rows(entry: &Regs, c: Converter) -> Result<Left, Refused> {
    let steps = entry.ecx >> 2;
    if steps == 0 || entry.edx == 0 {
        return Err(BadInput(
            "fewer than four pixels a row or no rows (the original's dec counts on)",
        )
        .into());
    }
    let (mut from, mut to) = (entry.esi, entry.edi);
    for _ in 0..entry.edx {
        for _ in 0..steps {
            unsafe {
                let q0 = (from as *const u64).read_unaligned();
                let q1 = ((from + 8) as *const u64).read_unaligned();
                (to as *mut u64).write_unaligned(step(&c, q0, q1));
            }
            from = from.wrapping_add(0x10);
            to = to.wrapping_add(8);
        }
        to = if c.format == 1 && !c.three_dnow {
            to.wrapping_add(entry.ebp)
        } else {
            to.wrapping_sub(entry.ebp)
        };
    }
    let left = Left::nothing().with(ECX, steps).with(EDX, 0).with(EDI, to);
    let left = if c.three_dnow {
        left.with(EBX, from).with(ESI, steps)
    } else {
        left.with(EBX, steps).with(ESI, from)
    };
    Ok(Left {
        zero: Some(true),
        ..left
    })
}

/// Whether the processor has 3DNow (CPUID 0x80000001, edx bit 31).
fn has_3dnow() -> bool {
    #[cfg(target_arch = "x86")]
    {
        use std::arch::x86::__cpuid;
        if __cpuid(0x8000_0000).eax < 0x8000_0001 {
            return false;
        }
        __cpuid(0x8000_0001).edx & (1 << 31) != 0
    }
    #[cfg(not(target_arch = "x86"))]
    false
}

pub fn dispatch_converter(
    hook: &crate::framework::RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    c: Converter,
) {
    // The destination: the pixels written, ecx / 4 * 8 bytes a row.
    let rows = r.edx.min(0x1_0000) as usize;
    let bytes = ((r.ecx >> 2) as usize * 8).saturating_mul(rows);
    let regions = if bytes == 0 || bytes > 0x100_0000 {
        vec![]
    } else {
        vec![(r.edi, bytes)]
    };
    dispatch_state(hook, r, call_original, &regions, &|r| convert_rows(r, c));
}

pub fn test_converter(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    c: Converter,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    if c.three_dnow && !has_3dnow() {
        return t.skip("the processor has no 3DNow: the original cannot run here");
    }
    for (width, rows, ebp) in [(4u32, 1u32, 0u32), (16, 3, 0), (13, 2, 0), (8, 2, 8)] {
        // a row's room: the steps' bytes and ebp's moves
        let pixels: Vec<u32> = (0..width * rows)
            .map(|n| n.wrapping_mul(0x9E37_79B9) ^ 0x1234_5678 ^ (n << 24))
            .collect();
        let room = (width as usize * 2 + ebp as usize * 2) * rows as usize + 64;
        let mut out = vec![0xAAu8; room];
        // ebp taken from edi after each row: start past the room it may go back
        let start = out.as_mut_ptr() as u32 + (ebp * rows);
        let o = out.as_mut_ptr() as u32;
        check_state(
            &mut t,
            &format!("format {}, {width} x {rows}, ebp {ebp}", c.format),
            call_original,
            Regs {
                esi: pixels.as_ptr() as u32,
                edi: start,
                ecx: width,
                edx: rows,
                ebp,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(o as *mut u8, 0xAA, room) },
            &[(o, room)],
            &|r| convert_rows(r, c),
        );
        drop((pixels, out));
    }
    t.finish()
}

/// A hook for one converter.
macro_rules! converter {
    ($module:ident, $name:literal, $address:expr, $format:expr, $constants:expr, $three_dnow:expr) => {
        pub mod $module {
            use super::*;
            use crate::framework::{RawHook, hook_asm};
            use std::sync::atomic::AtomicUsize;

            const CONVERTER: Converter = Converter {
                format: $format,
                constants: $constants,
                three_dnow: $three_dnow,
            };

            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

            pub static HOOK: RawHook =
                RawHook::new($name, rva($address), thunk, &ORIGINAL).self_test(self_test);

            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL
            );

            extern "C" fn dispatch(r: &mut Regs) {
                dispatch_converter(&HOOK, r, call_original, CONVERTER);
            }

            fn self_test() -> (usize, usize) {
                test_converter(&HOOK, call_original, CONVERTER)
            }
        }
    };
}

converter!(
    rows_565_mmx,
    "gfx::rows_565_mmx",
    0x4070F9,
    1,
    0x4F51F8,
    false
);
converter!(
    rows_555_mmx,
    "gfx::rows_555_mmx",
    0x407171,
    2,
    0x4F5218,
    false
);
converter!(
    rows_1555_mmx,
    "gfx::rows_1555_mmx",
    0x4071ED,
    3,
    0x4F5238,
    false
);
converter!(
    rows_4444_mmx,
    "gfx::rows_4444_mmx",
    0x407276,
    4,
    0x4F5260,
    false
);
converter!(
    rows_565_3dnow,
    "gfx::rows_565_3dnow",
    0x4072FB,
    1,
    0x4F51F8,
    true
);
converter!(
    rows_555_3dnow,
    "gfx::rows_555_3dnow",
    0x407377,
    2,
    0x4F5218,
    true
);
converter!(
    rows_1555_3dnow,
    "gfx::rows_1555_3dnow",
    0x4073F7,
    3,
    0x4F5238,
    true
);
converter!(
    rows_4444_3dnow,
    "gfx::rows_4444_3dnow",
    0x407484,
    4,
    0x4F5260,
    true
);

/// The eight, for `gfx/mod.rs`.
pub static HOOKS: [&dyn crate::framework::GameHook; 8] = [
    &rows_565_mmx::HOOK,
    &rows_555_mmx::HOOK,
    &rows_1555_mmx::HOOK,
    &rows_4444_mmx::HOOK,
    &rows_565_3dnow::HOOK,
    &rows_555_3dnow::HOOK,
    &rows_1555_3dnow::HOOK,
    &rows_4444_3dnow::HOOK,
];
