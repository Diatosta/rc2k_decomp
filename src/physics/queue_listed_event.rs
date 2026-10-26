//! `physics::queue_listed_event` (0x44194D): for ebx below 0x20, the event
//! `dword_614C7C[ebx]` (into ebx); unless it is -1, `physics::queue_car_event` with no
//! delay (esi 0), inside `pusha` / `popa`.
//!
//! In:  ebx the event's number
//! Out: ebx the table's entry (when below 0x20); the rest kept; no flags read (the
//!      caller's next is an `inc`)
//!
//! Caller: `sub_4418B3`.
//! Refuses what `queue_car_event` refuses.
//! Self-test: a copy of the original (the table the test's, its call to
//! `queue_car_event`'s test copy on test cells) against the core: numbers in and past the
//! table, entries -1 and not, queue counts 0 .. 7.

use crate::errors::Refused;
use crate::framework::{EBX, RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::queue_car_event::{QueuePlaces, queue_car_event_at};
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use rc2k_core::physics::car_events::listed_event;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::queue_listed_event",
    rva(0x44194D),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TABLE_RVA: usize = rva(0x614C7C); // dword_614C7C, 0x20 entries

/// The core: the table at `table`, the queue at `q`.
// Converted: `rc2k_core::physics::car_events::listed_event`.
pub unsafe fn queue_listed_event_at(
    table: u32,
    q: &QueuePlaces,
    ebx: u32,
) -> Result<Left, Refused> {
    let entry = |n: u32| unsafe { ((table + 4 * n) as *const u32).read_unaligned() };
    let Some((event, queued)) = listed_event(ebx, entry) else {
        return Ok(Left::nothing());
    };
    if queued {
        unsafe { queue_car_event_at(q, 0, event) }?;
    }
    Ok(Left::nothing().with(EBX, event))
}

/// The Rust version.
pub unsafe fn queue_listed_event(ebx: u32) -> Result<Left, Refused> {
    unsafe { queue_listed_event_at(game_addr(TABLE_RVA) as u32, &QueuePlaces::game(), ebx) }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { queue_listed_event(r.ebx) }
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
    // The clock, 7 times, 7 events, the count, +0xF9C, +0xFA0, then the table of 0x20.
    let mut cells = vec![0u32; 18 + 0x20];
    let base = cells.as_mut_ptr() as u32;
    let q = QueuePlaces {
        clock: base,
        times: base + 4,
        events: base + 0x20,
        count: base + 0x3C,
        f9c: base + 0x40,
        fa0: base + 0x44,
    };
    let table = base + 0x48;
    let inner = match crate::audio::test_copy::make(
        &crate::physics::queue_car_event::HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4419EB,
            len: 0x4C,
            entry: &[0xA1, 0x30, 0x38, 0x61, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x613830, q.clock),
            (0x71AF30, q.times),
            (0x71AF50, q.events),
            (0x71AF70, q.count),
            (0x71AF74, q.f9c),
            (0x71AF78, q.fa0),
        ],
        9,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let target = inner.addr();
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x44194D,
            len: 0x1E,
            entry: &[0x83, 0xFB, 0x20, 0x73, 0x18],
            calls: &[0x17],
            jumps_out: &[],
        },
        &|r| (r == rva(0x4419EB)).then_some(target),
        &[(0x614C7C, table)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 4 * cells.len())];
    for (n, number) in [0u32, 3, 0x1F, 0x20, 0xFFFF_FFFF].into_iter().enumerate() {
        for count in [0u32, 6, 7] {
            let fill = || unsafe {
                for k in 0..(18 + 0x20u32) {
                    ((base + 4 * k) as *mut u32).write(0x5A5A_0000 | k);
                }
                ((q.count) as *mut u32).write(count);
                for k in 0..0x20u32 {
                    let v = if (k + n as u32).is_multiple_of(3) {
                        u32::MAX
                    } else {
                        0x100 + k
                    };
                    ((table + 4 * k) as *mut u32).write(v);
                }
            };
            let before = Regs {
                ebx: number,
                ..Regs::SENTINEL
            };
            fill();
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { queue_listed_event_at(table, &q, number) };
            let mine = capture(&all);
            let regs_ok = left.is_ok_and(|l| l.matches(&before, &ro));
            t.check(regs_ok && theirs == mine, || {
                format!("number {number:#x}, count {count}: regs_ok={regs_ok}; the memory differs")
            });
        }
    }
    drop((copy, inner, cells));
    t.finish()
}
