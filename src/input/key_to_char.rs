//! Turns a key (a scan code) into the character it types with the current keyboard layout,
//! shift state and pending dead key.
//!
//! The layout (`input::layouts`) has a list of dead keys {key, plain, shifted} ended by a 0
//! key, a plain table, a shifted one and one used while AltGr is down. A dead key is stored
//! (`dead_key`); the next key is then looked up, by its plain character, in the list of
//! {dead key, base, plain result, shifted result} in the keyboard tables. Keys whose bit
//! is set in the keyboard tables' bitmap type nothing. In layout mode 2, Alt or AltGr with
//! a shift key switches the layout: left shift picks the keyboard file's first layout,
//! right shift its second.
//! Refuses: no layout; a dead key pending with no plain table; lists without an end within 256
//! entries.

use crate::errors::BadInput;
use crate::input::layouts::Current;
use crate::input::owner::Input;
use crate::input::store::Devices;

// The key-down table (`keys_down`) is indexed by scan code: left shift 0x2A, right shift
// 0x36, Alt 0x38, AltGr 0x64.
const LEFT_SHIFT: usize = 0x2A;
const RIGHT_SHIFT: usize = 0x36;
const ALT: usize = 0x38;
const ALT_GR: usize = 0x64;

fn down(devices: &Devices, key: usize) -> bool {
    devices.keys_down.get(key) != 0
}

/// True if a keyboard layout is set.
pub fn has_layout(input: &Input) -> bool {
    input.layouts().current() != Current::None
}

/// The character a key types, for the yes/no question, which in layout mode 2 always uses
/// the keyboard file's first layout and leave the current one as it was.
pub fn key_to_char_for_text(input: &Input, key_word: u32) -> Result<u32, BadInput> {
    typed(input, key_word)
}

fn typed(input: &Input, key_word: u32) -> Result<u32, BadInput> {
    let down = |key| down(&input.devices, key);
    use rc2k_core::input::keyboard::{self, Switch};
    let key = key_word as u8;
    if key & 0x80 != 0 {
        return Ok(key_word & !0xFF);
    }
    let mut layouts = input.layouts();
    let switched = keyboard::switch(
        layouts.mode(),
        down(ALT),
        down(ALT_GR),
        down(LEFT_SHIFT),
        down(RIGHT_SHIFT),
    )
    .map(|which| match which {
        Switch::First => Current::File(0),
        Switch::Second => Current::File(1),
    });
    let own_layout = layouts.mode() == 2;
    let which = match switched {
        Some(layout) => layout,
        None if own_layout => Current::File(0),
        None => layouts.current(),
    };
    let layout = layouts
        .layout(which)
        .ok_or(BadInput("no keyboard layout"))?;
    let silent_keys: Vec<u8> = crate::memory::session::read_all(&input.keyboard().silent_keys)
        .iter()
        .flat_map(|dword| dword.to_le_bytes())
        .collect();
    let typed = keyboard::key_to_char(
        key,
        &layout.tables()?,
        &input.keyboard().compose.read(),
        &silent_keys,
        down(LEFT_SHIFT) || down(RIGHT_SHIFT),
        down(ALT_GR),
        input.devices.dead_key.get(),
    )?;
    if let (Some(layout), false) = (switched, own_layout) {
        layouts.set_current(layout);
    }
    if let Some(pending) = typed.pending {
        input.devices.dead_key.set(pending);
    }
    Ok(typed.code)
}
