//! `input::key_to_char` (0x407E43): turns a key (a scan code) into the character it types
//! with the current keyboard layout, shift state and pending dead key.
//!
//! In:  al = key
//! Out: al = the character, 0 if the key types none; carry = (al == 0), zero = (al == 1);
//!      ebx, ecx, edx, esi, edi, ebp preserved. The rest of eax: untouched for a key release
//!      (bit 7 set); else zero, except ah = the dead key that was stored or used.
//! The layout (`input::layouts`; `dword_4F07D4` pointed at the current one in the original)
//! has a list of dead keys {key, plain, shifted} ended by a 0 key, a plain table, a shifted
//! one and one used while `byte_4F6B40` is down (read as AltGr). A dead key is
//! stored in `byte_4F6AD0`; the next key is then looked up, by its plain character, in the
//! list of {dead key, base, plain result, shifted result} at 0x4F8000. Keys whose bit is set
//! in the bitmap at 0x4F7FF0 type nothing. In layout mode 2 (`byte_4F07D2`), Alt or AltGr
//! with a shift key switches the layout: left shift picks the keyboard file's first layout,
//! right shift its second.
//! Refuses: no layout; a dead key pending with no plain table (the original reads through
//! null); lists without an end within 256 entries.

use crate::errors::BadInput;
use crate::input::layouts::{Current, layouts};
use crate::input::owner::{Input, input};
use crate::input::store::Devices;
use crate::pointers::GamePtr;

// ---- game data ----

// The key-down table (`g_keysDown`) is indexed by scan code: left shift 0x2A
// (`byte_4F6B06`), right shift 0x36 (`byte_4F6B12`), Alt 0x38 (`byte_4F6B14`), AltGr 0x64
// (`byte_4F6B40`).
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

// ---- Rust version ----

/// eax as the original leaves it; the character is its low byte.
pub fn key_to_char(eax: u32) -> Result<u32, BadInput> {
    typed(input(), eax, false)
}

/// The same for the name editors and the yes/no question, which in layout mode 2 always use
/// the keyboard file's first layout and leave the current one as it was (the original put
/// that layout in place for the call and the old one back after).
pub fn key_to_char_for_text(eax: u32) -> Result<u32, BadInput> {
    typed(input(), eax, true)
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
    let typed = keyboard::key_to_char(
        key,
        &layout.tables()?,
        &GamePtr::to(&input.keyboard.compose).bytes("the compose table")?,
        &GamePtr::to(&input.keyboard.silent_keys).bytes("the silent keys")?,
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
    Ok(typed.eax)
}
