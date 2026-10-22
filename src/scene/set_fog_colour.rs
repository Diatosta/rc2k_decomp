//! `scene::set_fog_colour` (0x44E76B): the fog's colour: eax kept in `dword_5AB964` and
//! given to Direct3D (`d3d_set_fog_colour`, 0x41AE25: inside `pusha`, render state 0x22 =
//! eax with its top byte set, through `d3d::set_render_state`).
//!
//! In:  eax = the colour
//! Out: every register kept; no flags read
//!
//! Callers: `race::set_up_fog` and ten more. The layer's call is made here (the two lines
//! of `d3d_set_fog_colour`), `d3d::set_render_state` itself through the game.
//! Self-test: the original and the Rust version recorded on the layer, nothing drawn;
//! the colour kept compared.

use crate::d3d::recorder::record;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_fog_colour", rva(0x44E76B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KEPT_RVA: usize = rva(0x5AB964); // dword_5AB964
const LAYER_RVA: usize = rva(0x41AB5B); // d3d::set_render_state

/// The Rust version (see the header).
// Converted: `rc2k_core::scene::render_states::fog_colour`
pub unsafe fn set_fog_colour(colour: u32) {
    unsafe {
        game_write(KEPT_RVA, colour);
        let s = rc2k_core::scene::render_states::fog_colour(colour);
        let mut r = Regs {
            eax: s.value,
            ebx: s.state,
            ..Regs::default()
        };
        crate::calls::call(LAYER_RVA, &mut r);
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    unsafe { set_fog_colour(r.eax) };
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { set_fog_colour(r.eax) };
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = [at(KEPT_RVA)];
    let saved = capture(&kept);
    for colour in [0u32, 0x0080_6040, 0xFF12_3456, 0x00FF_FFFF] {
        let before = Regs {
            eax: colour,
            ..Regs::SENTINEL
        };
        let ((theirs, their_kept), their_calls) = record(true, || {
            let mut r = before;
            unsafe { call_original(&mut r) };
            (r, unsafe { game_read::<u32>(KEPT_RVA) })
        });
        unsafe { game_write(KEPT_RVA, 0xAAAA_AAAAu32) };
        let (my_kept, my_calls) = record(true, || unsafe {
            set_fog_colour(colour);
            game_read::<u32>(KEPT_RVA)
        });
        t.check(
            theirs.preserved_except(&before, 0) && their_calls == my_calls && their_kept == my_kept,
            || format!("colour {colour:#x}: calls {their_calls:x?} / {my_calls:x?}, kept {their_kept:#x} / {my_kept:#x}"),
        );
    }
    restore(&kept, &saved);
    t.finish()
}
