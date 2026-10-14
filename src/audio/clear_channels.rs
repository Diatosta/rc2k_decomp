//! `audio::clear_channels` (0x40ACA1): the first 0x10 mixer channels made free.
//!
//! In:  nothing
//! Out: every register preserved (the flags of `test eax, eax`, which no caller reads)
//!
//! Each channel's `used`, `voice` and the five dwords from +0x44 on are 0. Five callers
//! (the mode's set-up, `audio::free_voices`, the mixer's start).
//! Compared in shadow mode. Self-test: a redirected copy on channels of the test's.

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::clear_channels", rva(0x40ACA1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn clear_channels(channels: u32) {
    for n in 0..CHANNELS_CLEARED {
        let c = unsafe { channel(channels + n * std::mem::size_of::<MixerChannel>() as u32) };
        c.used = 0;
        c.voice = 0;
        c.gap_44 = [0; 5];
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    let channels = World::game().channels;
    let regions = [(
        channels,
        CHANNELS_CLEARED as usize * std::mem::size_of::<MixerChannel>(),
    )];
    let rust = |_: &Regs| -> Result<Left, Refused> {
        unsafe { clear_channels(channels) };
        Ok(Left::nothing())
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let words = 0x11 * 0x16;
    let mut block = vec![0u32; words];
    let at = block.as_mut_ptr() as u32;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40ACA1,
            len: 0x46,
            entry: &[0x60, 0xBB, 0x10, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x506354, at)],
        1,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    for pattern in [0u32, 0xFFFF_FFFF, 0x1234_5678] {
        let fill = |b: &mut Vec<u32>| {
            for (i, w) in b.iter_mut().enumerate() {
                *w = pattern ^ i as u32;
            }
        };
        fill(&mut block);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = block.clone();
        fill(&mut block);
        unsafe { clear_channels(at) };
        let differ = (0..words).find(|&i| theirs[i] != block[i]);
        t.check(
            differ.is_none() && ro.preserved_except(&Regs::SENTINEL, 0),
            || format!("pattern {pattern:#x}: first difference at word {differ:x?}"),
        );
    }
    t.finish()
}
