//! What the service area's small value functions share: the twelve that step a setup value
//! one up or down, and the four that select or deselect a change.

use crate::errors::{BadInput, Refused};
use crate::service::state::current_player;
use crate::session::Dword;

// ---- steppers ----

#[derive(Clone, Copy)]
pub enum Target {
    /// A game variable.
    Global(&'static Dword),
    /// A dword of the current player, by offset.
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

/// The stepped value, read and written where the target says.
fn with_target(stepper: Stepper, f: impl FnOnce(u32) -> u32) -> Result<(), BadInput> {
    match stepper.target {
        Target::Global(cell) => cell.set(f(cell.get())),
        Target::CarField(offset) => {
            let player = current_player().ok_or(BadInput(
                "no current player object: the original reads through null",
            ))?;
            let next = f(player.get().word(offset as usize));
            player.update(|q| q.store(&[(offset as usize, next)]));
        }
    }
    Ok(())
}

pub fn step(stepper: Stepper) -> Result<(), Refused> {
    // Converted: `rc2k_core::service::repairs::step`.
    with_target(stepper, |value| {
        rc2k_core::service::repairs::step(value as i32, stepper.max, stepper.up) as u32
    })?;
    Ok(())
}

// ---- selecting a change ----

/// A setup value that costs service time to change: `chosen` is what the keys step,
/// `installed` what the car has.
#[derive(Clone, Copy)]
pub struct Change {
    pub chosen: &'static Dword,
    pub installed: &'static Dword,
}
