//! `hud::overlay_states` (0x45A943): the 2D overlay's render states: 0x15 = 4, 0x1B = 1,
//! 0x13 = 5, 0x14 = 6, 0x1C = 0.
//!
//! In:  nothing
//! Out: eax 0, ebx 0x1C; the rest as the state calls leave them
//!
//! Callers: `sub_45A893` (0x45A8A2), code at 0x4559AF, 0x4559C8. The states through
//! `part` (1 with eax the value, ebx the state), in the game `set_render_state`'s Rust
//! version. Shadow mode runs the original alone (it tells the device). Self-test: a copy
//! whose calls go to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::overlay_states", rva(0x45A943), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// A call by place: 1 `set_render_state`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::scene::render_states::OVERLAY`
pub unsafe fn overlay_states(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    for s in rc2k_core::scene::render_states::OVERLAY {
        [g.eax, g.ebx] = [s.value, s.state];
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `set_render_state`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    let (value, state) = (r.eax, r.ebx);
    unsafe { crate::scene::set_render_state::set_render_state(r, value, state) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { overlay_states(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, 4
    let entry: &[u8] = &[0xB8, 0x04, 0x00, 0x00, 0x00];
    let copy = match copy(
        0x45A943,
        0x4C,
        entry,
        &[0xA, 0x19, 0x28, 0x37, 0x46],
        &[(0x44EB72, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    run_both(&mut t, "once", &copy, &[], &|| (), &|r| unsafe {
        overlay_states(r, &mut test_part)
    });
    t.finish()
}
