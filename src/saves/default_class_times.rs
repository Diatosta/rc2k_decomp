//! `saves::default_class_times` (0x47C8E3): fills the table at `dword_7556DC` with made-up
//! entries.
//!
//! In:  nothing
//! Out: every register preserved
//! The table (0x15E0 bytes) is seven groups of ten entries of 0x50 bytes. In each group
//! the entries get 100,000 and then 0xAB3 less each time at +0x14, and a random driver
//! (`sub_47CB2D`, a plain function here: a number under 0xAA): the driver's +8 to the
//! entry's +0x1C, +0x14 to +8, +0x18 to +0x10, the id of the name's string to +0xC and
//! the name (without a NUL) to +0x20.
//! What the table is has not been worked out; "class times" is a guess.
//! One caller, `menu::set_up_front_end`, when `var\hst.dat` was not loaded.
//! Refuses a missing table and a name that does not fit.

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
    "saves::default_class_times",
    rva(0x47C8E3),
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

const TABLE_RVA: usize = rva(0x7556DC); // dword_7556DC
const GROUPS: u32 = 7;
const GROUP_LEN: u32 = 0x320;
const ENTRIES: u32 = 10;
const ENTRY_LEN: u32 = 0x50;

// ---- Rust version ----

/// A random driver into the entry at `entry`: +0x1C, +8, +0x10, the name's id at +0xC and
/// the name at +0x20 (`sub_47CB2D`). Returns what is to be written, so that the caller
/// can refuse before writing.
pub unsafe fn random_driver() -> Result<([u32; 4], Vec<u8>), Refused> {
    unsafe {
        let driver = Driver::number(random_below(0xAA)?)?;
        let (id, name) = driver.name()?;
        Ok((
            [driver.dword(8), driver.dword(0x14), driver.dword(0x18), id],
            name,
        ))
    }
}

/// Writes what `random_driver` gave.
pub unsafe fn put_driver(entry: u32, driver: &([u32; 4], Vec<u8>)) {
    unsafe {
        put(entry, 0x1C, driver.0[0]);
        put(entry, 8, driver.0[1]);
        put(entry, 0x10, driver.0[2]);
        put(entry, 0xC, driver.0[3]);
        put_bytes(entry, 0x20, &driver.1);
    }
}

pub unsafe fn default_class_times() -> Result<(), Refused> {
    unsafe {
        let table: u32 = game_read(TABLE_RVA);
        if table == 0 {
            return Err(BadInput("no table").into());
        }
        let mut entries = Vec::new();
        for group in 0..GROUPS {
            for n in 0..ENTRIES {
                let entry = table + group * GROUP_LEN + n * ENTRY_LEN;
                entries.push((entry, 100_000 - n * 0xAB3, random_driver()?));
            }
        }
        for (entry, value, driver) in entries {
            put(entry, 0x14, value);
            put_driver(entry, &driver);
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
    unsafe { default_class_times() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    if table_at(TABLE_RVA) == 0 {
        HOOK.count_call();
        let e = BadInput("no table");
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
        return t.skip("the table is not there yet");
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
