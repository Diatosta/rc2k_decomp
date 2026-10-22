//! `scene::kind_is_hidden` (0x452303): with `dword_71C3E0` not 0 or `dword_62389C` at 2,
//! whether the object eax is of kind 0x30 or 0x4E (`g_objectKinds`): CF set when it is,
//! clear otherwise (and clear without either setting).
//!
//! In:  eax the object
//! Out: CF; eax its kind when a setting was on, else kept; the rest kept (ZF as the last
//!      compare leaves it, not read: callers `jb`)
//!
//! Callers: `scene::object_hidden_by_kind` (0x4522F0, inlined in
//! `scene::draw_shown_object`), `scene::kind_hidden_at` (0x4522FC).
//! Refuses an object past 0xFF while a setting is on (the original reads past the table).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): both settings, objects 0 .. 0xFF with test kinds.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::kind_is_hidden", rva(0x452303), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SWITCH_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const MODE_RVA: usize = rva(0x62389C); // dword_62389C
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds (0x100 dwords)

/// The Rust version: the object's (hidden, eax left).
pub unsafe fn kind_is_hidden(object: u32) -> Result<(bool, u32), Refused> {
    // Converted: `rc2k_core::scene::kinds::kind_is_hidden`.
    let settings = rc2k_core::scene::kinds::HideSettings {
        switch: unsafe { game_read(SWITCH_RVA) },
        mode: unsafe { game_read(MODE_RVA) },
    };
    Ok(rc2k_core::scene::kinds::kind_is_hidden(
        settings,
        object,
        |n| unsafe { game_read(KINDS_RVA + 4 * n) },
    )?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (hidden, eax) = unsafe { kind_is_hidden(r.eax) }?;
    Ok(Left::carry(hidden).with(EAX, eax))
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
    let all: Vec<Region> = vec![
        (game_addr(SWITCH_RVA) as u32, 4),
        (game_addr(MODE_RVA) as u32, 4),
        (game_addr(KINDS_RVA) as u32, 0x400),
    ];
    let saved = capture(&all);
    for k in 0..0x100usize {
        let kind = [0x30u32, 0x4E, 0, 0x2F, 0x31, 0x4D, 0x4F, 0x130][k % 8];
        unsafe { game_write(KINDS_RVA + 4 * k, kind) };
    }
    for (switch, mode) in [(0u32, 0u32), (0, 2), (1, 0), (5, 2), (0, 3)] {
        for object in (0..0x100u32).step_by(3) {
            let set = || unsafe {
                game_write(SWITCH_RVA, switch);
                game_write(MODE_RVA, mode);
            };
            check_state(
                &mut t,
                &format!("switch {switch}, mode {mode}, object {object:#x}"),
                call_original,
                Regs {
                    eax: object,
                    ..Regs::SENTINEL
                },
                &set,
                &[],
                &rust,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
