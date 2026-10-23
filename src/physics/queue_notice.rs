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
//! Self-test: a copy of the original whose fourteen references are the test's, against
//! the Rust core on lists of 0 .. 0x20 entries, matching and new keys, ebx 0 and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EDX, Regs, game_addr, rva};
use crate::physics::cells::{get, get_list, put_changed, put_list_changed};
use crate::screens::harness::Left;
use rc2k_core::physics::notices::NoticeQueue;

/// The variables' addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct Contacts {
    pub flags: u32,
    pub count: u32,
    pub esis: u32,
    pub eaxs: u32,
    pub ebxs: u32,
}

impl Contacts {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            flags: a(0x71A1F8),
            count: a(0x71A450),
            esis: a(0x71A454),
            eaxs: a(0x71A4D4),
            ebxs: a(0x71A554),
        }
    }

    /// The queue at these places.
    pub unsafe fn load(&self) -> NoticeQueue {
        unsafe {
            NoticeQueue {
                count: get(self.count),
                ids: get_list(self.esis),
                values: get_list(self.eaxs),
                levels: get_list(self.ebxs),
            }
        }
    }

    /// What changed from `was` to `now` stored back.
    pub unsafe fn store(&self, was: &NoticeQueue, now: &NoticeQueue) {
        unsafe {
            put_list_changed(self.esis, &was.ids, &now.ids);
            put_list_changed(self.eaxs, &was.values, &now.values);
            put_list_changed(self.ebxs, &was.levels, &now.levels);
            put_changed(self.count, was.count, now.count);
        }
    }
}

/// The Rust version on the variables at `p`, with the registers at entry.
// Converted: `rc2k_core::physics::notices::NoticeQueue::queue`.
pub unsafe fn queue_notice_at(p: &Contacts, r: &Regs) -> Result<Left, Refused> {
    unsafe {
        let was = p.load();
        let mut q = was;
        let index = q.queue(get(p.flags), r.esi, r.ebx, r.eax)?;
        p.store(&was, &q);
        Ok(index.map_or(Left::nothing(), |k| Left::nothing().with(EDX, k)))
    }
}

/// The Rust version.
pub unsafe fn queue_notice(r: &Regs) -> Result<Left, Refused> {
    unsafe { queue_notice_at(&Contacts::game(), r) }
}
