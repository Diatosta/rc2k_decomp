//! `race::load_lights_data` (0x438AE5): `files\\s\\data\\ldata` read and copied into the light records (`render3d loadlightsdat`).
//!
//! In:  nothing
//! Out: as `copy_lights_data` leaves them
//!
//! See `race/stage_prefs.rs`. Self-test: the original and the Rust version on the game's
//! variables with values of the test's, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::stage_prefs::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::load_lights_data", rva(0x438AE5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![
        (game_addr(LIGHTS_RVA) as u32, LIGHTS_LEN as usize),
        (
            game_addr(LIGHT_RECORDS_RVA) as u32,
            (LIGHT_RECORDS * 0x134) as usize,
        ),
        at(crate::files::read_whole_file::BYTES_READ_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe {
        load_lights_data(r, &mut crate::race::level_init::game_part)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let _ = (at as fn(usize) -> Region, game_addr as fn(usize) -> usize);
    check_state(
        &mut t,
        "the game's file",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &saved),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
