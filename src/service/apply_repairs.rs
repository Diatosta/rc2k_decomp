//! Carries out the work chosen in the service area on leaving it: each chosen repair is shown
//! being done, then the chosen setup is installed.
//!
//! The car's dword runs of the chosen gearbox, body, lights, suspension and brakes are zeroed
//! (the tyres through `reset_tyre_wear`); then each chosen repair, in the workshop's order, runs
//! frames until its damage has counted down to 0 (2 off per 5 ms or more), clears its flag and
//! installs the chosen gear ratio or tyre, followed by 500 ms more of frames. Ride height and
//! suspension stiffness, if chosen, install their value and run 500 ms of frames. Frames, clock
//! and sleep come in through `Driver`. Refuses a missing player.

use crate::clock::tick_count::tick_count;
use crate::errors::{BadInput, Refused};
use crate::memory::session::DwordLike;
use crate::owners::Owners;
use crate::service::pages::{GEAR_RATIO, RIDE_HEIGHT, SUSPENSION_STIFFNESS, TYRE};
use crate::service::reset_tyre_wear::reset_tyre_wear;
use crate::service::state::{current_player, repair_flag_in, set_repair_flag};
use rc2k_core::fatal::Context;

use crate::menu::owner::Menus;
use rc2k_core::service::workshop::{Setting, Timer, Workshop};

/// Where time, frames and sleep come from.
pub trait Driver {
    fn clock(&mut self) -> u32;
    fn frame(&mut self);
    fn sleep(&mut self, milliseconds: u32);
    /// Whether a key was pressed (it is taken).
    fn key_pressed(&mut self) -> bool {
        false
    }
}

/// The real clock, frames and sleep.
pub struct GameDriver {
    o: Owners<'static>,
}

impl Driver for GameDriver {
    fn clock(&mut self) -> u32 {
        tick_count(&self.o.system.session.picture_clock)
    }
    /// `screens::run_frame` with the service area's frame (`screens::service_area_frame`).
    fn frame(&mut self) {
        let ran = crate::shell::frame::run_frame_now(self.o, || {
            crate::screens::service_area_frame::service_area_frame(self.o)
        });
        if let Some(Err(e)) = ran {
            crate::menu::table_calls::refused(
                "screens::service_area_frame",
                e,
                &Context::default(),
            );
        }
    }
    /// A pause a key ends, the window's events going on.
    fn sleep(&mut self, milliseconds: u32) {
        crate::system::pause::pause(self.o, milliseconds);
    }
    fn key_pressed(&mut self) -> bool {
        crate::input::take_key::take_key(self.o.input) != 0
    }
}

/// A clock that jumps a second at every reading, with no frames and no sleeping: the work goes
/// through at once (a launch's tune, with no screen to show it).
#[derive(Default)]
struct QuietDriver {
    now: u32,
}

impl Driver for QuietDriver {
    fn clock(&mut self) -> u32 {
        self.now = self.now.wrapping_add(1000);
        self.now
    }
    fn frame(&mut self) {}
    fn sleep(&mut self, _milliseconds: u32) {}
}

/// The chosen work carried out as `apply_repairs` does, without showing it.
pub fn apply_repairs_unshown(menus: &Menus) -> Result<(), Refused> {
    apply_repairs(menus, &mut QuietDriver::default())
}

/// The work on the current player's car, with `driver`'s time, frames and sleep.
struct Game<'a> {
    menus: &'a Menus,
    player: crate::menu::standings::PlayerCell<'a>,
    driver: &'a mut dyn Driver,
}

impl Workshop for Game<'_> {
    type Error = Refused;
    fn flag(&mut self, item: u32) -> u32 {
        repair_flag_in(&self.menus.service, item)
    }
    fn set_flag(&mut self, item: u32, value: u32) {
        set_repair_flag(&self.menus.service, item, value);
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
        timer(self.menus, t).get()
    }
    fn set_timer(&mut self, t: Timer, value: u32) {
        timer(self.menus, t).set(value);
    }
    fn reset_tyre_wear(&mut self) -> Result<(), Refused> {
        Ok({ reset_tyre_wear(self.menus) }?)
    }
    fn install(&mut self, s: Setting) -> u32 {
        let page = match s {
            Setting::GearRatio => GEAR_RATIO,
            Setting::Tyre => TYRE,
            Setting::RideHeight => RIDE_HEIGHT,
            Setting::Stiffness => SUSPENSION_STIFFNESS,
        };
        let value = page.change.chosen(&self.menus.service).get();
        page.change.installed(&self.menus.service).set(value);
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
    fn key_pressed(&mut self) -> bool {
        self.driver.key_pressed()
    }
}

fn timer(menus: &Menus, t: Timer) -> &dyn DwordLike {
    let screens = &menus.screens;
    match t {
        Timer::Start => &screens.state.menus.fade_start,
        Timer::Elapsed => &screens.state.menus.fade_gone,
        Timer::LastStep => &menus.service.repair_last_step,
    }
}

pub fn apply_repairs(menus: &Menus, driver: &mut dyn Driver) -> Result<(), Refused> {
    let Some(player) = current_player(menus) else {
        return Err(BadInput("no current player object: the original writes through null").into());
    };
    rc2k_core::service::workshop::apply_repairs(&mut Game {
        menus,
        player,
        driver,
    })
}

/// The repairs with the real clock, frames and sleep. A refusal is the caller's.
pub fn apply_repairs_in_game_in(o: Owners<'static>) -> Result<(), Refused> {
    apply_repairs(o.menus, &mut GameDriver { o })
}
