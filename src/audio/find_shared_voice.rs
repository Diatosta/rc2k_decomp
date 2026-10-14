//! `audio::find_shared_voice` (0x40B027): a voice already made from a record's file.
//!
//! In:  ebx = the sample record
//! Out: eax = the first of the `g_voiceCount` voices whose `data_key` is the record's, or 0;
//!      everything else preserved
//!
//! One caller, command 3 (`sub_40AEB9`): such a voice's data is shared, not read again.
//! Compared in shadow mode. Self-test: a redirected copy on voices of the test's, 30 cases.

use crate::audio::samples::find_shared_voice;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::find_shared_voice", rva(0x40B027), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let record = unsafe { *(r.ebx as *const SampleRecord) };
    let count: u32 = unsafe { game_read(VOICE_COUNT_RVA) };
    let found =
        unsafe { find_shared_voice(&record, game_addr(VOICES_RVA) as u32, count.min(VOICES)) };
    Ok(Left::nothing().with(EAX, found))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut voices = vec![Voice::default(); 6];
    let mut count = Box::new(0u32);
    let count_at = &mut *count as *mut u32 as u32;
    let base = voices.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40B027,
            len: 0x25,
            entry: &[0x51, 0x52, 0x8B, 0x53, 0x0C],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x4F8348, count_at), (0x4F834C, base)],
        2,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for (n, v) in voices.iter_mut().enumerate() {
        v.data_key = [0x10, 0x20, 0, 0x20, 0x30, 0x10][n];
    }
    for c in 0..6u32 {
        for key in [0u32, 0x10, 0x20, 0x30, 0x40] {
            *count = c;
            let record = SampleRecord {
                data_key: key,
                ..SampleRecord::default()
            };
            let before = Regs {
                ebx: &record as *const SampleRecord as u32,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = unsafe { find_shared_voice(&record, base, c) };
            t.check(ro.eax == mine && ro.preserved_except(&before, EAX), || {
                format!(
                    "{c} voices, key {key:#x}: original {:#x}; rust {mine:#x}",
                    ro.eax
                )
            });
        }
    }
    t.finish()
}
