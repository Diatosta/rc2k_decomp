//! `physics::queue_notice` (0x4B1BD0, first named `note_contact`): queues a notice
//! (esi an id, 0xCF .. on; ebx its level, 1 .. 4; eax a value) in the work copy's queue
//! of up to 0x20 at `dword_71A450` (the count; then `dword_71A454` the esi's,
//! `dword_71A4D4` the eax's, `dword_71A554` the ebx's, 0x20 dwords each), when the work
//! copy's `dword_71A1F8` has bit 2 and the queue is not full. With ebx not 0, an entry of
//! the same eax and a non-zero ebx is kept to the largest ebx (its esi with it), unsigned;
//! else (ebx 0, or no such entry) a new entry. `physics::take_notice` takes entries off;
//! `race::start_car` empties it.
//! The only caller is `physics::damage_notices` (76 calls), run every physics step from
//! `physics::car_physics_step` (0x43B95C); each call is behind a state variable (`dword_71A398` ..)
//! whose value passes a threshold of a car figure (`flt_71A28C` >= 1, ...), so a call comes only
//! when such a figure rises to a new level: damage notices, by the look of the levels (a reading).
//! A stage driven cleanly gives none (round 3, 2026-10-17: 0 calls).
//!
//! Refuses a count above 0x20 (the original only stops at 0x20 exactly, and would write
//! past the lists), before writing.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::cells::{set_changed, set_values_changed, values};
use crate::race::owner_shared::Shared;
use rc2k_core::physics::notices::NoticeQueue;

/// The work car's notice queue.
pub fn load(shared: &Shared) -> NoticeQueue {
    NoticeQueue {
        count: shared.car.notice_count.get(),
        ids: values(&shared.car.notice_ids),
        values: values(&shared.car.notice_values),
        levels: values(&shared.car.notice_ranks),
    }
}

/// What changed from `was` to `now` stored back.
pub fn store(shared: &Shared, was: &NoticeQueue, now: &NoticeQueue) {
    set_values_changed(&shared.car.notice_ids, &was.ids, &now.ids);
    set_values_changed(&shared.car.notice_values, &was.values, &now.values);
    set_values_changed(&shared.car.notice_ranks, &was.levels, &now.levels);
    set_changed(&shared.car.notice_count, was.count, now.count);
}

pub fn queue_notice(shared: &Shared, notice: u32, rank: u32, value: u32) -> Result<(), Refused> {
    let was = load(shared);
    let mut q = was;
    q.queue(shared.car.option_bits.get(), notice, rank, value)?;
    store(shared, &was, &q);
    Ok(())
}
