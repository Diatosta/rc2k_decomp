//! The sound module's owner: the sound stores in one struct, grouped by what they serve, handed by
//! reference to the code that runs them. `sound()` is the boundary for the callers that do not
//! take a group yet.

use crate::session::{GameThread, PlainDword};
use crate::sound::sets_store::{
    CueLists, MenuSounds, SoundKinds, SoundRequest, SoundSets, SpotVoices, SurfaceSets,
};
use crate::sound::store::{
    CarStream, CdAudio, RaceClock, RaceSounds, SettingsApplied, SoundOptions, SoundShared,
    SpotPicks, StageSounds, SurfaceSounds,
};

/// The race's sounds: the objects, the clock, the surface and stage constants, the sets and
/// kinds, the spot voices and the car stream.
///
/// `shared` (the input thread's force-feedback effects read the player counts) and `sets` (the
/// audio thread loads its lists) stay atomic; the rest is the game thread's.
pub struct RaceSoundState {
    pub shared: SoundShared,
    pub sounds: GameThread<RaceSounds>,
    pub clock: GameThread<RaceClock>,
    pub surface: GameThread<SurfaceSounds>,
    pub stage: GameThread<StageSounds>,
    pub spot_picks: GameThread<SpotPicks>,
    pub sets: SoundSets,
    /// The request each sound is sent with (a work area).
    pub request: GameThread<SoundRequest>,
    pub cue_lists: GameThread<CueLists>,
    pub kinds: GameThread<SoundKinds>,
    pub surface_sets: GameThread<SurfaceSets>,
    pub spot_voices: GameThread<SpotVoices>,
    pub car_stream: GameThread<CarStream>,
    /// 1 at start; the race's sounds switched.
    pub sounds_on: GameThread<PlainDword>,
}

impl RaceSoundState {
    pub const fn new() -> Self {
        RaceSoundState {
            shared: SoundShared::new(),
            sounds: GameThread::new(RaceSounds::new()),
            clock: GameThread::new(RaceClock::new()),
            surface: GameThread::new(SurfaceSounds::new()),
            stage: GameThread::new(StageSounds::new()),
            spot_picks: GameThread::new(SpotPicks::new()),
            sets: SoundSets::new(),
            request: GameThread::new(SoundRequest::new()),
            cue_lists: GameThread::new(CueLists::new()),
            kinds: GameThread::new(SoundKinds::new()),
            surface_sets: GameThread::new(SurfaceSets::new()),
            spot_voices: GameThread::new(SpotVoices::new()),
            car_stream: GameThread::new(CarStream::new()),
            sounds_on: GameThread::new(PlainDword::new(1)),
        }
    }
}

/// The menus' sounds and the kept sound settings. `menu_sounds` (the audio thread loads its
/// list) and `options` (it reads the tune turn) stay atomic.
pub struct MenuSoundState {
    pub menu_sounds: MenuSounds,
    /// The clock the next menu sound may be heard at.
    pub next_sound_time: GameThread<PlainDword>,
    pub options: SoundOptions,
    pub settings_applied: GameThread<SettingsApplied>,
    pub cd_audio: GameThread<CdAudio>,
}

impl MenuSoundState {
    pub const fn new() -> Self {
        MenuSoundState {
            menu_sounds: MenuSounds::new(),
            next_sound_time: GameThread::new(PlainDword::new(0)),
            options: SoundOptions::new(),
            settings_applied: GameThread::new(SettingsApplied::new()),
            cd_audio: GameThread::new(CdAudio::new()),
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
