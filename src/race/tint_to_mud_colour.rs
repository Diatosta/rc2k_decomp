//! `race::tint_to_mud_colour` (0x44A441): a colour brought toward the mud's
//! (`dword_60AD00` red, `dword_60AD04` green, `dword_60AD08` blue, whole dwords): as
//! `gfx::fade_colour`, each low byte s of ebx becoming s + ((t - s) x ebp >> 8), held to
//! 0 .. 0xFF, with t the dword (blue for the lowest byte); the top byte kept.
//!
//! In:  ebx = the colour, ebp = how far (0x100 all the way)
//! Out: eax = ebx = the colour; esi the third byte before; edx the high dword of the last
//!      `imul`; the rest kept; no flags read
//!
//! See `race/mud.rs`. One caller, `race::muddy_texel`. Self-test: the original and the
//! Rust version on colours, amounts and mud colours by lot and at the edges; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::tint_to_mud_colour", rva(0x44A441), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const MUD_COLOUR_RVA: usize = rva(0x60AD00); // dword_60AD00 .. dword_60AD08: red, green, blue

/// The mud's colour, as its three dwords (red, green, blue).
pub fn mud_colour() -> [u32; 3] {
    unsafe { game_read(MUD_COLOUR_RVA) }
}

/// The Rust version (see the header): the registers it leaves.
pub fn tint_to_mud_colour(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::mud::tint`.
    let t = rc2k_core::race::mud::tint(entry.ebx, mud_colour(), entry.ebp);
    Regs {
        ebx: t.colour,
        eax: t.colour,
        esi: entry.ebx >> 16 & 0xFF,
        edx: t.high,
        ..*entry
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&tint_to_mud_colour(r)))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [(crate::framework::game_addr(MUD_COLOUR_RVA) as u32, 12)];
    let saved = capture(&regions);
    let mut x = 0x9E37_79B9u32;
    let mut next = move || {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x
    };
    for case in 0..32u32 {
        let colour = [
            [0x40u32, 0x80, 0xC0],
            [0, 0xFF, 0x100],
            [0xFFFF_FFFF, 0x7FFF_FFFF, 0x8000_0000],
            [next() & 0xFF, next() & 0xFF, next() & 0xFF],
            [next(), next(), next()],
        ][(case % 5) as usize];
        let ebx = if case == 0 { 0 } else { next() };
        let ebp = [0x100, 0, 0x80, next() & 0x1FF, next()][(case % 5) as usize];
        check_state(
            &mut t,
            &format!("{ebx:#x} by {ebp:#x} toward {colour:x?}"),
            call_original,
            Regs {
                ebx,
                ebp,
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(MUD_COLOUR_RVA, colour) },
            &[],
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
