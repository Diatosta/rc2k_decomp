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

use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, EDI, ESI, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva,
};
use crate::physics::cells::{get, put, put_changed};
use crate::physics::queue_notice::Contacts;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, state_entry};
use rc2k_core::physics::notices::Notice;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::take_notice", rva(0x4B1C56), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { take_notice() })
}

fn regions() -> Vec<Region> {
    let s = Shown::game();
    let mut r = Contacts::game().regions();
    r.extend([(s.id, 4), (s.time_a, 4), (s.time_b, 4)]);
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
    // busy, id, time a, time b, flags, count, the three lists.
    let mut cells = vec![0u32; 6 + 3 * 0x20];
    let base = cells.as_mut_ptr() as u32;
    let s = Shown {
        busy: base,
        id: base + 4,
        time_a: base + 8,
        time_b: base + 12,
    };
    let q = Contacts {
        flags: base + 16,
        count: base + 20,
        esis: base + 24,
        eaxs: base + 24 + 0x80,
        ebxs: base + 24 + 0x100,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4B1C56,
            len: 0x7A,
            entry: &[0x83, 0x3D, 0x00, 0xA7, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x71A700, s.busy),
            (0x71A6F4, s.id),
            (0x71A708, s.time_a),
            (0x71A704, s.time_b),
            (0x71A450, q.count),
            (0x71A454, q.esis),
            (0x71A458, q.esis + 4),
            (0x71A4D4, q.eaxs),
            (0x71A4D8, q.eaxs + 4),
            (0x71A554, q.ebxs),
            (0x71A558, q.ebxs + 4),
        ],
        14,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all: [Region; 1] = [(base, 4 * cells.len())];
    let fill = |busy: u32, id: u32, n: u32| {
        let c = base as *mut u32;
        for k in 0..(6 + 3 * 0x20) {
            unsafe { c.add(k).write_volatile(0xC000_0000 | k as u32) };
        }
        unsafe {
            put(s.busy, busy);
            put(s.id, id);
            put(q.count, n);
        }
    };
    for (busy, id) in [(0u32, 0u32), (1, 0), (0, 0xCF), (5, 7)] {
        for n in [0u32, 1, 2, 0x1F, 0x20] {
            fill(busy, id, n);
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill(busy, id, n);
            let mine = unsafe { take_notice_at(&s, &q) };
            let state = capture(&all);
            let regs_ok = mine.matches(&Regs::SENTINEL, &ro);
            t.check(regs_ok && theirs == state, || {
                format!(
                    "busy {busy}, shown {id:#x}, count {n}: regs_ok={regs_ok}; the memory differs"
                )
            });
        }
    }
    drop((copy, cells));
    t.finish()
}
