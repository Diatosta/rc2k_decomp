//! `drive::load_replay_frame` (0x46020A): the replay's frame `dword_6EF7E0` read
//! (`drive::read_replay_frame_0`), its marks (`drive::replay_frame_marks`), and the frame
//! (0xA04 dwords at `dword_9613A4`) copied to the replay's starting state `unk_7172EC`.
//!
//! In:  nothing
//! Out: eax the frame, ecx 0, esi and edi past the copy; the rest kept; no flags read
//!
//! Caller: `drive::play_replay_frame`.
//! Refuses what `read_replay_frame` refuses.
//! Self-test: the original and the Rust version with `read_replay_frame`'s test replay.

use crate::drive::read_replay_frame::{FRAME, regions, test_regions, test_replay};
use crate::errors::Refused;
use crate::framework::{EAX, ECX, EDI, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::load_replay_frame", rva(0x46020A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SAVED: usize = 0x7172EC;
const DWORDS: usize = 0xA04;

/// The Rust version.
pub unsafe fn load_replay_frame() -> Result<Left, Refused> {
    let frame = g(0x6EF7E0);
    unsafe { crate::drive::read_replay_frame_0::read_replay_frame_0(frame) }?;
    crate::drive::replay_frame_marks::replay_frame_marks();
    unsafe { std::ptr::copy(a(FRAME) as *const u32, a(SAVED) as *mut u32, DWORDS) };
    Ok(Left::nothing()
        .with(EAX, frame)
        .with(ECX, 0)
        .with(ESI, a(FRAME) + 4 * DWORDS as u32)
        .with(EDI, a(SAVED) + 4 * DWORDS as u32))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_replay_frame() }
}

fn all_regions() -> Vec<Region> {
    let mut all = regions();
    all.push((a(SAVED), 4 * DWORDS));
    all
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &all_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = test_regions();
    all.extend([
        (a(SAVED), 4 * DWORDS),
        (a(0x6EF7E0), 4),
        (a(0x963BB4), 0x20),
    ]);
    let saved = capture(&all);
    let Some(file) = test_replay() else {
        restore(&all, &saved);
        return t.skip("no test file in the temp folder");
    };
    for frame in [0u32, 1, 3] {
        let set = || {
            p(0x5BEC88, 1);
            p(0x6EF7E0, frame);
            file.rewind();
        };
        check_state(
            &mut t,
            &format!("frame {frame}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &all_regions(),
            &rust,
        );
    }
    drop(file);
    restore(&all, &saved);
    t.finish()
}
