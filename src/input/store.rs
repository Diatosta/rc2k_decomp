//! The input module's variables, kept with the image's layout (`notes/conversion.md`).
//! Three blocks, one `#[repr(C)]` struct each (the keyboard file's layouts are
//! `input::layouts`):
//! - `Devices` (`DEVICES`), 0x4F68D8 .. 0x4F7E18: the settings' switches (`nojoysticks` ...),
//!   the DirectInput objects, the joystick's state, caps and properties, the key tables (down,
//!   now, before, pressed, repeat: a byte a key), the mouse word, the two device records
//!   (0x4F8 bytes each, `types::InputDevice`; `input_devices` points at them), the
//!   controllers' table (`types::Controller`) and the input command;
//! - `Keyboard` (`KEYBOARD`), 0x4F7FD8 .. 0x4F8294: the yes/no key, the built-in layout's
//!   tables, the silent keys' bitmap and the dead keys' list (0x4F8000);
//! - `Mouse` (`MOUSE`), 0x5C6924 .. 0x5C6948: the mouse's position, clicks and buttons.
//!
//! The effects' GUIDs and the DirectInput identifiers (0x4F7E18 .. 0x4F7FA8) stay in the
//! image: the device records' effect slots point at them, and they are constants.
//!
//! The pointers in the data that name these blocks keep IDA's addresses (the layouts'
//! pointers to their tables); `input_devices` names the records by handle (`DEVICE_TAG`,
//! `Devices::device`).
//!
//! `ForceFeedback` (IDA 0x518084 .. 0x5180F0): the force-feedback effects' clock, the race
//! switch, the surfaces' effect kinds and the effects' constants (split from sound's block
//! around it).
//!
//! Threads: the input thread polls and the game's thread reads, as the image's bytes were
//! shared (atomic cells and volatile bytes).

use crate::errors::BadInput;
use crate::input::types::{Controller, InputDevice};
use crate::session::{Byte, Bytes, Dword, Float, PlainBytes, PlainDword};

// ---- first values, from the image (`store_data.rs`) ----

type Runs = &'static [(usize, &'static [u8])];

/// The image's byte at `at` of a block whose non-zero bytes are `runs`.
const fn byte_at(runs: Runs, at: usize) -> u8 {
    let mut r = 0;
    while r < runs.len() {
        let (offset, run) = runs[r];
        if at >= offset && at < offset + run.len() {
            return run[at - offset];
        }
        r += 1;
    }
    0
}

const fn init<const N: usize>(runs: Runs, at: usize) -> [u8; N] {
    let mut out = [0u8; N];
    let mut k = 0;
    while k < N {
        out[k] = byte_at(runs, at + k);
        k += 1;
    }
    out
}

const fn dword(runs: Runs, at: usize) -> Dword {
    Dword::new(u32::from_le_bytes(init::<4>(runs, at)))
}

const fn dwords<const N: usize>(runs: Runs, at: usize) -> [Dword; N] {
    let mut out = [const { Dword::new(0) }; N];
    let mut k = 0;
    while k < N {
        out[k] = dword(runs, at + 4 * k);
        k += 1;
    }
    out
}

const fn bytes<const N: usize>(runs: Runs, at: usize) -> Bytes<N> {
    Bytes::new(init::<N>(runs, at))
}

const fn plain_dword(runs: Runs, at: usize) -> PlainDword {
    PlainDword::new(u32::from_le_bytes(init::<4>(runs, at)))
}

const fn plain_dwords<const N: usize>(runs: Runs, at: usize) -> [PlainDword; N] {
    let mut out = [const { PlainDword::new(0) }; N];
    let mut k = 0;
    while k < N {
        out[k] = plain_dword(runs, at + 4 * k);
        k += 1;
    }
    out
}

const fn plain_bytes<const N: usize>(runs: Runs, at: usize) -> PlainBytes<N> {
    PlainBytes::new(init::<N>(runs, at))
}

const fn byte(runs: Runs, at: usize) -> Byte {
    Byte::new(byte_at(runs, at))
}

// ---- the devices: 0x4F68D8 .. 0x4F7E18 ----

/// A key table: a byte a scan code (0x80 down / pressed).
pub const KEYS: usize = 0x80;
/// A device record's size (`types::InputDevice`).
pub const RECORD_LEN: usize = 0x4F8;
/// A controller's entry (`types::Controller`).
pub const CONTROLLER_LEN: usize = 0x50;

/// The input thread's and the settings' variables.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Devices {
    /// `dword_4F68D8`: the input thread's last tick (ms).
    pub last_tick: Dword,
    /// `g_noJoysticks` (`nojoy`).
    pub no_joysticks: Dword,
    /// `dword_4F68E0` (`joymode`): the configuration's kind.
    pub joy_mode: Dword,
    /// `dword_4F68E4` (`joycode`): its axes' code.
    pub joy_code: Dword,
    /// `dword_4F68E8` (`joylowerdz`): its dead zone.
    pub joy_dead_zone: Dword,
    /// `dword_4F68EC` (`joyupperdz`): its end zone.
    pub joy_end_zone: Dword,
    /// `dword_4F68F0` (`alt2axcode`): its +0x1C.
    pub alt_axes_code: Dword,
    /// `g_noForceFeedback` (`noforcefeedback`).
    pub no_force_feedback: Dword,
    /// `dword_4F68F8` (`joyffgain`): its strength.
    pub ff_gain: Dword,
    /// `g_twoDevicesOnePlayer` (`mergejoy`).
    pub two_devices_one_player: Dword,
    /// `dword_4F6900` (`nonlinearsteering`).
    pub nonlinear_steering: Dword,
    /// `g_directInput`: the IDirectInputA.
    pub direct_input: Dword,
    /// `dword_4F6908` and `dword_4F690C`: nothing reads them.
    pub _unused_6908: [Dword; 2],
    /// `g_newJoystick`: a device being made.
    pub new_joystick: Dword,
    /// `g_newJoystick2`: its IDirectInputDevice2A.
    pub new_joystick2: Dword,
    /// `byte_4F6918` (0x100 bytes) and `dword_4F6A18` (16): nothing reads them.
    pub _gap_6918: Bytes<0x110>,
    /// `g_deviceState`: the device's state, 0x50 bytes (DirectInput's `DIJOYSTATE`).
    pub device_state: [Dword; 20],
    /// `g_joystickCaps`: a DIDEVCAPS (0x2C bytes).
    pub joystick_caps: Bytes<0x2C>,
    /// `g_dwordProperty`: a DIPROPDWORD (0x14).
    pub dword_property: Bytes<0x14>,
    /// `g_rangeProperty`: a DIPROPRANGE (0x18).
    pub range_property: Bytes<0x18>,
    /// `byte_4F6AD0`: the dead key pending (`key_to_char`).
    pub dead_key: Byte,
    /// 0x4F6AD1 .. 0x4F6AD4, `dword_4F6AD4`, `dword_4F6AD8`: nothing reads them.
    pub _gap_6ad1: Bytes<0xB>,
    /// `g_keysDown` (0x4F6ADC): what the window's messages keep, by scan code; its first
    /// byte the pending key of `input::take_key`.
    pub keys_down: Bytes<KEYS>,
    /// `g_keysNow` (0x4F6B5C): the frame's copy.
    pub keys_now: Bytes<KEYS>,
    /// `g_keysBefore` (0x4F6BDC): last frame's.
    pub keys_before: Bytes<KEYS>,
    /// 0x4F6C5C .. 0x4F6CDC: nothing reads it.
    pub _gap_6c5c: Bytes<KEYS>,
    /// `g_keysPressed` (0x4F6CDC).
    pub keys_pressed: Bytes<KEYS>,
    /// `g_keyRepeat` (0x4F6D5C): the frames a key has been held.
    pub key_repeat: Bytes<KEYS>,
    /// `dword_4F6DDC` .. `dword_4F6DEC`: nothing reads them.
    pub _unused_6ddc: [Dword; 5],
    /// `g_mouseWord`: buttons, x << 8, y << 20.
    pub mouse_word: Dword,
    /// `dword_4F6DF4` .. 0x4F6E04: nothing reads them.
    pub _unused_6df4: [Dword; 5],
    /// `unk_4F6E08`: the force-feedback effects' parameter rows (16 bytes each).
    pub effect_rows: [[Dword; 4]; 14],
    /// `g_inputDevices` and `off_4F6EEC`: the two records, by handle (`DEVICE_TAG | n`; the
    /// original kept their addresses and never changes them; 0 none).
    pub input_devices: [Dword; 2],
    /// `g_inputRecord1` and `g_inputRecord2` (`types::InputDevice`).
    pub records: [InputDevice; 2],
    /// `g_controllerCount`.
    pub controller_count: Dword,
    /// `g_controllers`: 16 entries (`types::Controller`).
    pub controllers: [Controller; 16],
    /// `g_inputCommand`.
    pub input_command: Dword,
    /// 0x4F7DE8 .. 0x4F7E18: three GUIDs the effect slots point at.
    pub guids: Bytes<0x30>,
}

const fn records(runs: Runs, at: usize) -> [InputDevice; 2] {
    [
        InputDevice::from_bytes(&with_guid_refs(init::<RECORD_LEN>(runs, at))),
        InputDevice::from_bytes(&with_guid_refs(init::<RECORD_LEN>(runs, at + RECORD_LEN))),
    ]
}

/// The six effects' `wanted` (+8 of each `types::ForceEffect`, 0xA4 apart from +0x3C of a
/// record): their GUIDs by handle (`guids::guid_ref`) where the image has their addresses.
pub const WANTED_AT: usize = 0x44;
pub const EFFECT_LEN: usize = 0xA4;
pub const EFFECTS: usize = 6;

const fn with_guid_refs(mut record: [u8; RECORD_LEN]) -> [u8; RECORD_LEN] {
    let mut k = 0;
    while k < EFFECTS {
        let o = WANTED_AT + EFFECT_LEN * k;
        let v = u32::from_le_bytes([record[o], record[o + 1], record[o + 2], record[o + 3]]);
        let h = crate::input::guids::guid_ref(v).to_le_bytes();
        record[o] = h[0];
        record[o + 1] = h[1];
        record[o + 2] = h[2];
        record[o + 3] = h[3];
        k += 1;
    }
    record
}

const fn controllers(runs: Runs, at: usize) -> [Controller; 16] {
    let mut out = [const { Controller::from_le_bytes(&[0; CONTROLLER_LEN]) }; 16];
    let mut k = 0;
    while k < 16 {
        out[k] = Controller::from_le_bytes(&init::<CONTROLLER_LEN>(runs, at + CONTROLLER_LEN * k));
        k += 1;
    }
    out
}

const fn rows(runs: Runs, at: usize) -> [[Dword; 4]; 14] {
    let mut out = [const { [const { Dword::new(0) }; 4] }; 14];
    let mut k = 0;
    while k < 14 {
        out[k] = dwords(runs, at + 16 * k);
        k += 1;
    }
    out
}

impl Devices {
    pub const fn new() -> Self {
        {
            use crate::input::store_data::DEVICES_RUNS as R;
            Devices {
                last_tick: dword(R, 0x0),
                no_joysticks: dword(R, 0x4),
                joy_mode: dword(R, 0x8),
                joy_code: dword(R, 0xC),
                joy_dead_zone: dword(R, 0x10),
                joy_end_zone: dword(R, 0x14),
                alt_axes_code: dword(R, 0x18),
                no_force_feedback: dword(R, 0x1C),
                ff_gain: dword(R, 0x20),
                two_devices_one_player: dword(R, 0x24),
                nonlinear_steering: dword(R, 0x28),
                direct_input: dword(R, 0x2C),
                _unused_6908: dwords(R, 0x30),
                new_joystick: dword(R, 0x38),
                new_joystick2: dword(R, 0x3C),
                _gap_6918: bytes(R, 0x40),
                device_state: dwords(R, 0x150),
                joystick_caps: bytes(R, 0x1A0),
                dword_property: bytes(R, 0x1CC),
                range_property: bytes(R, 0x1E0),
                dead_key: byte(R, 0x1F8),
                _gap_6ad1: bytes(R, 0x1F9),
                keys_down: bytes(R, 0x204),
                keys_now: bytes(R, 0x284),
                keys_before: bytes(R, 0x304),
                _gap_6c5c: bytes(R, 0x384),
                keys_pressed: bytes(R, 0x404),
                key_repeat: bytes(R, 0x484),
                _unused_6ddc: dwords(R, 0x504),
                mouse_word: dword(R, 0x518),
                _unused_6df4: dwords(R, 0x51C),
                effect_rows: rows(R, 0x530),
                input_devices: [Dword::new(DEVICE_TAG), Dword::new(DEVICE_TAG | 1)],
                records: records(R, 0x618),
                controller_count: dword(R, 0x1008),
                controllers: controllers(R, 0x100C),
                input_command: dword(R, 0x150C),
                guids: bytes(R, 0x1510),
            }
        }
    }
}

#[cfg(test)]
pub const DEVICES_IDA: usize = 0x4F68D8;
#[cfg(test)]
pub const DEVICES_LEN: usize = core::mem::size_of::<Devices>();

impl Devices {
    /// `g_keysDown` and `g_keysNow` as one table of 0x100 bytes (the key table the device
    /// bindings read: a key code past 0x7F reads `g_keysNow`).
    pub fn key_table(&self) -> [u8; 0x100] {
        let mut table = [0u8; 0x100];
        table[..KEYS].copy_from_slice(&self.keys_down.read());
        table[KEYS..].copy_from_slice(&self.keys_now.read());
        table
    }

    /// Device `n`'s record (`g_inputDevices[n]`, by handle: `device_of`); `None` for a
    /// device the table holds as 0.
    pub fn device(&self, n: usize) -> Option<&InputDevice> {
        device_of(self, self.input_devices[n].get())
    }

    /// `g_inputDevices[n]` as the original indexes it: past the two pointers (`n` up to 7)
    /// it reads the first record's bytes (the drive code only refuses `n` past 7). What
    /// those hold is no handle: a 0 is no device, any other value the original would
    /// follow as an address, which is refused here.
    pub fn input_device(&self, n: usize) -> Result<Option<&InputDevice>, BadInput> {
        if n < self.input_devices.len() {
            return Ok(self.device(n));
        }
        let at = 4 * (n - self.input_devices.len());
        // A run-on past the pointers into the first record's fields, as the original.
        let held = self.records[0]
            .to_bytes()
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .ok_or(BadInput("an input device past the table"))?;
        if held == 0 {
            return Ok(None);
        }
        Err(BadInput(
            "an input device past the table that holds no record's handle",
        ))
    }

    /// Key `key`'s state in that table (the window's messages write it: 0xFF down).
    pub fn set_key_state(&self, key: u8, state: u8) {
        let key = key as usize;
        if key < KEYS {
            self.keys_down.set(key, state);
        } else {
            self.keys_now.set(key - KEYS, state);
        }
    }
}

/// A device record by handle: `DEVICE_TAG | n` for `records[n]`.
pub const DEVICE_TAG: u32 = 0xCE00_0000;

/// The record a handle names (`None` for 0). A value that is no handle ends the program.
pub fn device_of(devices: &Devices, handle: u32) -> Option<&InputDevice> {
    if handle == 0 {
        return None;
    }
    match devices.records.get((handle ^ DEVICE_TAG) as usize) {
        Some(record) => Some(record),
        None => crate::fatal::raise::end(
            "input::store::device_of",
            crate::errors::BadInput("a device record's handle that names none"),
        ),
    }
}

// ---- the keyboard: 0x4F7FD8 .. 0x4F8294 ----

/// The yes/no key, the built-in layout and the dead keys.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Keyboard {
    /// `g_yesNoKeyGiven`: 1 = the key comes from the caller.
    pub yes_no_key_given: PlainDword,
    /// `unk_4F7FDC`: the built-in layout's five pointers (nothing reads them: `input::layouts`
    /// copies the tables below).
    pub _built_in_layout: [PlainDword; 5],
    /// `dword_4F7FF0`: one bit per key; the keys set type nothing.
    pub silent_keys: [PlainDword; 4],
    /// `unk_4F8000`: the dead keys' list {dead key, base, plain result, shifted result}.
    pub compose: PlainBytes<0x6C>,
    /// 0x4F806C: the built-in layout's plain table.
    pub plain: PlainBytes<0x80>,
    /// 0x4F80EC: its shifted table.
    pub shifted: PlainBytes<0x80>,
    /// 0x4F816C .. 0x4F826C: its key names' table.
    pub key_names: PlainBytes<0x100>,
    /// `dword_4F826C`.
    pub _unused_826c: PlainDword,
    /// `unk_4F8270`: the axes' rest values (6 dwords, taken by `input::axis_moved`).
    pub axes_rest: [PlainDword; 6],
    /// 0x4F8288 .. 0x4F8294.
    pub _tail: PlainBytes<0xC>,
}

impl Keyboard {
    pub const fn new() -> Self {
        {
            use crate::input::store_data::KEYBOARD_RUNS as R;
            Keyboard {
                yes_no_key_given: plain_dword(R, 0x0),
                _built_in_layout: plain_dwords(R, 0x4),
                silent_keys: plain_dwords(R, 0x18),
                compose: plain_bytes(R, 0x28),
                plain: plain_bytes(R, 0x94),
                shifted: plain_bytes(R, 0x114),
                key_names: plain_bytes(R, 0x194),
                _unused_826c: plain_dword(R, 0x294),
                axes_rest: plain_dwords(R, 0x298),
                _tail: plain_bytes(R, 0x2B0),
            }
        }
    }
}

#[cfg(test)]
pub const KEYBOARD_IDA: usize = 0x4F7FD8;
#[cfg(test)]
pub const KEYBOARD_LEN: usize = core::mem::size_of::<Keyboard>();

// ---- the mouse: 0x5C6924 .. 0x5C6948 ----

/// The mouse's position, clicks and buttons.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Mouse {
    /// `g_mouseX`, `g_mouseY` (signed).
    pub x: PlainDword,
    pub y: PlainDword,
    /// `g_lastMouseX`, `g_lastMouseY`: last frame's.
    pub last_x: PlainDword,
    pub last_y: PlainDword,
    /// `g_leftClick`, `g_rightClick`.
    pub left_click: PlainDword,
    pub right_click: PlainDword,
    /// `g_mouseButtons`: the buttons' bits; `g_leftButton` (bit 0), `g_rightButton` (bit 1).
    pub buttons: PlainDword,
    pub left_button: PlainDword,
    pub right_button: PlainDword,
}

impl Mouse {
    pub const fn new() -> Self {
        Mouse {
            x: PlainDword::new(0),
            y: PlainDword::new(0),
            last_x: PlainDword::new(0),
            last_y: PlainDword::new(0),
            left_click: PlainDword::new(0),
            right_click: PlainDword::new(0),
            buttons: PlainDword::new(0),
            left_button: PlainDword::new(0),
            right_button: PlainDword::new(0),
        }
    }
}

#[cfg(test)]
pub const MOUSE_LEN: usize = core::mem::size_of::<Mouse>();

/// The force-feedback effects' clock, surface kinds and constants (IDA 0x518084 .. 0x5180f0, the image's layout).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ForceFeedback {
    /// `g_effectsLastTime`: the force-feedback effects' last clock.
    pub last_time: Dword,
    /// `g_effectsRate`: their rate.
    pub rate: Dword,
    /// `g_effectsSteps`: their steps.
    pub steps: Dword,
    /// `g_effectsRaceOn`: the effects run in a race.
    pub race_on: Dword,
    /// `0x518094`, `0x518098`: nothing reads or writes them.
    pub _gap_94: [Dword; 2],
    /// `g_surfaceEffectKinds` (`0x51809C`): a surface's effect kind, 16 of them.
    pub surface_effect_kinds: [Dword; 16],
    /// `flt_5180DC`: 10.0.
    pub effect_ten: Float,
    /// `flt_5180E0`: 162.97.
    pub effect_angle_scale: Float,
    /// `flt_5180E4`: 256.0.
    pub effect_full: Float,
    /// `0x5180E8`: nothing reads or writes it.
    pub _gap_e8: Dword,
    /// `flt_5180EC`: 0.1.
    pub effect_tenth: Float,
}

impl ForceFeedback {
    pub const fn new() -> Self {
        ForceFeedback {
            last_time: Dword::new(0),
            rate: Dword::new(0),
            steps: Dword::new(0),
            race_on: Dword::new(0),
            _gap_94: [Dword::new(0), Dword::new(0)],
            surface_effect_kinds: [
                Dword::new(0),
                Dword::new(1),
                Dword::new(2),
                Dword::new(3),
                Dword::new(6),
                Dword::new(9),
                Dword::new(4),
                Dword::new(0xD),
                Dword::new(0xC),
                Dword::new(8),
                Dword::new(0xA),
                Dword::new(7),
                Dword::new(0xB),
                Dword::new(0),
                Dword::new(0),
                Dword::new(0),
            ],
            effect_ten: Float::new(10.0),
            effect_angle_scale: Float::new(162.97466),
            effect_full: Float::new(256.0),
            _gap_e8: Dword::new(0x42200000),
            effect_tenth: Float::new(0.1),
        }
    }
}

impl ForceFeedback {
    pub const START: usize = 0x518084;
    pub const LEN: usize = 0x6c;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::first_values;
    use core::mem::offset_of;

    /// The game's devices block (a named read of `data/first_values.golden`) and its store.
    const DEVICES_IMAGE: &str = "input.owner.INPUT.devices/5440";
    const DEVICES_STORE: &str = "store:input.owner.INPUT.devices";

    #[test]
    fn the_devices_start_as_the_games() {
        // The two records' places and the effects' wanted GUIDs are handles where the game
        // holds addresses: the golden bytes are made handles first.
        let _g = crate::test_support::test_lock();
        let mut image = first_values::bytes(DEVICES_IMAGE).to_vec();
        let devices = offset_of!(Devices, input_devices);
        for r in 0..2 {
            let o = devices + 4 * r;
            let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
            assert_eq!(
                v as usize,
                DEVICES_IDA + offset_of!(Devices, records) + RECORD_LEN * r
            );
            image[o..o + 4].copy_from_slice(&(DEVICE_TAG | r as u32).to_le_bytes());
        }
        let records = offset_of!(Devices, records);
        for r in 0..2 {
            for k in 0..EFFECTS {
                let o = records + RECORD_LEN * r + WANTED_AT + EFFECT_LEN * k;
                let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                image[o..o + 4].copy_from_slice(&crate::input::guids::guid_ref(v).to_le_bytes());
            }
        }
        assert!(first_values::store_bytes(DEVICES_STORE) == image);
    }

    #[test]
    fn the_blocks_keep_the_images_layout() {
        assert_eq!(DEVICES_LEN, 0x1540);
        for (offset, games) in [
            (offset_of!(Devices, direct_input), 0x2C),
            (offset_of!(Devices, new_joystick2), 0x3C),
            (offset_of!(Devices, device_state), 0x150),
            (offset_of!(Devices, joystick_caps), 0x1A0),
            (offset_of!(Devices, dword_property), 0x1CC),
            (offset_of!(Devices, range_property), 0x1E0),
            (offset_of!(Devices, dead_key), 0x1F8),
            (offset_of!(Devices, keys_down), 0x204),
            (offset_of!(Devices, keys_now), 0x284),
            (offset_of!(Devices, keys_before), 0x304),
            (offset_of!(Devices, keys_pressed), 0x404),
            (offset_of!(Devices, key_repeat), 0x484),
            (offset_of!(Devices, mouse_word), 0x518),
            (offset_of!(Devices, effect_rows), 0x530),
            (offset_of!(Devices, input_devices), 0x610),
            (offset_of!(Devices, records), 0x618),
            (offset_of!(Devices, controller_count), 0x1008),
            (offset_of!(Devices, controllers), 0x100C),
            (offset_of!(Devices, input_command), 0x150C),
            (offset_of!(Devices, guids), 0x1510),
        ] {
            assert_eq!(offset, games);
        }
        assert_eq!(KEYBOARD_LEN, 0x2BC);
        assert_eq!(offset_of!(Keyboard, silent_keys), 0x18);
        assert_eq!(offset_of!(Keyboard, compose), 0x28);
        assert_eq!(offset_of!(Keyboard, plain), 0x94);
        assert_eq!(offset_of!(Keyboard, axes_rest), 0x298);
        assert_eq!(MOUSE_LEN, 0x24);
    }

    #[test]
    fn force_feedback_starts_as_the_games() {
        assert_eq!(size_of::<ForceFeedback>(), ForceFeedback::LEN);
    }
}
