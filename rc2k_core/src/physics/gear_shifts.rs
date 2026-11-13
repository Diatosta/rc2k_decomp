//! The gearbox's shifts (from `physics::manual_gearbox` 0x43C2AE,
//! `physics::auto_gearbox` 0x43C5C2, `physics::change_gear` 0x43C20D and
//! `physics::pick_start_gear` 0x43C8AC; the engine's sums and their compares stay with
//! the caller's x87, which hands the compares in as answers): where each box goes from a gear
//! (-1 reverse, 0 neutral, 1 .. the top). The steps run in the original's order; a step
//! that ends in another (`Shift`) is the original's jump to it.

/// Where a step of the box goes on to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    /// The change up (`manual`: 0x43C340; `auto`: 0x43C66D).
    Up,
    /// The change down (`manual`: 0x43C379; `auto`: 0x43C6B7).
    Down,
    /// Neutral and settled.
    Neutral,
    /// `physics::pick_start_gear`.
    Start,
    /// Into reverse.
    Reverse,
    /// Nothing more.
    Done,
}

/// The reverse gear.
pub const REVERSE: i32 = -1;

/// The manual box's change up from `gear`: from reverse to neutral, from neutral a start
/// gear, at the top (`top()`, `g_gearCount`) nothing; None: the next free gear up is
/// looked for.
pub fn manual_up(gear: i32, top: impl FnOnce() -> u32) -> Option<Shift> {
    match gear {
        REVERSE => Some(Shift::Neutral),
        0 => Some(Shift::Start),
        g if g as u32 == top() => Some(Shift::Done),
        _ => None,
    }
}

/// The manual box's change down from `gear`: from 1 to neutral, from neutral into
/// reverse, in reverse nothing; None: the next free gear down is looked for.
pub fn manual_down(gear: i32) -> Option<Shift> {
    match gear {
        1 => Some(Shift::Neutral),
        0 => Some(Shift::Reverse),
        REVERSE => Some(Shift::Done),
        _ => None,
    }
}

/// The start line's keys (down `input_gear_down` first, then up `input_gear_up`): down from
/// neutral into reverse, from a forward gear (or neutral's other) to neutral; up from
/// neutral a start gear, from reverse to neutral. None: neither applies.
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

/// A forward gear (1 or more, by the sign of gear - 1): the engine looked at for a
/// change up.
pub fn forward(gear: i32) -> bool {
    gear.wrapping_sub(1) >= 0
}

/// A gear with one below it to change down to (2 or more, by the sign of gear - 2).
pub fn above_first(gear: i32) -> bool {
    gear.wrapping_sub(2) >= 0
}

/// The engine's share for a change down: the clutch's (`shift_down_clutch_point`) with the clutch
/// down (`brake`), else the free one (`shift_down_point`).
pub fn down_point_clutch(clutch: u32) -> bool {
    clutch != 0
}

/// The automatic's change up from `gear`: at the top nothing, from reverse a start gear;
/// None: up one.
pub fn auto_up(gear: i32, top: u32) -> Option<Shift> {
    if gear as u32 == top {
        Some(Shift::Done)
    } else if gear == REVERSE {
        Some(Shift::Start)
    } else {
        None
    }
}

/// The automatic's tests (0x43C798), each asked only where the original reads or
/// compares: up when in a forward gear on the ground (`all_off()` false) with the engine
/// at its share (`over_share()`); a start gear from reverse with the clutch down
/// (`clutch()`) and the speed at the start's (`fast()`); down (`down()`: from gear 2 or
/// more, the engine under the lower gear's share and the point); reverse from first with
/// the clutch down and the speed under the reverse's (`slow()`).
#[allow(clippy::too_many_arguments)] // reason: each compare is the original's, asked in turn
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

/// `physics::change_gear`'s way from `gear` to the gear asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// The gear already.
    Same,
    /// From neutral: the gear set, the wheels from the revs.
    FromNeutral,
    /// To neutral.
    ToNeutral,
    /// The new gear tested (`physics::gear_bit`): refused, neutral; else the wheels from
    /// the revs, the gear set, the revs from the wheels.
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

/// `physics::pick_start_gear`: gears 1 .. 6 tried in turn (`try_gear(g)`: not refused
/// and the engine's share under the change point); the first taken, or None (the last
/// tried stays).
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
mod tests {
    use super::*;

    #[test]
    fn manual() {
        assert_eq!(manual_up(REVERSE, || 5), Some(Shift::Neutral));
        assert_eq!(manual_up(0, || 5), Some(Shift::Start));
        assert_eq!(manual_up(5, || 5), Some(Shift::Done));
        assert_eq!(manual_up(3, || 5), None);
        assert_eq!(manual_down(1), Some(Shift::Neutral));
        assert_eq!(manual_down(0), Some(Shift::Reverse));
        assert_eq!(manual_down(REVERSE), Some(Shift::Done));
        assert_eq!(manual_down(4), None);
        assert_eq!(start_line_keys(0, true, || true), Some(Shift::Reverse));
        assert_eq!(start_line_keys(3, true, || false), Some(Shift::Neutral));
        assert_eq!(
            start_line_keys(REVERSE, true, || true),
            Some(Shift::Neutral)
        );
        assert_eq!(start_line_keys(REVERSE, true, || false), None);
        assert_eq!(start_line_keys(0, false, || true), Some(Shift::Start));
        assert_eq!(start_line_keys(3, false, || true), None);
        assert_eq!(manual_keys(true, || true), Shift::Up);
        assert_eq!(manual_keys(false, || true), Shift::Down);
        assert!(forward(1) && !forward(0) && !forward(REVERSE));
        assert!(above_first(2) && !above_first(1));
    }

    #[test]
    fn automatic() {
        assert_eq!(auto_up(5, 5), Some(Shift::Done));
        assert_eq!(auto_up(REVERSE, 5), Some(Shift::Start));
        assert_eq!(auto_up(2, 5), None);
        let never = || -> bool { panic!() };
        let no = || -> Result<bool, ()> { Ok(false) };
        assert_eq!(
            auto_tests::<()>(2, || false, never, || true, never, || panic!(), never),
            Ok(Shift::Up)
        );
        assert_eq!(
            auto_tests::<()>(REVERSE, never, || true, never, || true, || panic!(), never),
            Ok(Shift::Start)
        );
        assert_eq!(
            auto_tests(1, || true, || true, never, never, no, || true),
            Ok(Shift::Reverse)
        );
        assert_eq!(
            auto_tests(0, never, never, never, never, no, never),
            Ok(Shift::Done)
        );
        assert_eq!(change(2, 2), Change::Same);
        assert_eq!(change(0, 2), Change::FromNeutral);
        assert_eq!(change(2, 0), Change::ToNeutral);
        assert_eq!(change(2, 3), Change::Through);
        assert_eq!(pick_start::<()>(|g| Ok(g == 3)), Ok(Some(3)));
        assert_eq!(pick_start::<()>(|_| Ok(false)), Ok(None));
    }
}
