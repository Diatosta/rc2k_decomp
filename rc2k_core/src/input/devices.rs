//! A player's input device read (converted 2026-10-20 from `input::read_device` 0x408E38,
//! `hat_reading` 0x408968, `hat_direction` 0x408992, `binding_place` 0x408DD6).
//!
//! A device record (0x4F8 bytes, the game's `InputDevice`) has twelve bindings of four
//! dwords, {key, the place of an axis in the record (0 none), the axis's sign, a button's
//! bit}: steering left and right, throttle, brake, then eight buttons. A reading comes from
//! the key (down: bit 0x80 of its byte in the key table), else the axis as it is, or with a
//! button bound too all of it when the axis is above 0x40 and the button down; else the
//! button alone.

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

/// A device record's bytes.
pub type Record = [u8; RECORD_LEN];
/// The key table the bindings' keys index (256 bytes from `g_keysDown`'s place).
pub type KeyTable = [u8; 0x100];
/// The device's state (`g_deviceState`, DirectInput's `DIJOYSTATE`, 0x50 bytes).
pub type State = [u8; 0x50];

/// The record's dword at an offset; `None` past the record.
pub fn dword(record: &Record, at: usize) -> Option<u32> {
    let bytes = record.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn field(record: &Record, at: usize) -> u32 {
    dword(record, at).expect("a field inside the record")
}

/// What `read_device` stores: steering, brake, throttle and the bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Readings {
    pub steering: u32,
    pub brake: u32,
    pub throttle: u32,
    pub bits: u32,
}

/// The hat as an angle in 256ths (`hat_reading`): with a device whose hat kind is 0 and a
/// hat (`state`'s +0x20) not centred (its low word not 0xFFFF), its hundredths of degrees
/// x 256 / 36000; else 0. Also the `div`'s ecx and edx when it was reached.
pub fn hat_reading(record: &Record, state: &State) -> (u32, Option<(u32, u32)>) {
    if field(record, DEVICE) == 0 || field(record, HAT_KIND) != 0 {
        return (0, None);
    }
    let hat = u32::from_le_bytes(state[0x20..0x24].try_into().unwrap());
    if hat & 0xFFFF == 0xFFFF {
        return (0, None);
    }
    let shifted = hat << 8;
    (shifted / 0x8CA0, Some((0x8CA0, shifted % 0x8CA0)))
}

/// The hat (or buttons standing for it) as a code in the second byte (`hat_direction`):
/// for a device whose hat kind is not 0, buttons 6, 7, 1, 2 as 0x100 .. 0x400, else the
/// hat straight on 0x500, 270 degrees 0x600, 180 0x700, 90 0xF00, else 0; without a device
/// or with kind 0, 0xE00 (none).
pub fn hat_direction(record: &Record, state: &State) -> u32 {
    if field(record, DEVICE) == 0 || field(record, HAT_KIND) == 0 {
        return 0xE00;
    }
    for (button, code) in [(0x35, 0x100), (0x36, 0x200), (0x30, 0x300), (0x31, 0x400)] {
        if state[button] != 0 {
            return code;
        }
    }
    match u32::from_le_bytes(state[0x20..0x24].try_into().unwrap()) {
        0 => 0x500,
        0x6978 => 0x600,
        0x4650 => 0x700,
        0x2328 => 0xF00,
        _ => 0,
    }
}

/// Where a binding's word says the record reads it (`binding_place`): for an acquired
/// device, the axis (bit 0x400: +0x2C negated, 0x800: +0x2C, 0x200: +0x34, 0x100: +0x30,
/// else none) and the button (the low byte n: bit n - 1; 0 none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    /// The axis's place (0 none).
    pub axis: u32,
    /// Its sign (1, -1; 0 when not acquired).
    pub sign: u32,
    /// The button's bit (0 none).
    pub button: u32,
    /// The low byte (less one when not 0) when acquired: what the original leaves in ecx.
    pub ecx: Option<u32>,
}

pub fn binding_place(word: u32, acquired: bool) -> Place {
    if !acquired {
        return Place {
            axis: 0,
            sign: 0,
            button: 0,
            ecx: None,
        };
    }
    let (axis, sign) = if word & 0x400 != 0 {
        (0x2C, u32::MAX)
    } else if word & 0x800 != 0 {
        (0x2C, 1)
    } else if word & 0x200 != 0 {
        (0x34, 1)
    } else if word & 0x100 != 0 {
        (0x30, 1)
    } else {
        (0, 1)
    };
    let low = word & 0xFF;
    let (button, ecx) = if low == 0 {
        (0, 0)
    } else {
        (1u32.wrapping_shl(low - 1), low - 1)
    };
    Place {
        axis,
        sign,
        button,
        ecx: Some(ecx),
    }
}

/// Bound keys past 0xFF and axis places past the record are refused.
pub fn check_bindings(record: &Record) -> Result<(), Invalid> {
    for k in 0..BINDINGS_COUNT {
        let g = BINDINGS + 0x10 * k;
        if field(record, g) > 0xFF {
            return Err(Invalid("a bound key past 0xFF"));
        }
        if field(record, g + 4) > RECORD_LEN as u32 - 4 {
            return Err(Invalid("a bound axis's place past the record"));
        }
    }
    Ok(())
}

struct Reader<'a> {
    record: &'a Record,
    keys: &'a KeyTable,
}

impl Reader<'_> {
    fn key_down(&self, g: usize) -> bool {
        self.keys[field(self.record, g) as usize & 0xFF] & 0x80 != 0
    }

    /// The record's dword at the binding's place times its sign (`imul`).
    fn axis_value(&self, g: usize) -> i32 {
        let place = field(self.record, g + 4) as usize;
        (field(self.record, place) as i32).wrapping_mul(field(self.record, g + 8) as i32)
    }

    fn buttons(&self) -> u32 {
        field(self.record, BUTTONS)
    }

    /// One of the four readings: the value, and whether it is all of it pressed.
    fn reading(&self, g: usize, full: u32, negate_axis: bool) -> (u32, bool) {
        if self.key_down(g) {
            return (full, true);
        }
        let button = field(self.record, g + 0xC);
        if field(self.record, g + 4) != 0 {
            let v = self.axis_value(g).max(0);
            if button == 0 {
                let v = v as u32;
                return (if negate_axis { v.wrapping_neg() } else { v }, false);
            }
            if v > 0x40 && self.buttons() & button != 0 {
                return (full, true);
            }
            return (0, false);
        }
        if button != 0 && self.buttons() & button != 0 {
            return (full, true);
        }
        (0, false)
    }

    /// One of the eight buttons.
    fn pressed(&self, g: usize) -> bool {
        if self.key_down(g) {
            return true;
        }
        let button = field(self.record, g + 0xC);
        if field(self.record, g + 4) != 0 {
            if self.axis_value(g) <= 0x40 {
                return false;
            }
            return button == 0 || self.buttons() & button != 0;
        }
        button != 0 && self.buttons() & button != 0
    }
}

/// Not 0 made all of it, by its sign.
fn whole_steering(v: u32) -> u32 {
    match v {
        0 => 0,
        v if (v as i32) < 0 => FULL_LEFT,
        _ => FULL,
    }
}

/// The four readings of a device (after its poll), or `None` while it is not read. See
/// the module's header; the steering is left plus right, and with a key or button pressed
/// held to all of it by its sign; on a digital device (kind 1) bits 0x7000 and every
/// reading not 0 made all of it. Refused: what `check_bindings` refuses.
pub fn read_device(
    record: &Record,
    keys: &KeyTable,
    state: &State,
) -> Result<Option<Readings>, Invalid> {
    check_bindings(record)?;
    if field(record, READING) != 1 {
        return Ok(None);
    }
    let r = Reader { record, keys };
    let mut bits = (hat_reading(record, state).0 << 24) | hat_direction(record, state);
    let mut steering = {
        let (left, l) = r.reading(BINDINGS, FULL_LEFT, true);
        let (right, rr) = r.reading(BINDINGS + 0x10, FULL, false);
        if l || rr {
            bits |= 0x1000;
        }
        left.wrapping_add(right)
    };
    if bits & 0x1000 != 0 {
        steering = whole_steering(steering);
    }
    let (mut throttle, t) = r.reading(BINDINGS + 0x20, FULL, false);
    if t {
        bits |= 0x4000;
    }
    let (mut brake, b) = r.reading(BINDINGS + 0x30, FULL, false);
    if b {
        bits |= 0x2000;
    }
    if field(record, KIND) == 1 {
        bits |= 0x7000;
        steering = whole_steering(steering);
        if brake != 0 {
            brake = FULL;
        }
        if throttle != 0 {
            throttle = FULL;
        }
    }
    for k in 0..8 {
        if r.pressed(BINDINGS + 0x40 + 0x10 * k) {
            bits |= 1 << k;
        }
    }
    Ok(Some(Readings {
        steering,
        brake,
        throttle,
        bits,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(record: &mut Record, at: usize, v: u32) {
        record[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }

    #[test]
    fn reads() {
        let mut record = [0u8; RECORD_LEN];
        put(&mut record, READING, 1);
        put(&mut record, DEVICE, 1);
        put(&mut record, 0x2C, 0x80); // the first axis
        put(&mut record, BINDINGS + 0x10 + 4, 0x2C); // right: the first axis
        put(&mut record, BINDINGS + 0x10 + 8, 1);
        put(&mut record, BINDINGS + 0x20, 0x48); // throttle: key 0x48
        put(&mut record, BINDINGS + 0x40 + 0xC, 4); // button 1: bit 2
        put(&mut record, BUTTONS, 4);
        let mut keys = [0u8; 0x100];
        keys[0x48] = 0x80;
        let mut state = [0u8; 0x50];
        state[0x20..0x24].copy_from_slice(&9000u32.to_le_bytes());
        let r = read_device(&record, &keys, &state).unwrap().unwrap();
        assert_eq!((r.steering, r.throttle, r.brake), (0x80, FULL, 0));
        assert_eq!(r.bits, (64 << 24) | 0xE00 | 0x4000 | 1); // hat kind 0: no direction
        put(&mut record, READING, 0);
        assert_eq!(read_device(&record, &keys, &state), Ok(None));
        put(&mut record, BINDINGS + 4, 0x4F8);
        assert!(read_device(&record, &keys, &state).is_err());
        assert_eq!(
            binding_place(0x403, true),
            Place {
                axis: 0x2C,
                sign: u32::MAX,
                button: 4,
                ecx: Some(2)
            }
        );
        assert_eq!(binding_place(0x403, false).ecx, None);
    }
}
