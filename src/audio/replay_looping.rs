//! `audio::replay_looping` (0x40A839): the playing voices of kind 2 played again.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `audio/restore.rs`: each voice on `g_playingVoices` whose +0x3C is 2 and that has a
//! buffer: `audio::play_buffer`. One caller, `audio::restore_lost`. Refuses a list without
//! its end. Shadow mode runs the original alone. Self-test: a redirected copy on voices of
//! the test's whose `play_buffer` call goes to a stand-in, 6 lists.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::restore::*;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::replay_looping", rva(0x40A839), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe {
        replay_looping(
            game_addr(crate::audio::voices::PLAYING_RVA) as u32,
            &mut game_part,
        )
    }?;
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
    // [0] the list's head; then four voices of 0x70 bytes.
    let mut memory = vec![0u32; 1 + 4 * 0x1C];
    let at = memory.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40A839,
            len: 0x29,
            entry: &[0x60, 0x8D, 0x1D, 0x40, 0x83, 0x4F, 0x00],
            calls: &[0x19],
            jumps_out: &[],
        },
        &[(0x4F8340, at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let voice = |n: u32| at + 4 + n * 0x70;
    // (the voices in the list, each one's +0x3C and buffer)
    let lists: [&[(u32, u32, u32)]; 6] = [
        &[],
        &[(0, 2, 7)],
        &[(0, 1, 7)],
        &[(0, 2, 0)],
        &[(2, 2, 7), (0, 0, 7), (3, 2, 9), (1, 2, 0)],
        &[(3, 2, 1), (1, 2, 2)],
    ];
    for list in lists {
        let run = |rust: bool| {
            unsafe {
                let m = at as *mut u32;
                for n in 0..memory.len() {
                    m.add(n).write_volatile(0);
                }
                m.write_volatile(list.first().map_or(0, |&(n, _, _)| voice(n)));
                for (i, &(n, kind, buffer)) in list.iter().enumerate() {
                    let v = voice(n) as *mut u32;
                    v.add(0x3C / 4).write_volatile(kind);
                    v.add(2).write_volatile(buffer);
                    let next = list.get(i + 1).map_or(0, |&(n, _, _)| voice(n));
                    v.add(0x48 / 4).write_volatile(next);
                }
            }
            noted();
            let before = Regs::SENTINEL;
            let mut ro = before;
            let done = if rust {
                unsafe { replay_looping(at, &mut test_part) }.is_ok()
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0)
            };
            (done, noted())
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("list {list:?}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(memory);
    t.finish()
}
