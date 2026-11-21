//! The gearbox's shifts: where each box goes from a gear (-1 reverse, 0 neutral, 1 .. the top).
//! The engine's compares are handed in as answers; a step that ends in another (`Shift`) goes on.

/// Where a step of the box goes on to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    /// The change up.
    Up,
    /// The change down.
    Down,
    /// Neutral and settled.
    Neutral,
    /// Pick a start gear.
    Start,
    /// Into reverse.
    Reverse,
    /// Nothing more.
    Done,
}

/// The reverse gear.
pub(crate) const REVERSE: i32 = -1;

/// The manual box's change up from `gear`: from reverse to neutral, from neutral a start
/// gear, at the top (`top()`) nothing; `None`: look for the next free gear up.
pub fn manual_up(gear: i32, top: impl FnOnce() -> u32) -> Option<Shift> {
    match gear {
        REVERSE => Some(Shift::Neutral),
        0 => Some(Shift::Start),
        g if g as u32 == top() => Some(Shift::Done),
        _ => None,
    }
}

/// The manual box's change down from `gear`: from 1 to neutral, from neutral into reverse, in
/// reverse nothing; `None`: look for the next free gear down.
pub fn manual_down(gear: i32) -> Option<Shift> {
    match gear {
        1 => Some(Shift::Neutral),
        0 => Some(Shift::Reverse),
        REVERSE => Some(Shift::Done),
        _ => None,
    }
}

/// The start line's keys (down first, then up): down from neutral into reverse, from a
/// forward gear to neutral; up from neutral a start gear, from reverse to neutral. `None`:
/// neither applies.
pub fn start_line_keys(gear: i32, down: bool, up: impl FnOnce() -> bool) -> Option<Shift> {
    if down {
        if gear == 0 {
            return Some(Shift::Reverse);
        }
        if gear >= 0 {
            return Some(Shift::Neutral);
        }
    }
    if up() {
        if gear == 0 {
            return Some(Shift::Start);
        }
        if gear < 0 {
            return Some(Shift::Neutral);
        }
    }
    None
}

/// The manual box's keys outside the start line: up first, then down (`down()`).
pub fn manual_keys(up: bool, down: impl FnOnce() -> bool) -> Shift {
    if up {
        Shift::Up
    } else if down() {
        Shift::Down
    } else {
        Shift::Done
    }
}

/// A forward gear (1 or more): the engine is looked at for a change up.
pub fn forward(gear: i32) -> bool {
    gear.wrapping_sub(1) >= 0
}

/// A gear with one below it to change down to (2 or more).
pub fn above_first(gear: i32) -> bool {
    gear.wrapping_sub(2) >= 0
}

/// Whether the engine's share for a change down is the clutch's point (clutch down) rather
/// than the free one.
pub fn down_point_clutch(clutch: u32) -> bool {
    clutch != 0
}

/// The automatic's change up from `gear`: at the top nothing, from reverse a start gear;
/// `None`: up one.
pub fn auto_up(gear: i32, top: u32) -> Option<Shift> {
    if gear as u32 == top {
        Some(Shift::Done)
    } else if gear == REVERSE {
        Some(Shift::Start)
    } else {
        None
    }
}

/// The automatic's tests, each asked only where it is needed: up when in a forward gear on
/// the ground (`all_off()` false) with the engine at its share (`over_share()`); a start gear
/// from reverse with the clutch down (`clutch()`) and the speed at the start's (`fast()`);
/// down (`down()`: from gear 2 or more, the engine under the lower gear's share); reverse
/// from first with the clutch down and the speed under the reverse's (`slow()`).
#[allow(clippy::too_many_arguments)] // reason: one answer per compare, asked in turn
pub fn auto_tests<E>(
    gear: i32,
    all_off: impl FnOnce() -> bool,
    clutch: impl Fn() -> bool,
    over_share: impl FnOnce() -> bool,
    fast: impl FnOnce() -> bool,
    down: impl FnOnce() -> Result<bool, E>,
    slow: impl FnOnce() -> bool,
) -> Result<Shift, E> {
    if forward(gear) && !all_off() && over_share() {
        return Ok(Shift::Up);
    }
    if gear == REVERSE && clutch() && fast() {
        return Ok(Shift::Start);
    }
    if down()? {
        return Ok(Shift::Down);
    }
    if gear == 1 && clutch() && slow() {
        return Ok(Shift::Reverse);
    }
    Ok(Shift::Done)
}

/// How `change` goes from `gear` to the gear asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// The gear already.
    Same,
    /// From neutral: the gear is set and the wheels follow from the revs.
    FromNeutral,
    /// To neutral.
    ToNeutral,
    /// The new gear tested: refused (neutral), else the wheels follow from the revs, the gear
    /// is set and the revs follow from the wheels.
    Through,
}

pub fn change(gear: i32, wanted: i32) -> Change {
    if gear == wanted {
        Change::Same
    } else if gear == 0 {
        Change::FromNeutral
    } else if wanted == 0 {
        Change::ToNeutral
    } else {
        Change::Through
    }
}

/// Gears 1 .. 6 tried in turn; the first taken, or `None` (the last tried stays).
pub fn pick_start<E>(mut try_gear: impl FnMut(i32) -> Result<bool, E>) -> Result<Option<i32>, E> {
    for gear in 1..=6 {
        if try_gear(gear)? {
            return Ok(Some(gear));
        }
    }
    Ok(None)
}

/// The hold after a change (frames before the next).
pub const HOLD: u32 = 20;

#[cfg(test)]
mod tests;
