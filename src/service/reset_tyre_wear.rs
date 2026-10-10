//! `service::reset_tyre_wear` (0x486EDD): zeroes six dwords of the current car: +0x1F8 ..
//! +0x208 and +0x108 (the tyre damage shown in the service area).
//!
//! In:  nothing (eax is saved and restored)
//! Out: every register preserved
//! Three callers. The one in `sub_46C14D` saves and restores +0x108 around the call, so there
//! only the five at +0x1F8 are cleared; what they hold has not been worked out, and "wear" is
//! a reading. Refuses a missing car object.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use crate::service::state::current_car;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::reset_tyre_wear", rva(0x486EDD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const WEAR_OFFSET: u32 = 0x1F8; // five dwords
pub const WEAR_DWORDS: u32 = 5;
pub const TYRE_DAMAGE_OFFSET: u32 = 0x108;

// ---- Rust version ----

pub unsafe fn reset_tyre_wear() -> Result<(), BadInput> {
    let car = current_car().ok_or(BadInput(
        "no current car object: the original writes through null",
    ))?;
    unsafe {
        for i in 0..WEAR_DWORDS {
            ((car + WEAR_OFFSET + 4 * i) as *mut u32).write_unaligned(0);
        }
        ((car + TYRE_DAMAGE_OFFSET) as *mut u32).write_unaligned(0);
    }
    Ok(())
}

// ---- hook ----

/// From the tyre damage to the end of the wear values, with what lies between them.
fn regions() -> Vec<Region> {
    current_car()
        .map(|car| {
            vec![(
                car + TYRE_DAMAGE_OFFSET - 4,
                (WEAR_OFFSET + 4 * WEAR_DWORDS + 4 - TYRE_DAMAGE_OFFSET + 4) as usize,
            )]
        })
        .unwrap_or_default()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { reset_tyre_wear() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the current car, put back afterwards ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.is_empty() {
        return t.skip("there is no current car object");
    }
    let saved = capture(&regions);
    // Only the six dwords are given test values: the game may be reading the rest of the car.
    let car = regions[0].0 + 4 - TYRE_DAMAGE_OFFSET;
    for fill in [0x1111_1111u32, 0xEEEE_EEEE] {
        check_state(
            &mut t,
            &format!("fill={fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for i in 0..WEAR_DWORDS {
                    ((car + WEAR_OFFSET + 4 * i) as *mut u32).write_unaligned(fill);
                }
                ((car + TYRE_DAMAGE_OFFSET) as *mut u32).write_unaligned(fill);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
