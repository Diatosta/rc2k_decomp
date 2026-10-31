//! The input module's variables, out of RAL.EXE's image (phase 5, `notes/conversion.md`;
//! typed in phase 7). Four blocks with the image's layout, one `#[repr(C)]` struct each:
//! - `KeyboardFile` (`KEYBOARD_FILE`), IDA 0x4F07C8 .. 0x4F0BC8: the keyboard file's 0x400
//!   bytes as `strings::load_language` reads it (its head, the layout mode, the current
//!   layout, the two layout blocks and the tables they point into; `key_to_char`). One
//!   block: the file is read over all of it and its layouts' offsets are made addresses in it
//!   (2026-10-24: moving only 0x4F07D0 .. 0x4F0820 left the file in the image and the layouts
//!   pointing past the block, "a keyboard layout without its key names" at the redefine page);
//! - `Devices` (`DEVICES`), 0x4F68D8 .. 0x4F7E18: the settings' switches (`nojoysticks` ...),
//!   the DirectInput objects, the joystick's state, caps and properties, the key tables (down,
//!   now, before, pressed, repeat: a byte a key), the mouse word, the two device records
//!   (0x4F8 bytes each, `types::InputDevice`; `input_devices` points at them), the
//!   controllers' table (`types::Controller`) and the input command;
//! - `Keyboard` (`KEYBOARD`), 0x4F7FD8 .. 0x4F8294: the yes/no key, the built-in layout, the
//!   silent keys' bitmap and the dead keys' list (0x4F8000) the keyboard file fills;
//! - `Mouse` (`MOUSE`), 0x5C6924 .. 0x5C6948: the mouse's position, clicks and buttons.
//!
//! The effects' GUIDs and the DirectInput identifiers (0x4F7E18 .. 0x4F7FA8) stay in the
//! image: the device records' effect slots point at them, and they are constants.
//!
//! The pointers in the data that name these blocks (IDA's addresses: the layouts' pointers
//! to their tables) keep working through `moved::MOVED`; `input_devices` names the records
//! by handle (`DEVICE_TAG`, `Devices::device`).
//!
//! `ForceFeedback` (IDA 0x518084 .. 0x5180F0): the force-feedback effects' clock, the race
//! switch, the surfaces' effect kinds and the effects' constants (split from sound's block
//! around it, 2026-10-25).
//!
//! Threads: the input thread polls and the game's thread reads, as the image's bytes were
//! shared (atomic cells and volatile bytes).

use crate::session::{Byte, Bytes, Dword, Float};

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

const fn byte(runs: Runs, at: usize) -> Byte {
    Byte::new(byte_at(runs, at))
}

/// A keyboard layout: five pointers (the dead keys' list, the plain table, the shifted one,
/// the one used while AltGr (`byte_4F6B40`) is down, and the key names' table, words of
/// string ids a key; `append_key_name`; the keyboard file of kind 0x0C patches three).
#[repr(C)]
pub struct KeyLayout {
    /// +0: a list of {dead key, plain, shifted} ended by a 0 key.
    pub dead_keys: Dword,
    /// +4: the plain table (a character a key).
    pub plain: Dword,
    /// +8: the shifted table.
    pub shifted: Dword,
    /// +0xC: the table used while AltGr is down.
    pub alt_gr: Dword,
    /// +0x10: the key names' table (`load_language` writes three words of it for kind
    /// 0x0C).
    pub key_names: Dword,
}

// ---- the keyboard file: 0x4F07C8 .. 0x4F0BC8 ----

/// The keyboard file's 0x400 bytes (`g_keyboardFile`): the file is read over all of it.
#[repr(C)]
pub struct KeyboardFile {
    /// `g_keyboardFile`: "KEYB" in a file.
    pub magic: Bytes<4>,
    /// 0x4F07CC (a reading: the file's second dword; nothing reads it).
    pub _version: Dword,
    /// `byte_4F07D0`: the file's kind (0x0C: patches the fourth table of its layout).
    pub kind: Byte,
    /// `byte_4F07D1`: 1 for the built-in layout.
    pub built_in: Byte,
    /// `byte_4F07D2`: the layout mode (2: Alt with a shift key switches the layout).
    pub mode: Byte,
    /// `byte_4F07D3`: the layout the file picks (times 0x14 from `layouts`).
    pub chosen: Byte,
    /// `g_keyboardLayout` (`dword_4F07D4`): the current layout block's address.
    pub layout: Dword,
    /// 0x4F07D8 .. 0x4F07F8: the rest of the file's head.
    pub _head: Bytes<0x20>,
    /// `dword_4F07F8` and `unk_4F080C`: the file's two layout blocks (the file holds
    /// offsets; `load_language` makes them addresses).
    pub layouts: [KeyLayout; 2],
    /// 0x4F0820 .. 0x4F0BC8: the tables the layouts point into.
    pub tables: Bytes<0x3A8>,
}

pub static KEYBOARD_FILE: KeyboardFile = KeyboardFile {
    magic: Bytes::new([0; 4]),
    _version: Dword::new(0),
    kind: Byte::new(0),
    built_in: Byte::new(0),
    mode: Byte::new(0),
    chosen: Byte::new(0),
    layout: Dword::new(0),
    _head: Bytes::new([0; 0x20]),
    layouts: [
        KeyLayout {
            dead_keys: Dword::new(0),
            plain: Dword::new(0),
            shifted: Dword::new(0),
            alt_gr: Dword::new(0),
            key_names: Dword::new(0),
        },
        KeyLayout {
            dead_keys: Dword::new(0),
            plain: Dword::new(0),
            shifted: Dword::new(0),
            alt_gr: Dword::new(0),
            key_names: Dword::new(0),
        },
    ],
    tables: Bytes::new([0; 0x3A8]),
};

pub const KEYBOARD_FILE_IDA: usize = 0x4F07C8;
pub const KEYBOARD_FILE_LEN: usize = core::mem::size_of::<KeyboardFile>();

/// Where the keyboard file is (`moved::MOVED`'s home for IDA 0x4F07C8).
pub fn keyboard_file_addr() -> u32 {
    crate::pointers::game_address(&raw const KEYBOARD_FILE)
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
    pub records: [Bytes<RECORD_LEN>; 2],
    /// `g_controllerCount`.
    pub controller_count: Dword,
    /// `g_controllers`: 16 entries (`types::Controller`).
    pub controllers: [Bytes<CONTROLLER_LEN>; 16],
    /// `g_inputCommand`.
    pub input_command: Dword,
    /// 0x4F7DE8 .. 0x4F7E18: three GUIDs the effect slots point at.
    pub guids: Bytes<0x30>,
}

const fn records(runs: Runs, at: usize) -> [Bytes<RECORD_LEN>; 2] {
    [
        Bytes::new(with_guid_refs(init::<RECORD_LEN>(runs, at))),
        Bytes::new(with_guid_refs(init::<RECORD_LEN>(runs, at + RECORD_LEN))),
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

const fn controllers(runs: Runs, at: usize) -> [Bytes<CONTROLLER_LEN>; 16] {
    let mut out = [const { Bytes::new([0; CONTROLLER_LEN]) }; 16];
    let mut k = 0;
    while k < 16 {
        out[k] = bytes(runs, at + CONTROLLER_LEN * k);
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

pub static DEVICES: Devices = {
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
};

pub const DEVICES_IDA: usize = 0x4F68D8;
pub const DEVICES_LEN: usize = core::mem::size_of::<Devices>();

/// Where the devices are (`moved::MOVED`'s home for IDA 0x4F68D8).
pub fn devices_addr() -> u32 {
    crate::pointers::game_address(&raw const DEVICES)
}

impl Devices {
    /// `g_keysDown` and `g_keysNow` as one table of 0x100 bytes (the key table the device
    /// bindings read: a key code past 0x7F reads `g_keysNow`).
    pub fn key_table(&self) -> [u8; 0x100] {
        let mut table = [0u8; 0x100];
        table[..KEYS].copy_from_slice(&self.keys_down.read());
        table[KEYS..].copy_from_slice(&self.keys_now.read());
        table
    }

    /// Device `n`'s record (`g_inputDevices[n]`, by handle: `device_of`).
    pub fn device(&self, n: usize) -> u32 {
        device_of(self.input_devices[n].get())
    }

    /// `g_inputDevices[n]` as the original indexes it: past the two pointers (`n` up to 7)
    /// it reads the first record's bytes (the drive code only refuses `n` past 7).
    pub fn input_device(&self, n: usize) -> u32 {
        use core::mem::offset_of;
        if n < self.input_devices.len() {
            return self.device(n);
        }
        let at = offset_of!(Devices, input_devices) + 4 * n;
        assert!(at + 4 <= core::mem::size_of::<Devices>());
        crate::pointers::read_or_end(
            crate::pointers::game_address(std::ptr::from_ref::<Self>(self)).wrapping_add(at as u32),
            "an input device past the table",
        )
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

/// A device record by handle (phase 13): `DEVICE_TAG | n` for `records[n]`.
pub const DEVICE_TAG: u32 = 0xCE00_0000;

/// Where the record a handle names is (0 for 0). A value that is no handle ends the
/// program.
pub fn device_of(handle: u32) -> u32 {
    if handle == 0 {
        return 0;
    }
    match DEVICES.records.get((handle ^ DEVICE_TAG) as usize) {
        Some(record) => record.addr(),
        None => crate::fatal::raise::end(
            "input::store::device_of",
            crate::errors::BadInput("a device record's handle that names none"),
        ),
    }
}

// ---- the keyboard: 0x4F7FD8 .. 0x4F8294 ----

/// The yes/no key, the built-in layout and the dead keys.
#[repr(C)]
pub struct Keyboard {
    /// `g_yesNoKeyGiven`: 1 = the key comes in eax.
    pub yes_no_key_given: Dword,
    /// `unk_4F7FDC`: the built-in layout (its pointers name the tables below).
    pub built_in_layout: KeyLayout,
    /// `dword_4F7FF0`: one bit per key; the keys set type nothing.
    pub silent_keys: [Dword; 4],
    /// `unk_4F8000`: the dead keys' list {dead key, base, plain result, shifted result}.
    pub compose: Bytes<0x6C>,
    /// 0x4F806C: the built-in layout's plain table.
    pub plain: Bytes<0x80>,
    /// 0x4F80EC: its shifted table.
    pub shifted: Bytes<0x80>,
    /// 0x4F816C .. 0x4F826C: its key names' table.
    pub key_names: Bytes<0x100>,
    /// `dword_4F826C`.
    pub _unused_826c: Dword,
    /// `unk_4F8270`: the axes' rest values (6 dwords, taken by `input::axis_moved`).
    pub axes_rest: [Dword; 6],
    /// 0x4F8288 .. 0x4F8294.
    pub _tail: Bytes<0xC>,
}

pub static KEYBOARD: Keyboard = {
    use crate::input::store_data::KEYBOARD_RUNS as R;
    Keyboard {
        yes_no_key_given: dword(R, 0x0),
        built_in_layout: KeyLayout {
            dead_keys: dword(R, 0x4),
            plain: dword(R, 0x8),
            shifted: dword(R, 0xC),
            alt_gr: dword(R, 0x10),
            key_names: dword(R, 0x14),
        },
        silent_keys: dwords(R, 0x18),
        compose: bytes(R, 0x28),
        plain: bytes(R, 0x94),
        shifted: bytes(R, 0x114),
        key_names: bytes(R, 0x194),
        _unused_826c: dword(R, 0x294),
        axes_rest: dwords(R, 0x298),
        _tail: bytes(R, 0x2B0),
    }
};

pub const KEYBOARD_IDA: usize = 0x4F7FD8;
pub const KEYBOARD_LEN: usize = core::mem::size_of::<Keyboard>();

/// The table at `at` (0 .. 0x10) of the keyboard layout block at `layout`, as an address, 0
/// for none. The built-in layouts' (this one's and the NEC's, `strings::keyboard_tables`)
/// tables are named by IDA's addresses, which are not addresses of the program's memory; a
/// keyboard file's layout holds the real places (`strings::load_language`).
pub fn layout_table(layout: u32, at: u32) -> Result<u32, crate::errors::BadInput> {
    let p: u32 = crate::pointers::GamePtr::new(layout)
        .offset(at)
        .read("a keyboard layout's table")?;
    if (KEYBOARD_IDA..KEYBOARD_IDA + KEYBOARD_LEN).contains(&(p as usize)) {
        return crate::moved::named_in(
            p,
            KEYBOARD_IDA,
            KEYBOARD_LEN,
            "a keyboard table off the built-in layout",
        );
    }
    let nec = crate::strings::keyboard_tables::KeyboardTables::START;
    let nec_len = crate::strings::keyboard_tables::KeyboardTables::LEN;
    if (nec..nec + nec_len).contains(&(p as usize)) {
        return crate::moved::named_in(p, nec, nec_len, "a keyboard table off the NEC layout");
    }
    Ok(p)
}

/// Where the keyboard is (`moved::MOVED`'s home for IDA 0x4F7FD8).
pub fn keyboard_addr() -> u32 {
    crate::pointers::game_address(&raw const KEYBOARD)
}

// ---- the mouse: 0x5C6924 .. 0x5C6948 ----

/// The mouse's position, clicks and buttons.
#[repr(C)]
pub struct Mouse {
    /// `g_mouseX`, `g_mouseY` (signed).
    pub x: Dword,
    pub y: Dword,
    /// `g_lastMouseX`, `g_lastMouseY`: last frame's.
    pub last_x: Dword,
    pub last_y: Dword,
    /// `g_leftClick`, `g_rightClick`.
    pub left_click: Dword,
    pub right_click: Dword,
    /// `g_mouseButtons`: the buttons' bits; `g_leftButton` (bit 0), `g_rightButton` (bit 1).
    pub buttons: Dword,
    pub left_button: Dword,
    pub right_button: Dword,
}

pub static MOUSE: Mouse = Mouse {
    x: Dword::new(0),
    y: Dword::new(0),
    last_x: Dword::new(0),
    last_y: Dword::new(0),
    left_click: Dword::new(0),
    right_click: Dword::new(0),
    buttons: Dword::new(0),
    left_button: Dword::new(0),
    right_button: Dword::new(0),
};

pub const MOUSE_IDA: usize = 0x5C6924;
pub const MOUSE_LEN: usize = core::mem::size_of::<Mouse>();

/// Where the mouse is (`moved::MOVED`'s home for IDA 0x5C6924).
pub fn mouse_addr() -> u32 {
    crate::pointers::game_address(&raw const MOUSE)
}

/// The force-feedback effects' clock, surface kinds and constants (IDA 0x518084 .. 0x5180f0, the image's layout; phase 7, typed state).
#[repr(C)]
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

pub static FORCE_FEEDBACK: ForceFeedback = ForceFeedback {
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
    effect_ten: Float::from_bits(0x41200000),
    effect_angle_scale: Float::from_bits(0x4322F983),
    effect_full: Float::from_bits(0x43800000),
    _gap_e8: Dword::new(0x42200000),
    effect_tenth: Float::from_bits(0x3DCCCCCD),
};

impl ForceFeedback {
    pub const START: usize = 0x518084;
    pub const LEN: usize = 0x6c;

    /// `moved::MOVED`'s home for `START`.
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const FORCE_FEEDBACK) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::offset_of;

    fn bytes_of(addr: usize, len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| unsafe {
                crate::pointers::host_at(addr + i)
                    .cast_const()
                    .read_volatile()
            })
            .collect()
    }

    #[test]
    fn the_blocks_start_as_the_images_bytes() {
        let _g = crate::store::test_lock();
        for (ida, len, addr) in [
            (KEYBOARD_FILE_IDA, KEYBOARD_FILE_LEN, keyboard_file_addr()),
            (DEVICES_IDA, DEVICES_LEN, devices_addr()),
            (KEYBOARD_IDA, KEYBOARD_LEN, keyboard_addr()),
            (MOUSE_IDA, MOUSE_LEN, mouse_addr()),
        ] {
            let mut image = image_bytes(ida, len);
            if ida == DEVICES_IDA {
                // The two records' places and the effects' wanted GUIDs are handles.
                let devices = offset_of!(Devices, input_devices);
                for r in 0..2 {
                    let o = devices + 4 * r;
                    let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                    assert_eq!(v as usize, 0x4F6EF0 + RECORD_LEN * r);
                    image[o..o + 4].copy_from_slice(&(DEVICE_TAG | r as u32).to_le_bytes());
                }
                let records = offset_of!(Devices, records);
                for r in 0..2 {
                    for k in 0..EFFECTS {
                        let o = records + RECORD_LEN * r + WANTED_AT + EFFECT_LEN * k;
                        let v = u32::from_le_bytes(image[o..o + 4].try_into().unwrap());
                        image[o..o + 4]
                            .copy_from_slice(&crate::input::guids::guid_ref(v).to_le_bytes());
                    }
                }
            }
            assert!(bytes_of(addr as usize, len) == image, "{ida:X}");
        }
    }

    #[test]
    fn the_blocks_keep_the_images_layout() {
        assert_eq!(KEYBOARD_FILE_LEN, 0x4F0BC8 - 0x4F07C8);
        assert_eq!(offset_of!(KeyboardFile, mode), 0x4F07D2 - 0x4F07C8);
        assert_eq!(offset_of!(KeyboardFile, layout), 0x4F07D4 - 0x4F07C8);
        assert_eq!(offset_of!(KeyboardFile, layouts), 0x4F07F8 - 0x4F07C8);
        assert_eq!(offset_of!(KeyboardFile, tables), 0x4F0820 - 0x4F07C8);
        assert_eq!(DEVICES_LEN, 0x4F7E18 - 0x4F68D8);
        for (offset, ida) in [
            (offset_of!(Devices, direct_input), 0x4F6904),
            (offset_of!(Devices, new_joystick2), 0x4F6914),
            (offset_of!(Devices, device_state), 0x4F6A28),
            (offset_of!(Devices, joystick_caps), 0x4F6A78),
            (offset_of!(Devices, dword_property), 0x4F6AA4),
            (offset_of!(Devices, range_property), 0x4F6AB8),
            (offset_of!(Devices, dead_key), 0x4F6AD0),
            (offset_of!(Devices, keys_down), 0x4F6ADC),
            (offset_of!(Devices, keys_now), 0x4F6B5C),
            (offset_of!(Devices, keys_before), 0x4F6BDC),
            (offset_of!(Devices, keys_pressed), 0x4F6CDC),
            (offset_of!(Devices, key_repeat), 0x4F6D5C),
            (offset_of!(Devices, mouse_word), 0x4F6DF0),
            (offset_of!(Devices, effect_rows), 0x4F6E08),
            (offset_of!(Devices, input_devices), 0x4F6EE8),
            (offset_of!(Devices, records), 0x4F6EF0),
            (offset_of!(Devices, controller_count), 0x4F78E0),
            (offset_of!(Devices, controllers), 0x4F78E4),
            (offset_of!(Devices, input_command), 0x4F7DE4),
            (offset_of!(Devices, guids), 0x4F7DE8),
        ] {
            assert_eq!(offset, ida - 0x4F68D8, "{ida:X}");
        }
        assert_eq!(KEYBOARD_LEN, 0x4F8294 - 0x4F7FD8);
        assert_eq!(offset_of!(Keyboard, silent_keys), 0x4F7FF0 - 0x4F7FD8);
        assert_eq!(offset_of!(Keyboard, compose), 0x4F8000 - 0x4F7FD8);
        assert_eq!(offset_of!(Keyboard, plain), 0x4F806C - 0x4F7FD8);
        assert_eq!(offset_of!(Keyboard, axes_rest), 0x4F8270 - 0x4F7FD8);
        assert_eq!(MOUSE_LEN, 0x24);
    }

    #[test]
    fn the_built_in_layouts_tables_resolve() {
        let _g = crate::store::test_lock();
        let layout = KEYBOARD.built_in_layout.dead_keys.addr();
        let image = image_bytes(0x4F7FDC, 0x14);
        for (at, ida) in [(4, 0x4F806C), (8, 0x4F80EC), (0x10, 0x4F816C)] {
            let name = u32::from_le_bytes(image[at..at + 4].try_into().unwrap());
            assert_eq!(name as usize, ida);
            assert_eq!(
                layout_table(layout, at as u32),
                Ok(keyboard_addr() + (ida - KEYBOARD_IDA) as u32)
            );
        }
        // The dead keys' list and the AltGr table are not named.
        assert_eq!(layout_table(layout, 0xC), Ok(0));
        // A place that is not a name is kept as it is.
        let place = KEYBOARD_FILE.tables.addr();
        KEYBOARD_FILE.layouts[0].plain.set(place);
        let file_layout = KEYBOARD_FILE.layouts[0].dead_keys.addr();
        assert_eq!(layout_table(file_layout, 4), Ok(place));
        KEYBOARD_FILE.layouts[0].plain.set(0u32);
    }

    #[test]
    fn force_feedback_starts_as_the_games() {
        assert_eq!(size_of::<ForceFeedback>(), ForceFeedback::LEN);
        let image = crate::moved::image_bytes(ForceFeedback::START, ForceFeedback::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ForceFeedback::home()).cast_const(),
                ForceFeedback::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
