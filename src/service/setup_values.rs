//! What the service area's small value functions share: the twelve that step a setup value
//! one up or down, and the four that select or deselect a change.

use crate::errors::{BadInput, Refused};
use crate::framework::game_addr;
use crate::service::state::current_player;

// ---- steppers ----

#[derive(Clone, Copy)]
pub enum Target {
    /// A game variable, by RVA.
    Global(usize),
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

fn target(stepper: Stepper) -> Result<u32, BadInput> {
    match stepper.target {
        Target::Global(rva) => Ok(game_addr(rva) as u32),
        Target::CarField(offset) => {
            let player = current_player().ok_or(BadInput(
                "no current player object: the original reads through null",
            ))?;
            Ok(player.wrapping_add(offset))
        }
    }
}

pub unsafe fn step(stepper: Stepper) -> Result<(), Refused> {
    let addr = target(stepper)?;
    let value = addr as *mut i32;
    unsafe {
        // Converted: `rc2k_core::service::repairs::step`.
        let next =
            rc2k_core::service::repairs::step(value.read_unaligned(), stepper.max, stepper.up);
        value.write_unaligned(next);
    }
    Ok(())
}

// ---- selecting a change ----

/// A setup value that costs service time to change: `chosen` is what the keys step,
/// `installed` what the car has.
#[derive(Clone, Copy)]
pub struct Change {
    pub chosen_rva: usize,
    pub installed_rva: usize,
}
