//! `physics::expire_car_events` (0x44196B): the work car's event queue
//! (`physics::queue_car_event`'s) aged: while the count (+0xF98) is not 0 and the first
//! entry's time is 3000 or more behind `g_sceneClock` (by the sign), the two lists shift
//! down by one (seven dwords from +4 each, `rep movsd`: each list has an eighth slot,
//! +0xF4C and +0xF6C, which the shift reads), the
//! count goes down, +0xF9C up 0x20 and +0xFA0 up 0x40; it loops by a `jmp` to its own
//! start. Then +0xFA0 moves 4 towards +0xF9C (not past it; left when at or below it, by
//! the sign).
//!
//! In:  nothing
//! Out: eax +0xFA0, ecx 4; after a shift esi 0x71AF70, edi 0x71AF6C; the rest kept; no
//!      flags read (the callers return)
//!
//! It jumps back to its own start: hooked by a private copy.
//! Callers: `sub_4417EC`, and itself.
//! Refuses (before anything) a count above 7.
//! Self-test: a copy of the original (the clock and the queue the test's) against the
//! core: counts 0 .. 7 with times behind and not, +0xFA0 about +0xF9C.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, ECX, EDI, ESI, RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::physics::queue_car_event::QueuePlaces;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::expire_car_events",
    rva(0x44196B),
    thunk,
    &ORIGINAL,
)
.private_copy(0x80)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The core (esi and edi after a shift: the events' eighth slot and the one after).
pub unsafe fn expire_car_events_at(p: &QueuePlaces) -> Result<Left, Refused> {
    unsafe {
        if at(p.count) > 7 {
            return Err(BadInput("a car event count above 7").into());
        }
        let mut left = Left::nothing();
        while at(p.count) != 0 {
            let behind = at(p.clock).wrapping_sub(at(p.times)).wrapping_sub(3000);
            if (behind as i32) < 0 {
                break;
            }
            for k in 0..7 {
                put(p.times + 4 * k, at(p.times + 4 * k + 4));
            }
            for k in 0..7 {
                put(p.events + 4 * k, at(p.events + 4 * k + 4));
            }
            put(p.count, at(p.count).wrapping_sub(1));
            put(p.f9c, at(p.f9c).wrapping_add(0x20));
            put(p.fa0, at(p.fa0).wrapping_add(0x40));
            left = left.with(ESI, p.events + 0x20).with(EDI, p.events + 0x1C);
        }
        let target = at(p.f9c);
        let mut eax = at(p.fa0);
        if eax != target && (eax.wrapping_sub(target) as i32) >= 0 {
            eax = eax.wrapping_sub(4);
            if (eax.wrapping_sub(target) as i32) < 0 {
                eax = target;
            }
        }
        put(p.fa0, eax);
        Ok(left.with(EAX, eax).with(ECX, 4))
    }
}

/// The Rust version.
pub unsafe fn expire_car_events() -> Result<Left, Refused> {
    unsafe { expire_car_events_at(&QueuePlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { expire_car_events() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let p = QueuePlaces::game();
    dispatch_state(&HOOK, r, call_original, &[(p.times, 0x4C)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The clock, 8 times, 8 events, the count, +0xF9C, +0xFA0 (as the game lays them).
    let mut cells = vec![0u32; 20];
    let base = cells.as_mut_ptr() as u32;
    let p = QueuePlaces {
        clock: base,
        times: base + 4,
        events: base + 0x24,
        count: base + 0x44,
        f9c: base + 0x48,
        fa0: base + 0x4C,
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x44196B,
            len: 0x80,
            entry: &[0x83, 0x3D, 0x70, 0xAF, 0x71, 0x00, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x613830, p.clock),
            (0x71AF30, p.times),
            (0x71AF34, p.times + 4),
            (0x71AF50, p.events),
            (0x71AF54, p.events + 4),
            (0x71AF70, p.count),
            (0x71AF74, p.f9c),
            (0x71AF78, p.fa0),
        ],
        15,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 80)];
    for count in 0..=7u32 {
        for (n, (f9c, fa0)) in [
            (0x100u32, 0x100u32),
            (0x100, 0x102),
            (0x100, 0x200),
            (0x100, 0x80),
        ]
        .into_iter()
        .enumerate()
        {
            let fill = || unsafe {
                put(p.clock, 10_000);
                for k in 0..8u32 {
                    // Times from long ago to recent, by the case.
                    put(p.times + 4 * k, 1000 * (k + n as u32 * 2));
                    put(p.events + 4 * k, 0x100 + k);
                }
                put(p.count, count);
                put(p.f9c, f9c);
                put(p.fa0, fa0);
            };
            fill();
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { expire_car_events_at(&p) };
            let mine = capture(&all);
            let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
            t.check(regs_ok && theirs == mine, || {
                format!("count {count}, case {n}: regs_ok={regs_ok} original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    drop((copy, cells));
    t.finish()
}
