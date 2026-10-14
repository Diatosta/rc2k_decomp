//! The six edit pages of the service area: what the keys do while one of the setup values or
//! the tyre or gear ratio choice is being edited ("locked", in the original's error text).
//! Key 0x68 steps the value down, 0x69 up, 0x1C confirms. Every key also copies
//! `dword_51899C` (a clock) into `dword_7556C4`.
//!
//! Two pages (steering sensitivity, brake balance) change the car directly: confirming just
//! leaves edit mode. The other four cost service time; confirming goes through this:
//!   - ride height and stiffness only: if the chosen value is what the car has, the item's
//!     flag is cleared and that is all;
//!   - if the value was not changed in this edit (it equals the value at entry), the item's
//!     selection is flipped (`toggle_change`), and unless that selected it, that is all;
//!   - the flag is cleared and `try_toggle_repair` says whether selecting it fits:
//!     - status 2 (no time left at all): the chosen value goes back to the installed one;
//!     - status 1 (fits only by overrunning): selected and marked (`choose_repair_marked`);
//!     - status 3 (fits): the marks are cleared and it is selected (`choose_repair`).
//!
//! Whatever happened, the four end with `update_repair_total`.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::input::take_key::{PENDING_KEY_RVA, pending_key, take_key};
use crate::screens::harness::CallOriginal;
use crate::service::choose_repair::choose_repair;
use crate::service::choose_repair_marked::choose_repair_marked;
use crate::service::clear_repair_marks::clear_repair_marks;
use crate::service::harness::{
    Region, RepairTest, Rust, at, capture, check_state_with, repair_regions, restore, set_flags,
};
use crate::service::setup_values::{Change, Stepper, step, toggle_change};
use crate::service::state::{
    EDIT_MODE_RVA, EDIT_PAGE_RVA, MENU_ITEM_RVA, STATUS_RVA, TIME_BUDGET_RVA, chosen_repairs_time,
    current_player, flagged_item, has_repair_time, leave_edit_mode, marks_ended, repair_flag,
    set_repair_flag,
};
use crate::service::try_toggle_repair::try_toggle_repair;
use crate::service::update_repair_total::update_repair_total;
use crate::service::{
    step_brake_balance_down, step_brake_balance_up, step_gear_ratio_down, step_gear_ratio_up,
    step_ride_height_down, step_ride_height_up, step_steering_sensitivity_down,
    step_steering_sensitivity_up, step_suspension_stiffness_down, step_suspension_stiffness_up,
    step_tyre_down, step_tyre_up, toggle_gear_ratio_change, toggle_ride_height_change,
    toggle_suspension_stiffness_change, toggle_tyre_change,
};
use std::sync::atomic::Ordering;

pub const KEY_DOWN: u8 = 0x68;
pub const KEY_UP: u8 = 0x69;
pub const KEY_CONFIRM: u8 = 0x1C;

const CLOCK_RVA: usize = rva(0x51899C); // dword_51899C
const LAST_KEY_TIME_RVA: usize = rva(0x7556C4); // dword_7556C4: the clock at the last key

/// A page whose value is changed in the car itself.
#[derive(Clone, Copy)]
pub struct SimplePage {
    pub down: Stepper,
    pub up: Stepper,
    /// Offset of the value in the player.
    pub car_offset: u32,
}

/// A page whose value costs service time to change.
#[derive(Clone, Copy)]
pub struct ChangePage {
    pub down: Stepper,
    pub up: Stepper,
    pub change: Change,
    /// The chosen value as it was when the edit started.
    pub entry_rva: usize,
    /// Where the car has this value, for the pages that compare with it.
    pub car_offset: Option<u32>,
    /// The page's menu item (the self-tests highlight it).
    pub item: u32,
}

pub const STEERING_SENSITIVITY: SimplePage = SimplePage {
    down: step_steering_sensitivity_down::STEPPER,
    up: step_steering_sensitivity_up::STEPPER,
    car_offset: 0x11C,
};
pub const BRAKE_BALANCE: SimplePage = SimplePage {
    down: step_brake_balance_down::STEPPER,
    up: step_brake_balance_up::STEPPER,
    car_offset: 0x110,
};
pub const GEAR_RATIO: ChangePage = ChangePage {
    down: step_gear_ratio_down::STEPPER,
    up: step_gear_ratio_up::STEPPER,
    change: toggle_gear_ratio_change::CHANGE,
    entry_rva: rva(0x75AC98), // dword_75AC98
    car_offset: None,
    item: 0xE,
};
pub const TYRE: ChangePage = ChangePage {
    down: step_tyre_down::STEPPER,
    up: step_tyre_up::STEPPER,
    change: toggle_tyre_change::CHANGE,
    entry_rva: rva(0x75ACA4), // dword_75ACA4
    car_offset: None,
    item: 0xF,
};
pub const RIDE_HEIGHT: ChangePage = ChangePage {
    down: step_ride_height_down::STEPPER,
    up: step_ride_height_up::STEPPER,
    change: toggle_ride_height_change::CHANGE,
    entry_rva: rva(0x75ACB0), // dword_75ACB0
    car_offset: Some(0x114),
    item: 0x12,
};
pub const SUSPENSION_STIFFNESS: ChangePage = ChangePage {
    down: step_suspension_stiffness_down::STEPPER,
    up: step_suspension_stiffness_up::STEPPER,
    change: toggle_suspension_stiffness_change::CHANGE,
    entry_rva: rva(0x75ACBC), // dword_75ACBC
    car_offset: Some(0x118),
    item: 0x13,
};

fn player() -> Result<u32, BadInput> {
    current_player().ok_or(BadInput(
        "no current player object: the original reads through null",
    ))
}

/// The key, with the clock noted and the value stepped if it was an arrow. 0 = no key.
unsafe fn key_and_step(down: Stepper, up: Stepper) -> Result<u8, Refused> {
    let key = take_key();
    if key == 0 {
        return Ok(0);
    }
    unsafe {
        game_write(LAST_KEY_TIME_RVA, game_read::<u32>(CLOCK_RVA));
        if key == KEY_DOWN {
            step(down)?;
        }
        if key == KEY_UP {
            step(up)?;
        }
    }
    Ok(key)
}

pub unsafe fn simple_page(page: SimplePage) -> Result<(), Refused> {
    // Nothing may be written before it is known that nothing will be refused.
    player()?;
    if unsafe { key_and_step(page.down, page.up) }? == KEY_CONFIRM {
        leave_edit_mode();
    }
    Ok(())
}

/// `confirm`: act as if the confirm key had been pressed, without taking a key (the callers
/// outside the key handler pass ebx = 1 for this).
pub unsafe fn change_page(page: ChangePage, confirm: bool) -> Result<(), Refused> {
    // Everything the steps below could refuse is checked first, so that an error never
    // leaves half of the work done.
    let item = flagged_item()?;
    if !has_repair_time(item) {
        return Err(BadInput("the highlighted item has no repair time").into());
    }
    chosen_repairs_time()?;
    if !marks_ended() {
        return Err(BadInput("the second flag table has lost its end mark").into());
    }
    let installed_in_car = match page.car_offset {
        Some(offset) => Some((player()?.wrapping_add(offset)) as *const u32),
        None => None,
    };

    unsafe {
        if !confirm && key_and_step(page.down, page.up)? != KEY_CONFIRM {
            return update_repair_total();
        }

        let chosen: u32 = game_read(page.change.chosen_rva);
        if installed_in_car.is_some_and(|value| value.read_unaligned() == chosen) {
            set_repair_flag(item, 0);
            leave_edit_mode();
            return update_repair_total();
        }
        if game_read::<u32>(page.entry_rva) == chosen {
            leave_edit_mode();
            toggle_change(page.change)?;
            if repair_flag(item) != 1 {
                return update_repair_total();
            }
        }
        set_repair_flag(item, 0);
        try_toggle_repair()?;
        match game_read::<u32>(STATUS_RVA) {
            2 => {
                game_write(
                    page.change.chosen_rva,
                    game_read::<u32>(page.change.installed_rva),
                );
                leave_edit_mode();
            }
            1 => {
                choose_repair_marked()?;
                leave_edit_mode();
            }
            _ => {
                leave_edit_mode();
                clear_repair_marks()?;
                choose_repair()?;
            }
        }
        update_repair_total()
    }
}

// ---- hook side ----

/// Everything a page may write except the pending key: the edit mode, the flag tables with
/// status and total, the chosen values and the car's values. The copy of the clock is left
/// out: the clock runs.
pub fn page_regions(chosen_rvas: &[usize], car_offsets: &[u32]) -> Vec<Region> {
    let mut regions = vec![at(EDIT_MODE_RVA)];
    regions.extend(repair_regions());
    regions.extend(chosen_rvas.iter().map(|&rva| at(rva)));
    if let Some(player) = current_player() {
        regions.extend(
            car_offsets
                .iter()
                .map(|&offset| (player.wrapping_add(offset), 4)),
        );
    }
    regions
}

pub fn simple_regions(page: SimplePage) -> Vec<Region> {
    page_regions(&[], &[page.car_offset])
}

pub fn change_regions(page: ChangePage) -> Vec<Region> {
    page_regions(&[page.change.chosen_rva], &[])
}

/// The pending key, which the self-tests compare as well (they set it themselves).
pub fn with_key(mut regions: Vec<Region>) -> Vec<Region> {
    regions.push((game_addr(PENDING_KEY_RVA) as u32, 1));
    regions
}

// ---- self-tests ----

const UNSET: u32 = 0x5EED_0003;

/// 1 if the clock was copied since `prepare`.
fn clock_noted() -> Vec<u8> {
    vec![(unsafe { game_read::<u32>(LAST_KEY_TIME_RVA) } != UNSET) as u8]
}

const KEYS: [u8; 5] = [0, KEY_DOWN, KEY_UP, KEY_CONFIRM, 0x41];

/// What a page test writes and puts back.
pub struct PageTest {
    repairs: RepairTest,
    regions: Vec<Region>,
    saved: Vec<u8>,
}

impl PageTest {
    /// None if there is no player.
    pub fn begin() -> Option<Self> {
        let player = current_player()?;
        let mut regions = vec![
            (game_addr(PENDING_KEY_RVA) as u32, 1),
            at(EDIT_MODE_RVA),
            at(EDIT_PAGE_RVA),
            at(LAST_KEY_TIME_RVA),
        ];
        for page in [GEAR_RATIO, TYRE, RIDE_HEIGHT, SUSPENSION_STIFFNESS] {
            regions.push(at(page.change.chosen_rva));
            regions.push(at(page.change.installed_rva));
            regions.push(at(page.entry_rva));
        }
        for offset in [0x110, 0x114, 0x118, 0x11C] {
            regions.push((player + offset, 4));
        }
        let repairs = RepairTest::begin();
        let saved = capture(&regions);
        RepairTest::set_known_times();
        Some(Self {
            repairs,
            regions,
            saved,
        })
    }

    pub fn end(self) {
        restore(&self.regions, &self.saved);
        self.repairs.end();
    }
}

/// The values of a change page: (chosen, at entry, installed, in the car).
pub const SCENARIOS: [(&str, [u32; 4]); 4] = [
    ("untouched", [3, 3, 3, 3]),
    ("changed now", [3, 2, 2, 2]),
    ("changed before", [3, 3, 1, 1]),
    ("back to the player's", [3, 2, 2, 3]),
];

/// Budgets that give status 3, 1 and 2 with the known times.
pub const BUDGETS: [i32; 3] = [100_000, 2000, 1000];

/// Puts a change page's inputs in place. Item 2 is always chosen besides.
pub fn prepare_change(page: ChangePage, key: u8, values: [u32; 4], flag: u32, budget: i32) {
    set_flags(&[2], 1);
    set_repair_flag(page.item, flag);
    unsafe {
        game_write(MENU_ITEM_RVA, page.item);
        game_write(TIME_BUDGET_RVA, budget);
        game_write(EDIT_MODE_RVA, 1u32);
        game_write(STATUS_RVA, 0x5EED_0002u32);
        game_write(LAST_KEY_TIME_RVA, UNSET);
        game_write(page.change.chosen_rva, values[0]);
        game_write(page.entry_rva, values[1]);
        game_write(page.change.installed_rva, values[2]);
        if let (Some(offset), Some(player)) = (page.car_offset, current_player()) {
            ((player + offset) as *mut u32).write_unaligned(values[3]);
        }
    }
    pending_key().store(key, Ordering::SeqCst);
}

pub fn prepare_simple(page: SimplePage, key: u8, value: u32) {
    unsafe {
        game_write(EDIT_MODE_RVA, 1u32);
        game_write(LAST_KEY_TIME_RVA, UNSET);
        if let Some(player) = current_player() {
            ((player + page.car_offset) as *mut u32).write_unaligned(value);
        }
    }
    pending_key().store(key, Ordering::SeqCst);
}

pub fn check_change_page(
    t: &mut SelfTest,
    call_original: CallOriginal,
    page: ChangePage,
    rust: Rust,
) {
    let Some(test) = PageTest::begin() else {
        return;
    };
    let regions = with_key(change_regions(page));
    for key in KEYS {
        for confirm in [0u32, 1] {
            for (name, values) in SCENARIOS {
                for flag in [0u32, 1] {
                    for budget in BUDGETS {
                        let before = Regs {
                            ebx: confirm,
                            ..Regs::SENTINEL
                        };
                        check_state_with(
                            t,
                            &format!(
                                "key={key:#x} ebx={confirm} {name} flag={flag} budget={budget}"
                            ),
                            call_original,
                            before,
                            &|| prepare_change(page, key, values, flag, budget),
                            &regions,
                            rust,
                            &clock_noted,
                        );
                    }
                }
            }
        }
    }
    test.end();
}

pub fn check_simple_page(
    t: &mut SelfTest,
    call_original: CallOriginal,
    page: SimplePage,
    rust: Rust,
) {
    let Some(test) = PageTest::begin() else {
        return;
    };
    let regions = with_key(simple_regions(page));
    for key in KEYS {
        for value in [0u32, 1, 50, 100] {
            check_state_with(
                t,
                &format!("key={key:#x} value={value}"),
                call_original,
                Regs::SENTINEL,
                &|| prepare_simple(page, key, value),
                &regions,
                rust,
                &clock_noted,
            );
        }
    }
    test.end();
}

/// Why the pages cannot be tested now, or None.
pub fn pages_unavailable() -> Option<&'static str> {
    current_player()
        .is_none()
        .then_some("there is no current player object")
}

/// The observer the dispatcher's test shares.
pub fn observe_clock() -> Vec<u8> {
    clock_noted()
}

pub const TEST_KEYS: [u8; 5] = KEYS;
