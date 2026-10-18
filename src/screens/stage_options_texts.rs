//! `screens::stage_options_texts` (0x4692A9): the stage options page's item texts, its stage line and caption (see `screens/page_settings.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In a page's unread list (0x75930A). Refuses (before anything) no stage records.
//! Shadow mode: by state. Self-test: the two switches each way; all put back (skipped without stage records).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::screens::page_settings::*;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::stage_options_texts",
    rva(0x4692A9),
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
    unsafe { stage_options_texts() }?;
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let mut v = vec![g(0xA8BD54, 0x200)];
    for n in 1..4 {
        v.push((game_addr(rva(0x74DCE0)) as u32 + 4 + 0x5C * n + 0x14, 4));
    }
    v.push(g(0xA8BE54, 0x100));
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
    kept.extend([at(rva(0x7556F8)), at(rva(0x7556FC))]);
    let saved = capture(&kept);
    for case in [(0u32, 0u32), (1, 1), (1, 0), (0, 1)] {
        unsafe {
            restore(&kept, &saved);
            game_write(rva(0x7556FC), case.1);
            if case.0 == 1 {
                game_write(rva(0x7556F8), 1u32);
            }
        }
        if unsafe { stage_options_texts_refuses() } {
            continue;
        }
        check_state(
            &mut t,
            &format!("case {case:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x7556FC), case.1);
                if case.0 == 1 {
                    game_write(rva(0x7556F8), 1u32);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
