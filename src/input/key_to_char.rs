//! `input::key_to_char` (0x407E43): turns a key (a scan code) into the character it types
//! with the current keyboard layout, shift state and pending dead key.
//!
//! In:  al = key
//! Out: al = the character, 0 if the key types none; carry = (al == 0), zero = (al == 1);
//!      ebx, ecx, edx, esi, edi, ebp preserved. The rest of eax: untouched for a key release
//!      (bit 7 set); else zero, except ah = the dead key that was stored or used.
//! The layout is a block of four pointers (`dword_4F07D4` points at the current one): +0 a
//! list of dead keys {key, plain, shifted} ended by a 0 key, +4 the plain table, +8 the
//! shifted one, +0xC the one used while `byte_4F6B40` is down (read as AltGr). A dead key is
//! stored in `byte_4F6AD0`; the next key is then looked up, by its plain character, in the
//! list of {dead key, base, plain result, shifted result} at 0x4F8000. Keys whose bit is set
//! in the bitmap at 0x4F7FF0 type nothing. In layout mode 2 (`byte_4F07D2`), Alt or AltGr
//! with a shift key switches the layout: left shift picks 0x4F07F8, right shift 0x4F080C.
//! Refuses: no layout; a dead key pending with no plain table (the original reads through
//! null); lists without an end within 256 entries.

use crate::errors::BadInput;
use crate::input::store::{DEVICES, KEYBOARD, KEYBOARD_FILE};

// ---- game data ----

// The key-down table (`g_keysDown`) is indexed by scan code: left shift 0x2A
// (`byte_4F6B06`), right shift 0x36 (`byte_4F6B12`), Alt 0x38 (`byte_4F6B14`), AltGr 0x64
// (`byte_4F6B40`).
const LEFT_SHIFT: usize = 0x2A;
const RIGHT_SHIFT: usize = 0x36;
const ALT: usize = 0x38;
const ALT_GR: usize = 0x64;

use crate::bridge::Raw;

unsafe fn dword(addr: u32) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}

fn down(key: usize) -> bool {
    DEVICES.keys_down.get(key) != 0
}

/// True if a keyboard layout is set.
pub fn has_layout() -> bool {
    KEYBOARD_FILE.layout.get() != 0
}

// ---- Rust version ----

/// eax as the original leaves it; the character is its low byte.
pub unsafe fn key_to_char(eax: u32) -> Result<u32, BadInput> {
    use rc2k_core::input::keyboard::{self, Layout, Switch};
    let key = eax as u8;
    if key & 0x80 != 0 {
        return Ok(eax & !0xFF);
    }
    // Converted: `rc2k_core::input::keyboard`; here the layout block's pointers as tables, and
    // the switch and the pending key stored.
    unsafe {
        let switched = keyboard::switch(
            KEYBOARD_FILE.mode.get(),
            down(ALT),
            down(ALT_GR),
            down(LEFT_SHIFT),
            down(RIGHT_SHIFT),
        );
        let layout: u32 = match switched {
            Some(Switch::First) => KEYBOARD_FILE.layouts[0].dead_keys.addr() as u32,
            Some(Switch::Second) => KEYBOARD_FILE.layouts[1].dead_keys.addr() as u32,
            None => KEYBOARD_FILE.layout.get(),
        };
        if layout == 0 {
            return Err(BadInput("no keyboard layout"));
        }
        // The built-in layout's pointers name its tables' homes (`moved::point_into_homes`);
        // a keyboard file's layout's are given as places.
        let table = |at: u32| {
            let p = dword(layout + at);
            (p != 0).then(|| Raw::new(p as *const u8))
        };
        let tables = [table(0), table(4), table(8), table(0xC)];
        fn as_bytes(t: &Option<Raw>) -> Option<&dyn rc2k_core::strings::bytes::Bytes> {
            t.as_ref()
                .map(|t| t as &dyn rc2k_core::strings::bytes::Bytes)
        }
        let typed = keyboard::key_to_char(
            key,
            &Layout {
                dead_keys: as_bytes(&tables[0]),
                plain: as_bytes(&tables[1]),
                shifted: as_bytes(&tables[2]),
                alt_gr: as_bytes(&tables[3]),
            },
            &Raw::new(KEYBOARD.compose.as_ptr()),
            &Raw::new(KEYBOARD.silent_keys[0].as_ptr().cast::<u8>()),
            down(LEFT_SHIFT) || down(RIGHT_SHIFT),
            down(ALT_GR),
            DEVICES.dead_key.get(),
        )?;
        if switched.is_some() {
            KEYBOARD_FILE.layout.set(layout);
        }
        if let Some(pending) = typed.pending {
            DEVICES.dead_key.set(pending);
        }
        Ok(typed.eax)
    }
}
