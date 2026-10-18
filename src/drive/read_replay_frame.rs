//! `drive::read_replay_frame` (0x41EBAB): a replay's frame eax (with the tenth ebx) read,
//! while a replay plays from its file (`dword_5BEC88` 1; else CF). Frames past 0x708 and
//! tenths past 100 are the fatal errors 0x80A61151 and 0x80A61152 (raised as the
//! original). At or past the last frame (`dword_5BECD0`): the last, `dword_5BEC90` 1, the
//! tenth 0. Else the tenth into `dword_5C2534`, `dword_5BEC90` 0, the frame into
//! `dword_5C2530`, the file (`h`) put at the frame's place (`lDistanceToMove[frame]`,
//! `files::seek`) and its bytes (`dword_5BECDC[frame]`, packed when below 0) read into
//! `dword_9613A4` (`drive::read_packed`). All in `pusha` / `popa`.
//!
//! In:  eax the frame, ebx the tenth
//! Out: CF set when not playing or the seek or read failed; every register kept
//!
//! Callers: `drive::read_replay_frame_0`, `sub_41DE81` (four), `sub_41E224`.
//! Refuses what `read_packed` refuses.
//! Self-test: the original and the Rust version with a test replay file (the temp
//! folder; `h` and the frame tables pointed at it, saved and put back): not playing,
//! frames plain and packed, the last frame.

use crate::drive::read_packed::TestFile;
use crate::errors::{FatalError, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, g, p};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::read_replay_frame", rva(0x41EBAB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// `dword_9613A4`: the frame read.
pub const FRAME: usize = 0x9613A4;
const FILE: usize = 0x5BEC8C; // h
const PLACES: usize = 0x5BECD8; // lDistanceToMove: (place, count) a frame

/// The Rust version: CF.
pub unsafe fn read_replay_frame(frame: u32, tenth: u32) -> Result<bool, Refused> {
    if g(0x5BEC88) != 1 {
        return Ok(true);
    }
    if frame > 0x708 {
        return Err(FatalError {
            code: 0x80A6_1151,
            message: "",
        }
        .into());
    }
    if tenth > 0x64 {
        return Err(FatalError {
            code: 0x80A6_1152,
            message: "",
        }
        .into());
    }
    if frame >= g(0x5BECD0) {
        p(0x5BEC90, 1);
        p(0x5C2530, g(0x5BECD0));
        p(0x5C2534, 0);
        return Ok(false);
    }
    p(0x5C2534, tenth);
    p(0x5BEC90, 0);
    p(0x5C2530, frame);
    let place = g(PLACES + 8 * frame as usize);
    if unsafe { crate::files::handle::seek(g(FILE), place as i32, 0) }.is_none() {
        return Ok(true);
    }
    let count = g(PLACES + 4 + 8 * frame as usize);
    let (_, carry, _) =
        unsafe { crate::drive::read_packed::read_packed(g(FILE), a(FRAME), count) }?;
    Ok(carry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { read_replay_frame(r.eax, r.ebx) }?))
}

/// What it writes.
pub fn regions() -> Vec<Region> {
    vec![
        (a(FRAME), 0x2810),
        (a(crate::drive::read_packed::PACKED), 0x400),
        (a(0x5BEC90), 4),
        (a(0x5C2530), 8),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// A test replay: two frames, plain and packed, at known places; `h`, the tables, the
/// count and the playing switch set (saved and put back by the caller).
pub fn test_replay() -> Option<TestFile> {
    let mut bytes: Vec<u8> = (0..0x20).map(|x| x as u8).collect();
    bytes.extend([0xFE, 0x11, 0x22, 0x03, 0x99, 0]);
    let file = TestFile::new("rc2k_replay_frame.tmp", &bytes)?;
    p(FILE, file.handle);
    p(PLACES, 0);
    p(PLACES + 4, 0x20);
    p(PLACES + 8, 0x20);
    p(PLACES + 12, (-6i32) as u32);
    p(0x5BECD0, 2);
    p(0x5BEC88, 1);
    Some(file)
}

/// What the tests set.
pub fn test_regions() -> Vec<Region> {
    let mut all = regions();
    all.extend([
        (a(FILE), 4),
        (a(PLACES), 0x18),
        (a(0x5BECD0), 4),
        (a(0x5BEC88), 4),
    ]);
    all
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
    for (frame, tenth, playing) in [
        (0u32, 5u32, 0u32),
        (0, 5, 1),
        (1, 0x64, 1),
        (2, 3, 1),
        (7, 9, 1),
    ] {
        let set = || {
            p(0x5BEC88, playing);
            file.rewind();
        };
        check_state(
            &mut t,
            &format!("frame {frame}, playing {playing}"),
            call_original,
            Regs {
                eax: frame,
                ebx: tenth,
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
