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

use crate::errors::Refused;
use crate::framework::{EDX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::cells::{get, get_list, put, put_changed, put_list_changed};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, state_entry};
use rc2k_core::physics::notices::{NoticeQueue, QUEUE_ROOM as MOST};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::queue_notice", rva(0x4B1BD0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

    pub fn regions(&self) -> Vec<Region> {
        vec![
            (self.count, 4),
            (self.esis, 0x80),
            (self.eaxs, 0x80),
            (self.ebxs, 0x80),
        ]
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { queue_notice(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Contacts::game().regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 2 + 3 * 0x20];
    let base = cells.as_mut_ptr() as u32;
    let p = Contacts {
        flags: base,
        count: base + 4,
        esis: base + 8,
        eaxs: base + 8 + 0x80,
        ebxs: base + 8 + 0x100,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4B1BD0,
            len: 0x86,
            entry: &[0xF7, 0x05, 0xF8, 0xA1, 0x71, 0x00, 0x04, 0x00, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x71A1F8, p.flags),
            (0x71A450, p.count),
            (0x71A454, p.esis),
            (0x71A4D4, p.eaxs),
            (0x71A554, p.ebxs),
        ],
        14,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, cells.len() * 4)];
    // Entries k: eax 0x100 + k % 5 (so keys repeat), ebx (k * 37) % 7 (some 0), esi k.
    let fill = |flags: u32, n: u32| unsafe {
        put(p.flags, flags);
        put(p.count, n);
        for k in 0..MOST {
            put(p.esis + 4 * k, 0xE000 + k);
            put(p.eaxs + 4 * k, 0x100 + k % 5);
            put(p.ebxs + 4 * k, (k * 37) % 7);
        }
    };
    for flags in [0u32, 4, 0xFFFF_FFFB, 0x84] {
        for n in [0u32, 1, 2, 3, 5, 9, 0x1F, 0x20] {
            for (eax, ebx) in [
                (0x100u32, 0u32),
                (0x100, 1),
                (0x100, 9),
                (0x101, 3),
                (0x102, 6),
                (0x103, 0xFFFF_FFFF),
                (0x777, 2),
                (0x104, 4),
            ] {
                let before = Regs {
                    eax,
                    ebx,
                    esi: 0x5151_0000 | ebx,
                    ..Regs::SENTINEL
                };
                fill(flags, n);
                let mut ro = before;
                unsafe { call_game(&mut ro, copy.addr()) };
                let theirs = capture(&all);
                fill(flags, n);
                let mine = unsafe { queue_notice_at(&p, &before) };
                let state = capture(&all);
                let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&before, &ro));
                t.check(regs_ok && theirs == state, || {
                    format!(
                        "flags {flags:#x}, count {n}, eax {eax:#x}, ebx {ebx:#x}: regs_ok={regs_ok} (original edx {:#x}) {mine:?}; the lists differ",
                        ro.edx
                    )
                });
            }
        }
    }
    fill(4, 0x21);
    let before = capture(&all);
    let refused = unsafe { queue_notice_at(&p, &Regs::SENTINEL) }.is_err();
    t.check(refused && capture(&all) == before, || {
        format!("count 0x21: refused {refused}, or something written")
    });
    drop((copy, cells));
    t.finish()
}
