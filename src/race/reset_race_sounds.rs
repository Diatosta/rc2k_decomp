//! `race::reset_race_sounds` (0x40D378): the race's sounds reset: commands waited for,
//! command 7 (and its end of turn), waited again, sounds held back, the race clock
//! started; the eight race sound objects' 32 fields, eleven variables, the 0x48 voices'
//! first and sixth dwords, the EAX environment 1.0; command 2 (eax 3); `dword_5124E4` 0.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/race_end.rs`. Callers: `race::stop_race_sounds`, `sub_40D34A`, `sub_40D366`,
//! `sub_41CDC2`, `sub_41D85F`. Refuses, before anything, a sound object missing (the
//! original writes through 0). Shadow mode runs the original alone (it sends commands).
//! Self-test: a copy whose five sound calls note themselves and whose object table and
//! voices are the test's (the audio thread reads the game's), against the Rust version on
//! the same; the plain variables put back.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::race_end::*;
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_race_sounds", rva(0x40D378), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { reset_race_sounds(r, &SoundVars::game(), &mut sound_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { reset_race_sounds(r, &SoundVars::game(), &mut sound_part) }?;
    Ok(())
}

/// The sound calls by place.
const CALLEES: [usize; 5] = [0x409C1E, 0x409B8C, 0x409B6B, 0x40F1E9, 0x409B81];

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let objects: Vec<Vec<u32>> = (0..8)
        .map(|n| vec![0x5A5A_0000 + n; OBJECT_LEN / 4])
        .collect();
    let table: Vec<u32> = objects.iter().map(|o| o.as_ptr() as u32).collect();
    let voices = vec![0x1234_5678u32; VOICES_LEN / 4];
    let v = SoundVars {
        objects: table.as_ptr() as u32,
        voices: voices.as_ptr() as u32,
    };
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x40D378,
            len: 0x21A,
            entry: &[0x60, 0xE8, 0xA0, 0xC8, 0xFF, 0xFF],
            calls: &[0x1, 0x6, 0xB, 0x10, 0x15, 0x209],
            jumps_out: &[],
        },
        &|c| {
            CALLEES
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| crate::install::stand_ins::stand_in(n as u32 + 1))
        },
        &[(SOUND_OBJECTS_IDA, v.objects), (VOICES_IDA, v.voices)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(crate::race::level_init::note);
    let mut regions: Vec<Region> = sound_regions();
    let saved = capture(&regions);
    regions.extend(objects.iter().map(|o| (o.as_ptr() as u32, OBJECT_LEN)));
    regions.push((v.voices, VOICES_LEN));
    for fill in [0u8, 0xA5] {
        crate::race::mud::run_both(
            &mut t,
            &format!("filled {fill:#x}"),
            &copy,
            &regions,
            &|| {
                for &(a, len) in &regions {
                    unsafe { std::ptr::write_bytes(a as *mut u8, fill, len) };
                }
            },
            &|r| unsafe { reset_race_sounds(r, &v, &mut crate::race::mud::test_part) },
        );
    }
    restore(&sound_regions(), &saved);
    drop((objects, table, voices));
    t.finish()
}
