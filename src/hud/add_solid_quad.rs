//! `hud::add_solid_quad` (0x4B7216): the quad record's texture places all on one texel
//! (0xF8 << 16, 8 << 16: a solid colour), then `hud::add_quad`.
//!
//! In:  nothing (the record's corners and colour set by the caller)
//! Out: as `add_quad` leaves them
//!
//! Callers: the HUD's drawers (code at 0x4B27E1, 0x4B3BF8, 0x4B5AD0, ...). The quad through
//! `part` (1), in the game its Rust version. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::hud::add_quad::QUAD_RVA;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::add_solid_quad", rva(0x4B7216), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `add_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::SOLID_PLACES`
pub unsafe fn add_solid_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    for (n, v) in rc2k_core::hud::quads::SOLID_PLACES {
        unsafe { game_write(QUAD_RVA + 4 * n, v) };
    }
    let mut g = *entry;
    part(1, &mut g)?;
    Ok(g)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { add_solid_quad(r, &mut crate::hud::add_triangle::game_part_quad) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { add_solid_quad(r, &mut crate::hud::add_triangle::game_part_quad) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov dword_75BFC8, 0F80000h
    let entry: &[u8] = &[0xC7, 0x05, 0xC8, 0xBF, 0x75, 0x00, 0x00, 0x00, 0xF8, 0x00];
    let copy = match copy(0x4B7216, 0x56, entry, &[0x50], &[(0x4B72D1, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![(game_addr(QUAD_RVA) as u32, 0x38)];
    let saved = capture(&regions);
    run_both(
        &mut t,
        "once",
        &copy,
        &regions,
        &|| unsafe { std::ptr::write_bytes(game_addr(QUAD_RVA) as *mut u8, 0xAA, 0x38) },
        &|r| unsafe { add_solid_quad(r, &mut test_part) },
    );
    restore(&regions, &saved);
    t.finish()
}
