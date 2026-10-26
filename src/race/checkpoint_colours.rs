//! `race::checkpoint_colours` (0x4548E3): from the current checkpoint's (`dword_71A070`)
//! face (`g_checkpointFaces`, 0x40-byte records at `g_courseFaces`): its +0x10 byte's low
//! and high nibbles (0 read as 0x10) to `dword_614D08` / `dword_614D0C`, `dword_614D10` =
//! `dword_9ED808[checkpoint]` - `dword_71A998`, and `dword_63F17C` the low nibble (twice
//! the distance below 0x40000000, signed), the high (from 0xC0000000), else the larger.
//!
//! In:  nothing
//! Out: eax that, ebx the checkpoint, esi the face (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4548A8` (0x4548D8), each view drawn. A reading:
//! colours (fog levels?) blended by the distance to the checkpoint. The tables are read
//! unchecked, as the original. Shadow mode: by state. Self-test: the variables and table
//! entries set (the face list pointed at a test record), distances around the bounds; all
//! put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::checkpoint_colours", rva(0x4548E3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CHECKPOINT: usize = rva(0x71A070); // dword_71A070
const FACE_LIST: usize = rva(0x9E3738); // g_checkpointFaces
const FACES: usize = rva(0x5CCCBC); // g_courseFaces (a pointer)
const DISTANCES: usize = rva(0x9ED808); // dword_9ED808
const NOW: usize = rva(0x71A998); // dword_71A998

fn regions() -> Vec<Region> {
    [0x614D08usize, 0x614D0C, 0x614D10, 0x63F17C]
        .iter()
        .map(|&a| at(rva(a)))
        .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { checkpoint_colours() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut faces = vec![0u8; 0x40 * 4];
    let base = faces.as_mut_ptr() as u32;
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        at(CHECKPOINT),
        at(FACES),
        at(NOW),
        at(FACE_LIST + 4 * 2),
        at(DISTANCES + 4 * 2),
    ]);
    let saved = capture(&kept);
    for (byte, distance) in [
        (0x35u8, 0i32),
        (0x00, 0x1FFF_FFFF),
        (0x53, 0x2000_0000),
        (0xF1, -0x2000_0000),
        (0x1F, 0x4000_0000),
        (0x80, 0x6000_0000),
        (0x08, i32::MIN),
        (0x91, -0x7000_0000),
    ] {
        check_state(
            &mut t,
            &format!("byte {byte:#x} distance {distance:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, len);
                }
                std::ptr::write_bytes(base as *mut u8, 0x5A, 0x100);
                ((base + 0x40 + 0x10) as *mut u8).write(byte);
                game_write(CHECKPOINT, 2u32);
                game_write(FACE_LIST + 4 * 2, 1u32);
                game_write(FACES, base);
                game_write(NOW, 1000u32);
                game_write(DISTANCES + 4 * 2, (distance as u32).wrapping_add(1000));
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(faces);
    t.finish()
}
