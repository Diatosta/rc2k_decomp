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
//! Converted: `rc2k_core::saves::defaults::class_times`.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::saves::default_times::{GameSource, to_nowhere};
use crate::saves::drivers::table_at;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::saves::defaults::{CLASS_TABLE_LEN, class_times, write_class_times};
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

// ---- Rust version ----

pub unsafe fn default_class_times() -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::defaults::class_times`.
    unsafe {
        let table: u32 = game_read(TABLE_RVA);
        if table == 0 {
            return Err(BadInput("no table").into());
        }
        let entries = class_times(&mut GameSource {
            log: &mut to_nowhere,
        })?;
        write_class_times(
            std::slice::from_raw_parts_mut(table as *mut u8, CLASS_TABLE_LEN),
            &entries,
        );
        Ok(())
    }
}
// ---- hook ----

fn regions() -> Vec<Region> {
    vec![(table_at(TABLE_RVA), CLASS_TABLE_LEN), at(STATE_RVA)]
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
