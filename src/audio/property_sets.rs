//! A 3D buffer's property sets: DirectSound's voice manager ("DSV",
//! setting `ignoredsv`) and EAX 1's reverb ("EAX", `ignoreeax`), each opened once through
//! the first 3D buffer made (`IKsPropertySet` from its `QueryInterface`), and the EAX room.
//! The platform's DirectSound offers no property sets (`platform_sound::query_interface`
//! answers `E_NOINTERFACE`, as a sound card without them): the openings end there, as
//! they did through the method tables; what the original does with a set it got (its
//! `QuerySupport` for get and set, the voice manager's mode 2, the log line, `found` 1,
//! the room's `Set`) is refused where it would be reached.

use crate::audio::own::AudioOwn;
use crate::audio::voices::SampleRecord;
use crate::errors::{BadInput, Refused};
use crate::session::{Dword, PlainDword, Shared};

/// What a property set the platform made would need (it makes none).
const NO_SETS: &str = "a DirectSound property set (the platform offers none)";

/// Where one opening's things are. (The original also has, per set, its `found` flag, its
/// id, its log line "AUD: Using DSV" / "AUD: Using EAX" and, for the voice manager, the
/// mode it sets: only reached with a set the platform never makes.)
#[derive(Clone, Copy)]
pub struct Opening<'a> {
    /// The `ignore...` setting.
    pub setting: &'a Dword,
    /// Where the property set goes.
    pub object: &'a PlainDword,
}

impl<'a> Opening<'a> {
    pub fn voice_manager(state: &'a AudioOwn) -> Self {
        Self {
            setting: &state.ignore_dsv_setting,
            object: &state.voice_manager,
        }
    }

    pub fn eax(state: &'a AudioOwn) -> Self {
        Self {
            setting: &state.ignore_eax_setting,
            object: &state.eax_properties,
        }
    }
}

/// What `audio::make_buffer` hands the openings to read the 3D buffer through (the original's
/// ebx, read as the address of a dword). Only the voice manager's call gives the 3D buffer; the
/// other values are the game's bug (`notes/bugs.md`): a record's is read as the original reads
/// it, a number refused.
#[derive(Clone, Copy)]
pub enum Holder<'a> {
    /// `audio::load_sounds`'s sample record: its first dword, the record's `number`, is read.
    Record(&'a Shared<SampleRecord>),
    /// The voice of this id's `buffer_3d` (stored before the openings).
    Buffer3d(u32),
    /// A number held as an address: the caller's sound mode (`audio::start`) or the voice's
    /// frequency after the create path (`audiobasefreq` or a sample's). It names nothing the
    /// game holds and is refused.
    Value,
}

impl Holder<'_> {
    /// The dword the original reads through the holder; refuses a number (`Value`): the
    /// original reads where it points, which is no place of the game's.
    fn read(self, state: &AudioOwn) -> Result<u32, Refused> {
        match self {
            Holder::Record(record) => Ok(record.get().number),
            Holder::Buffer3d(voice) => Ok(crate::audio::voices::voice_of(state, voice).buffer_3d),
            Holder::Value => {
                Err(BadInput("the holder value is no 3D buffer (make_buffer's EAX bug)").into())
            }
        }
    }
}

/// `audio::open_voice_manager` and `audio::open_eax`: true for the carry clear. `holder`
/// holds the 3D buffer (`Holder`). Nothing with the setting on or the set already opened;
/// else the set from the 3D buffer's `QueryInterface`, its `QuerySupport` (get and set
/// both), for the voice manager its mode set to 2; then logged and `found` 1. A failure
/// leaves the set where it went (it is released at the close). Refuses a holder that is
/// no pointer (the original reads through it: `make_buffer`'s EAX bug, `notes/bugs.md`).
pub fn open_property_set(state: &AudioOwn, o: &Opening, holder: Holder) -> Result<bool, Refused> {
    if o.setting.is_set() || o.object.is_set() {
        return Ok(false);
    }
    let buffer = holder.read(state)?;
    if buffer == 0 {
        return Ok(false);
    }
    match crate::platform_sound::query_interface(buffer, state.iid_property_set.get_u32(0)) {
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

/// `audio::set_eax_room`: the room's reverb (`g_eaxRooms`) set when EAX is open. Refuses a room
/// past the table (the original reads past it).
pub fn set_eax_room(state: &AudioOwn, room: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::audio::levels::eax_room_offset`. The room's entry lies inside
    // the table; a room past it is refused.
    rc2k_core::audio::levels::eax_room_offset(room)?;
    let p = state.eax_properties.get();
    if p != 0 {
        // `IKsPropertySet::Set` of the reverb's property 0 with the room's 0x10 bytes.
        return Err(BadInput(NO_SETS).into());
    }
    Ok(())
}
