//! What the service area's small value functions share: the twelve that step a setup value
//! one up or down, and the four that select or deselect a change.

use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::menu::owner::Menus;
use crate::service::owner::Service;
use crate::service::owner::SetupItem;
use crate::service::state::current_player;

#[derive(Clone, Copy)]
pub enum Target {
    /// The chosen value of a setup item (`setups[n].chosen`).
    Chosen(usize),
    /// A setup value of the current player's car.
    CarField(rc2k_core::menu::player::SetupField),
}

/// One step down stops at 0 (a value that is already negative goes on down; kept); one step
/// up stops at `max`, and a value above `max` becomes `max`.
#[derive(Clone, Copy)]
pub struct Stepper {
    pub target: Target,
    pub max: i32,
    pub up: bool,
}

/// The stepped value, read and written where the target says.
fn with_target(
    menus: &Menus,
    stepper: Stepper,
    f: impl FnOnce(u32) -> u32,
) -> Result<(), BadInput> {
    match stepper.target {
        Target::Chosen(setup) => {
            let cell = &setup_item(&menus.service, setup).chosen;
            cell.set(f(cell.get()));
        }
        Target::CarField(field) => {
            let player = current_player(menus).ok_or(BadInput(
                "no current player object: the original reads through null",
            ))?;
            player.update(|q| {
                let value = field.of(q);
                *value = f(*value);
            });
        }
    }
    Ok(())
}

pub fn step(menus: &Menus, stepper: Stepper) -> Result<(), Refused> {
    with_target(menus, stepper, |value| {
        rc2k_core::service::repairs::step(value as i32, stepper.max, stepper.up) as u32
    })?;
    Ok(())
}

/// A setup value that costs service time to change: `chosen` is what the keys step,
/// `installed` what the car has.
#[derive(Clone, Copy)]
pub struct Change {
    pub setup: usize,
}

impl Change {
    pub fn chosen<'a>(&self, service: &'a Service) -> &'a PlainDword {
        &setup_item(service, self.setup).chosen
    }
    pub fn installed<'a>(&self, service: &'a Service) -> &'a PlainDword {
        &setup_item(service, self.setup).installed
    }
    /// The chosen value as it was when the item's edit started.
    pub fn start<'a>(&self, service: &'a Service) -> &'a PlainDword {
        &setup_item(service, self.setup).start
    }
}

/// The setup items: gear ratio, tyre, ride height, suspension stiffness.
pub fn setup_item(service: &Service, setup: usize) -> &SetupItem {
    &service.setups[setup]
}
