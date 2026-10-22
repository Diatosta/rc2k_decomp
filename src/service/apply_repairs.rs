//! `service::apply_repairs` (0x46C14D): carries out the work chosen in the service area, on
//! leaving it: each chosen repair is shown being done, then the chosen setup is installed.
//!
//! In:  nothing
//! Out: every register preserved
//! What it does, in order (a "chosen" item is one whose flag in `dword_759B6E` is exactly 1):
//!   - `dword_75AB38` = 1;
//!   - runs of the car's dwords are zeroed for five chosen items: gearbox (0xE) ten at +0x17C,
//!     body (3) six at +0x1A4, lights (7) seven at +0x1BC, suspension (0xB) five at +0x1D8,
//!     brakes (0xD) three at +0x1EC; for the tyres (0xF) `reset_tyre_wear`, keeping the
//!     damage at +0x108;
//!   - for the items 2, 4, 6, 8, 0xA, 0xC, 0xE, 3, 5, 7, 9, 0xB, 0xD, 0xF, in that order, if
//!     chosen: frames are run, and each time 5 ms or more have passed 2 is taken off the
//!     car's damage (+0xCC + 4 * item) until it is 0 or less; then the damage is 0, the flag
//!     is cleared, the gearbox also installs the chosen gear ratio (car +0x120) and the tyres
//!     the chosen tyre (+0x10C); then 500 ms more of frames;
//!   - ride height (0x12) and suspension stiffness (0x13), if chosen: flag cleared, chosen
//!     value installed (car +0x114, +0x118), 500 ms of frames. (The original has a countdown
//!     here too, but jumps over it.)
//!   - tyre damage 0 sets bits 0x1E in the car's dword at +0x278; body damage 0 sets all its
//!     other bits and zeroes eight dwords at +0x24C;
//!   - `sleep(1000)`.
//!
//! A frame is the game's frame function `sub_405A71` with the service area's draw callback
//! `sub_46CFD7`; neither is ported, so the Rust version calls them with `call_game`. The
//! clock is `tick_count`. They come in through `Driver`, so that the logic can be run and
//! compared without real frames.
//! Called from `sub_48E0FD` and `sub_4A16AB`. Refuses a missing player.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::service::pages::{GEAR_RATIO, RIDE_HEIGHT, SUSPENSION_STIFFNESS, TYRE};
use crate::service::reset_tyre_wear::reset_tyre_wear;
use crate::service::state::{current_player, repair_flag, set_repair_flag};
use crate::system::sleep::sleep;
use crate::system::tick_count::tick_count;

// ---- game data ----

const WORKING_RVA: usize = rva(0x75AB38); // dword_75AB38: set to 1 here
const START_RVA: usize = rva(0x75AB48); // dword_75AB48: the clock when a phase started
const ELAPSED_RVA: usize = rva(0x75AB4C); // dword_75AB4C
const LAST_STEP_RVA: usize = rva(0x75AB54); // dword_75AB54: `elapsed` at the last step

const FRAME_FUNCTION_RVA: usize = rva(0x405A71); // sub_405A71
const DRAW_CALLBACK_RVA: usize = rva(0x46CFD7); // sub_46CFD7

// Converted: `rc2k_core::service::workshop` (the car's offsets, the items, the order).
use rc2k_core::service::workshop::{Setting, Timer, Workshop};
// ---- Rust version ----

/// Where time, frames and sleep come from.
pub trait Driver {
    fn clock(&mut self) -> u32;
    fn frame(&mut self);
    fn sleep(&mut self, milliseconds: u32);
}

/// The game's own.
pub struct GameDriver;

impl Driver for GameDriver {
    fn clock(&mut self) -> u32 {
        tick_count()
    }
    fn frame(&mut self) {
        let mut regs = Regs {
            eax: game_addr(DRAW_CALLBACK_RVA) as u32,
            ..Regs::default()
        };
        crate::calls::call(FRAME_FUNCTION_RVA, &mut regs);
    }
    fn sleep(&mut self, milliseconds: u32) {
        sleep(milliseconds);
    }
}

unsafe fn field(player: u32, offset: u32) -> *mut i32 {
    player.wrapping_add(offset) as *mut i32
}

/// The work on the game's variables and the current player's car, with `driver`'s time,
/// frames and sleep.
struct Game<'a> {
    player: u32,
    driver: &'a mut dyn Driver,
}

impl Workshop for Game<'_> {
    type Error = Refused;
    fn flag(&mut self, item: u32) -> u32 {
        repair_flag(item)
    }
    fn set_flag(&mut self, item: u32, value: u32) {
        set_repair_flag(item, value);
    }
    fn car(&mut self, offset: u32) -> u32 {
        unsafe { field(self.player, offset).read_unaligned() as u32 }
    }
    fn set_car(&mut self, offset: u32, value: u32) {
        unsafe { field(self.player, offset).write_unaligned(value as i32) };
    }
    fn set_working(&mut self) {
        unsafe { game_write(WORKING_RVA, 1u32) };
    }
    fn timer(&mut self, t: Timer) -> u32 {
        unsafe { game_read(timer_rva(t)) }
    }
    fn set_timer(&mut self, t: Timer, value: u32) {
        unsafe { game_write(timer_rva(t), value) };
    }
    fn reset_tyre_wear(&mut self) -> Result<(), Refused> {
        Ok(unsafe { reset_tyre_wear() }?)
    }
    fn install(&mut self, s: Setting) -> u32 {
        let page = match s {
            Setting::GearRatio => GEAR_RATIO,
            Setting::Tyre => TYRE,
            Setting::RideHeight => RIDE_HEIGHT,
            Setting::Stiffness => SUSPENSION_STIFFNESS,
        };
        unsafe {
            let value: u32 = game_read(page.change.chosen_rva);
            game_write(page.change.installed_rva, value);
            value
        }
    }
    fn clock(&mut self) -> u32 {
        self.driver.clock()
    }
    fn frame(&mut self) {
        self.driver.frame();
    }
    fn sleep(&mut self, milliseconds: u32) {
        self.driver.sleep(milliseconds);
    }
}

fn timer_rva(t: Timer) -> usize {
    match t {
        Timer::Start => START_RVA,
        Timer::Elapsed => ELAPSED_RVA,
        Timer::LastStep => LAST_STEP_RVA,
    }
}

pub unsafe fn apply_repairs(driver: &mut dyn Driver) -> Result<(), Refused> {
    let Some(player) = current_player() else {
        return Err(BadInput("no current player object: the original writes through null").into());
    };
    // Converted: `rc2k_core::service::workshop::apply_repairs`.
    rc2k_core::service::workshop::apply_repairs(&mut Game { player, driver })
}

/// For Rust callers: the Rust version, or, for a missing player or a refusal, the original
/// (as the hook did; it has not written anything then). Every register kept.
pub unsafe fn apply_repairs_in_game(r: &mut Regs) {
    unsafe {
        if current_player().is_none() || apply_repairs(&mut GameDriver).is_err() {
            crate::calls::call(rva(0x46C14D), r);
        }
    }
}
