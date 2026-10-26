//! `physics::pace_notes_step` (0x4417EC): the work car's pace notes, each physics step.
//! Only with `unk_71A080`, `unk_71A07C`, `unk_71A078`, `g_holdCarRevs`, `dword_71C3E0`,
//! `unk_719FF4`, `unk_71A22C` all 0, `unk_71A234` 1 and `unk_719FEC` 1000 or more
//! (unsigned). Then the checkpoints ahead: `g_carSpeed` x `flt_614C78` / `flt_5C5D50`
//! (`fistp`) into `dword_62400C`; ebx the car's checkpoint (`unk_71A06C`) plus that when
//! not negative, 0 when at or past the count (by the sign); past the last called
//! (`unk_71A074`, unsigned): more than 16 past moves it up to ebx first, then every
//! checkpoint after it up to ebx (at least one) has its group called
//! (`physics::queue_pace_note`, edx the checkpoint, inside `pusha` / `popa`), and
//! `unk_71A074` = ebx. Then (unless it left at the start) `physics::expire_car_events`.
//!
//! In:  nothing
//! Out: the start's gates closed: every register kept; else ebx and edx as above (edx not
//!      set when nothing was called) and what `expire_car_events` leaves; no flags read
//!
//! Caller: `physics::car_physics_step`, which calls the Rust version directly. Its calls
//! by `physics/queue_pace_note.rs`' places (5 the group, 6 the expiry).
//! Refuses what its calls refuse. Shadow mode runs the original alone (the calls queue
//! sounds and car events). Self-test: a copy whose two calls go to stand-ins that note
//! them, against the Rust version with the same, on the game's variables (put back): each
//! gate, speeds forward and back, checkpoints about the count and the last called.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::queue_pace_note::{EXPIRE, PACE, Part, both, copy_of, dispatch_pace};
use crate::service::harness::{capture, restore};
use rc2k_core::physics::pace_calls::{call_target, calling, calls};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::pace_notes_step", rva(0x4417EC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The gates: (variable, the value it must have); the last is "1000 or more".
const ZERO: [usize; 7] = [
    0x71A080, 0x71A07C, 0x71A078, 0x5BEC94, 0x71C3E0, 0x719FF4, 0x71A22C,
];
const ONE: usize = 0x71A234;
const TIME: usize = 0x719FEC;
const SPEED: usize = 0x71A184; // g_carSpeed
const SCALE: usize = 0x614C78; // flt_614C78
const DIVISOR: usize = 0x5C5D50; // flt_5C5D50
const AHEAD: usize = 0x62400C; // dword_62400C
const CHECKPOINT: usize = 0x71A06C;
const LAST_CALLED: usize = 0x71A074;
const COUNT: usize = 0x623640; // g_checkpointCount
const ONE_F32: u32 = 0x3F80_0000;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::pace_calls::{calling, call_target, calls}`.
pub fn pace_notes_step_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if !calling(ZERO.map(g), g(ONE), g(TIME)) {
        return Ok(());
    }
    // `fld; fmul; fdiv; fistp` (the extra division by 1.0 is exact).
    let ahead = crate::math::x87::mul_div_div_to_int(g(SPEED), g(SCALE), g(DIVISOR), ONE_F32);
    p(AHEAD, ahead as u32);
    r.ebx = call_target(g(CHECKPOINT), ahead, g(COUNT));
    if let Some((last, gap)) = calls(r.ebx, g(LAST_CALLED)) {
        r.eax = gap;
        p(LAST_CALLED, last);
        r.edx = last;
        loop {
            r.edx = r.edx.wrapping_add(1);
            let mut c = *r;
            part(PACE, &mut c)?;
            if r.edx >= r.ebx {
                break;
            }
        }
        p(LAST_CALLED, r.ebx);
    }
    part(EXPIRE, r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_pace(&HOOK, r, call_original, pace_notes_step_in);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x4417EC,
        0xC7,
        &[0x83, 0x3D, 0x80, 0xA0, 0x71, 0x00, 0x00],
        &[0xB1, 0xC1],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| (game_addr(rva(x)) as u32, 4);
    let mut all: Vec<_> = ZERO.iter().map(|&v| a(v)).collect();
    all.extend(
        [ONE, TIME, SPEED, AHEAD, CHECKPOINT, LAST_CALLED, COUNT]
            .iter()
            .map(|&v| a(v)),
    );
    let saved = capture(&all);
    let f = |x: f32| x.to_bits();
    // (closed gate, speed, checkpoint, last called, count)
    let mut cases = vec![];
    for gate in 0..10u32 {
        cases.push((gate, f(40.0), 10u32, 8u32, 50u32));
    }
    for speed in [0.0f32, 4.0, 12.0, 20.0, 63.9, -9.0, 200.0] {
        for (cp, last, count) in [
            (10u32, 8u32, 50u32),
            (10, 10, 50),
            (10, 30, 50),
            (48, 47, 50),
            (49, 2, 50),
            (0, 0xFFFF_FFF0, 50),
            (30, 2, 50),
        ] {
            cases.push((10, f(speed), cp, last, count));
        }
    }
    for (gate, speed, cp, last, count) in cases {
        let fill = || {
            restore(&all, &saved);
            for (n, &v) in ZERO.iter().enumerate() {
                p(v, (gate == n as u32) as u32);
            }
            p(ONE, if gate == 7 { 2 } else { 1 });
            p(
                TIME,
                if gate == 8 {
                    999
                } else if gate == 9 {
                    1000
                } else {
                    5000
                },
            );
            p(SPEED, speed);
            p(CHECKPOINT, cp);
            p(LAST_CALLED, last);
            p(COUNT, count);
            p(AHEAD, 0xAAAA_AAAA);
        };
        both(
            &mut t,
            &format!("gate {gate}, speed {speed:#x}, checkpoint {cp}, last {last:#x}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            pace_notes_step_in,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
