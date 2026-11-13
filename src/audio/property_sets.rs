//! A 3D buffer's property sets: DirectSound's voice manager ("DSV", setting `ignoredsv`) and EAX's
//! reverb ("EAX", `ignoreeax`), each opened once through the first 3D buffer made, and the EAX
//! room. The platform offers no property sets, so the openings end at `QueryInterface`; what would
//! be done with a set is refused where it would be reached.

use crate::audio::own::AudioOwn;
use crate::errors::{BadInput, Refused};
use crate::session::{Dword, PlainDword};

/// What a property set the platform made would need (it makes none).
const NO_SETS: &str = "a DirectSound property set (the platform offers none)";

/// Where one opening's things are.
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

/// What `audio::make_buffer` hands the openings to read the 3D buffer through. Only the voice
/// manager's call gives the 3D buffer; the other values are a game bug kept.
#[derive(Clone, Copy)]
pub enum Holder {
    /// `audio::load_sounds`'s sample record: its first dword, the record's `number`, is read.
    Record(u32),
    /// The voice of this id's `buffer_3d` (stored before the openings).
    Buffer3d(u32),
    /// A number held as an address: the caller's sound mode (`audio::start`) or the voice's
    /// frequency after the create path (`audiobasefreq` or a sample's). It names nothing the game
    /// holds and is refused.
    Value,
}

impl Holder {
    /// The dword read through the holder; refuses a number (`Value`), which names nothing the game
    /// holds.
    fn read(self, state: &AudioOwn) -> Result<u32, Refused> {
        match self {
            Holder::Record(number) => Ok(number),
            Holder::Buffer3d(voice) => Ok(crate::audio::voices::voice_of(state, voice).buffer_3d),
            Holder::Value => {
                Err(BadInput("the holder value is no 3D buffer (make_buffer's EAX bug)").into())
            }
        }
    }
}

/// `audio::open_voice_manager` and `audio::open_eax`: true when it worked. `holder` holds the 3D
/// buffer (`Holder`). Nothing with the setting on or the set already opened; else the set comes
/// from the 3D buffer's `QueryInterface` and `QuerySupport` (get and set both), for the voice
/// manager its mode set to 2; then logged. A failure leaves the set where it went (released at the
/// close). Refuses a holder that is no pointer (`make_buffer`'s kept EAX bug).
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

/// The room's reverb set when EAX is open. Refuses a room past the table.
pub fn set_eax_room(state: &AudioOwn, room: u32) -> Result<(), Refused> {
    // The room's entry lies inside the table; a room past it is refused.
    rc2k_core::audio::levels::eax_room_offset(room)?;
    let p = state.eax_properties.get();
    if p != 0 {
        // The reverb's property 0 set with the room's 0x10 bytes.
        return Err(BadInput(NO_SETS).into());
    }
    Ok(())
}
