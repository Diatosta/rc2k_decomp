//! `race::car_part_matrices` (0x460E7C): car esi's marks' switches and its four wheels'
//! matrices (see `race/car_parts.rs`).
//!
//! In:  esi = the car's block
//! Out: the registers as it leaves them (inside its caller's `pusha`)
//!
//! Refuses (before anything) axle values whose scaling would fault. Caller:
//! `race::ready_car_parts`. Shadow mode runs the original alone. Self-test: a copy (its
//! calls to the game's, hooked) against the Rust version on a copy of car 0's block, its
//! state, switches, broken parts and size varied; everything put back (skipped without a
//! race).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::race::car_parts::car_part_matrices;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::car_part_matrices", rva(0x460E7C), thunk, &ORIGINAL).self_test(self_test);

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
    if let Err(e) = unsafe { car_part_matrices(r.esi) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { car_part_matrices(r.esi) }?;
    Ok(())
}

/// Everything the function writes but the block.
pub fn written() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x5E9F8C, 0x538),
        g(0x6417E0, 0x100),
        g(0x641760, 0x40),
        g(0x608C48, 0x18),
        g(0x635038, 0x4C),
        g(0x63501C, 0xC),
        g(0x624148, 0x18),
        at(rva(0x62400C)),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let first: u32 = unsafe { game_read(rva(0x71B3B8)) };
    if first == 0 {
        return t.skip("no race loaded");
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x460E7C,
            len: 0x7ED,
            entry: &[0xD9, 0x86, 0xD8, 0x01, 0x00, 0x00, 0xD9, 0x9E],
            calls: &[
                0x58E, 0x5BD, 0x5C8, 0x5DF, 0x5EA, 0x601, 0x619, 0x630, 0x64C, 0x657, 0x66E, 0x679,
                0x690, 0x6A8, 0x6BF, 0x6E5, 0x6F0, 0x707, 0x712, 0x729, 0x741, 0x758, 0x774, 0x77F,
                0x796, 0x7A1, 0x7B8, 0x7D0, 0x7E7,
            ],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let mut block = vec![0u8; 0x1000];
    let b = block.as_mut_ptr() as u32;
    let mut regions = written();
    regions.push((b, 0x1000));
    let saved = capture(&regions);
    for (on, switches, broken, size) in [
        (1u32, 0x3E0u32, 0u32, 2.5f32),
        (0xFF, 0x200, 1, -1.0),
        (0, 0x3E0, 0, 9.0),
        (0x0F, 0, 1, f32::NAN),
    ] {
        let prepare = || unsafe {
            restore(&regions, &saved);
            std::ptr::copy(first as *const u8, b as *mut u8, 0xFE0);
            let set = |o: u32, v: u32| ((b + o) as *mut u32).write_unaligned(v);
            set(0x84, on);
            set(0x298, switches);
            set(0x1D8, size.to_bits());
            for o in [0x410u32, 0x418, 0x420] {
                set(o, broken);
            }
        };
        prepare();
        let mut ro = Regs {
            esi: b,
            ..Regs::SENTINEL
        };
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&regions);
        prepare();
        let done = unsafe { car_part_matrices(b) };
        let mine = capture(&regions);
        t.check(done.is_ok() && theirs == mine, || {
            let at = theirs.iter().zip(&mine).position(|(a, b)| a != b);
            format!("on {on:#x} switches {switches:#x} broken {broken} size {size}: first difference at {at:x?}")
        });
    }
    restore(&regions, &saved);
    drop(block);
    t.finish()
}
