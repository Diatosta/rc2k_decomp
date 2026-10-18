//! `sound::commands_pending` (0x409C01): whether commands still wait for the audio
//! thread; without audio (`g_noAudio` not 0) the count is cleared first.
//!
//! In:  nothing
//! Out: zero flag set = none waiting (`cmp g_soundCommandCount, 0`), carry clear; every
//!      register preserved
//! One caller, `sound::wait_for_commands` (`jz`), whose Rust version uses
//! `sound::queue::pending` itself.
//! Shadow mode runs the original alone (the count is the audio thread's as well).
//! Self-test: a copy whose two variables are the test's, against `queue::pending` on
//! the same.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{Queue, pending};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::commands_pending", rva(0x409C01), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x1D;
const ENTRY: [u8; 7] = [0x83, 0x3D, 0xB4, 0x82, 0x4F, 0x00, 0x00];
/// The operands naming `g_noAudio` and (twice) the count.
const NO_AUDIO_AT: usize = 2;
const COUNT_AT: [usize; 2] = [0xB, 0x13];

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let waiting = unsafe { pending(&Queue::game()) };
    r.set_zero(!waiting);
    r.set_carry(false);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy =
        unsafe { RedirectedCopy::new(game_addr(rva(0x409C01)), LEN, &ENTRY, &[], &|_| None) };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut vars = [0u32; 4];
    let v = vars.as_mut_ptr();
    unsafe {
        copy.patch_u32(NO_AUDIO_AT, v as u32);
        for at in COUNT_AT {
            copy.patch_u32(at, v.add(1) as u32);
        }
    }
    let queue = Queue {
        no_audio: unsafe { v.add(2) },
        count: unsafe { v.add(3) },
        write: std::ptr::null_mut(),
        records: std::ptr::null_mut(),
    };
    for (no_audio, count) in [(0u32, 0u32), (0, 3), (1, 0), (1, 5), (0xFF, 0x400)] {
        let mut o = Regs::SENTINEL;
        let (theirs, mine) = unsafe {
            v.write_volatile(no_audio);
            v.add(1).write_volatile(count);
            call_game(&mut o, copy.addr());
            v.add(2).write_volatile(no_audio);
            v.add(3).write_volatile(count);
            let waiting = pending(&queue);
            (
                (o.eflags & 0x40 != 0, v.add(1).read_volatile()),
                (!waiting, v.add(3).read_volatile()),
            )
        };
        t.check(
            theirs == mine && !o.carry() && o.preserved_except(&Regs::SENTINEL, 0),
            || format!("no audio {no_audio}, count {count}: original {theirs:?}, rust {mine:?}"),
        );
    }
    t.finish()
}
