//! `race::cast_car_shadows` (0x45754C): every race car's shadow (see `race/car_shadow.rs`):
//! the depth test off; for each car with +0x6FC set, loaded into the work copy, its corner
//! count from `g_car` +0x260, `race::cast_car_shadow` once with `dword_6238B0` and once
//! more with `dword_60933C` 0x40; the depth test on.
//!
//! In:  nothing
//! Out: ecx the cars, esi the last block, eax 1, ebx 0xE (the depth test's call; a `call`
//!      follows)
//!
//! It begins with its call: private copy (0x75, its five calls relocated), manual patch.
//! Caller: `sub_4557AB` (0x45584E), each view drawn. Refuses (before anything) no cars or
//! more than eight, or a null block. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose shadow and depth-test calls go to stand-ins (the car's load
//! real), against the Rust version with the same; the cars' +0x6FC set by pattern; all
//! put back (skipped with a null block).

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::car_shadow::*;
use crate::race::cast_car_shadow::faces_in_game;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

const CALLS: [usize; 5] = [0x00, 0x2B, 0x45, 0x5A, 0x6F];

pub static HOOK: RawHook = RawHook::new("race::cast_car_shadows", rva(0x45754C), thunk, &ORIGINAL)
    .private_copy(0x75)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let done = unsafe {
        cast_car_shadows(r, &mut depth_part, &mut |e| {
            cast_car_shadow(e, &mut faces_in_game).map(|_| ())
        })
    };
    match done {
        Ok(after) => {
            let esp = r.esp;
            *r = after;
            r.esp = esp;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// Each call: its place (1 depth off, 2 the shadow, 3 depth on), the current car, the
/// corner count, `dword_60933C`.
static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

fn state(place: u32) -> [u32; 4] {
    unsafe {
        [
            place,
            game_read(rva(0x71B3F8)),
            game_read(COUNT_RVA),
            game_read(rva(0x60933C)),
        ]
    }
}

fn stand_in(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push(state(place));
    match place {
        1 => {
            r.eax = 0;
            r.ebx = 0xE;
        }
        3 => {
            r.eax = 1;
            r.ebx = 0xE;
        }
        _ => {}
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The ground under the car is searched in the stage's records.
    if !crate::physics::move_slot::in_race() {
        return t.skip("not in a race (no stage's records)");
    }
    let blocks: Vec<u32> = (0..8)
        .map(|n| unsafe { game_read::<u32>(rva(0x71B3B8 + 4 * n)) })
        .collect();
    if blocks.contains(&0) {
        return t.skip("a car block is null (no race loaded)");
    }
    crate::install::stand_ins::set_handler(stand_in);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45754C,
            len: 0x75,
            entry: &[0xE8, 0x80, 0x6E, 0xFF, 0xFF, 0xC7, 0x05, 0x28],
            calls: &CALLS,
            jumps_out: &[],
        },
        &|c| match c {
            x if x == rva(0x44E3D1) => Some(crate::install::stand_ins::stand_in(1)),
            x if x == rva(0x4575C1) => Some(crate::install::stand_ins::stand_in(2)),
            x if x == rva(0x44E3E1) => Some(crate::install::stand_ins::stand_in(3)),
            _ => None,
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions = vec![
        g(0x719FD8, 0xFE0),
        at(rva(0x71B3F8)),
        at(COUNT_RVA),
        at(rva(0x641928)),
        at(rva(0x60933C)),
    ];
    let mut kept = regions.clone();
    kept.extend([at(rva(0x63DB90)), at(rva(0x6238B0))]);
    kept.extend(blocks.iter().map(|&b| (b + 0x6FC, 4)));
    let saved = capture(&kept);
    for (cars, pattern, both) in [(8u32, 0x5Au32, 1u32), (3, 0x07, 0), (8, 0, 1)] {
        let prepare = || unsafe {
            restore(&kept, &saved);
            game_write(rva(0x63DB90), cars);
            game_write(rva(0x6238B0), both);
            for (n, &b) in blocks.iter().enumerate() {
                ((b + 0x6FC) as *mut u32).write_unaligned((pattern >> n) & 1);
            }
            CALLED.lock().unwrap().clear();
        };
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            regs(&ro),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let done = unsafe {
            cast_car_shadows(
                &Regs::SENTINEL,
                &mut |p, r| {
                    stand_in(if p == 1 { 1 } else { 3 }, r);
                    Ok(())
                },
                &mut |_| {
                    CALLED.lock().unwrap().push(state(2));
                    Ok(())
                },
            )
        };
        let mine = (
            done.as_ref().map(regs).unwrap_or_default(),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!("cars {cars} pattern {pattern:#x} both {both}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}", theirs.0, theirs.1, mine.0, mine.1)
        });
    }
    restore(&kept, &saved);
    t.finish()
}
