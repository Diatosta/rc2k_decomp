//! `scene::set_texture` (0x44E77E): `d3d::set_texture` with eax, unless `dword_5CDA8C` is 0.
//!
//! In:  eax the texture's number
//! Out: as `d3d::set_texture` leaves the registers (ecx 0 ..), or all kept
//!
//! 21 callers, none reading flags. `d3d::set_texture` is the platform's: called through
//! its hook (`call_game`, section 15). Shadow mode runs the original alone (it tells the
//! device). Self-test: a copy whose call goes to a stand-in noting the registers, against
//! the Rust version on the same stand-in, the switch 0 and 1.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_texture", rva(0x44E77E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SWITCH_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const LAYER_RVA: usize = rva(0x41ACF5); // d3d::set_texture

/// The Rust version (see the header); `part` makes the layer's call.
pub unsafe fn set_texture(r: &mut Regs, part: &mut dyn FnMut(&mut Regs)) {
    if unsafe { game_read::<u32>(SWITCH_RVA) } != 0 {
        part(r);
    }
}

/// In the game: the layer through its hook.
pub fn game_part(r: &mut Regs) {
    crate::calls::call(LAYER_RVA, r);
}

/// `set_texture` with the game's layer, for Rust callers.
pub unsafe fn set_texture_in_game(r: &mut Regs) {
    unsafe { set_texture(r, &mut game_part) };
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { set_texture_in_game(r) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_texture_in_game(r) };
    Ok(())
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // cmp dword_5CDA8C, 0 (seven bytes: all the patch cuts)
    let entry: &[u8] = &[0x83, 0x3D, 0x8C, 0xDA, 0x5C, 0x00, 0x00];
    let copy = match copy(0x44E77E, 0xF, entry, &[0x9], &[(0x41ACF5, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let vars = [at(SWITCH_RVA)];
    let saved = capture(&vars);
    for on in [0u32, 1, 0x8000_0000] {
        run_both(
            &mut t,
            &format!("switch {on:#x}"),
            &copy,
            &vars,
            &|| unsafe { game_write(SWITCH_RVA, on) },
            &|r| {
                let mut g = *r;
                unsafe {
                    set_texture(&mut g, &mut |r| {
                        let _ = test_part(1, r);
                    })
                };
                Ok(g)
            },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
