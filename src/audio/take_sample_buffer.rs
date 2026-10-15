//! `audio::take_sample_buffer` (0x40B1D0): the work buffer samples are read into.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `g_sampleBufferSize` = `g_sampleBufferKb` x 0x400 (0: 0x400), then `memory::allocate`
//! on `g_sampleBufferMemory` into `g_sampleBuffer` (its failure leaves for a fatal stub at
//! 0x4D86D6; the Rust `allocate` raises "Need more virtual memory!" itself). One caller,
//! the audio thread's start-up. See `audio/thread_parts.rs`. Shadow mode runs the original
//! alone. Self-test: a redirected copy whose variables are the test's, 4 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::thread_parts::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::take_sample_buffer", rva(0x40B1D0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe { take_sample_buffer(&SampleBufferVars::game()) }?;
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
    // [0] KB, [1] size, [2] buffer, [3..7] the list; used through `at` only.
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
        0x40B1D0,
        0x31,
        &[
            0x60, 0x69, 0x05, 0xEC, 0x68, 0x50, 0x00, 0x00, 0x04, 0x00, 0x00,
        ],
        &[0x1F],
        &[0x24],
        &[
            (0x5068EC, v.kb),
            (0x506934, v.size),
            (0x506930, v.buffer),
            (0x5068F0, v.list),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let figures = [(game_addr(crate::memory::MOST_HELD_RVA) as u32, 12)];
    let saved = capture(&figures);
    for kb in [0u32, 1, 4, 0x40] {
        let run = |rust: bool| {
            unsafe { (at as *mut [u32; 7]).write_volatile([kb, 0x55, 0x55, 0, 0, 0, 0]) };
            restore(&figures, &saved);
            let before = Regs::SENTINEL;
            let mut ro = before;
            let done = if rust {
                unsafe { take_sample_buffer(&v) }.is_ok()
            } else {
                unsafe { call_game(&mut ro, copy.addr()) };
                ro.preserved_except(&before, 0)
            };
            let after = unsafe { (at as *const [u32; 7]).read_volatile() };
            let seen = (done, after[1], after[2] != 0, after[4], capture(&figures));
            let _ = unsafe { crate::memory::free_list::free_list(v.list as *mut _) };
            seen
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("KB {kb}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&figures, &saved);
    drop(vars);
    t.finish()
}
