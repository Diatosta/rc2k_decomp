//! `gfx::list_secondary_modes` (0x404401): with a 3D secondary card (`dword_5AB4B0` 1,
//! `gfx::find_3d_cards`' "secondary card"), its screen modes listed into the modes' table
//! and marked for Direct3D; DirectDraw closed in any case.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! With the card: `sub_4048EC` with eax its GUID (`unk_5AB4B4`), `sub_404A31` with eax
//! 0x2000, `gfx::mark_3d_modes` with eax 0x2000. Then `sub_404935` (eax 0x2000, or the
//! entry's without the card). The calls are as in `gfx::list_primary_modes`, whose parts
//! these are (DirectDraw through the game; nothing in the standalone program). One caller,
//! `start` (0x40109D). Shadow mode runs the original alone. Self-test: a copy whose
//! variables are the test's and whose four calls go to stand-ins noting eax, against the
//! Rust version, both switches.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::gfx::list_primary_modes::{CardVars, Part, call, compare, game_part, get};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::list_secondary_modes", rva(0x404401), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const SECONDARY: u32 = 0x2000;

// ---- Rust version ----

pub unsafe fn list_secondary_modes(entry: &Regs, v: &CardVars, part: Part) -> Result<(), Refused> {
    let mut eax = entry.eax;
    if unsafe { get(v.secondary_3d) } == 1 {
        call(entry, 0, v.secondary_guid, part)?;
        call(entry, 1, SECONDARY, part)?;
        call(entry, 2, SECONDARY, part)?;
        eax = SECONDARY;
    }
    call(entry, 3, eax, part)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { list_secondary_modes(&entry, &CardVars::game(), &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    compare(
        &mut t,
        &HOOK,
        (
            0x404401,
            0x2B,
            &[0x60, 0x83, 0x3D, 0xB0, 0xB4, 0x5A, 0x00, 0x01],
            &[0x10, 0x1A, 0x1F, 0x24],
            2,
        ),
        list_secondary_modes,
    );
    t.finish()
}
