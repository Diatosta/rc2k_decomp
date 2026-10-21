//! `race::start_frame_pictures` (0x41F35E): the replay's frames to be saved as pictures
//! (`race::replay_screenshot` does it each frame): the rate `dword_5C2CEF` held to 1 ..
//! 100 (signed); the time a picture (`dword_5C2CF3`) eax, or for eax 0 1000 / the rate;
//! the folder `rpl` made (`sub_403113`: `CreateDirectoryA`, edx the name);
//! `g_replayRecording` 1, `dword_5C2CD4` and `dword_5C2CD0` 0.
//!
//! In:  eax the ms a picture, or 0 for the rate's
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28: eax 0, 0xA0, 0x50, 0x28). Its call
//! through `part` (1 the folder made; in the game `CreateDirectoryA`). Shadow mode runs
//! the original alone (it makes a folder). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version: rates below, inside and past the range, eax 0 or
//! not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{
    PICTURE_RATE, PICTURE_TIME, PICTURES_STARTED, PICTURES_STARTED_2, Part, RECORDING, Saved,
    dispatch_parts, dword, get, put, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::start_frame_pictures",
    rva(0x41F35E),
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

const FOLDER: usize = 0x5C2CD8; // "rpl"

/// The Rust version.
pub fn start_frame_pictures(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer::{picture_rate, picture_time}`.
    use rc2k_core::race::viewer::{picture_rate, picture_time};
    let rate = picture_rate(get(PICTURE_RATE));
    if get(PICTURE_RATE) != rate {
        put(PICTURE_RATE, rate);
    }
    let mut r = *entry;
    r.eax = picture_time(r.eax, rate);
    put(PICTURE_TIME, r.eax);
    r.edx = a(FOLDER);
    part(1, &mut r)?;
    put(RECORDING, 1);
    put(PICTURES_STARTED, 0);
    put(PICTURES_STARTED_2, 0);
    Ok(*entry)
}

/// In the game: the folder made.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateDirectoryA(
            r.edx as *const u8,
            std::ptr::null(),
        )
    };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        start_frame_pictures(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; cmp dword_5C2CEF, 1
    let copy = match crate::race::viewer::copy(
        0x41F35E,
        0x68,
        &[0x60, 0x83, 0x3D, 0xEF, 0x2C, 0x5C, 0x00, 0x01],
        &[0x43],
        &[(0x403113, 1)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let regions = [
        dword(PICTURE_RATE),
        dword(PICTURE_TIME),
        dword(RECORDING),
        dword(PICTURES_STARTED),
        dword(PICTURES_STARTED_2),
    ];
    let saved = Saved::new(&regions);
    for (rate, eax) in [
        (0u32, 0u32),
        (7, 0),
        (100, 0),
        (101, 0),
        (0xFFFF_FFF0, 0),
        (30, 0xA0),
        (30, 0x28),
    ] {
        run_both(
            &mut t,
            &format!("rate {rate:#x}, eax {eax:#x}"),
            copy.addr(),
            Regs {
                eax,
                ..Regs::SENTINEL
            },
            &regions,
            0,
            &|| {
                put(PICTURE_RATE, rate);
                put(PICTURE_TIME, 0xAAAA_AAAA);
                put(RECORDING, 0);
                put(PICTURES_STARTED, 7);
                put(PICTURES_STARTED_2, 7);
            },
            &|r| start_frame_pictures(r, &mut test_part),
        );
    }
    drop(saved);
    t.finish()
}
