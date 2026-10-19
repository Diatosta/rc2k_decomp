//! `physics::cue_listed_car_sound_last` (0x45D1D3): `physics::cue_listed_car_sound`'s twin
//! for the pace notes: eax the current car's entry in `dword_641680`; unless -1,
//! `physics::cue_car_sound_last` with it.
//!
//! In:  ebx the cue
//! Out: eax the entry; the rest kept; no flags read
//!
//! Caller: `physics::queue_pace_note` (inside `pusha` / `popa`).
//! Refuses (before anything) a current car past 7.
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version: cars 0 ..
//! 7 listed and not.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::sound_cues::{CUE_LAST, Part, both, copy_of, dispatch_cue};
use crate::service::harness::{capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::cue_listed_car_sound_last",
    rva(0x45D1D3),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

/// The steps on `r`.
pub fn cue_listed_car_sound_last_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let car = g(0x71B3F8);
    if car > 7 {
        return Err(BadInput("a current car past 7").into());
    }
    r.eax = g(0x641680 + 4 * car as usize);
    if r.eax != u32::MAX {
        part(CUE_LAST, r)?;
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_cue(&HOOK, r, call_original, cue_listed_car_sound_last_in);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x45D1D3, 0x17, &[0xA1, 0xF8, 0xB3, 0x71, 0x00], &[0x11]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [(a(0x71B3F8), 4), (a(0x641680), 0x20)];
    let saved = capture(&all);
    for car in 0..8u32 {
        let fill = || unsafe {
            game_write(rva(0x71B3F8), car);
            for k in 0..8usize {
                game_write(
                    rva(0x641680 + 4 * k),
                    if (k as u32 + car).is_multiple_of(3) {
                        u32::MAX
                    } else {
                        k as u32 % 2
                    },
                );
            }
        };
        both(
            &mut t,
            &format!("car {car}"),
            &copy,
            Regs {
                ebx: 0x52,
                ..Regs::SENTINEL
            },
            &fill,
            &all,
            cue_listed_car_sound_last_in,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
