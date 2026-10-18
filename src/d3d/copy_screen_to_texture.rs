//! `d3d::copy_screen_to_texture` (0x455D38): while `dword_6415E0` is set (the view's
//! mirror picture wanted, from its block's +0x6F0), the back surface (`g_backSurface`)
//! copied into the mirror picture's system texture (`dword_624234`, its record's surface
//! +0x14) by DirectDraw's `Blt` without rectangles (stretched to the texture).
//!
//! In:  nothing
//! Out: ebx the record's kind's address and eax .. edx as `Blt` leaves them on the copy's
//!      way; no flags read (its caller goes on with a `mov`)
//!
//! Caller: `race::draw_race_scene` (one view). Platform: the original always runs. In the
//! standalone program the screen is copied into the texture's memory of ours
//! (`platform_3d::copy_screen_to_system_texture`). No self-test.

use crate::framework::{RawHook, Regs, game_read, hook_asm, rva, standalone};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "d3d::copy_screen_to_texture",
    rva(0x455D38),
    thunk,
    &ORIGINAL,
);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const WANTED_RVA: usize = rva(0x6415E0); // dword_6415E0
#[cfg(feature = "standalone")]
const PICTURE_RVA: usize = rva(0x624234); // dword_624234: the mirror picture's texture

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if standalone() {
        if unsafe { game_read::<u32>(WANTED_RVA) } != 0 {
            #[cfg(feature = "standalone")]
            crate::platform_3d::copy_screen_to_system_texture(unsafe { game_read(PICTURE_RVA) });
        }
        return;
    }
    unsafe { call_original(r) };
}
