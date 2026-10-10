//! What the service area's small value functions share: the twelve that step a setup value
//! one up or down, and the four that select or deselect a change.

use crate::errors::{BadInput, Refused};
use crate::framework::{EDI, RawHook, Regs, SelfTest, game_addr, game_read, game_write};
use crate::screens::harness::{CallOriginal, Left};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::state::{
    FLAG_TABLE_LEN, MENU_ITEM_RVA, REPAIR_FLAGS_RVA, current_car, flagged_item, repair_flag,
    set_repair_flag,
};

// ---- steppers ----

#[derive(Clone, Copy)]
pub enum Target {
    /// A game variable, by RVA.
    Global(usize),
    /// A dword of the current car object, by offset. These functions leave the car in edi.
    CarField(u32),
}

/// One step down stops at 0 (a value that is already negative goes on down, as in the
/// original); one step up stops at `max`, and a value above `max` becomes `max`.
#[derive(Clone, Copy)]
pub struct Stepper {
    pub target: Target,
    pub max: i32,
    pub up: bool,
}

fn target(stepper: Stepper) -> Result<(u32, Left), BadInput> {
    match stepper.target {
        Target::Global(rva) => Ok((game_addr(rva) as u32, Left::nothing())),
        Target::CarField(offset) => {
            let car = current_car().ok_or(BadInput(
                "no current car object: the original reads through null",
            ))?;
            Ok((car.wrapping_add(offset), Left::nothing().with(EDI, car)))
        }
    }
}

pub unsafe fn step(stepper: Stepper) -> Result<Left, Refused> {
    let (addr, left) = target(stepper)?;
    let value = addr as *mut i32;
    unsafe {
        let now = value.read_unaligned();
        let next = if stepper.up {
            if now < stepper.max {
                now.wrapping_add(1)
            } else {
                stepper.max
            }
        } else if now != 0 {
            now.wrapping_sub(1)
        } else {
            0
        };
        value.write_unaligned(next);
    }
    Ok(left)
}

pub fn dispatch_stepper(
    hook: &RawHook,
    r: &mut Regs,
    call_original: CallOriginal,
    stepper: Stepper,
) {
    let regions: Vec<Region> = target(stepper)
        .map(|(addr, _)| vec![(addr, 4)])
        .unwrap_or_default();
    dispatch_state(hook, r, call_original, &regions, &|_| unsafe {
        step(stepper)
    });
}

/// Why a stepper cannot be tested now, or None.
pub fn stepper_unavailable(stepper: Stepper) -> Option<&'static str> {
    target(stepper)
        .is_err()
        .then_some("there is no current car object")
}

/// Values on both sides of both limits. The value is put back afterwards.
pub fn check_stepper(t: &mut SelfTest, call_original: CallOriginal, stepper: Stepper) {
    let Ok((addr, _)) = target(stepper) else {
        return;
    };
    let regions = [(addr, 4)];
    let saved = capture(&regions);
    let max = stepper.max;
    for value in [
        -3,
        -1,
        0,
        1,
        2,
        max - 1,
        max,
        max + 1,
        max + 50,
        i32::MAX,
        i32::MIN,
    ] {
        check_state(
            t,
            &format!("value={value}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { (addr as *mut i32).write_unaligned(value) },
            &regions,
            &|_| unsafe { step(stepper) },
        );
    }
    restore(&regions, &saved);
}

// ---- selecting a change ----

/// A setup value that costs service time to change: `chosen` is what the keys step,
/// `installed` what the car has.
#[derive(Clone, Copy)]
pub struct Change {
    pub chosen_rva: usize,
    pub installed_rva: usize,
}

/// Flips the flag of the highlighted menu item; if it is then not 1, the chosen value goes
/// back to the installed one.
pub unsafe fn toggle_change(change: Change) -> Result<Left, Refused> {
    let item = flagged_item()?;
    let flag = repair_flag(item) ^ 1;
    set_repair_flag(item, flag);
    if flag != 1 {
        unsafe { game_write(change.chosen_rva, game_read::<u32>(change.installed_rva)) };
    }
    Ok(Left::nothing())
}

fn change_regions(change: Change) -> [Region; 2] {
    [
        (game_addr(REPAIR_FLAGS_RVA) as u32, FLAG_TABLE_LEN),
        at(change.chosen_rva),
    ]
}

pub fn dispatch_change(hook: &RawHook, r: &mut Regs, call_original: CallOriginal, change: Change) {
    dispatch_state(
        hook,
        r,
        call_original,
        &change_regions(change),
        &|_| unsafe { toggle_change(change) },
    );
}

/// Several items and flag values. Everything written is put back.
pub fn check_change(t: &mut SelfTest, call_original: CallOriginal, change: Change) {
    let regions = change_regions(change);
    let inputs = [at(MENU_ITEM_RVA), at(change.installed_rva)];
    let saved = (capture(&regions), capture(&inputs));
    for item in [0u32, 2, 0xE, 0xF, 0x12, 0x13] {
        for flag in [0u32, 1, 2, 3] {
            check_state(
                t,
                &format!("item={item:#x} flag={flag}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    game_write(MENU_ITEM_RVA, item);
                    set_repair_flag(item, flag);
                    game_write(change.chosen_rva, 0x5EED_0001u32);
                    game_write(change.installed_rva, 0x5EED_0002u32);
                },
                &regions,
                &|_| unsafe { toggle_change(change) },
            );
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
}
