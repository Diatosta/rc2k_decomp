//! The gearbox's rules (converted 2026-10-21 from `physics::gear_bit` 0x43C5B7,
//! `higher_free_gear` 0x43C5A0, `lower_free_gear` 0x43C58E, `allowed_gear` 0x4651B5,
//! `gear_input_rules` 0x439BF6, `gear_change_sound` 0x45C551, `gearbox_step` 0x43C15C
//! and `take_asked_gear` 0x43C1E3): which gears cannot be taken, the search for the next
//! free one up or down, the gear asked for let through or not, the gearbox's held inputs
//! by the car's state, the change heard, and which box runs a turn.

use crate::Invalid;

/// The gears that cannot be taken (`g_carGearBits`, the work copy's +0x434): bit
/// gear + 1 set (gears -1 .. 7 in the game, so bits 0 .. 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearBits(pub u32);

/// One gear's test: its bit (gear + 1, the original's eax) and whether it is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GearTest {
    pub bit: u32,
    pub barred: bool,
}

/// A search's end: the gear reached (the original's ecx), the last bit tested (eax; as
/// it came in when none was) and whether a free gear was found (CF clear).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Search {
    pub gear: u32,
    pub bit: u32,
    pub found: bool,
}

impl GearBits {
    /// `physics::gear_bit`: the gear's bit. A bit past the dword is refused (the original's
    /// `bt` on memory reads on along the work copy).
    pub fn test(self, gear: u32) -> Result<GearTest, Invalid> {
        let bit = gear.wrapping_add(1);
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

    /// `physics::higher_free_gear`: from `gear` up, the first gear whose bit is clear,
    /// up to `count` (the forward gears, unsigned); not found with the gear one past it.
    pub fn higher_free(self, gear: u32, count: u32) -> Result<Search, Invalid> {
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
            if at > count {
                return Ok(Search {
                    gear: at,
                    bit: t.bit,
                    found: false,
                });
            }
        }
    }

    /// `physics::lower_free_gear`: from `gear` down, the first gear whose bit is clear;
    /// not found on reaching 0 (neutral). `bit` is what eax held at entry.
    pub fn lower_free(self, gear: u32, bit: u32) -> Result<Search, Invalid> {
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

/// `physics::allowed_gear`: the gear asked for, or 0: 0 when the gearbox is locked (the
/// work copy's +0x2B8 holds the bits of 100.0), when a gear of 1 or more (by the sign of
/// gear - 1) is barred, or unless it is open (+0x3F4 not 0). `bits` is asked only for a
/// gear of 1 or more, `open` only past the first two tests, as the original reads them.
pub fn allowed_gear(
    locked: bool,
    gear: u32,
    bits: impl FnOnce() -> GearBits,
    open: impl FnOnce() -> u32,
) -> Result<u32, Invalid> {
    if locked {
        return Ok(0);
    }
    if (gear.wrapping_sub(1) as i32) >= 0 && bits().test(gear)?.barred {
        return Ok(0);
    }
    Ok(if open() != 0 { gear } else { 0 })
}

/// What a held input is set to when the inputs are cleared (0x1F4).
pub const HELD: u32 = 500;

/// The gearbox's input state (0x613890 .. 0x6138A8, seven dwords; what they are is a
/// reading at best: an automatic box's held inputs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GearInputs {
    /// 0x613890.
    pub w90: u32,
    /// 0x613894: the first input (the joined one).
    pub first: u32,
    /// 0x613898: the second input.
    pub second: u32,
    /// 0x61389C: the hold.
    pub hold: u32,
    /// 0x6138A0.
    pub wa0: u32,
    /// 0x6138A4.
    pub wa4: u32,
    /// 0x6138A8: which of the two was the larger when joined (1, or -1 for the second).
    pub joined: u32,
}

/// What `gear_input_rules` reads of the car (the work copy) and the race.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GearCar {
    /// `dword_71A244` (the grades' mode): rules apply when it is 0 or -1.
    pub state: u32,
    /// `dword_71A010`: 2 joins or swaps the two inputs.
    pub box_mode: u32,
    /// `dword_719FEC`: the start timer (below 0x320 by its sign: still starting).
    pub timer: u32,
    /// `dword_62389C`: the race mode (1 keeps the inputs while starting).
    pub race_mode: u32,
    /// `dword_71A07C`, `dword_71A078`, `dword_71A080`: three switches that clear them.
    pub switches: [u32; 3],
}

impl GearInputs {
    fn clear(&mut self, gear: &mut u32) {
        self.wa4 = 0;
        self.wa0 = 0;
        self.first = 0;
        self.second = 0;
        self.hold = HELD;
        *gear = 0;
    }

    /// `physics::gear_input_rules`: the inputs by the car's state; `gear` is `g_carGear`.
    /// The joined input when the two were joined (the original's eax), else `None`.
    pub fn apply_rules(&mut self, gear: &mut u32, car: &GearCar) -> Option<u32> {
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
            } else if (*gear as i32) < 0 {
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

/// A gear change heard (a reading).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearChange {
    Up,
    Down,
}

/// `physics::gear_change_sound`'s rule: the work copy's +0x238 (`dword_71A210`) 1 is a
/// change up, -1 down, anything else none.
pub fn gear_change(word: u32) -> Option<GearChange> {
    match word {
        1 => Some(GearChange::Up),
        u32::MAX => Some(GearChange::Down),
        _ => None,
    }
}

/// What `physics::gearbox_step` (0x43C15C, converted 2026-10-21) runs this turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearboxTurn {
    /// Nothing (finished, or before the start's 0x320 outside race mode 1).
    Idle,
    /// `physics::manual_gearbox` (grade mode 2, or the box's kind not 2).
    Manual,
    /// `physics::take_asked_gear`, and the turn ends there.
    Asked,
    /// `physics::auto_gearbox` (the box's kind, +0x38, 2).
    Auto,
}

/// No gear asked for: +0x468's bits 8 .. 11 (`ASKED_MASK`) 0xE.
pub const ASKED_MASK: u32 = 0xF00;
pub const NONE_ASKED: u32 = 0xE00;

/// `physics::gearbox_step`'s choice: the car's finish (+0xA8), the race mode, the start
/// timer (+0x14, by the sign against 0x320), the grade mode (+0x25C), the inputs word
/// (+0x468) and the box's kind (+0x38), each asked only where the original reads it.
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

/// `physics::take_asked_gear` (0x43C1E3, converted 2026-10-21): the gear asked for,
/// bits 8 .. 11 of the inputs word as a signed nibble.
pub fn asked_gear(inputs: u32) -> u32 {
    (((inputs & ASKED_MASK) << 20) as i32 >> 28) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_and_searches() {
        let b = GearBits(0b1_0110);
        assert_eq!(
            b.test(1),
            Ok(GearTest {
                bit: 2,
                barred: true
            })
        );
        assert_eq!(
            b.test(u32::MAX),
            Ok(GearTest {
                bit: 0,
                barred: false
            })
        );
        assert!(b.test(31).is_err());
        // Bits 1, 2, 4 set: gears 0, 1, 3 barred.
        assert_eq!(
            b.higher_free(0, 5),
            Ok(Search {
                gear: 2,
                bit: 3,
                found: true
            })
        );
        assert_eq!(
            GearBits(u32::MAX >> 1).higher_free(1, 4),
            Ok(Search {
                gear: 5,
                bit: 5,
                found: false
            })
        );
        assert_eq!(
            b.lower_free(4, 9),
            Ok(Search {
                gear: 2,
                bit: 3,
                found: true
            })
        );
        assert_eq!(
            b.lower_free(1, 9),
            Ok(Search {
                gear: 0,
                bit: 9,
                found: false
            })
        );
        assert!(GearBits(u32::MAX).lower_free(0, 0).is_err());
    }

    #[test]
    fn allowed() {
        let bits = || GearBits(0b100);
        assert_eq!(allowed_gear(true, 3, bits, || 1), Ok(0));
        assert_eq!(allowed_gear(false, 1, bits, || 1), Ok(0));
        assert_eq!(allowed_gear(false, 2, bits, || 1), Ok(2));
        assert_eq!(allowed_gear(false, 2, bits, || 0), Ok(0));
        assert_eq!(allowed_gear(false, 0, || unreachable!(), || 1), Ok(0));
        assert_eq!(
            allowed_gear(false, u32::MAX, || unreachable!(), || 1),
            Ok(u32::MAX)
        );
    }

    #[test]
    fn changes() {
        assert_eq!(gear_change(1), Some(GearChange::Up));
        assert_eq!(gear_change(u32::MAX), Some(GearChange::Down));
        assert_eq!(gear_change(2), None);
    }

    #[test]
    fn input_rules() {
        let car = GearCar::default();
        let mut i = GearInputs {
            first: 3,
            second: 7,
            hold: 1,
            ..Default::default()
        };
        let mut gear = 0;
        let both = GearCar {
            box_mode: 2,
            timer: 0x320,
            ..car
        };
        assert_eq!(i.apply_rules(&mut gear, &both), Some(7));
        assert_eq!((i.first, i.second, i.joined, i.hold), (7, 0, u32::MAX, 1));
        let mut gear = u32::MAX;
        let mut j = GearInputs {
            first: 3,
            second: 7,
            ..Default::default()
        };
        assert_eq!(j.apply_rules(&mut gear, &both), None);
        assert_eq!((j.first, j.second), (7, 3));
        let mut gear = 4;
        let mut k = GearInputs {
            w90: 1,
            first: 2,
            second: 3,
            wa0: 4,
            wa4: 5,
            ..Default::default()
        };
        k.apply_rules(&mut gear, &car);
        assert_eq!(
            k,
            GearInputs {
                first: 2,
                hold: HELD,
                ..Default::default()
            }
        );
        let off = GearCar { state: 5, ..car };
        let mut gear = 4;
        k.apply_rules(&mut gear, &off);
        assert_eq!(gear, 0);
    }

    #[test]
    fn turns() {
        let t = |f, m, tm, g, i, k| gearbox_turn(f, || m, || tm, || g, || i, || k);
        assert_eq!(t(1, 1, 0, 0, 0, 0), GearboxTurn::Idle);
        assert_eq!(t(0, 0, 0x31F, 0, 0, 0), GearboxTurn::Idle);
        assert_eq!(t(0, 1, 0, 2, 0, 0), GearboxTurn::Manual);
        assert_eq!(t(0, 0, 0x320, 0, 0x300, 0), GearboxTurn::Asked);
        assert_eq!(t(0, 0, 0x320, 0, 0xE00, 2), GearboxTurn::Auto);
        assert_eq!(t(0, 0, 0x320, 0, 0xE00, 1), GearboxTurn::Manual);
        assert_eq!(asked_gear(0xF00), u32::MAX);
        assert_eq!(asked_gear(0x700), 7);
        assert_eq!(asked_gear(0x800), (-8i32) as u32);
    }
}
