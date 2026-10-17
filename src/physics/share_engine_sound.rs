//! `physics::share_engine_sound` (0x45D294): the current car's sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`): when `flt_71A208` is above its +0xD0 (`fcomp`:
//! not equal, not below, not unordered), the work car's 0x71A1FC .. 0x71A208 into its
//! +0xC4 .. +0xD0; always `dword_71A1E0` into +0xD4 and +0x10 = 1. By the look of it the
//! engine's figures for the sounds (a reading).
//!
//! In:  nothing
//! Out: edi the sound object; the rest kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B99F), every physics step.
//! Refuses (before anything) a null sound object.
//! Self-test: a copy of the original (the current car, the objects' table and the work
//! car's five the test's) on a test object, values above, at, below and NaN.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EDI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva,
};
use crate::math::x87_run::compare_full;
use crate::screens::harness::Left;
use crate::service::harness::{capture, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::share_engine_sound",
    rva(0x45D294),
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

/// The current car, the objects' table, the work car's 0x71A1FC .. 0x71A208, 0x71A1E0.
const VARIABLES: [usize; 7] = [
    0x71B3F8, 0x512A10, 0x71A1FC, 0x71A200, 0x71A204, 0x71A208, 0x71A1E0,
];

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core on the variables at `v` (`VARIABLES`' order).
pub unsafe fn share_at(v: &[u32]) -> Result<Left, Refused> {
    unsafe {
        let o = get(v[1] + 4 * (get(v[0]) & 7));
        if o == 0 {
            return Err(BadInput("a null sound object").into());
        }
        let status = compare_full(get(v[5]), get(o + 0xD0));
        if status & 0x4000 == 0 && status & 0x100 == 0 {
            for k in 0..4 {
                put(o + 0xC4 + 4 * k, get(v[2 + k as usize]));
            }
        }
        put(o + 0xD4, get(v[6]));
        put(o + 0x10, 1);
        Ok(Left::nothing().with(EDI, o))
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn share_engine_sound() -> Result<Left, Refused> {
    unsafe { share_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { share_engine_sound() }
}

extern "C" fn dispatch(r: &mut Regs) {
    let o: u32 =
        unsafe { game_read(rva(0x512A10) + 4 * (game_read::<u32>(rva(0x71B3F8)) & 7) as usize) };
    let regions = if o == 0 {
        Vec::new()
    } else {
        vec![(o + 0x10, 4), (o + 0xC4, 0x14)]
    };
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; VARIABLES.len() + 8 + 0x40];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..VARIABLES.len() as u32).map(|k| base + 4 * k).collect();
    let table = base + 4 * VARIABLES.len() as u32;
    let object = table + 0x20;
    let variables: Vec<(usize, u32)> = VARIABLES.iter().copied().zip(v.iter().copied()).collect();
    let mut variables = variables;
    variables[1].1 = table;
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45D294,
            len: 0x71,
            entry: &[0x8B, 0x3D, 0xF8, 0xB3, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &variables,
        8,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut at = v.clone();
    at[1] = table;
    let all = [(base, cells.len() * 4)];
    for (k, (ours, theirs)) in [
        (5.0f32, 3.0f32),
        (3.0, 3.0),
        (1.0, 3.0),
        (f32::NAN, 3.0),
        (2.0, f32::NAN),
    ]
    .into_iter()
    .enumerate()
    {
        let fill = || unsafe {
            for i in 0..cells.len() as u32 {
                put(base + 4 * i, 0x7700_0000 | i);
            }
            put(v[0], 9 + k as u32);
            for c in 0..8 {
                put(table + 4 * c, object);
            }
            put(v[5], ours.to_bits());
            put(object + 0xD0, theirs.to_bits());
        };
        fill();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let before_state = capture(&all);
        fill();
        let mine = unsafe { share_at(&at) };
        let state = capture(&all);
        let ok =
            mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro)) && before_state == state;
        t.check(ok, || {
            format!("ours {ours}, theirs {theirs}: {mine:?}, the memory differs")
        });
    }
    drop((copy, cells));
    t.finish()
}
