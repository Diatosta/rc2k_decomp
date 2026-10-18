//! `sound::play_cue_queue` (0x40ED4A): a car's queue of spoken cues played, one sound
//! after the other.
//!
//! In:  ebx the car's sound object
//! Out: (from a `play_sound`) eax the sound, ebx and edx -0x6F, ecx, edi, ebp 0; (the
//!      queue run out) eax 0, ebx the place, ecx `drop_cue`'s, edi the queue; (not due)
//!      eax the clock, edi the queue; esi preserved
//! Only for an object in use (+8 is 1) with cues queued (+0x298 +0xC, the queue of
//! `physics::queue_sound_cue`): the queue's +4 is the place in the first cue (-1: none
//! begun), +8 the time the next sound is due. Unless none is begun, nothing until then
//! (`js` on the clock's difference). Then the place moves on; past the cue's sounds (its
//! +4) the cue is dropped (`sound::drop_cue`, place 0) and the next begun at 0, or with
//! none left the place is -1. Else its sound: due again after
//! `sound::sample_duration` (the sample's own rate) + 25 ms (sounds 0x13F and 0x140:
//! 0x1000000 later, so never: a reading), and `sound::play_sound` with ebx (the volume)
//! and edx -0x6F, ecx, edi, ebp 0.
//! Two callers, `sound::play_cue_queues`.
//! Refuses a null object, a queue of more than 16 or a cue of more than six sounds
//! (before anything), and the duration's refusals (what was written put back).
//! A near `jnz` in its first five bytes: private copy (0xA4 bytes, the three calls
//! relocated), manual patch.
//! Shadow mode runs the original alone (it sends sounds).
//! Self-test: a copy whose calls (`drop_cue`, which the stand-in does, `play_sound`) go
//! to stand-ins (the one of
//! `sample_duration` left), against the Rust version, on a test object.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::physics::flow::guard;
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::drop_cue::MOST_CUES;
use crate::sound::sample_duration::sample_duration;
use crate::sound::sample_settings::OWN;
use crate::sound::turn_parts::{
    CLOCK, OBJECT_LEN, PLAY, Part, both, copy_of, dispatch_turn, run_copy, test_object,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::play_cue_queue", rva(0x40ED4A), thunk, &ORIGINAL)
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

const LEN: usize = 0xA4;
/// `drop_cue`, `sample_duration` (left), `play_sound`.
const OUTS: [usize; 3] = [0x37, 0x5B, 0x9E];
const ENTRY: [u8; 10] = [0x83, 0x7B, 0x08, 0x01, 0x0F, 0x85, 0x99, 0x00, 0x00, 0x00];
const DROP: u32 = 0x40EE32;
const QUEUE: u32 = 0x298;

// ---- Rust version ----

pub fn play_cue_queue_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    let o = c.ebx;
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    if w(8) != 1 {
        return Ok(());
    }
    let q = o + QUEUE;
    let get = |n: u32| unsafe { ((q + 4 * n) as *const u32).read_unaligned() };
    let put = |n: u32, v: u32| unsafe { ((q + 4 * n) as *mut u32).write_unaligned(v) };
    c.edi = q;
    let count = get(3);
    if count == 0 {
        return Ok(());
    }
    if count > MOST_CUES {
        return Err(BadInput("a cue queue of more than 16").into());
    }
    if (0..count).any(|k| get(5 + 8 * k) > 6) {
        return Err(BadInput("a cue of more than six sounds").into());
    }
    let clock: u32 = unsafe { game_read(rva(CLOCK)) };
    if get(1) != u32::MAX {
        c.eax = clock;
        if (clock.wrapping_sub(get(2)) as i32) < 0 {
            return Ok(());
        }
    }
    guard(&[(o, OBJECT_LEN)], || {
        put(1, get(1).wrapping_add(1));
        loop {
            c.ebx = get(1);
            if c.ebx < get(5) {
                break;
            }
            c.eax = 0;
            part(DROP, c)?;
            put(1, 0);
            if get(3) == 0 {
                put(1, u32::MAX);
                return Ok(());
            }
        }
        let sound = get(6 + c.ebx);
        c.edx = OWN;
        c.eax = sample_duration(sound, OWN)?
            .wrapping_add(0x19)
            .wrapping_add(clock);
        put(2, c.eax);
        if sound == 0x13F || sound == 0x140 {
            put(2, get(2).wrapping_add(0x100_0000));
        }
        c.eax = get(6 + get(1));
        (c.ebx, c.edx, c.ecx, c.edi, c.ebp) = (OWN, OWN, 0, 0, 0);
        part(PLAY, c)
    })
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, play_cue_queue_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy_of(0x40ED4A, LEN, &ENTRY, &OUTS, &[DROP, PLAY]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x298, &[]);
    let o = object.as_mut_ptr() as u32;
    let all: [Region; 2] = [(o, OBJECT_LEN), at(rva(CLOCK))];
    let saved = capture(&all);
    for active in [1u32, 0] {
        for count in [0u32, 1, 3, 17] {
            for (place, due) in [(u32::MAX, 0u32), (0, 900), (0, 1100), (1, 1000), (5, 0)] {
                for sounds in [3u32, 0, 6, 7] {
                    let fill = || unsafe {
                        let p = o as *mut u32;
                        p.add(2).write(active);
                        let q = p.add(QUEUE as usize / 4);
                        // The whole queue: the drops move every cue's dwords.
                        for n in 0..0x84usize {
                            q.add(n).write(0x5000 + n as u32);
                        }
                        q.add(1).write(place);
                        q.add(2).write(due);
                        q.add(3).write(count);
                        for k in 0..16usize {
                            q.add(5 + 8 * k).write(if k == 0 { sounds } else { 2 });
                            for n in 0..6usize {
                                q.add(6 + 8 * k + n)
                                    .write([0x13F, 0x20, 0x140, 0x1CC, 3, 0][n]);
                            }
                        }
                        game_write(rva(CLOCK), 1000u32);
                    };
                    both(
                        &mut t,
                        &format!(
                            "active {active}, count {count}, place {place:#x}, due {due}, sounds {sounds}"
                        ),
                        &run,
                        Regs {
                            ebx: o,
                            ..Regs::SENTINEL
                        },
                        &fill,
                        &all,
                        play_cue_queue_in,
                        active == 1 && (count > MOST_CUES || (count > 0 && sounds > 6)),
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    t.finish()
}
