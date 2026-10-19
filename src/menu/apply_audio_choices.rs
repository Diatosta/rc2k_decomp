//! `menu::apply_audio_choices` (0x470766): the advanced audio page's work values through
//! `audio::choose_modes` and back (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! An item action of the advanced audio page. Shadow mode runs the original alone (the
//! modes' choice writes the log). Self-test: `option_steps::test_audio`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::option_steps::{apply_audio_choices, audio_part, test_audio};
use crate::menu::parts::dispatch_parted;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::apply_audio_choices", rva(0x470766), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parted(
        &HOOK,
        r,
        call_original,
        &|r, part| unsafe { apply_audio_choices(r, part) },
        audio_part,
    );
}

fn self_test() -> (usize, usize) {
    test_audio(&HOOK)
}
