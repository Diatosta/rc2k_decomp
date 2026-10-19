//! `race::play_replay_range` (0x41D846): the viewer plays from the block read: its
//! cameras into the block (`drive::replay_frame_marks`), the race's sounds started unless
//! the frames are being saved as pictures (`race::start_race_sounds`, not with
//! `g_replayRecording` 1), the replay played (`race::play_movie`), the sounds reset
//! (`race::reset_race_sounds`), the key that ended it (`input::take_key`) into
//! `dword_5C2550`, `dword_5C2554` 1 (keys held), the block shown held to the last
//! (`dword_5BECD0` at most, unsigned; then frame 0), `dword_5BEC98` 1 (stopped).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::play_marked`. Its calls through `part`: 1 `replay_frame_marks`, 2
//! `start_race_sounds`, 3 `play_movie`, 4 `reset_race_sounds`, 5 `input::take_key`; in the
//! game their Rust versions. Ungated (it plays a replay). Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins, against the Rust version:
//! recording or not, the block before, at and past the last.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    BLOCK, BLOCKS, FRAME, KEY_AFTER_PLAY, Part, RECORDING, STOPPED, Saved, WAIT_RELEASE, answer,
    dispatch_parts, dword, get, put, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::play_replay_range", rva(0x41D846), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version: the registers it leaves (the entry's).
pub fn play_replay_range(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(1, &mut r)?;
    if get(RECORDING) != 1 {
        part(2, &mut r)?;
    }
    part(3, &mut r)?;
    part(4, &mut r)?;
    part(5, &mut r)?;
    r.eax &= 0xFF;
    put(KEY_AFTER_PLAY, r.eax);
    put(WAIT_RELEASE, 1);
    if get(BLOCKS) <= get(BLOCK) {
        put(BLOCK, get(BLOCKS));
        put(FRAME, 0);
    }
    put(STOPPED, 1);
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::race_end;
    unsafe {
        match place {
            1 => crate::drive::replay_frame_marks::replay_frame_marks(),
            2 => crate::race::sounds::start_race_sounds(r, &mut crate::race::sounds::game_part)?,
            3 => {
                *r =
                    crate::race::play_movie::play_movie(r, &mut crate::race::play_movie::game_part)?
            }
            4 => {
                race_end::reset_race_sounds(
                    r,
                    &race_end::SoundVars::game(),
                    &mut race_end::sound_part,
                )?;
            }
            _ => crate::race::viewer_load_stage::take_key(r),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        play_replay_range(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; call drive::replay_frame_marks
    let copy = match crate::race::viewer::copy(
        0x41D846,
        0x5D,
        &[0x60, 0xE8, 0x57, 0x00, 0x00, 0x00],
        &[0x1, 0xF, 0x14, 0x19, 0x1E],
        &[
            (0x41D8A3, 1),
            (0x40D592, 2),
            (0x460776, 3),
            (0x40D378, 4),
            (0x407F95, 5),
        ],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [
        dword(KEY_AFTER_PLAY),
        dword(WAIT_RELEASE),
        dword(BLOCK),
        dword(FRAME),
        dword(STOPPED),
        dword(BLOCKS),
        dword(RECORDING),
    ];
    let saved = Saved::new(&regions);
    for (recording, block, blocks) in [(0u32, 3u32, 9u32), (1, 9, 9), (0, 12, 9)] {
        run_both(
            &mut t,
            &format!("recording {recording}, block {block} of {blocks}"),
            copy.addr(),
            Regs::SENTINEL,
            &regions,
            0,
            &|| {
                put(RECORDING, recording);
                put(BLOCK, block);
                put(BLOCKS, blocks);
                put(FRAME, 0x33);
                put(STOPPED, 0);
                put(WAIT_RELEASE, 0);
                put(KEY_AFTER_PLAY, 0xAAAA_AAAA);
                answer(Box::new(|place, r| {
                    if place == 5 {
                        r.eax = r.eax & !0xFF | 0x1C;
                    }
                }));
            },
            &|r| play_replay_range(r, &mut test_part),
        );
    }
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
