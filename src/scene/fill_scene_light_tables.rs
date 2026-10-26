//! `scene::fill_scene_light_tables` (0x4433AD): the light tables from the scene's own
//! colours, whatever the switch says.
//!
//! In:  nothing
//! Out: as `scene::fill_light_tables` leaves them
//!
//! `dword_621480` is kept, set to 1 for `scene::fill_light_tables` (which then takes
//! the scene's colours) and put back.
//! Two callers.
//! Self-test: three settings of the switch on the game's tables, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::scene::fill_light_tables::fill_light_tables;
use crate::scene::light_tables::{
    CAR_LIGHT_TABLE_RVA, LAMP_TABLE_A_RVA, LAMP_TABLE_B_RVA, LAMP_TABLE_C_RVA, LIGHT_TABLE_RVA,
    table,
};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::fill_scene_light_tables",
    rva(0x4433AD),
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

const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480

// ---- Rust version ----

pub unsafe fn fill_scene_light_tables() -> Left {
    unsafe {
        let kept: u32 = game_read(OTHER_WAY_RVA);
        game_write(OTHER_WAY_RVA, 1u32);
        let left = fill_light_tables();
        game_write(OTHER_WAY_RVA, kept);
        left
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = [
        LIGHT_TABLE_RVA,
        LAMP_TABLE_A_RVA,
        LAMP_TABLE_B_RVA,
        CAR_LIGHT_TABLE_RVA,
        LAMP_TABLE_C_RVA,
    ]
    .map(table)
    .to_vec();
    regions.push(at(OTHER_WAY_RVA));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { fill_scene_light_tables() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for switch in [0u32, 1, 0xFFFF_FF00] {
        check_state(
            &mut t,
            &format!("the switch at {switch:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                game_write(OTHER_WAY_RVA, switch);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
