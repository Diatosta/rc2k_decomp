//! `race::stop_frame_pictures` (0x41F3C6): the replay's frames no longer saved as pictures
//! (`g_replayRecording` and `dword_5C2CD4` 0; see `race::replay_screenshot`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28). Shadow mode compares. Self-test:
//! the two filled, against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    PICTURES_STARTED, RECORDING, Saved, check_original, dispatch_regs, dword, put,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::stop_frame_pictures", rva(0x41F3C6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version.
pub fn stop_frame_pictures() {
    put(RECORDING, 0);
    put(PICTURES_STARTED, 0);
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    stop_frame_pictures();
    Ok(*r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = [dword(RECORDING), dword(PICTURES_STARTED)];
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [dword(RECORDING), dword(PICTURES_STARTED)];
    let saved = Saved::new(&regions);
    check_original(
        &mut t,
        "recording",
        call_original,
        Regs::SENTINEL,
        &regions,
        0,
        &|| {
            put(RECORDING, 1);
            put(PICTURES_STARTED, 0xAAAA_AAAA);
        },
        &rust,
    );
    drop(saved);
    t.finish()
}
