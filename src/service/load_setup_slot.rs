//! `service::load_setup_slot` (0x46EECC): the action of a slot on the "load setup" menu:
//! reads the slot's saved car setup and makes it the chosen setup.
//!
//! In:  nothing
//! Out: every register preserved
//! The slot is `dword_755244 * 30 + item - 1`, and one less from item 0x11 on (`dword_755244`
//! is read as the page of the slot list). If `read_setup_slot` fails, nothing more happens.
//! Otherwise: all repair choices are cleared; the file's six values go to the chosen tyre
//! (+0x30), the car's brake balance (+0x34), the chosen ride height (+0x38) and suspension
//! stiffness (+0x3C), the car's steering sensitivity (+0x40) and the chosen gear ratio
//! (+0x44); both flag tables are cleared; then, for gear ratio, tyre, ride height and
//! stiffness in that order, if the chosen value differs from the installed one, its edit page
//! is run as if confirmed, with its menu item highlighted (so the change is selected if the
//! service time allows); the highlighted item is put back, and `menu::go_back` leaves the
//! menu.
//! Refuses, before anything is written: no car object, a slot past 99, a flag table without
//! its end mark.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::go_back;
use crate::screens::harness::Left;
use crate::service::clear_repair_flags::clear_repair_flags;
use crate::service::clear_repair_marks::clear_repair_marks;
use crate::service::edit_gear_ratio::edit_gear_ratio;
use crate::service::edit_ride_height::edit_ride_height;
use crate::service::edit_suspension_stiffness::edit_suspension_stiffness;
use crate::service::edit_tyre::edit_tyre;
use crate::service::harness::{
    Region, RepairTest, at, capture, check_state, dispatch_state, repair_regions, restore,
};
use crate::service::pages::{
    BRAKE_BALANCE, ChangePage, GEAR_RATIO, RIDE_HEIGHT, STEERING_SENSITIVITY, SUSPENSION_STIFFNESS,
    TYRE,
};
use crate::service::read_setup_slot::{
    SETUP_FILE_RVA, SLOT_COUNT, read_setup_slot, regions as slot_regions,
};
use crate::service::state::{
    EDIT_MODE_RVA, ITEM_COUNT, MENU_ITEM_RVA, TIME_BUDGET_RVA, current_car, marks_ended, menu_item,
    repair_flag,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::load_setup_slot", rva(0x46EECC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const SLOT_PAGE_RVA: usize = rva(0x755244); // dword_755244
const SLOTS_PER_PAGE: u32 = 30;

/// The slot the highlighted menu item stands for.
pub fn highlighted_slot() -> u32 {
    let item = menu_item();
    let mut slot = unsafe { game_read::<u32>(SLOT_PAGE_RVA) }
        .wrapping_mul(SLOTS_PER_PAGE)
        .wrapping_add(item)
        .wrapping_sub(1);
    if (item as i32) >= 0x11 {
        slot = slot.wrapping_sub(1);
    }
    slot
}

// ---- Rust version ----

pub unsafe fn load_setup_slot() -> Result<(), Refused> {
    let slot = highlighted_slot();
    let Some(car) = current_car() else {
        return Err(BadInput("no current car object: the original writes through null").into());
    };
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table").into());
    }
    if repair_flag(ITEM_COUNT) != u32::MAX || !marks_ended() {
        return Err(BadInput("a flag table has lost its end mark").into());
    }

    unsafe {
        if !read_setup_slot(slot)? {
            return Ok(());
        }
        clear_repair_flags()?;

        let value = |offset: usize| game_read::<u32>(SETUP_FILE_RVA + offset);
        let in_car = |offset: u32| (car.wrapping_add(offset)) as *mut u32;
        game_write(TYRE.change.chosen_rva, value(0x30));
        in_car(BRAKE_BALANCE.car_offset).write_unaligned(value(0x34));
        game_write(RIDE_HEIGHT.change.chosen_rva, value(0x38));
        game_write(SUSPENSION_STIFFNESS.change.chosen_rva, value(0x3C));
        in_car(STEERING_SENSITIVITY.car_offset).write_unaligned(value(0x40));
        game_write(GEAR_RATIO.change.chosen_rva, value(0x44));

        // The original clears both tables again here, this time checking each entry for the
        // end mark before writing it. With the end marks in place that is the same thing.
        clear_repair_flags()?;
        clear_repair_marks()?;

        let highlighted = menu_item();
        let differs = |page: ChangePage| {
            game_read::<u32>(page.change.chosen_rva) != game_read::<u32>(page.change.installed_rva)
        };
        if differs(GEAR_RATIO) {
            game_write(MENU_ITEM_RVA, GEAR_RATIO.item);
            edit_gear_ratio(true)?;
        }
        if differs(TYRE) {
            game_write(MENU_ITEM_RVA, TYRE.item);
            edit_tyre(true)?;
        }
        if differs(RIDE_HEIGHT) {
            game_write(MENU_ITEM_RVA, RIDE_HEIGHT.item);
            edit_ride_height(true)?;
        }
        if differs(SUSPENSION_STIFFNESS) {
            game_write(MENU_ITEM_RVA, SUSPENSION_STIFFNESS.item);
            edit_suspension_stiffness(true)?;
        }
        game_write(MENU_ITEM_RVA, highlighted);
        go_back::go_back();
    }
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = slot_regions(highlighted_slot());
    regions.extend(repair_regions());
    regions.extend(go_back::regions());
    regions.push(at(MENU_ITEM_RVA));
    regions.push(at(EDIT_MODE_RVA));
    for page in [GEAR_RATIO, TYRE, RIDE_HEIGHT, SUSPENSION_STIFFNESS] {
        regions.push(at(page.change.chosen_rva));
    }
    if let Some(car) = current_car() {
        regions.push((car + BRAKE_BALANCE.car_offset, 4));
        regions.push((car + STEERING_SENSITIVITY.car_offset, 4));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { load_setup_slot() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the first 30 slots, with whatever setup files the owner has saved ----
//
// Slots without a file only show that nothing happens. Each slot is tried with a service time
// that lets every change in, one that lets them in only by overrunning, and none.

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if current_car().is_none() {
        return t.skip("there is no current car object");
    }
    let test = RepairTest::begin();
    RepairTest::set_known_times();
    let inputs = [at(SLOT_PAGE_RVA)];
    let saved_inputs = capture(&inputs);

    // Items 1 .. 31 on page 0 are slots 0 .. 29 (items 0x10 and 0x11 are both slot 15).
    for item in 1u32..=31 {
        unsafe {
            game_write(SLOT_PAGE_RVA, 0u32);
            game_write(MENU_ITEM_RVA, item);
        }
        let regions = regions();
        let saved = capture(&regions);
        for budget in [100_000i32, 2000, 0] {
            check_state(
                &mut t,
                &format!("item={item} budget={budget}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    restore(&regions, &saved);
                    unsafe {
                        game_write(MENU_ITEM_RVA, item);
                        game_write(TIME_BUDGET_RVA, budget);
                    }
                },
                &regions,
                &rust,
            );
        }
        restore(&regions, &saved);
    }

    restore(&inputs, &saved_inputs);
    test.end();
    t.finish()
}
