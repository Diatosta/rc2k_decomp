//! `marks::squares_step` (0x4C0654): `dword_81623A` 0; for each step since the squares were
//! last looked at (`dword_814714` less `dword_84460C`, low byte; `dword_84460C` the clock)
//! `marks::pick_shapes` and `marks::expire_squares`, `dword_84460C` on one (low byte); then
//! `dword_814714` it.
//!
//! In:  nothing
//! Out: as the last step leaves them (unchanged but eax with none)
//!
//! Caller: `sub_4BF60F` (0x4BF809). The calls are their Rust versions. Refuses
//! `pick_shapes`' refusals (all it may write put back first). Self-test: the original and
//! the Rust version on the game's items (as `expire_squares`'), clocks by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::expire_squares::{self, expire_squares, test_items};
use crate::marks::pick_shapes::{self, pick_shapes};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::squares_step", rva(0x4C0654), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CLEARED_RVA: usize = rva(0x81623A); // dword_81623A
const CLOCK_RVA: usize = rva(0x814714); // dword_814714
const LATEST_RVA: usize = rva(0x84460C); // dword_84460C

pub fn regions() -> Vec<Region> {
    let mut r = expire_squares::regions();
    r.extend(pick_shapes::regions());
    r.extend([CLEARED_RVA, CLOCK_RVA, LATEST_RVA].map(at));
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn squares_step(entry: &Regs) -> Result<Regs, Refused> {
    let written = regions();
    let saved = capture(&written);
    let done = unsafe { run(entry) };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

// Converted: `rc2k_core::marks::steps::{steps, next_step}`
unsafe fn run(entry: &Regs) -> Result<Regs, Refused> {
    use rc2k_core::marks::steps;
    let mut g = *entry;
    unsafe {
        game_write(CLEARED_RVA, 0u32);
        let clock: u32 = game_read(CLOCK_RVA);
        g.eax = steps::steps(game_read(LATEST_RVA), clock);
        game_write(LATEST_RVA, clock);
        for _ in 0..g.eax {
            g = pick_shapes(&g)?;
            g = expire_squares(&g);
            game_write(LATEST_RVA, steps::next_step(game_read(LATEST_RVA)));
        }
        game_write(CLOCK_RVA, game_read::<u32>(LATEST_RVA));
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { squares_step(r) }?))
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
    let extra = [at(rva(0x82FC56)), at(rva(0x833D7E))];
    let saved_extra = capture(&extra);
    for case in 0..5u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &saved);
                restore(&extra, &saved_extra);
                test_items(case);
                game_write(LATEST_RVA, case * 2);
                game_write(CLOCK_RVA, [0u32, 1, 3, 2, 0xFF][case as usize]);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    restore(&extra, &saved_extra);
    t.finish()
}
