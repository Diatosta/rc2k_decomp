//! `scene::forget_slot_texture` (0x44EB66): slot eax's loaded texture forgotten
//! (`g_slotLoadedTextures[eax]` -1), so that it is loaded again.
//!
//! In:  eax the slot
//! Out: every register kept
//!
//! 12 callers (among them the marks' snow textures). Refuses a slot past 0x3FF (the table
//! is 0x400 dwords). Self-test: the original and the Rust version on slots 0, 7 and 0x3FF,
//! the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::forget_slot_texture",
    rva(0x44EB66),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LOADED_RVA: usize = rva(0x5DB968); // g_slotLoadedTextures
const SLOTS: u32 = 0x400;

/// The Rust version (see the header).
pub unsafe fn forget_slot_texture(slot: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::scene::textures::forget_slot_texture`.
    let loaded = unsafe { crate::scene::textures::slice(LOADED_RVA, SLOTS) };
    Ok(rc2k_core::scene::textures::forget_slot_texture(
        loaded, slot,
    )?)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { forget_slot_texture(r.eax) }?;
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [(game_addr(LOADED_RVA) as u32 + 4 * r.eax.min(SLOTS - 1), 4)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    let regions = [(game_addr(LOADED_RVA) as u32, 4 * SLOTS as usize)];
    let saved = capture(&regions);
    for slot in [0u32, 7, SLOTS - 1] {
        check_state(
            &mut t,
            &format!("slot {slot:#x}"),
            call_original,
            Regs {
                eax: slot,
                ..Regs::SENTINEL
            },
            &|| unsafe { std::ptr::write_bytes(regions[0].0 as *mut u8, 0x11, regions[0].1) },
            &regions,
            &rust,
        );
    }
    t.check(unsafe { forget_slot_texture(SLOTS) }.is_err(), || {
        "slot 0x400: the Rust version went on".into()
    });
    restore(&regions, &saved);
    t.finish()
}
