//! `physics::queue_car_event` (0x4419EB): an entry added to the work car's queue of up
//! to 7 (`g_car` +0xF98 the count): eax = `g_sceneClock` + esi; unless the count is 7,
//! the time into +0xF58[count], ebx into +0xF78[count], the count up by one; then +0xF9C
//! 0 and +0xFA0 0 for the first entry, else +0xF9C less 0x20. A reading: timed events
//! of the car (under `physics::pace_notes_step`).
//!
//! In:  esi a delay, ebx the event
//! Out: eax the time, ecx the count before; the rest kept; no flags read (the caller
//!      `popa`s)
//!
//! Caller: `sub_44194D`.
//! Refuses (before anything) a count above 7 (the original writes past the queue).
//! Self-test: a copy of the original (the clock and the car's fields the test's) against
//! the core, counts 0 .. 7, and the refusal.

use crate::errors::Refused;
use crate::framework::{EAX, ECX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::cells::{get, get_list, put, put_changed, put_list_changed};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use rc2k_core::physics::car_events::CarEvents;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::queue_car_event", rva(0x4419EB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Where the variables are: the clock, the times, the events, the count, +0xF9C, +0xFA0.
#[derive(Clone, Copy)]
pub struct QueuePlaces {
    pub clock: u32,
    pub times: u32,
    pub events: u32,
    pub count: u32,
    pub f9c: u32,
    pub fa0: u32,
}

impl QueuePlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            clock: g(0x613830),
            times: g(0x71AF30),
            events: g(0x71AF50),
            count: g(0x71AF70),
            f9c: g(0x71AF74),
            fa0: g(0x71AF78),
        }
    }
}

impl QueuePlaces {
    /// The queue at these places.
    pub unsafe fn load(&self) -> CarEvents {
        unsafe {
            CarEvents {
                times: get_list(self.times),
                events: get_list(self.events),
                count: get(self.count),
                level: get(self.f9c),
                shown: get(self.fa0),
            }
        }
    }

    /// What changed from `was` to `now` stored back.
    pub unsafe fn store(&self, was: &CarEvents, now: &CarEvents) {
        unsafe {
            put_list_changed(self.times, &was.times, &now.times);
            put_list_changed(self.events, &was.events, &now.events);
            put_changed(self.count, was.count, now.count);
            put_changed(self.f9c, was.level, now.level);
            put_changed(self.fa0, was.shown, now.shown);
        }
    }
}

/// The Rust version on the variables at `p`.
// Converted: `rc2k_core::physics::car_events::CarEvents::queue`.
pub unsafe fn queue_car_event_at(p: &QueuePlaces, esi: u32, ebx: u32) -> Result<Left, Refused> {
    unsafe {
        let was = p.load();
        let mut q = was;
        let time = q.queue(get(p.clock), esi, ebx)?;
        p.store(&was, &q);
        Ok(Left::nothing().with(EAX, time).with(ECX, was.count))
    }
}

/// The Rust version.
pub unsafe fn queue_car_event(esi: u32, ebx: u32) -> Result<Left, Refused> {
    unsafe { queue_car_event_at(&QueuePlaces::game(), esi, ebx) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { queue_car_event(r.esi, r.ebx) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let p = QueuePlaces::game();
    // The times, the events, the count, +0xF9C and +0xFA0: all (until 2026-10-18 the count
    // and four events were left out, so in shadow mode the Rust version's count stood and the
    // original queued one further on).
    let regions = [(p.times, (p.fa0 + 4 - p.times) as usize)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The clock, 7 times, 7 events, the count, +0xF9C, +0xFA0.
    let mut cells = vec![0u32; 1 + 7 + 7 + 3];
    let base = cells.as_mut_ptr() as u32;
    let p = QueuePlaces {
        clock: base,
        times: base + 4,
        events: base + 0x20,
        count: base + 0x3C,
        f9c: base + 0x40,
        fa0: base + 0x44,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4419EB,
            len: 0x4C,
            entry: &[0xA1, 0x30, 0x38, 0x61, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x613830, p.clock),
            (0x71AF30, p.times),
            (0x71AF50, p.events),
            (0x71AF70, p.count),
            (0x71AF74, p.f9c),
            (0x71AF78, p.fa0),
        ],
        9,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for count in 0..=7u32 {
        let fill = || unsafe {
            for k in 0..18u32 {
                put(base + 4 * k, 0x5A5A_0000 | k);
            }
            put(p.count, count);
        };
        let before = Regs {
            esi: 0x40 + count,
            ebx: 0x1234_0000 | count,
            ..Regs::SENTINEL
        };
        fill();
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        fill();
        let left = unsafe { queue_car_event_at(&p, before.esi, before.ebx) };
        let mine = capture(&all);
        let regs_ok = left.is_ok_and(|l| l.matches(&before, &ro));
        t.check(regs_ok && theirs == mine, || {
            format!("count {count}: regs_ok={regs_ok}; the memory differs")
        });
    }
    unsafe { put(p.count, 8) };
    let refused = unsafe { queue_car_event_at(&p, 0, 0) }.is_err();
    t.check(refused, || "count 8 not refused".into());
    drop((copy, cells));
    t.finish()
}
