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
//! Self-test: a copy of the original whose fourteen references are the test's, the two
//! gates and counts 0 .. 0x20.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, ECX, EDI, ESI, game_addr, rva};
use crate::physics::cells::{get, put_changed};
use crate::physics::queue_notice::Contacts;
use crate::screens::harness::Left;
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
        let p = |x: usize| game_addr(rva(x)) as u32;
        Self {
            busy: p(0x71A700),
            id: p(0x71A6F4),
            time_a: p(0x71A708),
            time_b: p(0x71A704),
        }
    }
}

/// The Rust version on the variables at `s` and `q`.
// Converted: `rc2k_core::physics::notices::Notice::take`.
pub unsafe fn take_notice_at(s: &Shown, q: &Contacts) -> Left {
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
        let Some(id) = notice.take(&mut now) else {
            return Left::nothing();
        };
        put_changed(s.id, was.id, notice.id);
        put_changed(s.time_a, was.w708, notice.w708);
        put_changed(s.time_b, was.time, notice.time);
        q.store(&queue, &now);
        Left::nothing()
            .with(EAX, id)
            .with(ECX, 0)
            .with(ESI, q.ebxs + 4 + 0x7C)
            .with(EDI, q.ebxs + 0x7C)
    }
}

/// The Rust version.
pub unsafe fn take_notice() -> Left {
    unsafe { take_notice_at(&Shown::game(), &Contacts::game()) }
}
