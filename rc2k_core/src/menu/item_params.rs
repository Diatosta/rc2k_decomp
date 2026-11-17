//! What the parameterised item functions (`tables_data::ItemFn`) are given: which volume, which
//! option switch, which counter, which event. One variant of `ItemFn` stands for a family of item
//! functions that differ only in such a value.

/// The volumes of the sound pages that have a "less" and a "more" item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolumeKind {
    /// The "general volume" slider, sent to the audio thread (`audio::levels::Levels::general_volume`).
    General,
    /// The "menu music" slider (`options.menu_music_volume`).
    MenuMusic,
    /// The "race music" slider (`options.race_music_level`), kept for the race.
    RaceMusic,
    /// The "race effects" slider (`audio::levels::Levels::race_effects_volume`), heard with a race sound.
    RaceEffects,
    /// The "menu voices" slider (`options.menu_voices_volume`).
    MenuVoices,
    /// The "co-driver's voice" slider (`audio::levels::Levels::codriver_voice_volume`).
    CodriverVoice,
    /// The "car engine" slider (`audio::levels::Levels::engine_volume`).
    Engine,
    /// The "menu effects" slider (`options.menu_effects_volume`), heard with a menu sound.
    MenuEffects,
    /// The "film volume" slider, kept in the options file (`platform::sound::FILM_VOLUME`).
    Film,
}

/// The switches of the options pages (the items flipped left or right).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionSwitch {
    /// `sound.menu.options.race_music_on`: the race music on or off (tune 8 plays only while it is
    /// 1), kept in the options file.
    RaceMusicOn,
    /// `play.champ_view.switches.records_set`: which stage record tables the stage pages show.
    StageRecordsSet,
    /// `play.champ_view.switches.records_alt`: the alternative tables of those (adds 2 to the
    /// table).
    StageRecordsAlt,
    /// Mipmapping (also flips its display choice).
    Mipmapping,
    /// Buffering (also flips its display choice).
    Buffering,
    /// The screen modes' switch (also flips the card choice and makes the mode list again).
    ScreenModeCard,
    /// Lens flares (also copied to the race settings).
    LensFlare,
    /// Reflections (also copied to the race settings).
    Reflections,
    /// Car shadows (also copied to the race settings).
    CarShadows,
    /// Headlights (also copied to the race settings).
    Headlights,
    /// DirectSound 3D.
    Ds3d,
    /// EAX.
    Eax,
    /// Dolby.
    Dolby,
    /// The channels.
    Channels,
    /// Sound while the window is not in front (`platform::sound::SOUND_IN_BACKGROUND`).
    SoundInBackground,
}

/// The stage pages' two counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageCounter {
    /// `champ_view.shown_stage.group`: the stage group shown, 0 .. 5.
    Group,
    /// `champ_view.shown_stage.number`: the stage shown in its group, 1 .. 6.
    InGroup,
}

/// The events the start menus' items begin; `starters::Seats` says for how many players.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventStart {
    SingleRally,
    TimeTrial,
    /// The championship (`starters::Championship::First`).
    Championship,
    /// The A8 championship (`starters::Championship::A8`).
    A8,
}
