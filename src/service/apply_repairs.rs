//! `service::apply_repairs` (0x46C14D): carries out the work chosen in the service area, on
//! leaving it: each chosen repair is shown being done, then the chosen setup is installed.
//!
//! What it does, in order (a "chosen" item is one whose flag in `inspection_flags_a` is exactly 1):
//!   - `service_flag` = 1;
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
//! A frame is the game's frame function 0x405A71 with the service area's draw callback
//! `screens::service_area_frame`; neither is ported, so this code calls them through
//! `Driver`. The clock is
//! `tick_count`. They come in through `Driver`, so that the logic can be run and compared without
//! real frames. Called from 0x48E0FD and 0x4A16AB. Refuses a missing player.

use crate::errors::{BadInput, Refused};
use crate::service::pages::{GEAR_RATIO, RIDE_HEIGHT, SUSPENSION_STIFFNESS, TYRE};
use crate::service::reset_tyre_wear::reset_tyre_wear;
use crate::service::state::{current_player_of, repair_flag_in, set_repair_flag};
use crate::session::DwordLike;
use crate::system::sleep::sleep;
use crate::system::tick_count::tick_count;
use rc2k_core::fatal::Context;

// ---- game data ----

// Converted: `rc2k_core::service::workshop` (the items, the order, the car's fields).
use crate::menu::owner::{Menus, Screens};
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
    /// `screens::run_frame` with the service area's frame (`screens::service_area_frame`).
    fn frame(&mut self) {
        let ran = crate::platform::run_frame_now(|| {
            crate::screens::service_area_frame::service_area_frame(crate::menu::owner::menus())
        });
        if let Some(Err(e)) = ran {
            crate::menu::table_calls::refused(
                "screens::service_area_frame",
                e,
                &Context::default(),
            );
        }
    }
    fn sleep(&mut self, milliseconds: u32) {
        sleep(milliseconds);
    }
}

/// The work on the game's variables and the current player's car, with `driver`'s time,
/// frames and sleep.
struct Game<'a> {
    menus: &'a Menus,
    player: crate::menu::standings::PlayerCell<'a>,
    driver: &'a mut dyn Driver,
}

impl Workshop for Game<'_> {
    type Error = Refused;
    fn flag(&mut self, item: u32) -> u32 {
        repair_flag_in(&self.menus.tables, item)
    }
    fn set_flag(&mut self, item: u32, value: u32) {
        set_repair_flag(&self.menus.tables, item, value);
    }
    fn car(&mut self) -> rc2k_core::menu::player::Player {
        self.player.get()
    }
    fn update_car(&mut self, change: impl FnOnce(&mut rc2k_core::menu::player::Player)) {
        self.player.update(change);
    }
    fn set_working(&mut self) {
        self.menus.screens.state.menus.service_flag.set(true);
    }
    fn timer(&mut self, t: Timer) -> u32 {
        timer(&self.menus.screens, t).get()
    }
    fn set_timer(&mut self, t: Timer, value: u32) {
        timer(&self.menus.screens, t).set(value);
    }
    fn reset_tyre_wear(&mut self) -> Result<(), Refused> {
        Ok({ reset_tyre_wear() }?)
    }
    fn install(&mut self, s: Setting) -> u32 {
        let page = match s {
            Setting::GearRatio => GEAR_RATIO,
            Setting::Tyre => TYRE,
            Setting::RideHeight => RIDE_HEIGHT,
            Setting::Stiffness => SUSPENSION_STIFFNESS,
        };
        let value = page.change.chosen.get();
        page.change.installed.set(value);
        value
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

fn timer(screens: &Screens, t: Timer) -> &dyn DwordLike {
    match t {
        Timer::Start => &screens.state.menus.fade_start,
        Timer::Elapsed => &screens.state.menus.fade_gone,
        Timer::LastStep => &screens.state.menus.repair_last_step,
    }
}

pub fn apply_repairs(menus: &Menus, driver: &mut dyn Driver) -> Result<(), Refused> {
    let Some(player) = current_player_of(&menus.screens) else {
        return Err(BadInput("no current player object: the original writes through null").into());
    };
    // Converted: `rc2k_core::service::workshop::apply_repairs`.
    rc2k_core::service::workshop::apply_repairs(&mut Game {
        menus,
        player,
        driver,
    })
}

/// For Rust callers: the repairs with the game's clock, frames and sleep. A refusal is the
/// caller's.
pub fn apply_repairs_in_game(menus: &Menus) -> Result<(), Refused> {
    apply_repairs(menus, &mut GameDriver)
}
