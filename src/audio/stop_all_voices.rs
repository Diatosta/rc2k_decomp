//! `audio::stop_all_voices` (0x40AFD9): command 7: every playing voice to the stopping list.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The playing list, if not empty, is put in front of the stopping list (its last voice's
//! `next` the stopping list's head) and emptied, and `g_channelsUsed` is 0. The channels
//! keep their `used` marks: the turn's update frees them as it stops the voices.
//! Refuses a playing list that does not end within 0x200 voices.
//! Compared in shadow mode. Self-test: a redirected copy on lists of the test's, 40 cases.

use crate::audio::voices::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::stop_all_voices", rva(0x40AFD9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `playing`, `stopping`, `used`: where the two lists' heads and the count are.
pub unsafe fn stop_all_voices(playing: u32, stopping: u32, used: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::voices::stop_all`.
    let w = World {
        playing,
        stopping,
        channels_used: used,
        ..World::game()
    };
    unsafe {
        let mut l = lists(&w);
        rc2k_core::audio::voices::stop_all(&mut GamePool::new(&w), &mut l)?;
        store_lists(&w, &l);
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    let w = World::game();
    let regions = [
        (
            game_addr(VOICES_RVA) as u32,
            VOICES as usize * std::mem::size_of::<Voice>(),
        ),
        (w.playing, 8),
        (w.channels_used, 4),
    ];
    let rust = |_: &Regs| -> Result<Left, Refused> {
        unsafe { stop_all_voices(w.playing, w.stopping, w.channels_used) }?;
        Ok(Left::nothing())
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Six voices and the three variables.
    let mut block = vec![0u32; 6 * 0x1C + 3];
    let base = block.as_mut_ptr() as u32;
    let voice_at = |n: u32| base + n * 0x70;
    let vars = base + 6 * 0x70;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40AFD9,
            len: 0x3D,
            entry: &[0x60, 0x8B, 0x0D, 0x40, 0x83, 0x4F, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(0x4F8340, vars), (0x4F8344, vars + 4), (0x506350, vars + 8)],
        5,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let mut case = 0;
    for length in 0..6u32 {
        for stopping in [0u32, 0x1234_5678] {
            for used in [0u32, 3, 0xFFFF_FFFF] {
                let order: Vec<u32> = (0..length).map(|i| (i * 4 + case) % 6).collect();
                let order: Vec<u32> = {
                    let mut seen = Vec::new();
                    for n in order.into_iter().chain(0..6) {
                        if !seen.contains(&n) && seen.len() < length as usize {
                            seen.push(n);
                        }
                    }
                    seen
                };
                case += 1;
                let prepare = |b: &mut Vec<u32>| unsafe {
                    for (i, w) in b.iter_mut().enumerate() {
                        *w = 0xA000_0000 | i as u32;
                    }
                    for n in 0..6 {
                        voice(voice_at(n)).next = 0;
                    }
                    for (i, &n) in order.iter().enumerate() {
                        voice(voice_at(n)).next = order.get(i + 1).map_or(0, |&m| voice_at(m));
                    }
                    write(vars, order.first().map_or(0, |&n| voice_at(n)));
                    write(vars + 4, stopping);
                    write(vars + 8, used);
                };
                prepare(&mut block);
                let mut ro = Regs::SENTINEL;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = block.clone();
                prepare(&mut block);
                let mine = unsafe { stop_all_voices(vars, vars + 4, vars + 8) };
                t.check(
                    mine.is_ok() && theirs == block && ro.preserved_except(&Regs::SENTINEL, 0),
                    || {
                        format!(
                            "voices {order:?}, stopping {stopping:#x}, used {used:#x}: original {:x?}; rust {mine:?} {:x?}",
                            &theirs[6 * 0x1C..],
                            &block[6 * 0x1C..]
                        )
                    },
                );
            }
        }
    }
    t.finish()
}
