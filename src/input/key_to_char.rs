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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::input::store::DeviceSlot;
use crate::input::store::KeyboardSlot;
use crate::input::store::LayoutSlot;
use crate::store::{game_addr, game_read, game_write};

// ---- game data ----

pub const LAYOUT_MODE_RVA: LayoutSlot = LayoutSlot::at(0x4F07D2); // byte_4F07D2
pub const LAYOUT_RVA: LayoutSlot = LayoutSlot::at(0x4F07D4); // dword_4F07D4: the current layout block
pub const LAYOUT_A_RVA: LayoutSlot = LayoutSlot::at(0x4F07F8); // dword_4F07F8
const LAYOUT_B_RVA: LayoutSlot = LayoutSlot::at(0x4F080C); // unk_4F080C
pub const DEAD_KEY_RVA: DeviceSlot = DeviceSlot::at(0x4F6AD0); // byte_4F6AD0
// The key-down table starts at the pending key, 0x4F6ADC, and is indexed by scan code.
pub const LEFT_SHIFT_RVA: DeviceSlot = DeviceSlot::at(0x4F6B06); // byte_4F6B06 (key 0x2A)
pub const RIGHT_SHIFT_RVA: DeviceSlot = DeviceSlot::at(0x4F6B12); // byte_4F6B12 (key 0x36)
pub const ALT_RVA: DeviceSlot = DeviceSlot::at(0x4F6B14); // byte_4F6B14 (key 0x38)
pub const ALT_GR_RVA: DeviceSlot = DeviceSlot::at(0x4F6B40); // byte_4F6B40
const SILENT_KEYS_RVA: KeyboardSlot = KeyboardSlot::at(0x4F7FF0); // dword_4F7FF0: one bit per key
const COMPOSE_RVA: KeyboardSlot = KeyboardSlot::at(0x4F8000); // unk_4F8000

use crate::bridge::Raw;

unsafe fn dword(addr: u32) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}

fn down(rva: impl crate::store::Where) -> bool {
    unsafe { crate::store::game_read::<u8>(rva) != 0 }
}

/// True if a keyboard layout is set.
pub fn has_layout() -> bool {
    unsafe { game_read::<u32>(LAYOUT_RVA) != 0 }
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
            game_read(LAYOUT_MODE_RVA),
            down(ALT_RVA),
            down(ALT_GR_RVA),
            down(LEFT_SHIFT_RVA),
            down(RIGHT_SHIFT_RVA),
        );
        let layout: u32 = match switched {
            Some(Switch::First) => game_addr(LAYOUT_A_RVA) as u32,
            Some(Switch::Second) => game_addr(LAYOUT_B_RVA) as u32,
            None => game_read(LAYOUT_RVA),
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
            &Raw::new(game_addr(COMPOSE_RVA) as *const u8),
            &Raw::new(game_addr(SILENT_KEYS_RVA) as *const u8),
            down(LEFT_SHIFT_RVA) || down(RIGHT_SHIFT_RVA),
            down(ALT_GR_RVA),
            game_read(DEAD_KEY_RVA),
        )?;
        if switched.is_some() {
            game_write(LAYOUT_RVA, layout);
        }
        if let Some(pending) = typed.pending {
            game_write(DEAD_KEY_RVA, pending);
        }
        Ok(typed.eax)
    }
}
