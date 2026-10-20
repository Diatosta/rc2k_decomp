//! `gfx::fade_colour` (0x44A4C5): a colour brought toward another: each of the three low
//! bytes s of ebx becomes s + ((t - s) x ebp >> 8) (`imul`, `sar`), held to 0 .. 0xFF, t
//! the byte of ecx in the same place; the top byte kept.
//!
//! In:  ebx = the colour, ecx = the colour faded to, ebp = how far (0x100 all the way)
//! Out: eax = ebx = the colour; ecx kept (turned twice); esi the third byte of the colour
//!      before; edx the high dword of the last `imul`; no flags read
//!
//! Callers: 0x425A23, `race::splash_mud` (0x44A403), 0x44CB15. Self-test: the original
//! and the Rust version on colours and amounts by lot and at the edges.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::fade_colour", rva(0x44A4C5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// One byte's step: (the byte, the high dword of the `imul`). Converted:
/// `rc2k_core::pixels::fade_step`.
use rc2k_core::pixels::fade_step as step;

/// The three low bytes of `from` brought toward `to` (their places' values, whole: the
/// mud's tint gives dwords), `amount` of 0x100; the registers it leaves on `entry`.
pub fn fade_bytes(entry: &Regs, from: u32, to: [u32; 3], amount: u32) -> Regs {
    let mut r = *entry;
    let [b0, b1, b2, b3] = from.to_le_bytes();
    let (r0, _) = step(b0.into(), to[0], amount);
    let (r1, _) = step(b1.into(), to[1], amount);
    let (r2, high) = step(b2.into(), to[2], amount);
    r.ebx = u32::from_le_bytes([r0 as u8, r1 as u8, r2 as u8, b3]);
    r.eax = r.ebx;
    r.esi = b2.into();
    r.edx = high;
    r
}

/// The Rust version (see the header): the registers it leaves.
pub fn fade_colour(entry: &Regs) -> Regs {
    let [t0, t1, t2, _] = entry.ecx.to_le_bytes();
    fade_bytes(
        entry,
        entry.ebx,
        [t0.into(), t1.into(), t2.into()],
        entry.ebp,
    )
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&fade_colour(r)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cases: Vec<(u32, u32, u32)> = vec![
        (0, 0xFF_FFFF, 0x100),
        (0xFF_FFFF, 0, 0x100),
        (0x1234_5678, 0x8765_4321, 0x80),
        (0xFF00_00FF, 0x00FF_FF00, 0),
        (0x0080_8080, 0x0080_8080, 0x7FFF_FFFF),
        (0x0010_2030, 0x00F0_E0D0, 0xFFFF_FFFF),
        (0x00FF_0000, 0x0000_00FF, 0x8000_0000),
    ];
    let mut x = 0x2545_F491u32;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    };
    for k in 0..24usize {
        let amount = [next() & 0x1FF, next(), next() & 0xFF][k % 3];
        cases.push((next(), next(), amount));
    }
    for (from, to, amount) in cases {
        check_state(
            &mut t,
            &format!("{from:#x} toward {to:#x} by {amount:#x}"),
            call_original,
            Regs {
                ebx: from,
                ecx: to,
                ebp: amount,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    t.finish()
}
