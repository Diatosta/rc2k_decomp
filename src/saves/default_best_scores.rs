//! `saves::default_best_scores` (0x47C923): fills the table of best scores with made-up
//! entries.
//!
//! In:  nothing
//! Out: every register preserved
//! The table at `dword_7556D4` (0xBB8 bytes) is five groups of ten entries of 0x3C bytes.
//! In each group the entries get 50, 49, .. 41 at +0x38 and a driver each
//! (`sub_47CDA7`, a plain function here): a random number under 0xB3 is drawn; for the
//! groups 0 .. 3 it is thrown away and the driver is a random number under
//! `dword_75AB00[group]` plus `dword_75AB10[group]`, for group 4 it is the driver. From
//! the driver's record: its +8 to the entry's +0x10, +0x14 to +8, +0x18 to +0xC, and the
//! name (without a NUL) to +0x18.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table, a group without drivers (the original divides by 0), a
//! driver past the 180, and a name that does not fit.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::math::random_below::random_below;
use crate::saves::drivers::{Driver, put, put_bytes, table_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::default_best_scores",
    rva(0x47C923),
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

// ---- game data ----

const TABLE_RVA: usize = rva(0x7556D4); // dword_7556D4
const GROUPS: u32 = 5;
const GROUP_LEN: u32 = 0x258;
const ENTRIES: u32 = 10;
const ENTRY_LEN: u32 = 0x3C;
const COUNTS_RVA: usize = rva(0x75AB00); // dword_75AB00[group]
const FIRSTS_RVA: usize = rva(0x75AB10); // dword_75AB10[group]

// ---- Rust version ----

/// What an entry gets, worked out before anything is written but the random numbers.
struct Entry {
    at: u32,
    score: u32,
    fields: [u32; 3],
    name: Vec<u8>,
}

pub unsafe fn default_best_scores() -> Result<(), Refused> {
    unsafe {
        let table: u32 = game_read(TABLE_RVA);
        if table == 0 {
            return Err(BadInput("no table of best scores").into());
        }
        for group in 0..GROUPS as usize {
            if group != 4 && game_read::<u32>(COUNTS_RVA + group * 4) == 0 {
                return Err(BadInput("a group without drivers").into());
            }
        }
        let mut entries = Vec::new();
        for group in 0..GROUPS {
            for n in 0..ENTRIES {
                let mut number = random_below(0xB3)?;
                if group != 4 {
                    number = random_below(game_read(COUNTS_RVA + group as usize * 4))?
                        .wrapping_add(game_read(FIRSTS_RVA + group as usize * 4));
                }
                let driver = Driver::number(number)?;
                entries.push(Entry {
                    at: table + group * GROUP_LEN + n * ENTRY_LEN,
                    score: 0x32 - n,
                    fields: [driver.dword(8), driver.dword(0x14), driver.dword(0x18)],
                    name: driver.name()?.1,
                });
            }
        }
        for entry in entries {
            put(entry.at, 0x38, entry.score);
            put(entry.at, 0x10, entry.fields[0]);
            put(entry.at, 8, entry.fields[1]);
            put(entry.at, 0xC, entry.fields[2]);
            put_bytes(entry.at, 0x18, &entry.name);
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    vec![
        (table_at(TABLE_RVA), (GROUPS * GROUP_LEN) as usize),
        at(STATE_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { default_best_scores() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    if table_at(TABLE_RVA) == 0 {
        HOOK.count_call();
        let e = BadInput("no table of best scores");
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    }
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the game's table, filled by both from the same random numbers, and put
// back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if table_at(TABLE_RVA) == 0 {
        return t.skip("the table of best scores is not there yet");
    }
    let regions = regions();
    let saved = capture(&regions);
    for seed in [1u32, 0x1234_5678, 0xDEAD_BEEF, 0x8000_0001] {
        check_state(
            &mut t,
            &format!("seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                game_write(STATE_RVA, seed);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
