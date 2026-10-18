//! `screens::replay_range_caption` (0x470501): the page's caption (`g_captionText`) string
//! `dword_75A2A6[dword_755240]` and its NUL (see `screens/best_scores.rs`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! In `menu::save_replay_loop`'s and `menu::load_replay_loop`'s lists, and called at 0x46D435,
//! 0x46D454. Shadow mode: by state (the caption). Self-test: page numbers, the caption put
//! back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::best_scores::{CAPTION_RVA, replay_range_caption};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::replay_range_caption",
    rva(0x470501),
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
    unsafe { replay_range_caption() };
    Ok(Left::nothing())
}

fn regions() -> [Region; 1] {
    [(game_addr(CAPTION_RVA) as u32, 0x100)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut kept = regions.to_vec();
    kept.push(at(rva(0x755248)));
    let saved = capture(&kept);
    for page in [0u32, 1, 4, 0x0CCC_CCCC] {
        check_state(
            &mut t,
            &format!("page {page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x755248), page);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
