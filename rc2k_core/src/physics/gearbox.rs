//! The gearbox's rules: which gears cannot be taken, the search for the next free gear, the gear
//! asked for, the held inputs by the car's state, the change heard, and which box runs a turn.

use crate::Invalid;

/// The gears that cannot be taken: bit gear + 1 set (gears -1 ..= 7, so bits 0 ..= 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearBits(pub u32);

/// One gear's test: its bit (gear + 1) and whether it is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearTest {
    pub bit: u32,
    pub barred: bool,
}

/// A search's end: the gear reached, the last bit tested (the one that came in when none
/// was) and whether a free gear was found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Search {
    pub gear: i32,
    pub bit: u32,
    pub found: bool,
}

impl GearBits {
    /// The gear's bit; a bit past the dword is refused.
    pub fn test(self, gear: i32) -> Result<GearTest, Invalid> {
        let bit = gear.wrapping_add(1) as u32;
        if bit > 31 {
            return Err(Invalid(
                "a bit past g_carGearBits' dword (gear + 1 past 31)",
            ));
        }
        Ok(GearTest {
            bit,
            barred: self.0 >> bit & 1 != 0,
        })
    }

    /// From `gear` up, the first gear whose bit is clear, up to `count` forward gears
    /// (unsigned); not found with the gear one past it.
    pub fn higher_free(self, gear: i32, count: u32) -> Result<Search, Invalid> {
        let mut at = gear.wrapping_add(1);
        loop {
            let t = self.test(at)?;
            if !t.barred {
                return Ok(Search {
                    gear: at,
                    bit: t.bit,
                    found: true,
                });
            }
            at = at.wrapping_add(1);
            if at as u32 > count {
                return Ok(Search {
                    gear: at,
                    bit: t.bit,
                    found: false,
                });
            }
        }
    }

    /// From `gear` down, the first gear whose bit is clear; not found on reaching 0 (neutral).
    /// `bit` is the bit tested at entry.
    pub fn lower_free(self, gear: i32, bit: u32) -> Result<Search, Invalid> {
        let (mut at, mut bit) = (gear, bit);
        loop {
            at = at.wrapping_sub(1);
            if at == 0 {
                return Ok(Search {
                    gear: 0,
                    bit,
                    found: false,
                });
            }
            let t = self.test(at)?;
            bit = t.bit;
            if !t.barred {
                return Ok(Search {
                    gear: at,
                    bit,
                    found: true,
                });
            }
        }
    }
}

/// The gear asked for, or 0 when the gearbox is locked, a gear of 1 or more is barred (by the
/// sign of gear - 1), or the box is not open. `bits` is asked only for a gear of 1 or more,
/// `open` only past the first two tests.
pub fn allowed_gear(
    locked: bool,
    gear: i32,
    bits: impl FnOnce() -> GearBits,
    open: impl FnOnce() -> u32,
) -> Result<i32, Invalid> {
    if locked {
        return Ok(0);
    }
    if gear.wrapping_sub(1) >= 0 && bits().test(gear)?.barred {
        return Ok(0);
    }
    Ok(if open() != 0 { gear } else { 0 })
}

/// What a held input is set to when the inputs are cleared.
pub const HELD: u32 = 500;

/// The gearbox's input state (seven dwords: an automatic box's held inputs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GearInputs {
    pub w90: u32,
    /// The first input (the joined one).
    pub first: u32,
    /// The second input.
    pub second: u32,
    /// The hold.
    pub hold: u32,
    pub wa0: u32,
    pub wa4: u32,
    /// Which of the two was the larger when joined (1, or -1 for the second).
    pub joined: u32,
}

/// What `GearInputs::apply_rules` reads of the car and the race.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GearCar {
    /// The tyre-change (grades) state: the rules apply when it is 0 or -1.
    pub state: u32,
    /// The box kind: 2 joins or swaps the two inputs.
    pub box_mode: u32,
    /// The start timer (below 0x320 by its sign: still starting).
    pub timer: u32,
    /// The race mode (1 keeps the inputs while starting).
    pub race_mode: u32,
    /// Three switches that clear the inputs.
    pub switches: [u32; 3],
}

impl GearInputs {
    fn clear(&mut self, gear: &mut i32) {
        self.wa4 = 0;
        self.wa0 = 0;
        self.first = 0;
        self.second = 0;
        self.hold = HELD;
        *gear = 0;
    }

    /// Applies the rules for the car's state to `gear` and the inputs; returns the joined
    /// input when the two were joined.
    pub fn apply_rules(&mut self, gear: &mut i32, car: &GearCar) -> Option<u32> {
        if car.state != 0 && car.state != u32::MAX {
            self.clear(gear);
            return None;
        }
        let mut joined = None;
        if car.box_mode == 2 {
            if *gear == 0 {
                self.joined = 1;
                let mut v = self.first;
                if (self.first.wrapping_sub(self.second) as i32) < 0 {
                    self.joined = u32::MAX;
                    v = self.second;
                }
                self.first = v;
                self.second = 0;
                joined = Some(v);
            } else if *gear < 0 {
                std::mem::swap(&mut self.first, &mut self.second);
            }
        }
        if (car.timer.wrapping_sub(0x320) as i32) < 0 {
            self.hold = HELD;
            if car.race_mode != 1 {
                self.second = 0;
                self.w90 = 0;
                self.wa0 = 0;
                self.wa4 = 0;
            }
        }
        if car.switches[0] != 0 {
            self.clear(gear);
        }
        if car.switches[1] != 0 {
            self.clear(gear);
        }
        if car.switches[2] != 0 {
            self.first = 0;
            self.second = 0;
            self.hold = HELD;
            *gear = 0;
        }
        joined
    }
}

/// A gear change whose sound is to be played.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearChange {
    Up,
    Down,
}

/// A gear-changed word of 1 is a change up, -1 down, anything else none.
pub fn gear_change(word: i32) -> Option<GearChange> {
    match word {
        1 => Some(GearChange::Up),
        -1 => Some(GearChange::Down),
        _ => None,
    }
}

/// What the gearbox runs this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearboxTurn {
    /// Nothing (finished, or before the start's 0x320 outside race mode 1).
    Idle,
    /// The manual box (grade mode 2, or the box's kind not 2).
    Manual,
    /// The gear asked for taken, and the turn ends there.
    Asked,
    /// The automatic box (kind 2).
    Auto,
}

/// The inputs word's bits 8 ..= 11; `NONE_ASKED` in them means no gear asked for.
pub const ASKED_MASK: u32 = 0xF00;
/// The asked-gear bits' value when no gear is asked for.
pub const NONE_ASKED: u32 = 0xE00;

/// Chooses the turn from the car's finish, the race mode, the start timer (by sign against
/// 0x320), the grade mode, the inputs word and the box's kind, each asked only where it is read.
pub fn gearbox_turn(
    finished: u32,
    race_mode: impl FnOnce() -> u32,
    timer: impl FnOnce() -> u32,
    grade_mode: impl FnOnce() -> u32,
    inputs: impl FnOnce() -> u32,
    kind: impl FnOnce() -> u32,
) -> GearboxTurn {
    if finished != 0 || (race_mode() != 1 && (timer().wrapping_sub(0x320) as i32) < 0) {
        return GearboxTurn::Idle;
    }
    if grade_mode() == 2 {
        GearboxTurn::Manual
    } else if inputs() & ASKED_MASK != NONE_ASKED {
        GearboxTurn::Asked
    } else if kind() == 2 {
        GearboxTurn::Auto
    } else {
        GearboxTurn::Manual
    }
}

/// The gear asked for: bits 8 ..= 11 of the inputs word as a signed nibble.
pub fn asked_gear(inputs: u32) -> i32 {
    ((inputs & ASKED_MASK) << 20) as i32 >> 28
}

#[cfg(test)]
mod tests;
