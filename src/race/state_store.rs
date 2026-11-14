//! The race's settings and results as the front end keeps them: the mode, the stage, the
//! direction, the two players' settings (`RaceCar`) and the two cars' results (`RaceResult`).
//! `#[repr(C)]` structs of atomic cells, one static. Ends in unreferenced bytes up to the damage
//! model.

use crate::session::{Bytes, Dword, GameThread, PlainBytes, PlainDword, PlainFloat};

/// A gap's first bytes: `runs` are (offset, bytes) of the non-zero ones.
const fn sparse<const N: usize>(runs: &[(usize, &[u8])]) -> [u8; N] {
    let mut out = [0u8; N];
    let mut r = 0;
    while r < runs.len() {
        let (at, bytes) = runs[r];
        let mut i = 0;
        while i < bytes.len() {
            out[at + i] = bytes[i];
            i += 1;
        }
        r += 1;
    }
    out
}

/// The binding of `RaceCar::bindings` no word of the player's record goes to: the ninth device
/// binding's (key byte 0x5C), left as the store's start value (0) and read as it is.
pub const UNSET_BINDING: usize = 9;

/// The first part of one player's settings as the race variables keep them (0x16C bytes each, the
/// controls (`PlayerControls`) the last 0x6C of them): copies of fields of the player's record
/// (`Player`), in the record's order less the words the race does not take. Only the game's
/// thread reaches it. Field docs name the record's field and offset.
#[derive(rc2k_derive::Cells)]
pub struct RaceCarOwn {
    /// `Player.tyre` (+0x10C).
    pub tyre: PlainDword,
    /// `Player.brake_balance` (+0x110).
    pub brake_balance: PlainDword,
    /// `Player.ride_height` (+0x114).
    pub ride_height: PlainDword,
    /// `Player.suspension_stiffness` (+0x118).
    pub suspension_stiffness: PlainDword,
    /// `Player.steering_sensitivity` (+0x11C).
    pub steering_sensitivity: PlainDword,
    /// `Player.gear_ratio` (+0x120).
    pub gear_ratio: PlainDword,
    /// `Player.damage` words +0xD4 .. +0x100 and +0x108 (+0x104 goes into `wear[9]`, whose own
    /// +0x1A0 follows it).
    pub damage: [PlainDword; 13],
    /// `Player.parts_wear` and `tyre_wear` (+0x17C .. +0x208, one word each).
    pub wear: [PlainDword; 36],
    /// `Player.map_area` (+0x158).
    pub map_area: PlainDword,
    /// `Player.rear_mirror` (+0x15C).
    pub rear_mirror: PlainDword,
    /// `Player.rev_counter` (+0x160).
    pub rev_counter: PlainDword,
    /// `Player.metric_units` (+0x178).
    pub metric_units: PlainDword,
    /// `Player.message_display` (+0x168).
    pub message_display: PlainDword,
    /// `Player.car_info` (+0x164).
    pub car_info: PlainDword,
    /// `Player.progress_bar` (+0x16C).
    pub progress_bar: PlainDword,
    /// `Player.driver_info` (+0x170).
    pub driver_info: PlainDword,
    /// `Player.pace_notes` (+0x174).
    pub pace_notes: PlainDword,
    /// `Player.race_number` (+0x28).
    pub race_number: PlainDword,
    /// `Player.codriver_sex` (+0x154).
    pub codriver_sex: PlainDword,
    /// `Player.view` (+0xC).
    pub view: PlainDword,
}

/// The controls of one player's settings (the last 0x6C bytes of the record, from
/// `Player.controller`): the input thread reads them to set a device up and makes the second
/// player's controller none when it is the first's (`input::device_command`), so they are atomic
/// cells beside the game thread's own part.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct PlayerControls {
    /// `Player.controller` (+0x2C).
    pub controller: Dword,
    /// `Player.gearing` (+0x30); the network lobby keeps the car's colours here.
    pub gearing: Dword,
    /// `Player.sensitivity_level` (+0x34).
    pub sensitivity_level: Dword,
    /// `Player.axis_codes` (+0x38), the four bytes as one word.
    pub axis_codes: Dword,
    /// `Player.controller_mode` (+0x3C).
    pub controller_mode: Dword,
    /// `Player.end_zone_pct` (+0x40): the end zone, percent.
    pub end_zone_pct: Dword,
    /// `Player.centre_dead_zone_pct` (+0x44): the centre dead zone, percent.
    pub centre_dead_zone_pct: Dword,
    /// `Player.strength` (+0x48).
    pub strength: Dword,
    /// The record's +0x4C: not taken.
    pub _unused_4c: Dword,
    /// The twelve bindings' controller words (`input::set_up_device`, by
    /// `rc2k_core::input::devices::DEVICE_BINDINGS`): `Player.bindings`' eleven words go to all but
    /// `UNSET_BINDING` (`race::single_rally`, `net::take_player_settings`).
    pub bindings: [Dword; 12],
    /// `Player.keys` (+0x1C .. +0x27): the 12 bytes of the controls.
    pub keys: Bytes<12>,
}

impl PlayerControls {
    /// The binding words the player's record's eleven go to, in order: all but `UNSET_BINDING`.
    pub fn recorded_bindings(&self) -> impl Iterator<Item = &Dword> {
        let (head, tail) = self.bindings.split_at(UNSET_BINDING);
        head.iter().chain(&tail[1..])
    }
}

/// One player's settings as the race variables keep them: the game thread's own fields and the
/// controls both threads reach. The own fields read through the record (`car.tyre`).
#[derive(rc2k_derive::Cells)]
pub struct RaceCar {
    /// The fields only the game's thread reaches.
    pub own: GameThread<RaceCarOwn>,
    /// The controls, `Player.controller` on.
    pub controls: PlayerControls,
}

impl core::ops::Deref for RaceCar {
    type Target = RaceCarOwn;
    fn deref(&self) -> &RaceCarOwn {
        &self.own
    }
}

/// A car's result as the race leaves it.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct RaceResult {
    /// The car's time.
    pub time: PlainDword,
    /// One more word.
    pub word: PlainDword,
    /// The seconds, a float.
    pub secs: PlainFloat,
    /// The result: -1 none, -2 left.
    pub result: PlainDword,
    /// The 0x40 bytes of data (the best line's text).
    pub data: PlainBytes<64>,
    /// The car's eight impact marks, from and back to `Player::impact_marks`.
    pub impact_marks: [PlainDword; 8],
}

/// The first twenty words of the race's settings: the mode, the stage, the direction and the
/// options, which only the game's thread reaches.
#[derive(rc2k_derive::Cells)]
pub struct SettingsHead {
    /// 1 in the A8 game, the new simulation unlocked (`menu::apply_cheats`).
    pub a8_game: PlainDword,
    /// A copy of the standings summary (`ChampView.summary`): 1 while the summary
    /// (`saves::best_times::SUMMARY`) is taken, or 0.
    pub rivals: PlainDword,
    /// The stage chosen.
    pub stage: PlainDword,
    /// The run's kind argument.
    pub run_kind: PlainDword,
    /// 1 when the race is not an event's and has more than one
    /// player.
    pub no_replay: PlainDword,
    /// 1, or 2 in the events' one-car modes.
    pub race_mode: PlainDword,
    /// A copy of `ScreenFlow.kept`, put back after the race.
    pub kept: PlainDword,
    /// The current player's car kind (Shaun's `race_car_kinds`): the record word at +8.
    pub car_kind_current: PlainDword,
    /// Player 1's record word at +8.
    pub car_kind_first: PlainDword,
    /// The stage's direction (the selected group's copy).
    pub direction: PlainDword,
    /// Never used; 0 at start.
    pub unk_75acf4: PlainDword,
    /// The track entry's +0x1C.
    pub track_1c: PlainDword,
    /// The track entry's +0x20.
    pub track_20: PlainDword,
    /// The lens flare option, copied from `MenuOptions.lens_flare`.
    pub lens_flare: PlainDword,
    /// Unused; 100 (0x64) at start.
    pub unused_percent_a: PlainDword,
    /// Unused; 100 (0x64) at start.
    pub unused_percent_b: PlainDword,
    /// The reflections option (graphics page), copied from
    /// `MenuOptions.reflections`.
    pub reflections: PlainDword,
    /// The car shadows option, copied from
    /// `MenuOptions.car_shadows`.
    pub car_shadows: PlainDword,
    /// The car detail option, copied from `MenuOptions.car_detail`.
    pub car_detail: PlainDword,
    /// The headlights option (all cars' lamps), copied from
    /// `MenuOptions.headlights`.
    pub headlights: PlainDword,
}

impl SettingsHead {
    /// Takes the visual options the player chose on the options pages.
    pub fn set_visual_options(&self, options: &crate::menu::options_store::OptionsSnapshot) {
        self.lens_flare.set(options.lens_flare);
        self.reflections.set(options.reflections);
        self.car_shadows.set(options.car_shadows);
        self.car_detail.set(options.car_detail);
        self.headlights.set(options.headlights);
    }
}

/// The race's settings as the front end keeps them ("Graham's" variables, which
/// `race::copy_settings_in` copies to the race's ("Shaun's")): the mode, the stage, the direction,
/// the two players' settings. The head's fields read through the record (`settings.stage`).
#[derive(rc2k_derive::Cells)]
pub struct RaceSettings {
    /// The mode, the stage, the direction and the options.
    pub inline: GameThread<SettingsHead>,
    /// The two players' settings: [0] the current player's (the "second set"), [1] player 1's (the
    /// "first set").
    pub cars: [RaceCar; 2],
}

impl core::ops::Deref for RaceSettings {
    type Target = SettingsHead;
    fn deref(&self) -> &SettingsHead {
        &self.inline
    }
}

/// The race's results.
#[derive(rc2k_derive::Cells)]
pub struct RaceResults {
    /// The arcade race's stage (Shaun's `RaceVars.race_stage`).
    pub race_stage: PlainDword,
    /// The arcade race's level (Shaun's `RaceVars.race_level`).
    pub race_level: PlainDword,
    /// A figure of the race (the record's +0x654 side;
    /// `race::race_shell`).
    pub figure_ffc: PlainDword,
    /// A figure of the race (Shaun's `RaceTables.start_place`).
    pub figure_000: PlainDword,
    /// The current player's record +0x288.
    pub service_spent_current: PlainDword,
    /// Player 1's record +0x288.
    pub service_spent_first: PlainDword,
    /// The current player's record +0x284 (the HUD's bits);
    /// goes back to it.
    pub hud_bits_current: PlainDword,
    /// Player 1's record +0x284.
    pub hud_bits_first: PlainDword,
    /// The current player's record +0x278.
    pub flags_current: PlainDword,
    /// Player 1's record +0x278.
    pub flags_first: PlainDword,
    /// The two cars' results: [0] the current car's, [1] the first car's.
    pub results: [RaceResult; 2],
    /// The arcade race's index into the stage table
    /// (`drive::finish_race`).
    pub stage_index: PlainDword,
    /// The finishing player (+0xAA8 of the car block).
    pub finish_player: PlainDword,
    /// The cars' kind (into `race_car_kinds`).
    pub finish_kind: PlainDword,
    /// 1 at the championship's finish / the arcade final.
    pub finished: PlainDword,
}

/// The race's settings and results.
#[derive(rc2k_derive::Cells)]
pub struct RaceState {
    /// The race's settings as the front end keeps them ("Graham's" variables).
    pub settings: RaceSettings,
    /// The race's results.
    pub results: GameThread<RaceResults>,
    /// Unreferenced bytes.
    pub _gap_75b10c: Bytes<564>,
}

impl RaceState {
    /// The starting values.
    pub const fn new() -> Self {
        RaceState {
            settings: RaceSettings {
                inline: GameThread::new(SettingsHead {
                    a8_game: PlainDword::new(0),
                    rivals: PlainDword::new(0),
                    stage: PlainDword::new(0),
                    run_kind: PlainDword::new(0),
                    no_replay: PlainDword::new(0),
                    race_mode: PlainDword::new(1),
                    kept: PlainDword::new(0),
                    car_kind_current: PlainDword::new(0),
                    car_kind_first: PlainDword::new(0),
                    direction: PlainDword::new(0),
                    unk_75acf4: PlainDword::new(0),
                    track_1c: PlainDword::new(0x2d),
                    track_20: PlainDword::new(0x19),
                    lens_flare: PlainDword::new(0),
                    unused_percent_a: PlainDword::new(0x64),
                    unused_percent_b: PlainDword::new(0x64),
                    reflections: PlainDword::new(0),
                    car_shadows: PlainDword::new(0),
                    car_detail: PlainDword::new(0),
                    headlights: PlainDword::new(0),
                }),
                cars: [
                    RaceCar {
                        own: GameThread::new(RaceCarOwn {
                            tyre: PlainDword::new(0),
                            brake_balance: PlainDword::new(0x32),
                            ride_height: PlainDword::new(0x32),
                            suspension_stiffness: PlainDword::new(0x32),
                            steering_sensitivity: PlainDword::new(0x32),
                            gear_ratio: PlainDword::new(2),
                            damage: [const { PlainDword::new(0) }; 13],
                            wear: [const { PlainDword::new(0) }; 36],
                            map_area: PlainDword::new(0),
                            rear_mirror: PlainDword::new(0),
                            rev_counter: PlainDword::new(0),
                            metric_units: PlainDword::new(0),
                            message_display: PlainDword::new(0),
                            car_info: PlainDword::new(1),
                            progress_bar: PlainDword::new(1),
                            driver_info: PlainDword::new(1),
                            pace_notes: PlainDword::new(1),
                            race_number: PlainDword::new(0x61),
                            codriver_sex: PlainDword::new(0),
                            view: PlainDword::new(1),
                        }),
                        controls: PlayerControls {
                            controller: Dword::new(0),
                            gearing: Dword::new(0),
                            sensitivity_level: Dword::new(0),
                            axis_codes: Dword::new(0x2011000),
                            controller_mode: Dword::new(0),
                            end_zone_pct: Dword::new(0),
                            centre_dead_zone_pct: Dword::new(0xa),
                            strength: Dword::new(0),
                            _unused_4c: Dword::new(0),
                            bindings: [
                                Dword::new(0x200),
                                Dword::new(0x100),
                                Dword::new(0x400),
                                Dword::new(0x800),
                                Dword::new(0x201),
                                Dword::new(0x101),
                                Dword::new(2),
                                Dword::new(0),
                                Dword::new(0),
                                Dword::new(0),
                                Dword::new(0),
                                Dword::new(0),
                            ],
                            keys: Bytes::new(sparse::<12>(&[(
                                0x0,
                                &[
                                    0x2c, 0x2d, 0x66, 0x6b, 0x39, 0x69, 0x68, 0x26, 0x27, 0x2e,
                                    0x19, 0x28,
                                ],
                            )])),
                        },
                    },
                    RaceCar {
                        own: GameThread::new(RaceCarOwn {
                            tyre: PlainDword::new(0),
                            brake_balance: PlainDword::new(0x32),
                            ride_height: PlainDword::new(0x32),
                            suspension_stiffness: PlainDword::new(0x32),
                            steering_sensitivity: PlainDword::new(0x32),
                            gear_ratio: PlainDword::new(2),
                            damage: [const { PlainDword::new(0) }; 13],
                            wear: [const { PlainDword::new(0) }; 36],
                            map_area: PlainDword::new(0),
                            rear_mirror: PlainDword::new(0),
                            rev_counter: PlainDword::new(0),
                            metric_units: PlainDword::new(0),
                            message_display: PlainDword::new(0),
                            car_info: PlainDword::new(1),
                            progress_bar: PlainDword::new(1),
                            driver_info: PlainDword::new(1),
                            pace_notes: PlainDword::new(1),
                            race_number: PlainDword::new(0x60),
                            codriver_sex: PlainDword::new(0),
                            view: PlainDword::new(1),
                        }),
                        controls: PlayerControls {
                            controller: Dword::new(0),
                            gearing: Dword::new(0),
                            sensitivity_level: Dword::new(0),
                            axis_codes: Dword::new(0x2011000),
                            controller_mode: Dword::new(0),
                            end_zone_pct: Dword::new(0),
                            centre_dead_zone_pct: Dword::new(0xa),
                            strength: Dword::new(0),
                            _unused_4c: Dword::new(0),
                            bindings: [
                                Dword::new(0x200),
                                Dword::new(0x100),
                                Dword::new(0x400),
                                Dword::new(0x800),
                                Dword::new(0x201),
                                Dword::new(0x101),
                                Dword::new(2),
                                Dword::new(0),
                                Dword::new(0),
                                Dword::new(0),
                                Dword::new(0),
                                Dword::new(0),
                            ],
                            keys: Bytes::new(sparse::<12>(&[(
                                0x0,
                                &[
                                    0x4f, 0x50, 0x4d, 0x51, 0x52, 0x4e, 0x53, 0x4a, 0x48, 0x49,
                                    0x4c, 0x4b,
                                ],
                            )])),
                        },
                    },
                ],
            },
            results: GameThread::new(RaceResults {
                race_stage: PlainDword::new(0),
                race_level: PlainDword::new(0),
                figure_ffc: PlainDword::new(0),
                figure_000: PlainDword::new(0),
                service_spent_current: PlainDword::new(0),
                service_spent_first: PlainDword::new(0),
                hud_bits_current: PlainDword::new(0xffffffff),
                hud_bits_first: PlainDword::new(0xffffffff),
                flags_current: PlainDword::new(0),
                flags_first: PlainDword::new(0),
                results: [
                    RaceResult {
                        time: PlainDword::new(0),
                        word: PlainDword::new(0),
                        secs: PlainFloat::new(0.0),
                        result: PlainDword::new(0),
                        data: PlainBytes::new([0xff; 64]),
                        impact_marks: [const { PlainDword::new(0) }; 8],
                    },
                    RaceResult {
                        time: PlainDword::new(0),
                        word: PlainDword::new(0),
                        secs: PlainFloat::new(0.0),
                        result: PlainDword::new(0),
                        data: PlainBytes::new([0xff; 64]),
                        impact_marks: [const { PlainDword::new(0) }; 8],
                    },
                ],
                stage_index: PlainDword::new(0),
                finish_player: PlainDword::new(1),
                finish_kind: PlainDword::new(0),
                finished: PlainDword::new(0),
            }),
            _gap_75b10c: Bytes::new(sparse::<564>(&[
                (0x8, &[0x78, 0x56, 0x34, 0x12]),
                (
                    0x16,
                    &[
                        0x32, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x8f, 0x40, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0xf0, 0xee, 0x40, 0x00, 0x00, 0x80, 0x43, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x80, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x90, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xa0, 0x40, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0xb0, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0xf0, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x41, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0xe0, 0x41, 0xce, 0xfa, 0x1d, 0xac, 0x00, 0x00,
                        0x80, 0x37, 0xce, 0xfa, 0x1d, 0xac, 0x33, 0xe3, 0xea, 0x3b, 0x35, 0x20,
                        0x99, 0x3f, 0x00, 0x00, 0x00, 0x00, 0xce, 0xfa, 0x1d, 0xac, 0x78, 0x56,
                        0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x78, 0x56,
                        0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x78, 0x56, 0x34, 0x12, 0x78, 0x56,
                        0x34, 0x12, 0x78, 0x56, 0x34, 0x12,
                    ],
                ),
                (
                    0xc8,
                    &[
                        0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0xbc, 0xbe, 0x4d, 0x00, 0x00,
                        0x8c, 0x42,
                    ],
                ),
            ])),
        }
    }
}
