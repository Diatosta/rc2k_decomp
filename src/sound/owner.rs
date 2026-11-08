//! The sound module's owner (step 11, owned state): the sound stores in one struct, grouped
//! by what they serve, handed by reference to the code that runs them in place of global
//! stores. `sound()` is the boundary for the callers that do not take a group yet
//! (`notes/memory_layer_plan.md`, step 11).

use crate::session::Dword;
use crate::sound::sets_store::{
    CueLists, MenuSounds, RaceSoundSets, SoundKinds, SpotVoices, SurfaceSets,
};
use crate::sound::store::{
    CarStream, CdAudio, RaceClock, RaceSounds, SettingsApplied, SoundOptions, SpotPicks,
    StageSounds, SurfaceSounds,
};

/// The race's sounds: the objects, the clock, the surface and stage constants, the sets and
/// kinds, the spot voices and the car stream.
pub struct RaceSoundState {
    pub sounds: RaceSounds,
    pub clock: RaceClock,
    pub surface: SurfaceSounds,
    pub stage: StageSounds,
    pub spot_picks: SpotPicks,
    pub sets: RaceSoundSets,
    pub cue_lists: CueLists,
    pub kinds: SoundKinds,
    pub surface_sets: SurfaceSets,
    pub spot_voices: SpotVoices,
    pub car_stream: CarStream,
    /// `g_raceSoundsOn` (0x6218F0): 1 at start; the race's sounds switched.
    pub sounds_on: Dword,
}

impl RaceSoundState {
    pub const fn new() -> Self {
        RaceSoundState {
            sounds: RaceSounds::new(),
            clock: RaceClock::new(),
            surface: SurfaceSounds::new(),
            stage: StageSounds::new(),
            spot_picks: SpotPicks::new(),
            sets: RaceSoundSets::new(),
            cue_lists: CueLists::new(),
            kinds: SoundKinds::new(),
            surface_sets: SurfaceSets::new(),
            spot_voices: SpotVoices::new(),
            car_stream: CarStream::new(),
            sounds_on: Dword::new(1),
        }
    }
}

/// The menus' sounds and the kept sound settings.
pub struct MenuSoundState {
    pub menu_sounds: MenuSounds,
    /// `g_nextMenuSoundTime` (IDA 0x511B0C): the clock the next menu sound may be heard at.
    pub next_sound_time: Dword,
    pub options: SoundOptions,
    pub settings_applied: SettingsApplied,
    pub cd_audio: CdAudio,
}

impl MenuSoundState {
    pub const fn new() -> Self {
        MenuSoundState {
            menu_sounds: MenuSounds::new(),
            next_sound_time: Dword::new(0),
            options: SoundOptions::new(),
            settings_applied: SettingsApplied::new(),
            cd_audio: CdAudio::new(),
        }
    }
}

/// The sound module's state.
pub struct Sound {
    pub race: RaceSoundState,
    pub menu: MenuSoundState,
}

/// The one instance. Reached through `sound()`; `moved` registers its stores' places.
pub static SOUND: Sound = Sound {
    race: RaceSoundState::new(),
    menu: MenuSoundState::new(),
};

/// The sound state, for a caller that does not take it as a parameter yet.
pub fn sound() -> &'static Sound {
    &SOUND
}
