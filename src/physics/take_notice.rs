//! `physics::take_notice` (0x4B1C56): when no notice is shown (`dword_71A700` and
//! `dword_71A6F4` both 0) and the queue (`physics::queue_notice`) is not empty: the first
//! entry's id into `dword_71A6F4`, `dword_71A708` = 0x32, `dword_71A704` = 0x12C (by the
//! look of them, times to show it), the count less 1, and the three lists moved down one
//! (0x1F dwords each, `rep movsd`).
//!
//! In:  nothing
//! Out: when one is taken: eax the id, ecx 0, esi 0x71A5D4, edi 0x71A5D0 (the last
//!      `rep movsd`'s ends); else every register kept
//!
//! Caller: `sub_43B635` (0x43B961), every physics step, right after `sub_4B0241`.

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put_changed};
use crate::physics::queue_notice::Contacts;
use rc2k_core::physics::notices::Notice;

/// The shown notice's variables: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct Shown {
    pub busy: u32,
    pub id: u32,
    pub time_a: u32,
    pub time_b: u32,
}

impl Shown {
    pub fn game() -> Self {
        Self {
            busy: (CAR.notice_shown.addr() as u32),
            id: (CAR.notice_id.addr() as u32),
            time_a: (CAR.notice_time_b.addr() as u32),
            time_b: (CAR.notice_time.addr() as u32),
        }
    }
}

/// The Rust version on the variables at `s` and `q`.
// Converted: `rc2k_core::physics::notices::Notice::take`.
pub unsafe fn take_notice_at(s: &Shown, q: &Contacts) {
    unsafe {
        let was = Notice {
            id: get(s.id),
            shown: get(s.busy),
            time: get(s.time_b),
            w708: get(s.time_a),
            ..Notice::default()
        };
        let queue = q.load();
        let (mut notice, mut now) = (was, queue);
        if notice.take(&mut now).is_none() {
            return;
        }
        put_changed(s.id, was.id, notice.id);
        put_changed(s.time_a, was.w708, notice.w708);
        put_changed(s.time_b, was.time, notice.time);
        q.store(&queue, &now);
    }
}

/// The Rust version.
pub unsafe fn take_notice() {
    unsafe { take_notice_at(&Shown::game(), &Contacts::game()) }
}
