//! `math::turn_scratch_xz` (0x41FEF8): the scratch point's x and z (floats at
//! `dword_5C5F50` and `dword_5C5F4C`) turned by an angle.
//!
//! In:  edx = the angle (a dword's turn is 2^32: `g_sines`/`g_cosines` entry (edx +
//!      0x80000) >> 20)
//! Out: edx = the entry; the rest preserved
//!
//! x' = x cos + z sin, z' = z cos - x sin, on the x87 as the original (`math::x87::
//! turn_in_place`). Six callers, `math::turn_point_xz` and its three siblings. Self-test:
//! the original and the Rust version on the scratch point, put back afterwards.

use crate::errors::Refused;
use crate::framework::{
    EDX, GameHook, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87_run::{DOUBLE, TEST_CONTROL_WORDS, with_control_word};
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, Tolerance, check_state_floats, dispatch_state_floats, note_control_word,
};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("math::turn_scratch_xz", rva(0x41FEF8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const Z_RVA: usize = rva(0x5C5F4C); // dword_5C5F4C
pub const X_RVA: usize = rva(0x5C5F50); // dword_5C5F50
pub const Y_RVA: usize = rva(0x5C5F54); // dword_5C5F54
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines

/// `math::turn_scratch_xz` exactly as the original (the x87, at whatever precision it is
/// set to): what the Rust callers in modules not yet converted use, so that their own
/// comparisons stay exact. The entry left in edx.
pub unsafe fn turn_scratch_xz(angle: u32) -> u32 {
    let entry = angle.wrapping_add(0x8_0000) >> 20;
    unsafe {
        x87_turn(entry);
    }
    entry
}

unsafe fn x87_turn(entry: u32) {
    let a = |r: usize| game_addr(r + 4 * entry as usize);
    unsafe {
        crate::math::x87::turn_in_place(
            a(SINES_RVA) as *const u32,
            a(COSINES_RVA) as *const u32,
            game_addr(X_RVA) as *mut u32,
            game_addr(Z_RVA) as *mut u32,
        )
    };
}

/// The converted version (`rc2k_core::geometry`, plain floats): the hook's, and what
/// converted callers use. The entry left in edx.
pub unsafe fn turn_scratch_xz_plain(angle: u32) -> u32 {
    use rc2k_core::geometry;
    let entry = geometry::turn_entry(angle);
    unsafe {
        let float = |rva: usize| f32::from_bits(game_read(rva));
        let (x, z) = geometry::turn_xz(
            float(X_RVA),
            float(Z_RVA),
            float(SINES_RVA + 4 * entry as usize),
            float(COSINES_RVA + 4 * entry as usize),
        );
        game_write(X_RVA, x.to_bits());
        game_write(Z_RVA, z.to_bits());
    }
    entry
}

pub fn scratch() -> Region {
    (game_addr(Z_RVA) as u32, 12)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EDX, unsafe { turn_scratch_xz_plain(r.edx) }))
}

/// What a test case at `word` compares within tolerances: nothing at double precision,
/// the game's.
pub fn point_floats<'a>(
    word: u16,
    memory: &'a [(Region, Tolerance)],
    regs: &'a [(u8, Tolerance)],
) -> Floats<'a> {
    if word == DOUBLE {
        Floats::EXACT
    } else {
        Floats { memory, regs }
    }
}

/// The scratch point's floats.
pub fn scratch_floats() -> [(Region, Tolerance); 1] {
    [(scratch(), FLOAT)]
}

extern "C" fn dispatch(r: &mut Regs) {
    note_control_word(HOOK.name());
    let floats = scratch_floats();
    let floats = Floats {
        memory: &floats,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[scratch()], &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [scratch()];
    let saved = capture(&regions);
    let tolerant = scratch_floats();
    for word in TEST_CONTROL_WORDS {
        // At double precision the same bits; at the others within the tolerance.
        let floats = if word == DOUBLE {
            Floats::EXACT
        } else {
            Floats {
                memory: &tolerant,
                regs: &[],
            }
        };
        with_control_word(word, &mut || {
            for (x, z, angle) in [
                (1.0f32, 0.0f32, 0u32),
                (123.456, -78.9, 0x4000_0000),
                (-5000.25, 3.5, 0x1234_5678),
                (0.1, 0.7, 0xFFF8_0000),
                (1e9, -1e-9, 0x8007_FFFF),
                (f32::MAX, 2.0, 0x3000_0000),
            ] {
                check_state_floats(
                    &mut t,
                    HOOK.name(),
                    &format!("control word {word:#x}, x {x}, z {z}, angle {angle:#x}"),
                    call_original,
                    Regs {
                        edx: angle,
                        ..Regs::SENTINEL
                    },
                    &|| unsafe {
                        game_write(Z_RVA, [z.to_bits(), x.to_bits(), 0x5555_5555]);
                    },
                    &regions,
                    &floats,
                    &rust,
                );
            }
        });
    }
    restore(&regions, &saved);
    t.finish()
}
