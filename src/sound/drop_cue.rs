//! `sound::drop_cue` (0x40EE32): one cue taken out of a sound object's queue.
//!
//! In:  eax the cue's place, edi the queue (the object +0x298: +0xC the count, cues of
//!      0x20 bytes from +0x10; `physics::queue_sound_cue`)
//! Out: cues moved: eax the place x 0x20, ecx 0; else ecx the count (less one when it
//!      dropped); everything else preserved
//! Nothing for a place at or past the count; else the count goes down by one and the
//! cues after the place move up (`rep movsd`). One caller, `sound::play_cue_queue`
//! (place 0: the cue played out).
//! Refuses (before anything) a null queue and a count over 16 (the move would run past
//! the object).
//! Shadow mode: both, the queue compared.
//! Self-test: the original against the Rust version on a test queue.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::service::harness::{Region, capture, restore};
use crate::sound::turn_parts::{Part, both, dispatch_turn};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::drop_cue", rva(0x40EE32), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The queue's length: the header and 16 cues.
pub const QUEUE_LEN: usize = 0x210;
pub const MOST_CUES: u32 = 16;

// ---- Rust version ----

/// Converted: `rc2k_core::sound::cars::drop_cue`.
pub fn drop_cue_in(c: &mut Regs, _: Part) -> Result<(), Refused> {
    use rc2k_core::sound::cars::{CueQueue, Dropped, drop_cue};
    let q = c.edi as *mut [u32; 0x84];
    if q.is_null() {
        return Err(BadInput("no queue").into());
    }
    let mut queue = CueQueue::from_words(&unsafe { q.read_unaligned() });
    match drop_cue(&mut queue, c.eax).map_err(|e| BadInput(e.0))? {
        Dropped::None { count } | Dropped::Last { count } => c.ecx = count,
        // `imul eax, 20h` (the cue's offset) is left in eax; `rep movsd` leaves ecx 0.
        Dropped::Moved => (c.eax, c.ecx) = (c.eax.wrapping_mul(0x20), 0),
    }
    unsafe { q.write_unaligned(queue.to_words()) };
    Ok(())
}

// ---- hook ----

fn regions(r: &Regs) -> Vec<Region> {
    vec![(r.edi, QUEUE_LEN)]
}

extern "C" fn dispatch(r: &mut Regs) {
    if r.edi == 0 {
        dispatch_turn(&HOOK, r, call_original, drop_cue_in, None);
        return;
    }
    dispatch_turn(&HOOK, r, call_original, drop_cue_in, Some(&regions));
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut queue = vec![0u32; QUEUE_LEN / 4];
    let q = queue.as_mut_ptr() as u32;
    let all: [Region; 1] = [(q, QUEUE_LEN)];
    let saved = capture(&all);
    for count in [0u32, 1, 2, 5, 15, 16, 17] {
        for place in [0u32, 1, 4, 14, 15, 16, 0xFFFF_FFFF] {
            let fill = || unsafe {
                let p = q as *mut u32;
                for n in 0..QUEUE_LEN / 4 {
                    p.add(n).write(0x1000 + n as u32);
                }
                p.add(3).write(count);
            };
            both(
                &mut t,
                &format!("count {count}, place {place:#x}"),
                &|r| unsafe { call_original(r) },
                Regs {
                    eax: place,
                    edi: q,
                    ..Regs::SENTINEL
                },
                &fill,
                &all,
                drop_cue_in,
                count > MOST_CUES,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
