//! `physics::off_line_warnings` (0x44148F): unless finished (+0xA8) or +0xA4 set, two
//! counters with their next cue times: +0x270 (`unk_71A248`) counts the steps the car is
//! far from the checkpoints (`g_nearestCheckpointDistance` not below `flt_614C6C`, or
//! +0x98 four or more from +0x94), else 0 with its time +0x278 300; +0x274 counts the
//! steps its heading is `dword_614C74` or more (unsigned) from the checkpoint's
//! (`dword_9ED808[+0x98]`), else 0 with +0x27C 300. Then, unless `dword_5BEC94` is set,
//! each counter at or past its time (by the sign) moves the time on 3000 and cues a sound
//! (`physics::cue_listed_car_sound`, ebx 0x50 turned, 0x52 off the line).
//!
//! In:  nothing
//! Out: eax, ebx and what the cue leaves; no flags read
//!
//! Caller: `sub_44157B`.
//! Refuses (before anything) a checkpoint (+0x98) past 0x1418.
//! Shadow mode runs the original alone (`physics/sound_cues.rs`).
//! Self-test: a copy whose cue goes to a stand-in, run on the game's variables (saved and
//! put back), against the Rust version: distances, checkpoints, headings, counters about
//! their times.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::physics::sound_cues::{LISTED, Part, both, copy_of, dispatch_cue};
use crate::service::harness::{capture, restore};
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::off_line_warnings",
    rva(0x44148F),
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

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The steps on `r`.
pub fn off_line_warnings_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    if g(0x71A080) != 0 || g(0x71A07C) != 0 {
        return Ok(());
    }
    let cp = g(0x71A070);
    if cp > 0x1418 {
        return Err(BadInput("a checkpoint past 0x1418").into());
    }
    let near = !matches!(
        f32::from_bits(g(0x614C68)).partial_cmp(&f32::from_bits(g(0x614C6C))),
        Some(Ordering::Greater | Ordering::Equal)
    );
    let mut off = true;
    if near {
        r.eax = cp.wrapping_sub(g(0x71A06C));
        if (r.eax as i32) < 0 {
            r.eax = r.eax.wrapping_neg();
        }
        off = (r.eax.wrapping_sub(4) as i32) >= 0;
    }
    if off {
        p(0x71A248, g(0x71A248).wrapping_add(1));
    } else {
        p(0x71A248, 0);
        p(0x71A250, 300);
    }
    r.ecx = cp;
    r.ebx = g(0x9ED808 + 4 * cp as usize);
    r.eax = g(0x71A998).wrapping_sub(r.ebx);
    if (r.eax as i32) < 0 {
        r.eax = r.eax.wrapping_neg();
    }
    if r.eax < g(0x614C74) {
        p(0x71A24C, 0);
        p(0x71A254, 300);
    } else {
        p(0x71A24C, g(0x71A24C).wrapping_add(1));
    }
    if g(0x5BEC94) != 0 {
        return Ok(());
    }
    for (count, time, cue) in [(0x71A24C, 0x71A254, 0x50), (0x71A248, 0x71A250, 0x52)] {
        r.eax = g(count);
        if (r.eax.wrapping_sub(g(time)) as i32) >= 0 {
            p(time, g(time).wrapping_add(3000));
            r.ebx = cue;
            part(LISTED, r)?;
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_cue(&HOOK, r, call_original, off_line_warnings_in);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(
        0x44148F,
        0xEC,
        &[0x83, 0x3D, 0x80, 0xA0, 0x71, 0x00, 0x00],
        &[0xC5, 0xE6],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [
        (a(0x71A060), 0x30),
        (a(0x71A248), 0x18),
        (a(0x71A998), 4),
        (a(0x614C68), 0x10),
        (a(0x9ED808), 0x40),
        (a(0x5BEC94), 4),
    ];
    let saved = capture(&all);
    for seed in 0..16u32 {
        let fill = || {
            p(0x71A080, (seed == 15) as u32);
            p(0x71A07C, 0);
            p(0x71A070, seed % 8);
            p(0x71A06C, (seed * 3) % 8);
            p(0x614C68, (seed as f32 * 10.0).to_bits());
            p(0x614C6C, 50.0f32.to_bits());
            p(0x614C74, 0x1000_0000);
            for k in 0..16usize {
                p(0x9ED808 + 4 * k, (k as u32).wrapping_mul(0x0800_0000));
            }
            p(0x71A998, seed.wrapping_mul(0x1300_0000));
            p(0x71A248, [0u32, 299, 300, 3300][(seed % 4) as usize]);
            p(0x71A250, 300);
            p(0x71A24C, [300u32, 0, 299, 301][(seed / 4 % 4) as usize]);
            p(0x71A254, 300);
            p(0x5BEC94, (seed == 7) as u32);
        };
        both(
            &mut t,
            &format!("seed {seed}"),
            &copy,
            Regs::SENTINEL,
            &fill,
            &all,
            off_line_warnings_in,
        );
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
