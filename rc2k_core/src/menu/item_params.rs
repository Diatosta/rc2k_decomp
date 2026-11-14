//! What the parameterised item functions (`tables_data::ItemFn`) are given: which volume, which
//! option switch, which counter, which event. One variant of `ItemFn` stands for a family of item
//! functions that differ only in such a value.

/// The volumes of the sound pages that have a "less" and a "more" item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VolumeKind {
    /// The general volume, sent to the audio thread.
    Master,
    /// The music's volume (`options.music_level`).
    Music,
    /// The race music's volume (`options.race_music_level`), kept for the race.
    RaceMusic,
    /// The race sounds' volume (`audio::levels::Levels::menu_sounds_a`), heard with a race sound.
    RaceSounds,
    /// The speech's volume (`options.speech_level`).
    Speech,
    /// The first of the other menu sounds' volumes (`audio::levels::Levels::sounds_a`).
    MenuSoundsA,
    /// The second of the other menu sounds' volumes (`audio::levels::Levels::sounds_b`).
    MenuSoundsB,
    /// The menu sounds' volume (`options.menu_sounds_level`), heard with a menu sound.
    MenuSounds,
}

/// The switches of the options pages (the items flipped left or right).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionSwitch {
    /// `sound.menu.options.race_music_on`: the race music on or off (tune 8 plays only while it is
    /// 1), kept in the options file.
    RaceMusicOn,
    /// `play.champ_view.switch_f8`: which stage record tables the stage pages show.
    StageRecordsSet,
    /// `play.champ_view.switch_fc`: the alternative tables of those (adds 2 to the table).
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
    /// A3D.
    A3d,
    /// Dolby.
    Dolby,
    /// The channels.
    Channels,
}

/// The stage pages' two counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StageCounter {
    /// `captions.shown_group`: the stage group shown, 0 .. 5.
    Group,
    /// `captions.shown_stage`: the stage shown in its group, 1 .. 6.
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
