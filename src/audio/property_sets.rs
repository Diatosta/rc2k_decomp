//! A 3D buffer's property sets (IDA, 2026-10-15): DirectSound's voice manager ("DSV",
//! setting `ignoredsv`) and EAX 1's reverb ("EAX", `ignoreeax`), each opened once through
//! the first 3D buffer made (`IKsPropertySet` from its `QueryInterface`), and the EAX room.

use crate::audio::com::method;
use crate::audio::store::AUDIO;
use crate::errors::{BadInput, Refused};
use crate::session::{Bytes, Dword};

const USING_DSV_TEXT: &std::ffi::CStr = c"AUD: Using DSV"; // "AUD: Using DSV"
const USING_EAX_TEXT: &std::ffi::CStr = c"AUD: Using EAX"; // "AUD: Using EAX"
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
    pub setting: &'static Dword,
    /// Where the property set goes.
    pub object: &'static Dword,
    /// Set to 1 when it is there.
    pub found: &'static Dword,
    /// Which set (its id); the text logged.
    pub set: &'static Bytes<16>,
    pub text: &'static std::ffi::CStr,
    /// The voice manager's mode is set; EAX's nothing.
    pub mode: bool,
}

impl Opening {
    pub fn voice_manager() -> Self {
        Self {
            setting: &AUDIO.ignore_dsv_setting,
            object: &AUDIO.voice_manager,
            found: &AUDIO.voice_manager_on,
            set: &AUDIO.voice_manager_set,
            text: USING_DSV_TEXT,
            mode: true,
        }
    }

    pub fn eax() -> Self {
        Self {
            setting: &AUDIO.ignore_eax_setting,
            object: &AUDIO.eax_properties,
            found: &AUDIO.has_eax,
            set: &AUDIO.eax_reverb_set,
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
        if o.setting.get() != 0 || o.object.get() != 0 {
            return Ok(false);
        }
        if holder < 0x1_0000 {
            return Err(BadInput("ebx holds no 3D buffer (make_buffer's EAX bug)").into());
        }
        let buffer = crate::audio::voices::read(holder);
        if buffer == 0 {
            return Ok(false);
        }
        let query: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(method(buffer, 0));
        if query(buffer, AUDIO.iid_property_set.addr(), o.object.addr()) != 0 {
            return Ok(false);
        }
        let p = o.object.get();
        let set = o.set.addr();
        let mut support = 0u32;
        let query_support: unsafe extern "system" fn(u32, u32, u32, u32) -> i32 =
            std::mem::transmute(method(p, QUERY_SUPPORT));
        if query_support(
            p,
            set,
            0,
            crate::pointers::game_address(&mut support as *mut u32),
        ) != 0
            || support & GET_AND_SET != GET_AND_SET
        {
            return Ok(false);
        }
        if o.mode {
            let value = VOICE_MANAGER_MODE;
            if set_property(
                p,
                set,
                crate::pointers::game_address(&value as *const u32),
                4,
            ) != 0
            {
                return Ok(false);
            }
        }
        log(crate::pointers::game_address(o.text.as_ptr()));
        o.found.set(1);
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
pub fn set_eax_room(room: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::audio::levels::eax_room_offset`.
    let at = crate::pointers::game_address(AUDIO.eax_rooms.as_ptr())
        + rc2k_core::audio::levels::eax_room_offset(room)?;
    let p = AUDIO.eax_properties.get();
    if p != 0 {
        unsafe { set_property(p, AUDIO.eax_reverb_set.addr(), at, 0x10) };
    }
    Ok(at)
}
