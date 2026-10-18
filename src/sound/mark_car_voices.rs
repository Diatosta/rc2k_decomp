//! `sound::mark_car_voices` (0x40D75A): each car's sound object told whether the car
//! sounds and whose voice it has.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! Only while `dword_514F8C` is 1: for each of the `dword_514F7C` objects in use
//! (`g_raceSoundObjects[n]`), +8 = 0; when its car (`g_carBlocks[n]`) has +0x25C not 0,
//! +8 = 1 and +0 = 0, and unless +0x25C is 1, +0 = !n (a negative number: a car of the
//! computer's, by the callers' reading). One caller, `sound::race_turn`.
//! Refuses (before writing) a count of 0 (the original's loop would not end) or over 8,
//! and a null object or car.
//! Shadow mode: both, the objects' two dwords compared.
//! Self-test: the original against the Rust version with the two tables pointing at the
//! test's objects and cars (put back).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::service::harness::{Region, at, capture, restore};
use crate::sound::turn_parts::{COUNT, OBJECT_LEN, OBJECTS, Part, both, dispatch_turn};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::mark_car_voices", rva(0x40D75A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// dword_514F8C: 1 while the cars' voices are to be marked.
const MARKING: usize = 0x514F8C;
/// g_carBlocks: the cars' blocks.
const CARS: usize = 0x71B3B8;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

// ---- Rust version ----

/// The (object, car) pairs in use, or a refusal.
fn pairs() -> Result<Vec<(u32, u32)>, BadInput> {
    let count = g(COUNT);
    if count == 0 || count > 8 {
        return Err(BadInput("a sound object count of 0 or over 8"));
    }
    let pairs: Vec<(u32, u32)> = (0..count as usize)
        .map(|n| (g(OBJECTS + 4 * n), g(CARS + 4 * n)))
        .collect();
    if pairs.iter().any(|&(o, c)| o == 0 || c == 0) {
        return Err(BadInput("a null sound object or car"));
    }
    Ok(pairs)
}

pub fn mark_car_voices_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    if g(MARKING) != 1 {
        return Ok(());
    }
    for (n, (object, car)) in pairs()?.into_iter().enumerate() {
        unsafe {
            let o = object as *mut u32;
            o.add(2).write_unaligned(0);
            let kind = ((car + 0x25C) as *const u32).read_unaligned();
            if kind != 0 {
                o.add(2).write_unaligned(1);
                o.write_unaligned(0);
                if kind != 1 {
                    o.write_unaligned(!(n as u32));
                }
            }
        }
    }
    Ok(())
}

// ---- hook ----

fn regions(_: &Regs) -> Vec<Region> {
    match pairs() {
        Ok(p) => p.iter().flat_map(|&(o, _)| [(o, 4), (o + 8, 4)]).collect(),
        Err(_) => Vec::new(),
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, mark_car_voices_in, Some(&regions));
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut objects = vec![0u32; 8 * OBJECT_LEN / 4];
    let mut cars = vec![0u32; 8 * 0x100];
    let (o, c) = (objects.as_mut_ptr() as u32, cars.as_mut_ptr() as u32);
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [
        (o, 8 * OBJECT_LEN),
        (c, 8 * 0x400),
        (a(OBJECTS), 0x20),
        (a(CARS), 0x20),
        at(rva(COUNT)),
        at(rva(MARKING)),
    ];
    let saved = capture(&all);
    for marking in [1u32, 0, 2] {
        for count in [1u32, 3, 8, 0, 9] {
            let fill = || unsafe {
                for n in 0..8u32 {
                    game_write(rva(OBJECTS + 4 * n as usize), o + n * OBJECT_LEN as u32);
                    game_write(rva(CARS + 4 * n as usize), c + n * 0x400);
                    let object = (o + n * OBJECT_LEN as u32) as *mut u32;
                    object.write(0x5555_0000 + n);
                    object.add(2).write(0x7777);
                    ((c + n * 0x400 + 0x25C) as *mut u32)
                        .write([0, 1, 2, 0xFFFF_FFFF][n as usize % 4]);
                }
                game_write(rva(COUNT), count);
                game_write(rva(MARKING), marking);
            };
            both(
                &mut t,
                &format!("marking {marking}, count {count}"),
                &|r| unsafe { call_original(r) },
                Regs::SENTINEL,
                &fill,
                &all,
                mark_car_voices_in,
                marking == 1 && (count == 0 || count > 8),
            );
        }
    }
    restore(&all, &saved);
    drop((objects, cars));
    t.finish()
}
