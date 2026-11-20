//! The sound module's owner: the sound stores in one struct, grouped by what they serve, handed by
//! reference to the code that runs them.

use std::cell::RefCell;

use crate::memory::session::{GameThread, PlainDword};
use crate::sound::sets_store::{
    CueLists, MenuSounds, SoundKinds, SoundRequest, SoundSets, SpotVoices, SurfaceSets,
};
use crate::sound::store::{
    CarStream, RaceClock, RaceSounds, SoundOptions, SoundShared, SoundSlot, SpotKinds, StageSounds,
    SurfaceSounds, stage_timed_slot,
};

/// The race's sounds: the objects, the clock, the surface and stage constants, the sets and
/// kinds, the spot voices and the car stream.
///
/// All of it is the game thread's; what other threads read is in `SoundForThreads`.
pub struct RaceSoundState {
    pub sounds: GameThread<RaceSounds>,
    pub clock: GameThread<RefCell<RaceClock>>,
    pub surface: GameThread<SurfaceSounds>,
    pub stage: GameThread<RefCell<StageSounds>>,
    /// The stage's timed sound's slot.
    pub timed_slot: GameThread<SoundSlot>,
    pub spot_kinds: GameThread<SpotKinds>,
    /// The request each sound is sent with (a work area).
    pub request: GameThread<SoundRequest>,
    pub cue_lists: GameThread<CueLists>,
    pub kinds: GameThread<SoundKinds>,
    pub surface_sets: GameThread<SurfaceSets>,
    pub spot_voices: GameThread<SpotVoices>,
    pub car_stream: GameThread<RefCell<CarStream>>,
    /// 1 at start; the race's sounds switched.
    pub sounds_on: GameThread<PlainDword>,
}

impl RaceSoundState {
    pub const fn new() -> Self {
        RaceSoundState {
            sounds: GameThread::new(RaceSounds::new()),
            clock: GameThread::new(RefCell::new(RaceClock::new())),
            surface: GameThread::new(SurfaceSounds::new()),
            stage: GameThread::new(RefCell::new(StageSounds::new())),
            timed_slot: GameThread::new(stage_timed_slot()),
            spot_kinds: GameThread::new(SpotKinds::new()),
            request: GameThread::new(SoundRequest::new()),
            cue_lists: GameThread::new(CueLists::new()),
            kinds: GameThread::new(SoundKinds::new()),
            surface_sets: GameThread::new(SurfaceSets::new()),
            spot_voices: GameThread::new(SpotVoices::new()),
            car_stream: GameThread::new(RefCell::new(CarStream::new())),
            sounds_on: GameThread::new(PlainDword::new(1)),
        }
    }
}

/// The menus' sound pacing and the kept sound settings: the game thread's alone (`options` keeps
/// atomic cells because the option tables and the saved file address them as cells; no other
/// thread reads them).
pub struct MenuSoundState {
    /// The clock the next menu sound may be heard at.
    pub next_sound_time: GameThread<PlainDword>,
    pub options: SoundOptions,
}

impl MenuSoundState {
    pub const fn new() -> Self {
        MenuSoundState {
            next_sound_time: GameThread::new(PlainDword::new(0)),
            options: SoundOptions::new(),
        }
    }
}

/// What threads other than the game's read of the sound state; the game's thread writes all of
/// it, and the cells are atomic.
///
/// - The input thread (force feedback) reaches `shared` only: whether one player races and how many
///   sound objects are in use (`input::input_tick` takes `&SoundShared`, nothing else).
/// - The audio thread reaches `sets` and `menu_sounds` only: the lists a load command names
///   (`audio::thread::AudioInputs`). The game's thread fills them before it queues that command.
pub struct SoundForThreads {
    pub shared: SoundShared,
    /// The sets of sounds a race loads.
    pub sets: SoundSets,
    /// The settings page's menu sounds.
    pub menu_sounds: MenuSounds,
}

impl SoundForThreads {
    pub const fn new() -> Self {
        SoundForThreads {
            shared: SoundShared::new(),
            sets: SoundSets::new(),
            menu_sounds: MenuSounds::new(),
        }
    }
}

/// The sound module's state: `threads` is what other threads read, `race` and `menu` are the
/// game thread's.
pub struct Sound {
    pub threads: SoundForThreads,
    pub race: RaceSoundState,
    pub menu: MenuSoundState,
}

impl Sound {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
            threads: SoundForThreads::new(),
            race: RaceSoundState::new(),
            menu: MenuSoundState::new(),
        }
    }
}
