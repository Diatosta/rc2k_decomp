//! The race's settings and results as the front end keeps them, out of RAL.EXE's image (IDA
//! 0x75ACCC .. 0x75B340; phase 7, typed): the mode, the stage, the direction, the two
//! players' settings (`RaceCar`) and the two cars' results (`RaceResult`). `#[repr(C)]`
//! structs of atomic cells with the image's layout, one static. Ends in unreferenced bytes up to the damage model.

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

/// The binding of `RaceCar::bindings` no word of the player's record goes to: the ninth
/// device binding's (key byte 0x5C), left as the store's start value (0) and read as it is.
pub const UNSET_BINDING: usize = 9;

/// The first part of one player's settings as the race variables keep them (IDA 0x75AD1C and
/// 0x75AE88, 0x16C bytes each, the controls (`PlayerControls`) the last 0x6C of them): copies of
/// fields of the player's record (`Player`), at the record's order less the words the race
/// does not take. Only the game's thread reaches it. Field docs name the record's field and
/// offset.
#[repr(C)]
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
    /// `Player.damage` words +0xD4 .. +0x100 and +0x108 (+0x104 goes into
    /// `wear[9]`, whose own +0x1A0 follows it).
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
/// player's controller none when it is the first's (`input::device_command`), so they are
/// atomic cells beside the game thread's own part.
#[repr(C)]
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
    /// `rc2k_core::input::devices::DEVICE_BINDINGS`): `Player.bindings`' eleven words go to
    /// all but `UNSET_BINDING` (`race::single_rally`, `net::take_player_settings`, as the
    /// original).
    pub bindings: [Dword; 12],
    /// `Player.keys` (+0x1C .. +0x27): the 12 bytes of the controls.
    pub keys: Bytes<12>,
}

impl PlayerControls {
    /// The binding words the player's record's eleven go to, in order: all but
    /// `UNSET_BINDING`.
    pub fn recorded_bindings(&self) -> impl Iterator<Item = &Dword> {
        let (head, tail) = self.bindings.split_at(UNSET_BINDING);
        head.iter().chain(&tail[1..])
    }
}

/// One player's settings as the race variables keep them (IDA 0x75AD1C and 0x75AE88, 0x16C
/// bytes each): the game thread's own fields and the controls both threads reach. The own
/// fields read through the record (`car.tyre`).
#[repr(C)]
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

/// A car's result as the race leaves it (IDA 0x75B01C and 0x75B08C, 0x70 bytes each).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct RaceResult {
    /// The car's time (`RaceState.results.results[0].time` / `RaceState.results.results[1].time`).
    pub time: PlainDword,
    /// One more word (`RaceState.results.results[0].word` / `RaceState.results.results[1].word`).
    pub word: PlainDword,
    /// The seconds, a float (`RaceState.results.results[0].secs` / `RaceState.results.results[1].secs`).
    pub secs: PlainFloat,
    /// The result (`g_raceResult` / `g_raceResult2`): -1 none, -2 left.
    pub result: PlainDword,
    /// The 0x40 bytes of data (`RaceState.results.results[0].data` / `RaceState.results.results[1].data`; the best line's text).
    pub data: PlainBytes<64>,
    /// The car's eight impact marks, from and back to `Player::impact_marks`.
    pub impact_marks: [PlainDword; 8],
}

/// The first twenty words of the race's settings (IDA 0x75ACCC ..): the mode, the stage, the
/// direction and the options, which only the game's thread reaches.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct SettingsHead {
    /// `g_newSimulationUnlocked`: 1 in the A8 game (`menu::apply_cheats`).
    pub a8_game: PlainDword,
    /// `RaceState.settings.rivals`: a copy of the standings summary (`ChampView.summary`, the game held its
    /// address): 1 while the summary (`saves::best_times::SUMMARY`) is taken, or 0.
    pub rivals: PlainDword,
    /// `RaceState.settings.stage`: the stage chosen.
    pub stage: PlainDword,
    /// `RaceState.settings.run_kind`: the run's kind argument.
    pub run_kind: PlainDword,
    /// `RaceState.settings.no_replay`: 1 when the race is not an event's and has more than one player.
    pub no_replay: PlainDword,
    /// `RaceState.settings.race_mode`: 1, or 2 in the events' one-car modes.
    pub race_mode: PlainDword,
    /// `RaceState.settings.kept`: a copy of `ScreenFlow.kept`, put back after the race.
    pub kept: PlainDword,
    /// `RaceState.settings.car_kind_current` (Shaun's `g_raceCarKinds`): the current player's record word at +8.
    pub car_kind_current: PlainDword,
    /// `RaceState.settings.car_kind_first`: player 1's record word at +8.
    pub car_kind_first: PlainDword,
    /// `RaceState.settings.direction`: the stage's direction (the selected group's copy).
    pub direction: PlainDword,
    /// `RaceState.settings.unk_75acf4`: no IDA xref and no Rust use; 0 at start.
    pub unk_75acf4: PlainDword,
    /// `RaceState.settings.track_1c`: the track entry's +0x1C.
    pub track_1c: PlainDword,
    /// `RaceState.settings.track_20`: the track entry's +0x20.
    pub track_20: PlainDword,
    /// `RaceState.settings.lens_flare`: the lens flare option, copied from `OptionWork.lens_flare`.
    pub lens_flare: PlainDword,
    /// `RaceState.settings.unused_percent_a`: read only in dead code outside any function (0x470060 .. 0x4700A1, beside the options toggles); 100 (0x64) at start.
    pub unused_percent_a: PlainDword,
    /// `RaceState.settings.unused_percent_b`: read only in dead code outside any function (0x4700C0 ..); 100 (0x64) at start.
    pub unused_percent_b: PlainDword,
    /// `RaceState.settings.reflections`: the reflections option (graphics page), copied from `OptionWork.reflections`.
    pub reflections: PlainDword,
    /// `RaceState.settings.car_shadows`: the car shadows option, copied from `OptionWork.car_shadows`.
    pub car_shadows: PlainDword,
    /// `RaceState.settings.car_detail`: the car detail option, copied from `OptionWork.car_detail`.
    pub car_detail: PlainDword,
    /// `RaceState.settings.headlights`: the headlights option (all cars' lamps), copied from `OptionWork.headlights`.
    pub headlights: PlainDword,
}

/// The race's settings as the front end keeps them (IDA 0x75ACCC .. 0x75AFF4; "Graham's"
/// variables, which `race::copy_settings_in` copies to the race's ("Shaun's")): the mode,
/// the stage, the direction, the two players' settings. The head's fields read through the
/// record (`settings.stage`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceSettings {
    /// The mode, the stage, the direction and the options.
    pub inline: GameThread<SettingsHead>,
    /// The two players' settings: [0] the current player's (the "second set", 0x75AD1C),
    /// [1] player 1's (the "first set", 0x75AE88).
    pub cars: [RaceCar; 2],
}

impl core::ops::Deref for RaceSettings {
    type Target = SettingsHead;
    fn deref(&self) -> &SettingsHead {
        &self.inline
    }
}

/// The race's results (IDA 0x75AFF4 .. 0x75B10C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceResults {
    /// `RaceState.results.race_stage`: the arcade race's stage (Shaun's `RaceVars.race_stage`).
    pub race_stage: PlainDword,
    /// `RaceState.results.race_level`: the arcade race's level (Shaun's `RaceVars.race_level`).
    pub race_level: PlainDword,
    /// `RaceState.results.figure_ffc`: a figure of the race (the record's +0x654 side; `race::race_shell`).
    pub figure_ffc: PlainDword,
    /// `RaceState.results.figure_000`: a figure of the race (Shaun's `RaceTables.start_place`).
    pub figure_000: PlainDword,
    /// `RaceState.results.service_spent_current`: the current player's record +0x288.
    pub service_spent_current: PlainDword,
    /// `RaceState.results.service_spent_first`: player 1's record +0x288.
    pub service_spent_first: PlainDword,
    /// `RaceState.results.hud_bits_current`: the current player's record +0x284 (the HUD's bits); goes back to it.
    pub hud_bits_current: PlainDword,
    /// `RaceState.results.hud_bits_first`: player 1's record +0x284.
    pub hud_bits_first: PlainDword,
    /// `RaceState.results.flags_current`: the current player's record +0x278.
    pub flags_current: PlainDword,
    /// `RaceState.results.flags_first`: player 1's record +0x278.
    pub flags_first: PlainDword,
    /// The two cars' results: [0] the current car's (0x75B01C), [1] the first car's (0x75B08C).
    pub results: [RaceResult; 2],
    /// `RaceState.results.stage_index`: the arcade race's index into the stage table (`drive::finish_race`).
    pub stage_index: PlainDword,
    /// `RaceState.results.finish_player`: the finishing player (+0xAA8 of the car block).
    pub finish_player: PlainDword,
    /// `RaceState.results.finish_kind`: the cars' kind (into `g_raceCarKinds`).
    pub finish_kind: PlainDword,
    /// `RaceState.results.finished`: 1 at the championship's finish / the arcade final.
    pub finished: PlainDword,
}

/// The race's settings and results.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceState {
    /// The race's settings as the front end keeps them (IDA 0x75ACCC .. 0x75AFF4; "Graham's"
    pub settings: RaceSettings,
    /// The race's results (IDA 0x75AFF4 .. 0x75B10C).
    pub results: GameThread<RaceResults>,
    /// Unreferenced bytes (IDA 0X75B10C .. 0X75B340).
    pub _gap_75b10c: Bytes<564>,
}

impl RaceState {
    /// First values the image's.
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
