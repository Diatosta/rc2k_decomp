//! `race::mud_texel` (0x44A549): a texel of the locked mud texture (`g_surface`) by its
//! place on a 256 x 256 square: with `dword_5CEEE0` not 0 the place is halved (x, then y;
//! an odd one skips), and halved again unless it is 1 (a reading: how many times the
//! texture is smaller); then `race::mud_pixel` (0 outside the surface).
//!
//! In:  ecx = x, edx = y (0 .. 0xFF), eax
//! Out: eax = the components, 0 outside, or as it came when the place is skipped; the
//!      rest kept; no flags read (the callers test eax's top bit)
//!
//! See `race/mud.rs`. Callers: `race::muddy_texel`, `race::splash_mud`. Self-test: the
//! original and the Rust version on test surfaces (16 and 32 bits), each halving, places
//! even, odd and outside.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::mud::{TestSurface, has_format, mud_pixel};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mud_texel", rva(0x44A549), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const HALVINGS_RVA: usize = rva(0x5CEEE0); // dword_5CEEE0

/// The texel's place on the texture, or None when it is skipped (an odd place halved).
pub fn texel_place(x: u32, y: u32) -> Option<(u32, u32)> {
    let halvings: u32 = unsafe { game_read(HALVINGS_RVA) };
    let (mut x, mut y) = (x, y);
    let times = match halvings {
        0 => 0,
        1 => 1,
        _ => 2,
    };
    for _ in 0..times {
        if x & 1 != 0 {
            return None;
        }
        x >>= 1;
        if y & 1 != 0 {
            return None;
        }
        y >>= 1;
    }
    Some((x, y))
}

/// The Rust version (see the header): eax as it leaves it.
pub unsafe fn mud_texel(eax: u32, x: u32, y: u32) -> u32 {
    match texel_place(x, y) {
        Some((x, y)) => unsafe { mud_pixel(x, y) },
        None => eax,
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { mud_texel(r.eax, r.ecx, r.edx) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The places the two texel tests use: even, odd in x or y, past 64 x 48 at each halving.
pub const TEST_PLACES: [(u32, u32); 10] = [
    (0, 0),
    (4, 8),
    (5, 8),
    (4, 9),
    (2, 6),
    (62, 46),
    (64, 4),
    (8, 96),
    (0xFC, 0xBC),
    (130, 2),
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !has_format() {
        return t.skip("no pixel format");
    }
    let kept = [at(HALVINGS_RVA)];
    let saved = capture(&kept);
    for bits in [16u32, 32] {
        let surface = TestSurface::new(bits);
        for halvings in [0u32, 1, 2, 3] {
            for (x, y) in TEST_PLACES {
                check_state(
                    &mut t,
                    &format!("{bits} bits, halvings {halvings}, ({x}, {y})"),
                    call_original,
                    Regs {
                        ecx: x,
                        edx: y,
                        eax: 0x8123_4567,
                        ..Regs::SENTINEL
                    },
                    &|| unsafe { game_write(HALVINGS_RVA, halvings) },
                    &[],
                    &rust,
                );
            }
        }
        drop(surface);
    }
    restore(&kept, &saved);
    t.finish()
}
