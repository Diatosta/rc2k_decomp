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
//! Converted: `rc2k_core::saves::defaults::best_scores`.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::saves::default_times::{GameSource, to_nowhere};
use crate::saves::drivers::table_at;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::saves::defaults::{SCORE_TABLE_LEN, best_scores, write_best_scores};
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
const COUNTS_RVA: usize = rva(0x75AB00); // dword_75AB00[group]
const FIRSTS_RVA: usize = rva(0x75AB10); // dword_75AB10[group]

// ---- Rust version ----

pub unsafe fn default_best_scores() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::best_scores`.
    unsafe {
        let table: u32 = game_read(TABLE_RVA);
        if table == 0 {
            return Err(BadInput("no table of best scores").into());
        }
        let counts = std::array::from_fn(|g| game_read(COUNTS_RVA + g * 4));
        let firsts = std::array::from_fn(|g| game_read(FIRSTS_RVA + g * 4));
        let entries = best_scores(
            &mut GameSource {
                log: &mut to_nowhere,
            },
            counts,
            firsts,
        )?;
        write_best_scores(
            std::slice::from_raw_parts_mut(table as *mut u8, SCORE_TABLE_LEN),
            &entries,
        );
        Ok(())
    }
}
// ---- hook ----

fn regions() -> Vec<Region> {
    vec![(table_at(TABLE_RVA), SCORE_TABLE_LEN), at(STATE_RVA)]
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
