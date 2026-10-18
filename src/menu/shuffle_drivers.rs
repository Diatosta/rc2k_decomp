//! `menu::shuffle_drivers` (0x470AB9): meant to shuffle four driver tables
//! (`unk_73EA7C`, `unk_740764`, `unk_7418BC`, `g_drivers`; 0x94 an entry; counts
//! `dword_75AB04`, `dword_75AB08`, `dword_75AB0C`, `dword_75AB00`), but it never adds the
//! two random entries' offsets: for each table, count times, two random numbers under the
//! count (`dword_7559EC`, `hhmm`) and the first entry swapped with itself through
//! `unk_755398` / `unk_75542C` (both left holding it). Section 16.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_473126` and one of the game's. The random numbers are
//! `math::random_below`'s. Refuses a count of 0 (the original's `div` traps, its loop runs
//! on), before anything. Self-test: the original and the Rust version on the game's
//! tables, counts and the random state by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::shuffle_drivers", rva(0x470AB9), thunk, &ORIGINAL)
    .private_copy(0x24F)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

/// Its calls (a private copy: the loop goes back to +3, inside the patched bytes).
const CALLS: [usize; 8] = [0x8, 0x17, 0x9B, 0xAA, 0x12E, 0x13D, 0x1C1, 0x1D0];

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TABLES: [(usize, usize); 4] = [
    (rva(0x73EA7C), rva(0x75AB04)),
    (rva(0x740764), rva(0x75AB08)),
    (rva(0x7418BC), rva(0x75AB0C)),
    (DRIVERS_RVA, rva(0x75AB00)),
];
const TEMP_RVA: [usize; 2] = [rva(0x755398), rva(0x75542C)]; // unk_755398, unk_75542C
const PICKS_RVA: [usize; 2] = [rva(0x7559EC), rva(0x7559F0)]; // dword_7559EC, hhmm
const RANDOM_RVA: usize = crate::math::next_random::STATE_RVA;
const EXTRA: [usize; 4] = [TABLES[0].1, TABLES[1].1, TABLES[2].1, TABLES[3].1];
const CASES: u32 = 3;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    unsafe {
        game_write(RANDOM_RVA, 0x1357_9BDFu32.wrapping_mul(case + 3));
        for (n, &(_, count)) in TABLES.iter().enumerate() {
            game_write(count, 1 + (n as u32 + case) % 5);
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn shuffle_drivers(entry: &Regs) -> Result<Regs, Refused> {
    use crate::math::random_below::random_below;
    unsafe {
        if TABLES.iter().any(|&(_, c)| game_read::<u32>(c) == 0) {
            return Err(BadInput("a driver table count of 0").into());
        }
        let [t1, t2] = TEMP_RVA.map(|r| game_addr(r) as *mut u8);
        for &(table, count) in &TABLES {
            let first = game_addr(table) as *mut u8;
            let n: u32 = game_read(count);
            for _ in 0..n {
                game_write(PICKS_RVA[0], random_below(n)?);
                game_write(PICKS_RVA[1], random_below(n)?);
                std::ptr::copy(first, t1, 0x94);
                std::ptr::copy(first, t2, 0x94);
                std::ptr::copy(t1, first, 0x94);
                std::ptr::copy(t2, first, 0x94);
            }
        }
    }
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let mut v: Vec<Region> = TABLES
        .iter()
        .map(|&(t, _)| (game_addr(t) as u32, 0x94))
        .collect();
    v.extend(TEMP_RVA.map(|r| (game_addr(r) as u32, 0x94)));
    v.extend([PICKS_RVA[0], PICKS_RVA[1], RANDOM_RVA].map(at));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { shuffle_drivers(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { shuffle_drivers(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(case),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
