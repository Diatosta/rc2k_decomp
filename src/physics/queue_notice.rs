//! `physics::queue_notice` (0x4B1BD0, first named `note_contact`): queues a notice
//! (esi an id, 0xCF .. on; ebx its level, 1 .. 4; eax a value) in the work copy's queue
//! of up to 0x20 at `dword_71A450` (the count; then `dword_71A454` the esi's,
//! `dword_71A4D4` the eax's, `dword_71A554` the ebx's, 0x20 dwords each), when the work
//! copy's `dword_71A1F8` has bit 2 and the queue is not full. With ebx not 0, an entry of
//! the same eax and a non-zero ebx is kept to the largest ebx (its esi with it), unsigned;
//! else (ebx 0, or no such entry) a new entry. `sub_4B1C56` takes entries off;
//! `race::start_car` empties it.
//! The only caller is `sub_4B0241` (76 calls), run every physics step from `sub_43B635`
//! (0x43B95C); each call is behind a state variable (`dword_71A398` ..) whose value passes
//! a threshold of a car figure (`flt_71A28C` >= 1, ...), so a call comes only when such a
//! figure rises to a new level: damage notices, by the look of the levels (a reading).
//! A stage driven cleanly gives none (round 3, 2026-10-17: 0 calls).
//!
//! In:  eax, ebx, esi
//! Out: edx the entry's index (0 when the list was empty, kept when nothing was done);
//!      the rest kept; flags not read (the callers `cmp` next)
//!
//! Refuses a count above 0x20 (the original only stops at 0x20 exactly, and would write
//! past the lists), before writing.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::cells::{set_changed, set_values_changed, values};
use rc2k_core::physics::notices::NoticeQueue;

/// The work car's notice queue.
pub fn load() -> NoticeQueue {
    NoticeQueue {
        count: CAR.notice_count.get(),
        ids: values(&CAR.notice_ids),
        values: values(&CAR.notice_values),
        levels: values(&CAR.notice_ranks),
    }
}

/// What changed from `was` to `now` stored back.
pub fn store(was: &NoticeQueue, now: &NoticeQueue) {
    set_values_changed(&CAR.notice_ids, &was.ids, &now.ids);
    set_values_changed(&CAR.notice_values, &was.values, &now.values);
    set_values_changed(&CAR.notice_ranks, &was.levels, &now.levels);
    set_changed(&CAR.notice_count, was.count, now.count);
}

/// The Rust version.
pub fn queue_notice(notice: u32, rank: u32, value: u32) -> Result<(), Refused> {
    let was = load();
    let mut q = was;
    q.queue(CAR.option_bits.get(), notice, rank, value)?;
    store(&was, &q);
    Ok(())
}
