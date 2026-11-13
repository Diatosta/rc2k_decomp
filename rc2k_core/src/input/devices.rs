//! A player's input device read (from `input::read_device` 0x408E38,
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

/// The bits reading with the hat centred.
pub const HAT_CENTRED: u32 = 0xE00;
/// What `input::clear_device_readings` (0x40883C) writes, in order:
/// the state copies (+0x2C .. +0x38) and the readings (+0x4DC .. +0x4E4) 0, the bits
/// reading the hat centred.
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
/// x 256 / 36000; else 0. Also the divisor and the remainder when the division was reached.
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
pub struct BindingSource {
    /// The axis's place (0 none).
    pub axis: u32,
    /// Its sign (1, -1; 0 when not acquired).
    pub sign: u32,
    /// The button's bit (0 none).
    pub button: u32,
}

pub fn binding_place(word: u32, acquired: bool) -> BindingSource {
    if !acquired {
        return BindingSource {
            axis: 0,
            sign: 0,
            button: 0,
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
    let button = if low == 0 {
        0
    } else {
        1u32.wrapping_shl(low - 1)
    };
    BindingSource { axis, sign, button }
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

/// Which axis moved past 0x80 from rest (`input::axis_moved` 0x40939F): of the six
/// (`now` less `rest`, signed) the first the controller has (`axes`, a bit an axis), its
/// number, + 0x10 when it went up. None: the last difference worked out.
pub fn axis_moved(now: &[u32; 6], rest: &[u32; 6], axes: u32) -> Result<u32, u32> {
    let mut last = 0;
    for n in 0..6 {
        let d = now[n].wrapping_sub(rest[n]) as i32;
        last = d as u32;
        if axes & (1 << n) == 0 {
            continue;
        }
        if d < -0x80 {
            return Ok(n as u32);
        }
        if d > 0x80 {
            return Ok(n as u32 | 0x10);
        }
    }
    Err(last)
}

/// One player on two devices (`input::input_tick` 0x408A56): the second's three readings
/// added to the first's, and of its bits those of 0x70FF.
pub fn merge_readings(first: &mut [u32; 4], second: &[u32; 4]) {
    for n in 0..3 {
        first[n] = first[n].wrapping_add(second[n]);
    }
    first[3] |= second[3] & 0x70FF;
}

/// The twelve bindings of a device's record (from
/// `input::set_up_device` 0x408BCD): each (the player's controls' key byte, the controls'
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

/// The device's kind from the controls' (0: 2).
pub fn device_kind(kind: u32) -> u32 {
    if kind == 0 { 2 } else { kind }
}

/// A binding's four dwords in the record: its key, and the place, sign and button of its
/// word (`binding_place`, `first` the record's +0x24 being 1).
pub fn binding(key: u8, word: u32, first: bool) -> [u32; 4] {
    let p = binding_place(word, first);
    [key as u32, p.axis, p.sign, p.button]
}

/// A step of the input thread's command (from `input::device_command`
/// 0x408AE5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandStep {
    /// `ForceFeedback.race_on`, the effects' switch.
    EffectsOn(bool),
    Release(usize),
    Acquire(usize),
    EndEffects,
    Tick,
    /// Device `device` set up from player `device`'s controls.
    SetUp(usize),
    /// The device's +0x414 (read).
    Reading(usize, bool),
    /// Player 2's controller made none when it is player 1's.
    SecondApart,
    Sleep(u32),
    BeginEffects,
}

/// The steps of command `command` (`device` too for 1): 1 the device given back and
/// acquired again; 2 (the race's end) the effects off and ended, a tick, both devices
/// given back and not read; 3 (the race's start) device 0 set up and read, device 1 too
/// unless one player on one device (`two_devices` 0 and `one_player` 1), a second's
/// sleep, the effects begun, a tick, the effects on. Refused: a device past 1 for 1.
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
            v.extend([Sleep(1000), BeginEffects, Tick, EffectsOn(true)]);
            v
        }
        _ => vec![],
    })
}

/// A control learnt from the controller's four values (from
/// `screens::classify_controller` 0x46B27A): the button (its number + 1, when exactly one is
/// down), and each of left (x below 0), right (x above 0), y and z shifted into its
/// history (a 1 for "under 0x80"); a direction held over 0x1D of the last 32 polls while
/// another's history is empty gives 0x400 (left empty) / 0x800 (right) / 0x100 (y) /
/// 0x200 (z) with the button; none empty, the button alone; else 0. The histories after,
/// and what was found.
/// The controller page's learning of a control: on,
/// when it began (the 3 s start), which control (1 .. 11).
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

pub fn classify(buttons: u32, x: u32, y: u32, z: u32, history: [u32; 4]) -> ([u32; 4], u32) {
    let mut button = 0u32;
    if buttons != 0 {
        let mut b = buttons;
        loop {
            button += 1;
            let out = b & 1;
            b >>= 1;
            if out != 0 {
                break;
            }
        }
        if b != 0 {
            button = 0;
        }
    }
    let left = if (x as i32) < 0 { x.wrapping_neg() } else { 0 };
    let right = if (x as i32) < 0 { 0 } else { x };
    let mut h = history;
    for (v, value) in h.iter_mut().zip([left, right, y, z]) {
        *v = (*v << 1) | u32::from(value < 0x80);
    }
    let [l, r, yy, zz] = h.map(u32::count_ones);
    let min = |a: u32, b: u32, c: u32| a.min(b).min(c);
    let bit = if l == 0 {
        (min(zz, r, yy) >= 0x1D).then_some(0x400)
    } else if r == 0 {
        (min(zz, l, yy) >= 0x1D).then_some(0x800)
    } else if yy == 0 {
        (min(zz, l, r) >= 0x1D).then_some(0x100)
    } else if zz == 0 {
        (min(l, r, yy) >= 0x1D).then_some(0x200)
    } else {
        Some(0)
    };
    (h, bit.map_or(0, |b| button | b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifying() {
        assert_eq!(classify(4, 0x100, 0x100, 0x100, [0; 4]).1, 0);
        assert_eq!(classify(4, 0x100, 0x100, 0x100, [u32::MAX; 4]).1, 3);
        assert_eq!(classify(6, 0x100, 0x100, 0x100, [u32::MAX; 4]).1, 0);
        let (h, f) = classify(0, 0x100, 0, 0, [0, 0, u32::MAX, u32::MAX]);
        assert_eq!((h[0], f), (1, 0));
    }

    #[test]
    fn commands() {
        assert!(command_steps(1, 2, 0, 0).is_err());
        assert_eq!(command_steps(3, 0, 0, 1).unwrap().len(), 6);
        assert_eq!(command_steps(3, 0, 1, 1).unwrap().len(), 9);
        assert!(command_steps(9, 0, 0, 0).unwrap().is_empty());
    }

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
            BindingSource {
                axis: 0x2C,
                sign: u32::MAX,
                button: 4,
            }
        );
    }

    #[test]
    fn axes_and_merges() {
        let rest = [0x100; 6];
        let mut now = rest;
        now[2] = 0x10;
        assert_eq!(axis_moved(&now, &rest, 0b100), Ok(2));
        assert_eq!(axis_moved(&now, &rest, 0b011), Err(0));
        now[1] = 0x200;
        assert_eq!(axis_moved(&now, &rest, 0b111), Ok(0x11));
        let mut a = [1, 2, 3, 0x8000];
        merge_readings(&mut a, &[1, 1, 1, 0xFFFF]);
        assert_eq!(a, [2, 3, 4, 0xF0FF]);
    }
}
