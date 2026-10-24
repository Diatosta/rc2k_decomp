//! A 3D buffer's property sets (IDA, 2026-10-15): DirectSound's voice manager ("DSV",
//! setting `ignoredsv`) and EAX 1's reverb ("EAX", `ignoreeax`), each opened once through
//! the first 3D buffer made (`IKsPropertySet` from its `QueryInterface`), and the EAX room.

use crate::audio::com::method;
use crate::audio::store::Slot;
use crate::audio::voices::{EAX_RVA, VOICE_MANAGER_RVA, read, write};
use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::store::addr32 as a;
use crate::store::game_addr;

pub const IID_PROPERTY_SET_RVA: Slot = Slot::at(0x506A78); // g_iidKsPropertySet
pub const VOICE_MANAGER_SET_RVA: Slot = Slot::at(0x506CC8); // g_voiceManagerSet
pub const EAX_REVERB_SET_RVA: Slot = Slot::at(0x506DAC); // g_eaxReverbSet
pub const EAX_ROOMS_RVA: Slot = Slot::at(0x506DCC); // g_eaxRooms: 10 of 0x10 bytes
pub const IGNORE_DSV_RVA: Slot = Slot::at(0x506C50); // g_ignoreDsvSetting
pub const IGNORE_EAX_RVA: Slot = Slot::at(0x506C6C); // g_ignoreEaxSetting
pub const VOICE_MANAGER_ON_RVA: Slot = Slot::at(0x506C54); // g_voiceManagerOn
pub const HAS_EAX_RVA: Slot = Slot::at(0x506C70); // g_hasEax
const USING_DSV_TEXT: usize = rva(0x4DA73E); // "AUD: Using DSV"
const USING_EAX_TEXT: usize = rva(0x4DA7AA); // "AUD: Using EAX"
/// `IKsPropertySet`'s `Set` and `QuerySupport`.
const SET: u32 = 0x10;
const QUERY_SUPPORT: u32 = 0x14;
/// `KSPROPERTY_SUPPORT_GET | KSPROPERTY_SUPPORT_SET`.
const GET_AND_SET: u32 = 3;
/// `DSPROPERTY_VMANAGER_MODE`'s value 2 (`DSPROPERTY_VMANAGER_MODE_REPORT`).
const VOICE_MANAGER_MODE: u32 = 2;

/// A text to the log (level 1, with a line's end).
pub type Log<'a> = &'a mut dyn FnMut(u32);

pub fn game_log(text: u32) {
    crate::audio::sound_device::log_text_1(text);
}

/// Where one opening's things are.
#[derive(Clone, Copy)]
pub struct Opening {
    /// The `ignore...` setting.
    pub setting: u32,
    /// Where the property set goes.
    pub object: u32,
    /// Set to 1 when it is there.
    pub found: u32,
    /// Which set; the text logged.
    pub set: Slot,
    pub text: usize,
    /// The voice manager's mode is set; EAX's nothing.
    pub mode: bool,
}

impl Opening {
    pub fn voice_manager() -> Self {
        Self {
            setting: a(IGNORE_DSV_RVA),
            object: a(VOICE_MANAGER_RVA),
            found: a(VOICE_MANAGER_ON_RVA),
            set: VOICE_MANAGER_SET_RVA,
            text: USING_DSV_TEXT,
            mode: true,
        }
    }

    pub fn eax() -> Self {
        Self {
            setting: a(IGNORE_EAX_RVA),
            object: a(EAX_RVA),
            found: a(HAS_EAX_RVA),
            set: EAX_REVERB_SET_RVA,
            text: USING_EAX_TEXT,
            mode: false,
        }
    }
}

/// `audio::open_voice_manager` and `audio::open_eax`: true for the carry clear. `holder`
/// (ebx) holds the 3D buffer. Nothing with the setting on or the set already opened;
/// else the set from the 3D buffer's `QueryInterface`, its `QuerySupport` (get and set
/// both), for the voice manager its mode set to 2; then logged and `found` 1. A failure
/// leaves the set where it went (it is released at the close). Refuses a holder that is
/// no pointer (the original reads through it: `make_buffer`'s EAX bug, `notes/bugs.md`).
pub unsafe fn open_property_set(o: &Opening, holder: u32, log: Log) -> Result<bool, Refused> {
    unsafe {
        if read(o.setting) != 0 || read(o.object) != 0 {
            return Ok(false);
        }
        if holder < 0x1_0000 {
            return Err(BadInput("ebx holds no 3D buffer (make_buffer's EAX bug)").into());
        }
        let buffer = read(holder);
        if buffer == 0 {
            return Ok(false);
        }
        let query: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(method(buffer, 0));
        if query(buffer, game_addr(IID_PROPERTY_SET_RVA) as u32, o.object) != 0 {
            return Ok(false);
        }
        let p = read(o.object);
        let set = game_addr(o.set) as u32;
        let mut support = 0u32;
        let query_support: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(method(p, QUERY_SUPPORT));
        if query_support(p, set, 0, &mut support as *mut u32 as u32) != 0
            || support & GET_AND_SET != GET_AND_SET
        {
            return Ok(false);
        }
        if o.mode {
            let value = VOICE_MANAGER_MODE;
            if set_property(p, set, &value as *const u32 as u32, 4) != 0 {
                return Ok(false);
            }
        }
        log(game_addr(o.text) as u32);
        write(o.found, 1);
        Ok(true)
    }
}

/// `IKsPropertySet::Set` of property 0, no instance data.
unsafe fn set_property(p: u32, set: u32, data: u32, bytes: u32) -> i32 {
    unsafe {
        let f: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(method(p, SET));
        f(p, set, 0, 0, 0, data, bytes)
    }
}

/// `audio::set_eax_room`: the room's reverb (`g_eaxRooms`) set when EAX is open; eax the
/// room's entry. Refuses a room past the table (the original reads past it).
pub unsafe fn set_eax_room(room: u32, properties: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::audio::levels::eax_room_offset`.
    let at = game_addr(EAX_ROOMS_RVA) as u32 + rc2k_core::audio::levels::eax_room_offset(room)?;
    unsafe {
        let p = read(properties);
        if p != 0 {
            set_property(p, game_addr(EAX_REVERB_SET_RVA) as u32, at, 0x10);
        }
    }
    Ok(at)
}
