//! A 3D buffer's property sets (IDA, 2026-10-15): DirectSound's voice manager ("DSV",
//! setting `ignoredsv`) and EAX 1's reverb ("EAX", `ignoreeax`), each opened once through
//! the first 3D buffer made (`IKsPropertySet` from its `QueryInterface`), and the EAX room.
//! The platform's DirectSound offers no property sets (`platform_sound::query_interface`
//! answers `E_NOINTERFACE`, as a sound card without them): the openings end there, as
//! they did through the method tables; what the original does with a set it got (its
//! `QuerySupport` for get and set, the voice manager's mode 2, the log line, `found` 1,
//! the room's `Set`) is refused where it would be reached.

use crate::audio::store::AUDIO;
use crate::errors::{BadInput, Refused};
use crate::session::Dword;

/// What a property set the platform made would need (it makes none).
const NO_SETS: &str = "a DirectSound property set (the platform offers none)";

/// Where one opening's things are. (The original also has, per set, its `found` flag, its
/// id, its log line "AUD: Using DSV" / "AUD: Using EAX" and, for the voice manager, the
/// mode it sets: only reached with a set the platform never makes.)
#[derive(Clone, Copy)]
pub struct Opening {
    /// The `ignore...` setting.
    pub setting: &'static Dword,
    /// Where the property set goes.
    pub object: &'static Dword,
}

impl Opening {
    pub fn voice_manager() -> Self {
        Self {
            setting: &AUDIO.ignore_dsv_setting,
            object: &AUDIO.voice_manager,
        }
    }

    pub fn eax() -> Self {
        Self {
            setting: &AUDIO.ignore_eax_setting,
            object: &AUDIO.eax_properties,
        }
    }
}

/// `audio::open_voice_manager` and `audio::open_eax`: true for the carry clear. `holder`
/// (ebx) holds the 3D buffer. Nothing with the setting on or the set already opened;
/// else the set from the 3D buffer's `QueryInterface`, its `QuerySupport` (get and set
/// both), for the voice manager its mode set to 2; then logged and `found` 1. A failure
/// leaves the set where it went (it is released at the close). Refuses a holder that is
/// no pointer (the original reads through it: `make_buffer`'s EAX bug, `notes/bugs.md`).
pub fn open_property_set(o: &Opening, holder: u32) -> Result<bool, Refused> {
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
    match crate::platform_sound::query_interface(buffer, AUDIO.iid_property_set.get_u32(0)) {
        Err(_) => {
            o.object.set(0);
            Ok(false)
        }
        Ok(set) => {
            o.object.set(set);
            Err(BadInput(NO_SETS).into())
        }
    }
}

/// `audio::set_eax_room`: the room's reverb (`g_eaxRooms`) set when EAX is open; eax the
/// room's entry. Refuses a room past the table (the original reads past it).
pub fn set_eax_room(room: u32) -> Result<crate::pointers::GamePtr, Refused> {
    // Converted: `rc2k_core::audio::levels::eax_room_offset`.
    let at = crate::pointers::GamePtr::to(&AUDIO.eax_rooms)
        .offset(rc2k_core::audio::levels::eax_room_offset(room)?);
    let p = AUDIO.eax_properties.get();
    if p != 0 {
        // `IKsPropertySet::Set` of the reverb's property 0 with the room's 0x10 bytes.
        return Err(BadInput(NO_SETS).into());
    }
    Ok(at)
}
