//! A player's input device: the record the input thread keeps for each (its layout), what a
//! reading is, the device command's steps. The reading itself is `input::profile::readings`.

use crate::Invalid;

pub const RECORD_LEN: usize = 0x4F8;
/// Offsets in the record.
pub const DEVICE: usize = 4;
pub const KIND: usize = 8;
pub const ACQUIRED: usize = 0x24;
pub const BUTTONS: usize = 0x38;
pub const READING: usize = 0x414;
pub const HAT_KIND: usize = 0x418;
pub const BINDINGS: usize = 0x41C;
pub const BINDINGS_COUNT: usize = 12;
pub const READINGS: usize = 0x4DC;
pub const BITS: usize = 0x4E8;
/// All of a reading: left (whose axis is negated), and the others.
pub const FULL_LEFT: u32 = 0xFFFF_FF00;
pub const FULL: u32 = 0x100;

/// The bits a reading has with the hat centred.
pub const HAT_CENTRED: u32 = 0xE00;
/// What `input::clear_device_readings` writes, in order: the state copies (+0x2C .. +0x38) and
/// the readings (+0x4DC .. +0x4E4) 0, the bits the hat-centred value.
pub const CLEARED_READINGS: [(usize, u32); 8] = [
    (0x2C, 0),
    (0x30, 0),
    (0x34, 0),
    (BUTTONS, 0),
    (READINGS, 0),
    (READINGS + 4, 0),
    (READINGS + 8, 0),
    (BITS, HAT_CENTRED),
];

pub type Record = [u8; RECORD_LEN];
/// The key table the bindings' keys index (256 bytes).
pub type KeyTable = [u8; 0x100];
/// The device's state (DirectInput's `DIJOYSTATE`, 0x50 bytes).
pub type State = [u8; 0x50];

/// The record's dword at an offset; `None` past the record.
pub fn dword(record: &Record, at: usize) -> Option<u32> {
    let bytes = record.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

/// A reading of a device: steering, brake, throttle and the bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Readings {
    pub steering: u32,
    pub brake: u32,
    pub throttle: u32,
    pub bits: u32,
}

/// One player on two devices: the second's three readings added to the first's, and of its bits
/// those of 0x70FF.
pub fn merge_readings(first: &mut [u32; 4], second: &[u32; 4]) {
    for n in 0..3 {
        first[n] = first[n].wrapping_add(second[n]);
    }
    first[3] |= second[3] & 0x70FF;
}

/// The twelve bindings of a device's record: each (the player's controls' key byte, the controls'
/// word, the record's place).
pub const DEVICE_BINDINGS: [(usize, usize, usize); 12] = [
    (0x54, 0x2C, 0x41C),
    (0x55, 0x30, 0x42C),
    (0x56, 0x24, 0x43C),
    (0x57, 0x28, 0x44C),
    (0x59, 0x34, 0x45C),
    (0x5A, 0x38, 0x46C),
    (0x58, 0x3C, 0x47C),
    (0x5B, 0x4C, 0x48C),
    (0x5C, 0x48, 0x49C),
    (0x5D, 0x40, 0x4AC),
    (0x5E, 0x44, 0x4BC),
    (0x5F, 0x50, 0x4CC),
];

/// The device's kind from the controls' kind (0 gives 2).
pub fn device_kind(kind: u32) -> u32 {
    if kind == 0 { 2 } else { kind }
}

/// A step of the input thread's device command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandStep {
    /// The force feedback's race-on flag, the effects' switch.
    EffectsOn(bool),
    Release(usize),
    Acquire(usize),
    EndEffects,
    Tick,
    /// Device `device` set up from player `device`'s controls.
    SetUp(usize),
    /// The device's read flag set to the value given.
    Reading(usize, bool),
    /// Player 2's controller made none when it is player 1's.
    SecondApart,
    BeginEffects,
}

/// The steps of command `command` (`device` too for 1): 1 gives the device back and acquires it
/// again; 2 (the race's end) turns the effects off and ends them, waits a tick, gives both devices
/// back and stops reading them; 3 (the race's start) sets up and reads device 0, device 1 too
/// unless one player is on one device, begins the effects, waits a tick and
/// turns them on. Refused: a device past 1 for 1.
pub fn command_steps(
    command: u8,
    device: u8,
    two_devices: u32,
    one_player: u32,
) -> Result<Vec<CommandStep>, Invalid> {
    use CommandStep::*;
    Ok(match command {
        1 => {
            if device > 1 {
                return Err(Invalid("a device past 1"));
            }
            vec![Release(device as usize), Acquire(device as usize)]
        }
        2 => vec![
            EffectsOn(false),
            EndEffects,
            Tick,
            Release(0),
            Reading(0, false),
            Release(1),
            Reading(1, false),
        ],
        3 => {
            let mut v = vec![SetUp(0), Reading(0, true)];
            if two_devices != 0 || one_player != 1 {
                v.extend([SecondApart, SetUp(1), Reading(1, true)]);
            }
            v.extend([BeginEffects, Tick, EffectsOn(true)]);
            v
        }
        _ => vec![],
    })
}

/// The controller page's learning of a control: on, when it began (the 5 s start), which control
/// (1 ..= 11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Learning {
    pub on: u32,
    pub started: u32,
    pub control: u32,
}

impl Learning {
    /// `menu::learn_binding_n`: control `control` learns from `now`.
    pub fn start(control: u32, now: u32) -> Self {
        Learning {
            on: 1,
            started: now,
            control,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands() {
        assert!(command_steps(1, 2, 0, 0).is_err());
        assert_eq!(command_steps(3, 0, 0, 1).unwrap().len(), 5);
        assert_eq!(command_steps(3, 0, 1, 1).unwrap().len(), 8);
        assert!(command_steps(9, 0, 0, 0).unwrap().is_empty());
    }

    #[test]
    fn merges() {
        let mut a = [1, 2, 3, 0x8000];
        merge_readings(&mut a, &[1, 1, 1, 0xFFFF]);
        assert_eq!(a, [2, 3, 4, 0xF0FF]);
    }
}
