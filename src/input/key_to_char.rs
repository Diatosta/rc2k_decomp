//! `input::key_to_char` (0x407E43): turns a key (a scan code) into the character it types
//! with the current keyboard layout, shift state and pending dead key.
//!
//! Takes: key
//! The layout (`input::layouts`) has a list of dead keys {key, plain, shifted} ended by a 0
//! key, a plain table, a shifted one and one used while AltGr is down. A dead key is stored
//! (`dead_key`); the next key is then looked up, by its plain character, in the list of
//! {dead key, base, plain result, shifted result} in the keyboard tables. Keys whose bit
//! is set in the keyboard tables' bitmap type nothing. In layout mode 2, Alt or AltGr with
//! a shift key switches the layout: left shift picks the keyboard file's first layout,
//! right shift its second.
//! Refuses: no layout; a dead key pending with no plain table (the original reads through
//! null); lists without an end within 256 entries.

use crate::errors::BadInput;
use crate::input::layouts::{Current, layouts};
use crate::input::owner::Input;
use crate::input::store::Devices;

// ---- game data ----

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
pub fn has_layout() -> bool {
    layouts().current() != Current::None
}

/// The character is the low byte of the result.
pub fn key_to_char(input: &Input, eax: u32) -> Result<u32, BadInput> {
    typed(input, eax, false)
}

/// The same for the name editors and the yes/no question, which in layout mode 2 always use
/// the keyboard file's first layout and leave the current one as it was (the original put
/// that layout in place for the call and the old one back after).
pub fn key_to_char_for_text(input: &Input, eax: u32) -> Result<u32, BadInput> {
    typed(input, eax, true)
}

fn typed(input: &Input, eax: u32, for_text: bool) -> Result<u32, BadInput> {
    let down = |key| down(&input.devices, key);
    use rc2k_core::input::keyboard::{self, Switch};
    let key = eax as u8;
    if key & 0x80 != 0 {
        return Ok(eax & !0xFF);
    }
    let mut layouts = layouts();
    // Converted: `rc2k_core::input::keyboard`; here the layouts, and the switch and the
    // pending key stored.
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
    let own_layout = for_text && layouts.mode() == 2;
    let which = match switched {
        Some(layout) => layout,
        None if own_layout => Current::File(0),
        None => layouts.current(),
    };
    let layout = layouts
        .layout(which)
        .ok_or(BadInput("no keyboard layout"))?;
    let silent_keys: Vec<u8> = crate::session::read_all(&input.keyboard().silent_keys)
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
