//! `menu::stage_range_caption` (0x4A1FBB): "first-last" into `g_captionText`: the first
//! (`dword_7556B8`) and it plus 9 (a reading: the stages shown on a page).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event pages' code (`sub_4A1F7A`, ..). `strings::append_u32`'s Rust
//! version; the minus and the NUL written in place. Self-test: the original and the Rust
//! version, firsts by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::stage_range_caption", rva(0x4A1FBB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FIRST_RVA: usize = rva(0x7556B8); // dword_7556B8
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn stage_range_caption(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(FIRST_RVA) };
    // Converted: `rc2k_core::menu::event::stage_range`.
    let mut text = rc2k_core::menu::event::stage_range(first);
    text.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(CAPTION_RVA) as *mut u8, text.len())
    };
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    vec![(game_addr(CAPTION_RVA) as u32, 0x20)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { stage_range_caption(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.push(at(FIRST_RVA));
    let saved = capture(&vars);
    for first in [0u32, 1, 9, 91, 4_294_967_290] {
        check_state(
            &mut t,
            &format!("first {first}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                std::ptr::write_bytes(game_addr(CAPTION_RVA) as *mut u8, 0xAA, 0x20);
                game_write(FIRST_RVA, first);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
