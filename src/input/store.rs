//! The input module's variables, three records of named fields (the keyboard file's layouts
//! are `input::layouts`):
//! - `Devices` (`DEVICES`): the settings' switches (`nojoysticks` ...), the DirectInput
//!   objects, the joystick's state, caps and properties, the key tables (down, now, before,
//!   pressed, repeat: a byte a key), the mouse word, the two device records
//!   (0x4F8 bytes each, `types::InputDevice`; `input_devices` points at them), the controllers'
//!   table (`types::Controller`) and the input command;
//! - `Keyboard` (`KEYBOARD`): the yes/no key, the built-in layout's tables, the silent keys'
//!   bitmap and the dead keys' list;
//! - `Mouse` (`MOUSE`): the mouse's position, clicks and buttons.
//!
//! The effects' GUIDs and the DirectInput identifiers are constants (`input::guids`): the device
//! records' effect slots name them by handle. `input_devices` names the records by handle
//! (`DEVICE_TAG`, `Devices::device`).
//!
//! `ForceFeedback`: the force-feedback effects' clock, the race switch, the surfaces' effect
//! kinds and the effects' constants.
//!
//! Threads: the input thread polls and the game's thread reads (atomic cells and volatile
//! bytes).

use crate::cells::ZeroCells;
use crate::errors::BadInput;
use crate::input::guids::EffectGuid;
use crate::input::types::{Controller, ForceEffect, InputDevice};
use crate::session::{Byte, Bytes, Dword, Float, PlainBytes, PlainDword};

// ---- the devices ----

/// A key table: a byte a scan code (0x80 down / pressed).
pub const KEYS: usize = 0x80;

/// The input thread's and the settings' variables.
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Devices {
    /// The input thread's last tick (ms).
    pub last_tick: Dword,
    /// `nojoy`: no joysticks listed.
    pub no_joysticks: Dword,
    /// `joymode`: the configuration's kind.
    pub joy_mode: Dword,
    /// `joycode`: its axes' code.
    pub joy_code: Dword,
    /// `joylowerdz`: its dead zone.
    pub joy_dead_zone: Dword,
    /// `joyupperdz`: its end zone.
    pub joy_end_zone: Dword,
    /// `alt2axcode`: its +0x1C.
    pub alt_axes_code: Dword,
    /// `noforcefeedback`.
    pub no_force_feedback: Dword,
    /// `joyffgain`: its strength.
    pub ff_gain: Dword,
    /// `mergejoy`: one player on both devices.
    pub two_devices_one_player: Dword,
    /// `nonlinearsteering`.
    pub nonlinear_steering: Dword,
    /// The `IDirectInputA`.
    pub direct_input: Dword,
    pub _unused_6908: [Dword; 2],
    /// A device being made.
    pub new_joystick: Dword,
    /// Its `IDirectInputDevice2A`.
    pub new_joystick2: Dword,
    /// The device's state, 0x50 bytes (DirectInput's `DIJOYSTATE`).
    pub device_state: [Dword; 20],
    /// A `DIDEVCAPS`.
    pub joystick_caps: Bytes<0x2C>,
    /// A `DIPROPDWORD`.
    pub dword_property: Bytes<0x14>,
    /// A `DIPROPRANGE`.
    pub range_property: Bytes<0x18>,
    /// The dead key pending (`key_to_char`).
    pub dead_key: Byte,
    /// Nothing reads them; they keep the key tables after them on a dword's boundary (the cells
    /// take no padding).
    pub _gap_6ad1: Bytes<0xB>,
    /// What the window's messages keep, by scan code; its first
    /// byte the pending key of `input::take_key`.
    pub keys_down: Bytes<KEYS>,
    /// The frame's copy.
    pub keys_now: Bytes<KEYS>,
    /// Last frame's.
    pub keys_before: Bytes<KEYS>,
    /// The keys that went down this frame, or auto-repeat.
    pub keys_pressed: Bytes<KEYS>,
    /// The frames a key has been held.
    pub key_repeat: Bytes<KEYS>,
    pub _unused_6ddc: [Dword; 5],
    /// Buttons, x << 8, y << 20.
    pub mouse_word: Dword,
    pub _unused_6df4: [Dword; 5],
    /// The force-feedback effects' parameter rows (16 bytes each).
    pub effect_rows: [[Dword; 4]; 14],
    /// The two records, by handle (`DEVICE_TAG | n`, never changed; 0 none).
    pub input_devices: [Dword; 2],
    pub records: [InputDevice; 2],
    pub controller_count: Dword,
    pub controllers: [Controller; 16],
    pub input_command: Dword,
    /// Three GUIDs the effect slots point at.
    pub guids: Bytes<0x30>,
}

const EFFECTS: usize = 6;

/// The six effects of a record, in order: the change function (`input::effect_N_change`), the
/// kind wanted (1 constant, 3 periodic, 4 condition) and the GUID wanted.
const EFFECT_KINDS: [(u32, u32, EffectGuid); EFFECTS] = [
    (0x409708, 4, EffectGuid::Spring),
    (0x4097FA, 3, EffectGuid::Sine),
    (0x409754, 3, EffectGuid::Sine),
    (0x4097B4, 1, EffectGuid::ConstantForce),
    (0x409871, 4, EffectGuid::Damper),
    (0x4098BF, 1, EffectGuid::ConstantForce),
];

/// A device record as the game starts it: the force feedback at 75 with the dead and end zones
/// at 10, the axes in the state's fifth, third and second dwords (`axis_code` 0x02011000), the
/// second axis pick, sensitivity 2, and its six effects set up by `EFFECT_KINDS`.
const fn record(controller: u32) -> InputDevice {
    let mut effects = [const { ForceEffect::ZERO }; EFFECTS];
    let mut k = 0;
    while k < EFFECTS {
        let (change, kind, wanted) = EFFECT_KINDS[k];
        effects[k] = ForceEffect {
            change: Dword::new(change),
            kind: Dword::new(kind),
            wanted: Dword::new(wanted.handle()),
            ..ForceEffect::ZERO
        };
        k += 1;
    }
    InputDevice {
        controller: Dword::new(controller),
        strength: Dword::new(75),
        dead_zone: Dword::new(10),
        end_zone: Dword::new(10),
        axis_code: Dword::new(0x0201_1000),
        axis_pick: Dword::new(1),
        sensitivity: Dword::new(2),
        effects,
        ..InputDevice::ZERO
    }
}

/// The three interface identifiers the device set-up passes to DirectInput (`IDirectInputA`,
/// `IDirectInput2A`, `IDirectInputDevice2A`), 16 bytes each.
const INTERFACE_GUIDS: [u8; 0x30] = *b"\x60\x13\x52\x89\x8a\xaa\xcf\x11\xbf\xc7\x44\x45\x53\x54\x00\x00\x62\xe6\x44\x59\x8a\xaa\xcf\x11\xbf\xc7\x44\x45\x53\x54\x00\x00\x80\xe6\x44\x59\x2e\xc9\xcf\x11\xbf\xc7\x44\x45\x53\x54\x00\x00";

const EFFECT_ROWS: [[u32; 4]; 14] = [
    [0x30, 0x64, 0x0, 0x226],
    [0x10, 0x32, 0x20, 0x14A],
    [0x1E, 0x64, 0x96, 0xC8],
    [0x20, 0x50, 0x80, 0xB4],
    [0x32, 0x4B, 0x64, 0x96],
    [0x23, 0x64, 0x80, 0xC8],
    [0xA, 0x64, 0x20, 0x12C],
    [0x28, 0x5A, 0x8C, 0x10E],
    [0x0, 0x190, 0x0, 0x320],
    [0x64, 0x50, 0xC8, 0x50],
    [0x7D, 0x14, 0xFA, 0xFA],
    [0x21, 0x64, 0x8C, 0xFA],
    [0x64, 0x64, 0xFA, 0xC8],
    [0x20, 0x32, 0x96, 0x50],
];

const fn effect_rows() -> [[Dword; 4]; 14] {
    let mut out: [[Dword; 4]; 14] = ZeroCells::ZERO;
    let mut k = 0;
    while k < 14 {
        let mut j = 0;
        while j < 4 {
            out[k][j] = Dword::new(EFFECT_ROWS[k][j]);
            j += 1;
        }
        k += 1;
    }
    out
}

impl Devices {
    pub const fn new() -> Self {
        Devices {
            // -1: none picked (the configuration's axes, zones and strength).
            joy_code: Dword::new(u32::MAX),
            joy_dead_zone: Dword::new(u32::MAX),
            joy_end_zone: Dword::new(u32::MAX),
            alt_axes_code: Dword::new(u32::MAX),
            ff_gain: Dword::new(u32::MAX),
            effect_rows: effect_rows(),
            input_devices: [Dword::new(DEVICE_TAG), Dword::new(DEVICE_TAG | 1)],
            records: [record(0), record(1)],
            guids: Bytes::new(INTERFACE_GUIDS),
            ..ZeroCells::ZERO
        }
    }
}

impl Devices {
    /// The key table the device bindings read, as one table of 0x100 bytes: a key code past 0x7F
    /// reads the frame's table.
    pub fn key_table(&self) -> [u8; 0x100] {
        let mut table = [0u8; 0x100];
        table[..KEYS].copy_from_slice(&self.keys_down.read());
        table[KEYS..].copy_from_slice(&self.keys_now.read());
        table
    }

    /// Device `n`'s record (by handle: `device_of`); `None` for a device the table holds as 0.
    pub fn device(&self, n: usize) -> Option<&InputDevice> {
        device_of(self, self.input_devices[n].get())
    }

    /// Device `n` as the drive code indexes it: past the two handles (`n` up to 7) it reads the
    /// first record's bytes (the drive code only refuses `n` past 7). What those hold is no
    /// handle: a 0 is no device, any other value is refused.
    pub fn input_device(&self, n: usize) -> Result<Option<&InputDevice>, BadInput> {
        if n < self.input_devices.len() {
            return Ok(self.device(n));
        }
        let at = 4 * (n - self.input_devices.len());
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

// ---- the keyboard ----

/// The yes/no key, the built-in layout and the dead keys.
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Keyboard {
    /// 1 = the key comes from the caller.
    pub yes_no_key_given: PlainDword,
    /// The built-in layout's five pointers (nothing reads them: `input::layouts`
    /// copies the tables below).
    pub _built_in_layout: [PlainDword; 5],
    /// One bit per key; the keys set type nothing.
    pub silent_keys: [PlainDword; 4],
    /// The dead keys' list {dead key, base, plain result, shifted result}.
    pub compose: PlainBytes<0x6C>,
    /// The built-in layout's plain table.
    pub plain: PlainBytes<0x80>,
    /// Its shifted table.
    pub shifted: PlainBytes<0x80>,
    /// Its key names' table.
    pub key_names: PlainBytes<0x100>,
    pub _unused_826c: PlainDword,
    /// The axes' rest values (6 dwords, taken by `input::axis_moved`).
    pub axes_rest: [PlainDword; 6],
}

impl Keyboard {
    pub const fn new() -> Self {
        use crate::cells::ZeroCells;
        use crate::session::{PlainBytes, PlainDword, with_runs};
        Self {
_built_in_layout: {
let mut v: [PlainDword; 5] = ZeroCells::ZERO;
v[1] = PlainDword::new(0x4F806C); // a game address nothing reads
v[2] = PlainDword::new(0x4F80EC); // a game address nothing reads
v[4] = PlainDword::new(0x4F816C); // a game address nothing reads
v
},
silent_keys: {
let mut v: [PlainDword; 4] = ZeroCells::ZERO;
v[0] = PlainDword::new(0x20000000);
v[1] = PlainDword::new(0x5400400);
v[2] = PlainDword::new(96);
v[3] = PlainDword::new(18);
v
},
compose: PlainBytes::new(with_runs::<108>(&[(0, b"\x01a\xe1\xc1\x01e\xe9\xc9\x01i\xed\xcd\x01o\xf3\xd3\x01u\xfa\xda\x01y\xfd\xdd\x04a\xe0\xc0\x04e\xe8\xc8\x04i\xec\xcc\x04o\xf2\xd2\x04u\xf9\xd9\x02a\xe2\xc2\x02e\xea\xca\x02i\xee\xce\x02o\xf4\xd4\x02u\xfb\xdb\x06a\xe4\xc4\x06e\xeb\xcb\x06i\xef\xcf\x06o\xf6\xd6\x06u\xfc\xdc\x06y\x9f\xff\x03a\xe5\xc5\x05a\xe3\xc3\x05o\xf5\xd5\x05n\xf1\xd1")])),
plain: PlainBytes::new(with_runs::<128>(&[(2, b"1234567890-=\x00\x00qwertyuiop[]\x00\x00asdfghjkl;'`\x00#zxcvbnm,./\x00*\x00 "), (74, b"-\x00\x00\x00+\x00\x00\x00\x00\x00\x00\x00\\"), (98, b"/")])),
shifted: PlainBytes::new(with_runs::<128>(&[(2, b"!\"\xa3$%^&*()_+\x00\x00QWERTYUIOP{}\x00\x00ASDFGHJKL:@\xac\x00~ZXCVBNM<>?\x00*\x00 "), (71, b"789-456+1230.\x00\x00|"), (98, b"/")])),
key_names: PlainBytes::new(with_runs::<256>(&[(2, b"\x05\x00\x06\x00\x07\x00\x08\x00\x09\x00\x0a\x00\x0b\x00\x0c\x00\x0d\x00\x0e\x00\x0f\x00\x10\x00\x11\x00\x12\x00\x13\x00\x14\x00\x15\x00\x16\x00\x17\x00\x18\x00\x19\x00\x1a\x00\x1b\x00\x1c\x00\x1d\x00\x1e\x00\x1f\x00 \x00!\x00\"\x00#\x00$\x00%\x00&\x00'\x00(\x00)\x00*\x00+\x00,\x00-\x00.\x00/\x000\x001\x002\x003\x004\x005\x006\x007\x008\x009\x00:\x00;\x00<\x00=\x00>\x00?\x00@\x00A\x00B\x00C\x00D\x00E\x00F\x00G\x00H\x00I\x00J\x00K\x00L\x00M\x00N\x00O\x00P\x00Q\x00R\x00S\x00T\x00U\x00V\x00W\x00X\x00\x00\x00Y\x00Z\x00["), (192, b"\\\x00]\x00^\x00_\x00`\x00a\x00b\x00c\x00d\x00e\x00f\x00g\x00h\x00i\x00j\x00\x00\x00k\x00l")])),
..ZeroCells::ZERO
}
    }
}

/// The mouse's position, clicks and buttons.
#[derive(rc2k_derive::Cells)]
pub struct Mouse {
    /// The mouse's x and y (signed).
    pub x: PlainDword,
    pub y: PlainDword,
    /// Last frame's x and y.
    pub last_x: PlainDword,
    pub last_y: PlainDword,
    /// The left and right clicks.
    pub left_click: PlainDword,
    pub right_click: PlainDword,
    /// The buttons' bits (bit 0 left, bit 1 right).
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

/// The force-feedback effects' clock, surface kinds and constants.
#[derive(rc2k_derive::Cells)]
pub struct ForceFeedback {
    /// The force-feedback effects' last clock.
    pub last_time: Dword,
    pub rate: Dword,
    pub steps: Dword,
    /// The effects run in a race.
    pub race_on: Dword,
    /// A surface's effect kind, 16 of them.
    pub surface_effect_kinds: [Dword; 16],
    /// 10.0.
    pub effect_ten: Float,
    /// 162.97.
    pub effect_angle_scale: Float,
    /// 256.0.
    pub effect_full: Float,
    /// 0.1.
    pub effect_tenth: Float,
}

impl ForceFeedback {
    pub const fn new() -> Self {
        ForceFeedback {
            last_time: Dword::new(0),
            rate: Dword::new(0),
            steps: Dword::new(0),
            race_on: Dword::new(0),
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
            effect_tenth: Float::new(0.1),
        }
    }
}
