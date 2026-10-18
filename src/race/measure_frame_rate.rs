//! `race::measure_frame_rate` (0x42009E): every 16th call (`dword_613838` counting, back to 0
//! at 0x10) the time-stamp counter (`rdtsc`) into `dword_6214E8`; with the cycles since the
//! last time (`dword_6214EC`, into `dword_62400C`) above 0 (signed), the frame rate
//! `flt_6214F0` = the CPU's clock `dword_5183E8` x `flt_5C5D18` x `flt_5C5D18` x
//! `flt_5C5D08` / the cycles (x87, as the original); the time kept for the next.
//!
//! In:  nothing
//! Out: unchanged but on the 16th: eax and edx the counter, ebx the cycles; no flags read
//!
//! Caller: `sub_4547AE` (0x4547FD); `flt_6214F0` is shown by `race::draw_frame_rate`.
//! The counter through `tsc` (in the game `rdtsc`). Self-test: a copy of the original
//! (its own `rdtsc`), then the Rust version given the value the copy read; counts before,
//! at and past the 16th, cycles above and below 0, under the three precisions.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{base, x87_run};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::measure_frame_rate", rva(0x42009E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COUNT_RVA: usize = rva(0x613838); // dword_613838
const NOW_RVA: usize = rva(0x6214E8); // dword_6214E8
const LAST_RVA: usize = rva(0x6214EC); // dword_6214EC
const RATE_RVA: usize = rva(0x6214F0); // flt_6214F0
const CYCLES_RVA: usize = rva(0x62400C); // dword_62400C, a scratch

/// `fild clock; fmul k; fmul k; fmul k2; fild cycles; fdivp; fstp rate`.
fn frame_rate() {
    x87_run!(
        "fild dword ptr [{b} + 0x1183E8]",
        "fmul dword ptr [{b} + 0x1C5D18]",
        "fmul dword ptr [{b} + 0x1C5D18]",
        "fmul dword ptr [{b} + 0x1C5D08]",
        "fild dword ptr [{b} + 0x22400C]",
        "fdivp st(1), st",
        "fstp dword ptr [{b} + 0x2214F0]";
        b = base()
    );
}

/// The Rust version (see the header): the registers it leaves; `tsc` gives the counter's
/// low and high halves.
pub unsafe fn measure_frame_rate(entry: &Regs, tsc: &mut dyn FnMut() -> (u32, u32)) -> Regs {
    let mut r = *entry;
    unsafe {
        let count = game_read::<u32>(COUNT_RVA).wrapping_add(1);
        game_write(COUNT_RVA, count);
        if count != 0x10 {
            return r;
        }
        game_write(COUNT_RVA, 0u32);
        (r.eax, r.edx) = tsc();
        game_write(NOW_RVA, r.eax);
        r.ebx = r.eax.wrapping_sub(game_read(LAST_RVA));
        if r.ebx as i32 > 0 {
            game_write(CYCLES_RVA, r.ebx);
            frame_rate();
        }
        r.eax = game_read(NOW_RVA);
        game_write(LAST_RVA, r.eax);
    }
    r
}

/// The time-stamp counter, as `rdtsc` gives it.
pub fn rdtsc() -> (u32, u32) {
    let t = unsafe { core::arch::x86::_rdtsc() };
    (t as u32, (t >> 32) as u32)
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    // Shadow mode runs the original alone: two reads of the counter never agree.
    if crate::framework::shadow() {
        unsafe { call_original(r) };
        return;
    }
    *r = Regs {
        esp: r.esp,
        ..unsafe { measure_frame_rate(r, &mut rdtsc) }
    };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // inc dword_613838
    let copy = match crate::race::mud::copy(
        0x42009E,
        0x68,
        &[0xFF, 0x05, 0x38, 0x38, 0x61, 0x00],
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 5] = [
        at(COUNT_RVA),
        at(NOW_RVA),
        at(LAST_RVA),
        at(RATE_RVA),
        at(CYCLES_RVA),
    ];
    let inputs = [at(rva(0x5183E8))];
    let saved = capture(&[&regions[..], &inputs[..]].concat());
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for (count, ahead, clock) in [
                (0u32, false, 450u32),
                (0xE, false, 450),
                (0xF, false, 450),
                (0xF, true, 1000),
                (0xF, false, 0),
                (0x10, false, 300),
                (0xFFFF_FFFF, false, 300),
            ] {
                let prepare = || unsafe {
                    game_write(COUNT_RVA, count);
                    game_write(RATE_RVA, 0x3F80_0000u32);
                    game_write(rva(0x5183E8), clock);
                    // The last time a little before now, or ahead of it (cycles below 0).
                    let now = rdtsc().0;
                    let last = if ahead {
                        now.wrapping_add(0x4000_0000)
                    } else {
                        now.wrapping_sub(0x0100_0000)
                    };
                    game_write(LAST_RVA, last);
                };
                prepare();
                let last: u32 = unsafe { game_read(LAST_RVA) };
                let mut ro = Regs::SENTINEL;
                unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
                let theirs = (regs(&ro), capture(&regions));
                let read = (unsafe { game_read::<u32>(NOW_RVA) }, ro.edx);
                prepare();
                unsafe { game_write(LAST_RVA, last) };
                let mine = unsafe { measure_frame_rate(&Regs::SENTINEL, &mut || read) };
                let mine = (regs(&mine), capture(&regions));
                t.check(theirs == mine, || {
                    format!(
                        "control word {word:#x}, count {count:#x}, last {last:#x}, clock {clock}: original {theirs:x?}; rust {mine:x?}"
                    )
                });
            }
        });
    }
    restore(&[&regions[..], &inputs[..]].concat(), &saved);
    t.finish()
}
