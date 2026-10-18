//! `drive::read_replay_frame_0` (0x41EBA1): `drive::read_replay_frame` of frame eax at
//! tenth 0 (ebx kept round it).
//!
//! In:  eax the frame
//! Out: CF as `read_replay_frame` leaves it; every register kept
//!
//! Callers: `drive::load_replay_frame`, `sub_41DE81` (six).
//! Self-test: as `read_replay_frame`'s, with its test replay.

use crate::drive::read_replay_frame::{regions, test_regions, test_replay};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::p;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "drive::read_replay_frame_0",
    rva(0x41EBA1),
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

/// The Rust version: CF.
pub unsafe fn read_replay_frame_0(frame: u32) -> Result<bool, Refused> {
    unsafe { crate::drive::read_replay_frame::read_replay_frame(frame, 0) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { read_replay_frame_0(r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = test_regions();
    let saved = capture(&all);
    let Some(file) = test_replay() else {
        restore(&all, &saved);
        return t.skip("no test file in the temp folder");
    };
    for frame in [0u32, 1, 5] {
        let set = || {
            p(0x5BEC88, 1);
            file.rewind();
        };
        check_state(
            &mut t,
            &format!("frame {frame}"),
            call_original,
            Regs {
                eax: frame,
                ..Regs::SENTINEL
            },
            &set,
            &regions(),
            &rust,
        );
    }
    drop(file);
    restore(&all, &saved);
    t.finish()
}
