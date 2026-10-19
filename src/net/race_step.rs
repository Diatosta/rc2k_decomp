//! `net::race_step` (0x464133): a car's step in a network race (nothing without one,
//! `dword_52A910` 0): for the own player's car (`g_currentCar` = `dword_52A920`) its
//! messages sent (`net::send_turn`) and the others' applied (`net::apply_received`); for
//! another's, its glide (`net::follow_remote`).
//!
//! In:  nothing
//! Out: eax `g_currentCar` (when on), or as the calls leave the registers
//!
//! See `net/mod.rs`. One caller, `drive::replay_step` (each car's step, after its
//! physics). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose three variables are the test's and whose calls go to noting stand-ins,
//! against the Rust version: off, the own car, another.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{APPLY_RECEIVED, FOLLOW, ON, PLAYER, Part, SEND_TURN, a, dispatch_net};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::race_step", rva(0x464133), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// g_currentCar.
const CURRENT: usize = 0x71B3F8;

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub on: u32,
    pub current: u32,
    pub player: u32,
}

/// The Rust version on the variables in `v`.
pub fn race_step_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    if rd(v.on) == 0 {
        return Ok(());
    }
    r.eax = rd(v.current);
    if r.eax == rd(v.player) {
        part(SEND_TURN, r)?;
        part(APPLY_RECEIVED, r)
    } else {
        part(FOLLOW, r)
    }
}

/// The Rust version.
pub fn race_step(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let v = Vars {
        on: a(ON),
        current: a(CURRENT),
        player: a(PLAYER),
    };
    race_step_with(r, &v, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, race_step);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 3];
    let base = mem.as_mut_ptr() as u32;
    let v = Vars {
        on: base,
        current: base + 4,
        player: base + 8,
    };
    // cmp dword_52A910, 0
    let entry = [0x83, 0x3D, 0x10, 0xA9, 0x52, 0x00, 0x00];
    let copy = match copy_with(
        0x464133,
        0x28,
        &entry,
        &[0x17, 0x1C, 0x22],
        &[SEND_TURN, APPLY_RECEIVED, FOLLOW],
        &[],
        &[
            (ON, 4, v.on),
            (CURRENT, 4, v.current),
            (PLAYER, 4, v.player),
        ],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for (on, current, player) in [(0u32, 1u32, 1u32), (1, 2, 2), (1, 2, 3), (5, 0, 7)] {
        compare(
            &mut t,
            &format!("on {on}, car {current}, player {player}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (base as *mut [u32; 3]).write_volatile([on, current, player]) },
            &[(base, 12)],
            &|r, part| race_step_with(r, &v, part),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
