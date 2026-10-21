//! `sound::play_timed_slot` (0x40E59E): a sound slot of the race's sounds started when
//! asked, and marked done when its time is up.
//!
//! In:  ebp the slot, six dwords: +0 the sound, +4 1 = start it, +8 1 = playing, +0xC
//!      when it ends (`dword_5124E8` time), +0x10 the volume, +0x14 the rate (-0x6F: the
//!      sample's own); esi the request (`unk_5124BC`), its place already in it
//! Out: started: eax the end, ecx the volume, edx the rate (`sound::sample_settings`);
//!      checked: eax the clock; else nothing changed
//! +4 at 1 is cleared; then it starts unless it is playing (sound 0xE9 starts even
//! then): the
//! request's sound, volume (+8) and rate (+4, floats by `fild`) set, the end = the
//! clock + `sound::sample_duration`, +8 1, and `sound::play_placed_sound` (a tail jump).
//! Else, playing: +8 cleared once the clock is at or past the end (`js` on the
//! difference). 14 callers.
//! Refuses a null slot or request, and what the two leaves refuse, before writing.
//! A short jump in its first five bytes: private copy (0x73 bytes, the two calls and the
//! jump out relocated), manual patch.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose jump to `play_placed_sound` goes to a stand-in, against the
//! Rust version, on a slot of the test's, real samples.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::service::harness::{at, capture, restore};
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::{OWN, sample_settings};
use crate::sound::turn_parts::{
    CLOCK, PLACED, Part, REQUEST, both, common_regions, copy_of, dispatch_turn, run_copy,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::play_timed_slot", rva(0x40E59E), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&OUTS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x73;
/// `sample_settings`, `sample_duration`, the jump to `play_placed_sound`.
const OUTS: [usize; 3] = [0x2E, 0x47, 0x5C];
const ENTRY: [u8; 6] = [0x83, 0x7D, 0x04, 0x01, 0x74, 0x07];
/// The sound that is started again even while it plays.
const KEPT: u32 = 0xE9;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::slot_turn`.
pub fn play_timed_slot_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{SlotTurn, TimedSlot, from_int, slot_turn};
    let (slot, request) = (r.ebp as *mut [u32; 6], r.esi as *mut u32);
    if slot.is_null() || request.is_null() {
        return Err(BadInput("no slot or no request").into());
    }
    let w = unsafe { slot.read_unaligned() };
    let mut s = TimedSlot {
        sound: w[0],
        start: w[1],
        playing: w[2],
        ends: w[3],
        volume: w[4],
        rate: w[5],
    };
    let clock: u32 = unsafe { game_read(rva(CLOCK)) };
    let turn = slot_turn(&mut s, clock, sample_settings, sample_duration)?;
    unsafe { slot.write_unaligned([s.sound, s.start, s.playing, s.ends, s.volume, s.rate]) };
    match turn {
        SlotTurn::Nothing => Ok(()),
        SlotTurn::Checked => {
            r.eax = clock;
            Ok(())
        }
        SlotTurn::Start { volume, rate, ends } => {
            unsafe {
                request.write_unaligned(s.sound);
                request.add(2).write_unaligned(from_int(volume as i32));
                request.add(1).write_unaligned(from_int(rate as i32));
            }
            (r.eax, r.ecx, r.edx) = (ends, volume, rate);
            part(PLACED, r)
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, play_timed_slot_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40E59E, LEN, &ENTRY, &OUTS, &[PLACED]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut slot = [0u32; 6];
    let s = slot.as_mut_ptr() as u32;
    let mut all = common_regions();
    all.extend([(s, 24), at(rva(CLOCK))]);
    let saved = capture(&all);
    for sound in [KEPT, 3, 0x40, 0x13F, 0x1CC, 0x1CD] {
        for (start, playing) in [(1u32, 0u32), (1, 1), (1, 2), (0, 1), (0, 0), (2, 1)] {
            for (volume, rate) in [(OWN, OWN), (0x30, 22050), (0x50, 0xFFFF_FF90), (0, 1)] {
                for (clock, end) in [(1000u32, 999u32), (1000, 1001), (0x8000_0010, 0x10)] {
                    let fill = || unsafe {
                        let p = s as *mut u32;
                        for (n, v) in [sound, start, playing, end, volume, rate]
                            .into_iter()
                            .enumerate()
                        {
                            p.add(n).write(v);
                        }
                        game_write(rva(CLOCK), clock);
                    };
                    let starts = start == 1 && (sound == KEPT || playing != 1);
                    let refuses = starts
                        && sample_settings(sound, volume, rate)
                            .and_then(|(_, r)| sample_duration(sound, r))
                            .is_err();
                    both(
                        &mut t,
                        &format!(
                            "sound {sound:#x}, start {start}, playing {playing}, volume {volume:#x}, \
                             rate {rate:#x}, clock {clock:#x}, end {end:#x}"
                        ),
                        &run,
                        Regs {
                            ebp: s,
                            esi: game_addr(rva(REQUEST)) as u32,
                            ..Regs::SENTINEL
                        },
                        &fill,
                        &all,
                        play_timed_slot_in,
                        refuses,
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
