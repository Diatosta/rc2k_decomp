//! `menu::drop_missing_controllers` (0x4748B6): a car whose number at +0x2C is above
//! `dword_4F78E0` gets 0 there.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players, compared unsigned. A reading of what the two are:
//! `dword_4F78E0` is counted up by the function that lists the game controllers
//! (`sub_408238`, at most 0x10), so +0x2C is the player's controller, 0 being none.
//! Callers: `sub_47486C` (before the main menu) and `saves::load_savegame`.
//! Refuses a missing player.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::player_names::{PLAYERS, cars};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::drop_missing_controllers",
    rva(0x4748B6),
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

const CONTROLLERS_RVA: usize = rva(0x4F78E0); // dword_4F78E0
const CAR_CONTROLLER: u32 = 0x2C;

// ---- Rust version ----

pub unsafe fn drop_missing_controllers() -> Result<(), BadInput> {
    unsafe {
        // Converted: `rc2k_core::menu::steps::kept_controller`.
        let most: u32 = game_read(CONTROLLERS_RVA);
        for car in cars()? {
            let controller = (car + CAR_CONTROLLER) as *mut u32;
            let c = controller.read_unaligned();
            let kept = rc2k_core::menu::steps::kept_controller(c, most);
            if kept != c {
                controller.write_unaligned(kept);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions(cars: &[u32; PLAYERS]) -> Vec<Region> {
    cars.iter().map(|&car| (car + CAR_CONTROLLER, 4)).collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { drop_missing_controllers() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    match unsafe { cars() } {
        Ok(cars) => dispatch_state(&HOOK, r, call_original, &regions(&cars), &rust),
        Err(e) => {
            HOOK.count_call();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

// ---- self-test: numbers of the test's in the game's cars, put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(cars) = (unsafe { cars() }) else {
        return t.skip("a player is missing");
    };
    let regions = regions(&cars);
    let inputs = [at(CONTROLLERS_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for (most, numbers) in [
        (0u32, [0u32, 1, 2, 0xFFFF_FFFF]),
        (2, [3, 2, 1, 0]),
        (0x10, [0x10, 0x11, 0x8000_0000, 5]),
        (0xFFFF_FFFF, [0xFFFF_FFFF, 0, 7, 0xFFFF_FFFE]),
        (0x8000_0000, [0x7FFF_FFFF, 0x8000_0001, 0x8000_0000, 1]),
    ] {
        check_state(
            &mut t,
            &format!("{most:#x} controllers, numbers {numbers:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(CONTROLLERS_RVA, most);
                for (&car, number) in cars.iter().zip(numbers) {
                    ((car + CAR_CONTROLLER) as *mut u32).write_unaligned(number);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
