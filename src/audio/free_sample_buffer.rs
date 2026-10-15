//! `audio::free_sample_buffer` (0x40B201): the samples' work buffer given back.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `memory::free_list` on `g_sampleBufferMemory`, then `g_sampleBufferSize` and
//! `g_sampleBuffer` 0. One caller, the audio thread's end. See `audio/thread_parts.rs`.
//! Refuses what `free_list` refuses. Shadow mode runs the original alone. Self-test: a
//! redirected copy whose variables are the test's, on lists of 0, 1 and 2 blocks.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::thread_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::free_sample_buffer", rva(0x40B201), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { free_sample_buffer(&SampleBufferVars::game()) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 7]);
    let at = vars.as_mut_ptr() as u32;
    let v = SampleBufferVars {
        kb: at,
        size: at + 4,
        buffer: at + 8,
        list: at + 12,
    };
    let copy = match copy_of(
        &HOOK,
        0x40B201,
        0x1A,
        &[0x60, 0x8D, 0x1D, 0xF0, 0x68, 0x50, 0x00],
        &[0x07],
        &[],
        &[(0x5068F0, v.list), (0x506934, v.size), (0x506930, v.buffer)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let figures = [(game_addr(crate::memory::MOST_HELD_RVA) as u32, 12)];
    let saved = capture(&figures);
    for blocks in 0..3 {
        let run = |rust: bool| {
            unsafe { (at as *mut [u32; 7]).write_volatile([1, 0x400, 0x55, 0, 0, 0, 0]) };
            restore(&figures, &saved);
            for _ in 0..blocks {
                let _ = unsafe { crate::memory::allocate::allocate(v.list as *mut _, 0x100) };
            }
            let before = Regs::SENTINEL;
            let mut ro = before;
            let done = if rust {
                unsafe { free_sample_buffer(&v) }.is_ok()
            } else {
                unsafe { call_game(&mut ro, copy.addr()) };
                ro.preserved_except(&before, 0)
            };
            (
                done,
                unsafe { (at as *const [u32; 7]).read_volatile() },
                capture(&figures),
            )
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("{blocks} blocks: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&figures, &saved);
    drop(vars);
    t.finish()
}
