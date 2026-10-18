//! `screens::stage_choice_texts` (0x469434): the stage choice page's two item values and its stage line (see `screens/page_settings.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a page's unread list (0x75932E). Refuses (before anything) no stage records or no item table.
//! Shadow mode: by state. Self-test: the page's choices as they are; all put back (skipped without stage records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::screens::harness::Left;
use crate::screens::page_settings::*;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::stage_choice_texts",
    rva(0x469434),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { stage_choice_texts() }?;
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = vec![g(0xA8BD54, 0x200)];
    v.push((game_addr(rva(0x74DB68)) as u32 + 4 + 0x14, 4));
    let pointed: u32 = unsafe { game_read(rva(0x74DB68)) };
    if pointed != 0 {
        v.push((pointed + 4 + 0x5C + 0x14, 4));
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
    if crate::screens::draw_menu_item_text::text_unavailable().is_some() {
        return t.skip("no strings");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(rva(0x7558C8))]);
    let saved = capture(&kept);
    for case in [0u32] {
        restore(&kept, &saved);
        if unsafe { stage_choice_texts_refuses() } {
            continue;
        }
        check_state(
            &mut t,
            &format!("case {case:?}"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&kept, &saved),
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
