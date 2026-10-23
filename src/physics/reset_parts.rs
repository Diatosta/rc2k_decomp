//! The car reset's steps under `sub_43B635` (stream B, 2026-10-17): `physics::false_start`
//! (0x43B9B4), `physics::reset_when_stuck` (0x43EFBA), `physics::reset_car` (0x43EFEF),
//! `physics::reset_car_on_line` (0x43FB00). Their calls go through a `Part` by place, as
//! stream A's `race/car_starts.rs` (its places 0 .. 12), with more of ours:
//! 13 `physics::reset_point_blocked`, 14 the log line (`log_write_l1_nl`, 0x40142B),
//! 15 `physics::reset_car`, 16 `physics::reset_car_on_line`, 17
//! `race::nearest_checkpoint_in`, 18 `physics::find_ground`, 19
//! `physics::count_ground_probe` (only the camera's ground checks call it). In the game the Rust
//! versions; in the self-tests copies whose calls go to the stand-ins (`handler`: stream
//! A's `note`, and for place 13 a carry by the case's pattern), and the Rust versions
//! with the same.
//!
//! Shadow mode runs these originals alone (A's `reset_car_motion` does: its calls into
//! the game).

use crate::errors::Refused;
use crate::framework::Regs;

pub const BLOCKED: u32 = 13;
pub const LOG: u32 = 14;
pub const RESET_CAR: u32 = 15;
pub const ON_LINE: u32 = 16;
pub const NEAREST_IN: u32 = 17;
pub const FIND_GROUND: u32 = 18;
pub const COUNT_PROBE: u32 = 19;

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            BLOCKED => crate::physics::reset_point_blocked::reset_point_blocked(r).apply(r),
            LOG => {
                // `log_write_l1_nl`: level 1, a line break.
                r.edx = 1;
                crate::logfile::log_write::log_write_regs(r);
            }
            RESET_CAR => *r = crate::physics::reset_car::reset_car(r)?,
            ON_LINE => *r = crate::physics::reset_car_on_line::reset_car_on_line(r)?,
            NEAREST_IN => *r = crate::race::start_place::nearest_checkpoint_in(r)?,
            FIND_GROUND => {
                crate::physics::find_ground::find_ground()?;
            }
            COUNT_PROBE => *r = crate::physics::count_ground_probe::count_ground_probe(r)?,
            _ => crate::race::car_starts::game_part(place, r)?,
        }
    }
    Ok(())
}

/// `dispatch_reset` without its hook (Rust callers; a refusal is the caller's).
pub fn reset_entry(
    r: &mut Regs,
    rust: unsafe fn(&Regs) -> Result<Regs, Refused>,
) -> Result<(), Refused> {
    let g = unsafe { rust(r) }?;
    *r = Regs {
        esp: r.esp,
        eflags: r.eflags,
        ..g
    };
    Ok(())
}

// ---- for the self-tests ----
