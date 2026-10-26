//! `marks::shapes_frame` (0x4C1148): `marks::copy_first_point`, `marks::tilt_shape`,
//! `marks::scale_setting`, `marks::run_shapes` (then `nullsub_19`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as `run_shapes` leaves them
//!
//! Caller: `sub_4C1270`. It begins with its call: a private copy (0x1A bytes, its five
//! calls relocated) with a manual patch. Refuses null settings before anything, and its
//! steps' refusals (all it may write is put back first). Self-test: the game's settings
//! (skipped without them), its steps' inputs by lot; compared on all it may write.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::marks::{copy_first_point, run_shapes, scale_setting, tilt_shape};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("marks::shapes_frame", rva(0x4C1148), thunk, &ORIGINAL)
    .private_copy(0x1A)
    .relocate_calls(&[0x0, 0x5, 0xA, 0xF, 0x14])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// What it may write.
pub fn regions() -> Vec<Region> {
    let mut r = copy_first_point::regions();
    r.extend(tilt_shape::regions());
    r.extend(scale_setting::regions());
    r.extend(run_shapes::regions());
    r
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn shapes_frame(entry: &Regs) -> Result<Regs, Refused> {
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Err(BadInput("no marks settings").into());
    }
    let written = regions();
    let saved = capture(&written);
    let done = unsafe {
        copy_first_point::copy_first_point();
        let g = tilt_shape::tilt_shape(entry);
        scale_setting::scale_setting(&g).and_then(|g| run_shapes::run_shapes(&g))
    };
    if done.is_err() {
        restore(&written, &saved);
    }
    done
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { shapes_frame(r) }?))
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
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return t.skip("no marks settings");
    }
    let regions = regions();
    // The steps' inputs: the first point, the tilt, the clock and ticks.
    let inputs = [
        rva(0x769306),
        rva(0x76930A),
        rva(0x76930E),
        rva(0x7692F6),
        rva(0x811BC3),
        rva(0x613830),
        rva(0x764FD0),
    ];
    let mut vars = regions.clone();
    vars.extend(inputs.map(at));
    let saved = capture(&vars);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                for (n, &a) in inputs[..5].iter().enumerate() {
                    game_write(a, ((case * 5 + n as u32) as f32 * 0.37 - 1.5).to_bits());
                }
                game_write(rva(0x613830), 1000 + case * 11);
                game_write(rva(0x764FD0), 1 + case % 3);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
