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
use crate::physics::cells::set_changed;
use crate::physics::queue_notice::{load, store};
use rc2k_core::physics::notices::Notice;

/// The Rust version.
pub fn take_notice() {
    let was = Notice {
        id: CAR.notice_id.get(),
        shown: CAR.notice_shown.get(),
        time: CAR.notice_time.get(),
        w708: CAR.notice_time_b.get(),
        ..Notice::default()
    };
    let queue = load();
    let (mut notice, mut now) = (was, queue);
    if notice.take(&mut now).is_none() {
        return;
    }
    set_changed(&CAR.notice_id, was.id, notice.id);
    set_changed(&CAR.notice_time_b, was.w708, notice.w708);
    set_changed(&CAR.notice_time, was.time, notice.time);
    store(&queue, &now);
}
