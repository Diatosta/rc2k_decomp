//! The race's settings and results as the front end keeps them: the mode, the stage, the
//! direction, the two players' settings (`RaceCar`) and the two cars' results (`RaceResult`).
//! `#[repr(C)]` structs of atomic cells, one static. Ends in unreferenced bytes up to the damage
//! model.

use crate::memory::session::{Bytes, Dword, GameThread, PlainDword, PlainFloat};

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
/// thread reaches it. Field docs name the record's field.
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct RaceCarOwn {
    /// `Player.tyre`.
    pub tyre: PlainDword,
    /// `Player.brake_balance`.
    pub brake_balance: PlainDword,
    /// `Player.ride_height`.
    pub ride_height: PlainDword,
    /// `Player.suspension_stiffness`.
    pub suspension_stiffness: PlainDword,
    /// `Player.steering_sensitivity`.
    pub steering_sensitivity: PlainDword,
    /// `Player.gear_ratio`.
    pub gear_ratio: PlainDword,
    /// `Player.damage`'s words but one (the one between them goes into `wear[9]`, which
    /// follows it).
    pub damage: [PlainDword; 13],
    /// `Player.parts_wear` and `tyre_wear`, one word each.
    pub wear: [PlainDword; 36],
    /// `Player.map_area`.
    pub map_area: PlainDword,
    /// `Player.rear_mirror`.
    pub rear_mirror: PlainDword,
    /// `Player.rev_counter`.
    pub rev_counter: PlainDword,
    /// `Player.metric_units`.
    pub metric_units: PlainDword,
    /// `Player.message_display`.
    pub message_display: PlainDword,
    /// `Player.car_info`.
    pub car_info: PlainDword,
    /// `Player.progress_bar`.
    pub progress_bar: PlainDword,
    /// `Player.driver_info`.
    pub driver_info: PlainDword,
    /// `Player.pace_notes`.
    pub pace_notes: PlainDword,
    /// `Player.race_number`.
    pub race_number: PlainDword,
    /// `Player.codriver_sex`.
    pub codriver_sex: PlainDword,
    /// `Player.view`.
    pub view: PlainDword,
}

/// The controls of one player's settings (the last 0x6C bytes of the record, from
/// `Player.controller`): the input thread reads them to set a device up and makes the second
/// player's controller none when it is the first's (`input::device_command`), so they are atomic
/// cells beside the game thread's own part.
#[cfg_attr(test, repr(C))]
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct PlayerControls {
    /// `Player.controller`.
    pub controller: Dword,
    /// `Player.gearing`; the network lobby keeps the car's colours here.
    pub gearing: Dword,
    /// `Player.sensitivity_level`.
    pub sensitivity_level: Dword,
    /// `Player.axis_codes`, the four bytes as one word.
    pub axis_codes: Dword,
    /// `Player.controller_mode`.
    pub controller_mode: Dword,
    /// `Player.end_zone_pct`: the end zone, percent.
    pub end_zone_pct: Dword,
    /// `Player.centre_dead_zone_pct`: the centre dead zone, percent.
    pub centre_dead_zone_pct: Dword,
    /// `Player.strength`.
    pub strength: Dword,
    /// The record's word after `strength`: not taken.
    pub _unused_4c: Dword,
    /// The twelve bindings' controller words (`input::set_up_device`, by
    /// `rc2k_core::input::devices::DEVICE_BINDINGS`): `Player.bindings`' eleven words go to all but
    /// `UNSET_BINDING` (`race::single_rally`, `net::take_player_settings`).
    pub bindings: [Dword; 12],
    /// `Player.keys`: the 12 bytes of the controls.
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
#[cfg_attr(test, derive(rc2k_derive::Cells))]
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
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct RaceResult {
    /// The car's time.
    pub time: PlainDword,
    /// One more word.
    pub word: PlainDword,
    /// The seconds, a float.
    pub secs: PlainFloat,
    /// The result: -1 none, -2 left.
    pub result: PlainDword,
    /// The 0x40 bytes of data (the best line's text), as sixteen little-endian words: the car's
    /// times at the splits too.
    pub data: [PlainDword; 16],
    /// The car's eight impact marks, from and back to `Player::impact_marks`.
    pub impact_marks: [PlainDword; 8],
}

impl RaceResult {
    /// The data's 0x40 bytes.
    pub fn data_bytes(&self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        for (chunk, word) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(&self.data) {
            *chunk = word.get().to_le_bytes();
        }
        bytes
    }

    /// The data's 0x40 bytes put in.
    pub fn set_data_bytes(&self, bytes: &[u8; 64]) {
        for (word, chunk) in self.data.iter().zip(bytes.as_chunks::<4>().0) {
            word.set(u32::from_le_bytes(*chunk));
        }
    }

    /// The car's time at split `split` kept in the record from its data on: sixteen words in
    /// `data`, then on into the impact marks; a split whose word is past the record is refused
    /// and nothing is written.
    pub fn set_split_time(&self, split: usize, time: u32) -> Result<(), crate::errors::BadInput> {
        let data_words = self.data.len();
        if split < data_words {
            self.data[split].set(time);
            return Ok(());
        }
        self.impact_marks
            .get(split - data_words)
            .ok_or(crate::errors::BadInput(
                "a split's time past the result's record",
            ))?
            .set(time);
        Ok(())
    }
}

/// The first twenty words of the race's settings: the mode, the stage, the direction and the
/// options, which only the game's thread reaches.
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct SettingsHead {
    /// 1 in the A8 game, the new simulation unlocked (`menu::apply_cheats`).
    pub a8_game: PlainDword,
    /// A copy of the standings summary (`ChampView.summary`): 1 while the summary
    /// (`BestTables::summary`) is taken, or 0.
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
    /// The current player's car kind (Shaun's `race_car_kinds`): the record's car kind word.
    pub car_kind_current: PlainDword,
    /// Player 1's record's car kind word.
    pub car_kind_first: PlainDword,
    /// The stage's direction (the selected group's copy).
    pub direction: PlainDword,
    /// Never used; 0 at start.
    pub unused_after_direction: PlainDword,
    /// The track entry's first extra word.
    pub track_1c: PlainDword,
    /// The track entry's second extra word.
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

/// The visual options the player chose on the options pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualOptions {
    pub lens_flare: u32,
    pub reflections: u32,
    pub car_shadows: u32,
    pub car_detail: u32,
    pub headlights: u32,
}

impl SettingsHead {
    /// Takes the visual options the player chose on the options pages.
    pub fn set_visual_options(&self, options: &VisualOptions) {
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
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct RaceSettings {
    /// The mode, the stage, the direction and the options.
    pub inline: GameThread<SettingsHead>,
    /// The players' settings: [0] the current player's (the "second set"), [1] player 1's (the
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
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct RaceResults {
    /// The arcade race's stage (Shaun's `RaceVars.race_stage`).
    pub race_stage: PlainDword,
    /// The arcade race's level (Shaun's `RaceVars.race_level`).
    pub race_level: PlainDword,
    /// A figure of the race (the record's start-place side;
    /// `race::race_shell`).
    pub figure_ffc: PlainDword,
    /// A figure of the race (Shaun's `RaceTables.start_place`).
    pub figure_000: PlainDword,
    /// The current player's record's service time.
    pub service_spent_current: PlainDword,
    /// Player 1's record's service time.
    pub service_spent_first: PlainDword,
    /// The current player's record's HUD bits;
    /// goes back to it.
    pub hud_bits_current: PlainDword,
    /// Player 1's record's HUD bits.
    pub hud_bits_first: PlainDword,
    /// The current player's record's flags.
    pub flags_current: PlainDword,
    /// Player 1's record's flags.
    pub flags_first: PlainDword,
    /// The two cars' results: [0] the current car's, [1] the first car's.
    pub results: [RaceResult; 2],
    /// The arcade race's index into the stage table
    /// (`drive::finish_race`).
    pub stage_index: PlainDword,
    /// The finishing player (the car block's finishing player field).
    pub finish_player: PlainDword,
    /// The cars' kind (into `race_car_kinds`).
    pub finish_kind: PlainDword,
    /// 1 at the championship's finish / the arcade final.
    pub finished: PlainDword,
}

/// The third and fourth players' of a split screen of more than two: their settings, car kinds,
/// results and the words the first two players' sides keep in `RaceResults`. Kept apart from the
/// first two players' so that theirs stay as the game laid them out.
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct RaceMore {
    pub car_kinds: [PlainDword; 2],
    pub results: [RaceResult; 2],
    pub service_spent: [PlainDword; 2],
    pub hud_bits: [PlainDword; 2],
    pub flags: [PlainDword; 2],
}

/// The race's settings and results.
#[cfg_attr(test, derive(rc2k_derive::Cells))]
pub struct RaceState {
    /// The race's settings as the front end keeps them ("Graham's" variables).
    pub settings: RaceSettings,
    /// The race's results.
    pub results: GameThread<RaceResults>,
    /// Unreferenced bytes.
    pub _gap_75b10c: Bytes<564>,
    /// Players 3 and 4's.
    pub more: GameThread<RaceMore>,
    /// Players 3 and 4's settings, beside the first two's: the input thread reads their controls
    /// (`input::device_command`), so they are shared cells as theirs are.
    pub more_cars: [RaceCar; 2],
}

impl RaceState {
    /// Player `k`'s settings: 0 the current player's, 1 player 1's, then 3 and 4's (`k` below 4).
    pub fn car(&self, k: usize) -> &RaceCar {
        match k {
            0 | 1 => &self.settings.cars[k],
            _ => &self.more_cars[k - 2],
        }
    }

    /// Player `k`'s record's car kind word (the same order).
    pub fn car_kind(&self, k: usize) -> &PlainDword {
        match k {
            0 => &self.settings.car_kind_current,
            1 => &self.settings.car_kind_first,
            _ => &self.more.car_kinds[k - 2],
        }
    }

    /// Player `k`'s car's result.
    pub fn result(&self, k: usize) -> &RaceResult {
        match k {
            0 | 1 => &self.results.results[k],
            _ => &self.more.results[k - 2],
        }
    }

    /// Player `k`'s record's service time.
    pub fn service_spent(&self, k: usize) -> &PlainDword {
        match k {
            0 => &self.results.service_spent_current,
            1 => &self.results.service_spent_first,
            _ => &self.more.service_spent[k - 2],
        }
    }

    /// Player `k`'s HUD bits word.
    pub fn hud_bits(&self, k: usize) -> &PlainDword {
        match k {
            0 => &self.results.hud_bits_current,
            1 => &self.results.hud_bits_first,
            _ => &self.more.hud_bits[k - 2],
        }
    }

    /// Player `k`'s parts-present word.
    pub fn flags(&self, k: usize) -> &PlainDword {
        match k {
            0 => &self.results.flags_current,
            1 => &self.results.flags_first,
            _ => &self.more.flags[k - 2],
        }
    }
}

/// The keys of the first player and of the others as the game starts them.
const FIRST_KEYS: [u8; 12] = [
    0x2c, 0x2d, 0x66, 0x6b, 0x39, 0x69, 0x68, 0x26, 0x27, 0x2e, 0x19, 0x28,
];
const SECOND_KEYS: [u8; 12] = [
    0x4f, 0x50, 0x4d, 0x51, 0x52, 0x4e, 0x53, 0x4a, 0x48, 0x49, 0x4c, 0x4b,
];

/// A race car's settings as the game starts them: the player's race number and the keys.
const fn starting_car(race_number: u32, keys: &[u8; 12]) -> RaceCar {
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
            race_number: PlainDword::new(race_number),
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
            keys: Bytes::new(sparse::<12>(&[(0x0, keys)])),
        },
    }
}

/// A car's result as the game starts it.
const fn starting_result() -> RaceResult {
    RaceResult {
        time: PlainDword::new(0),
        word: PlainDword::new(0),
        secs: PlainFloat::new(0.0),
        result: PlainDword::new(0),
        data: [const { PlainDword::new(0xffff_ffff) }; 16],
        impact_marks: [const { PlainDword::new(0) }; 8],
    }
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
                    unused_after_direction: PlainDword::new(0),
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
                    starting_car(0x61, &FIRST_KEYS),
                    starting_car(0x60, &SECOND_KEYS),
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
                results: [const { starting_result() }; 2],
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
            more: GameThread::new(RaceMore {
                car_kinds: [const { PlainDword::new(0) }; 2],
                results: [const { starting_result() }; 2],
                service_spent: [const { PlainDword::new(0) }; 2],
                hud_bits: [const { PlainDword::new(0xffffffff) }; 2],
                flags: [const { PlainDword::new(0) }; 2],
            }),
            more_cars: [
                starting_car(0x5F, &SECOND_KEYS),
                starting_car(0x5E, &SECOND_KEYS),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Players 0 and 1 are the game's own two sides; 2 and 3 are the ones kept apart, each with a
    /// side of its own.
    #[test]
    fn four_players_have_four_sides() {
        let state = RaceState::new();
        assert!(std::ptr::eq(
            state.car(0),
            std::ptr::from_ref(&state.settings.cars[0])
        ));
        assert!(std::ptr::eq(
            state.car(1),
            std::ptr::from_ref(&state.settings.cars[1])
        ));
        assert!(std::ptr::eq(
            state.result(1),
            std::ptr::from_ref(&state.results.results[1])
        ));
        let sides: Vec<*const RaceCar> = (0..4).map(|k| std::ptr::from_ref(state.car(k))).collect();
        for (a, first) in sides.iter().enumerate() {
            for second in &sides[a + 1..] {
                assert!(!std::ptr::eq(*first, *second));
            }
        }
        let kinds: Vec<*const PlainDword> = (0..4)
            .map(|k| std::ptr::from_ref(state.car_kind(k)))
            .collect();
        assert!(
            kinds
                .iter()
                .enumerate()
                .all(|(a, x)| kinds[..a].iter().all(|y| !std::ptr::eq(x, y)))
        );
        // The third and fourth start with their own race numbers.
        assert_eq!(state.car(2).race_number.get(), 0x5F);
        assert_eq!(state.car(3).race_number.get(), 0x5E);
        assert_eq!(state.hud_bits(3).get(), 0xFFFF_FFFF);
    }
}
