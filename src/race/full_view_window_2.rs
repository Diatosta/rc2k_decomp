//! `race::full_view_window_2` (0x456004): the view window the whole screen, `scene::set_view_window`,
//! `dword_63993C` 0 (see `race/frame_views.rs`).
//!
//! In:  nothing
//! Out: as `scene::set_view_window` leaves them (eax the height before it)
//!
//! Caller: `sub_45429B`'s frame (0x454874). Shadow mode: by state. Self-test: the game's
//! screen sizes and two test sizes; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_draw::all_regs;
use crate::race::frame_views::full_view_window;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::full_view_window_2", rva(0x456004), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { full_view_window(r, false) }))
}

fn regions() -> Vec<Region> {
    let mut v = crate::scene::set_view_window::regions().to_vec();
    for a in [
        0x62408Cusize,
        0x624090,
        0x623FB0,
        0x623FB4,
        0x624080,
        0x6365C4,
        0x63993C,
    ] {
        v.push(at(rva(a)));
    }
    v
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(rva(0x4F494C)), at(rva(0x4F4950))]);
    kept.extend(crate::scene::set_view_window::inputs());
    let saved = capture(&kept);
    for size in [None, Some((640u32, 480u32)), Some((1024, 768))] {
        check_state(
            &mut t,
            &format!("size {size:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                if let Some((x, y)) = size {
                    game_write(rva(0x4F494C), x);
                    game_write(rva(0x4F4950), y);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
