//! The race's settings and results as the front end keeps them, out of RAL.EXE's image (IDA
//! 0x75ACCC .. 0x75B340; phase 7, typed): the mode, the stage, the direction, the two
//! players' settings (`RaceCar`) and the two cars' results (`RaceResult`). `#[repr(C)]`
//! structs of atomic cells with the image's layout, one static; `moved::MOVED` points the
//! range at it. Ends in unreferenced bytes up to the damage model.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, Dword, Float};

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

/// One player's settings as the race variables keep them (IDA 0x75AD1C and 0x75AE88,
/// 0x16C bytes each): copies of fields of the player's record (`Player`), at the record's
/// order less the words the race does not take. Field docs name the record's field and
/// offset.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceCar {
    /// `Player.tyre` (+0x10C).
    pub tyre: Dword,
    /// `Player.brake_balance` (+0x110).
    pub brake_balance: Dword,
    /// `Player.ride_height` (+0x114).
    pub ride_height: Dword,
    /// `Player.suspension_stiffness` (+0x118).
    pub suspension_stiffness: Dword,
    /// `Player.steering_sensitivity` (+0x11C).
    pub steering_sensitivity: Dword,
    /// `Player.gear_ratio` (+0x120).
    pub gear_ratio: Dword,
    /// `Player.damage` words +0xD4 .. +0x100 and +0x108 (+0x104 goes into
    /// `wear[9]`, whose own +0x1A0 follows it).
    pub damage: [Dword; 13],
    /// `Player.parts_wear` and `tyre_wear` (+0x17C .. +0x208, one word each).
    pub wear: [Dword; 36],
    /// `Player.map_area` (+0x158).
    pub map_area: Dword,
    /// `Player.rear_mirror` (+0x15C).
    pub rear_mirror: Dword,
    /// `Player.rev_counter` (+0x160).
    pub rev_counter: Dword,
    /// `Player.metric_units` (+0x178).
    pub metric_units: Dword,
    /// `Player.message_display` (+0x168).
    pub message_display: Dword,
    /// `Player.car_info` (+0x164).
    pub car_info: Dword,
    /// `Player.progress_bar` (+0x16C).
    pub progress_bar: Dword,
    /// `Player.driver_info` (+0x170).
    pub driver_info: Dword,
    /// `Player.pace_notes` (+0x174).
    pub pace_notes: Dword,
    /// `Player.race_number` (+0x28).
    pub race_number: Dword,
    /// `Player.codriver_sex` (+0x154).
    pub codriver_sex: Dword,
    /// `Player.view` (+0xC).
    pub view: Dword,
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

impl RaceCar {
    /// The binding words the player's record's eleven go to, in order: all but
    /// `UNSET_BINDING`.
    pub fn recorded_bindings(&self) -> impl Iterator<Item = &Dword> {
        let (head, tail) = self.bindings.split_at(UNSET_BINDING);
        head.iter().chain(&tail[1..])
    }
}

/// A car's result as the race leaves it (IDA 0x75B01C and 0x75B08C, 0x70 bytes each).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceResult {
    /// The car's time (`dword_75B01C` / `dword_75B08C`).
    pub time: Dword,
    /// One more word (`dword_75B020` / `dword_75B090`).
    pub word: Dword,
    /// The seconds, a float (`flt_75B024` / `flt_75B094`).
    pub secs: Float,
    /// The result (`g_raceResult` / `g_raceResult2`): -1 none, -2 left.
    pub result: Dword,
    /// The 0x40 bytes of data (`unk_75B02C` / `unk_75B09C`; the best line's text).
    pub data: Bytes<64>,
    /// The car's eight impact marks, from and back to `Player::impact_marks`.
    pub impact_marks: [Dword; 8],
}

/// The race's settings as the front end keeps them (IDA 0x75ACCC .. 0x75AFF4; "Graham's"
/// variables, which `race::copy_settings_in` copies to the race's ("Shaun's")): the mode,
/// the stage, the direction, the two players' settings.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceSettings {
    /// `g_newSimulationUnlocked`: 1 in the A8 game (`menu::apply_cheats`).
    pub a8_game: Dword,
    /// `dword_75ACD0`: a copy of the standings summary (`dword_7556F4`, the game held its
    /// address): 1 while the summary (`saves::best_times::SUMMARY`) is taken, or 0.
    pub rivals: Dword,
    /// `dword_75ACD4`: the stage chosen.
    pub stage: Dword,
    /// `dword_75ACD8`: the run's kind argument.
    pub run_kind: Dword,
    /// `dword_75ACDC`: 1 when the race is not an event's and has more than one player.
    pub no_replay: Dword,
    /// `dword_75ACE0`: 1, or 2 in the events' one-car modes.
    pub race_mode: Dword,
    /// `dword_75ACE4`: a copy of `dword_7554C8`, put back after the race.
    pub kept: Dword,
    /// `dword_75ACE8` (Shaun's `g_raceCarKinds`): the current player's record word at +8.
    pub car_kind_current: Dword,
    /// `dword_75ACEC`: player 1's record word at +8.
    pub car_kind_first: Dword,
    /// `dword_75ACF0`: the stage's direction (the selected group's copy).
    pub direction: Dword,
    /// `unk_75ACF4`: no IDA xref and no Rust use; 0 at start.
    pub unk_75acf4: Dword,
    /// `dword_75ACF8`: the track entry's +0x1C.
    pub track_1c: Dword,
    /// `dword_75ACFC`: the track entry's +0x20.
    pub track_20: Dword,
    /// `dword_75AD00`: the lens flare option, copied from `dword_755650`.
    pub lens_flare: Dword,
    /// `dword_75AD04`: read only in dead code outside any function (0x470060 .. 0x4700A1, beside the options toggles); 100 (0x64) at start.
    pub unused_percent_a: Dword,
    /// `dword_75AD08`: read only in dead code outside any function (0x4700C0 ..); 100 (0x64) at start.
    pub unused_percent_b: Dword,
    /// `dword_75AD0C`: the reflections option (graphics page), copied from `dword_75565C`.
    pub reflections: Dword,
    /// `dword_75AD10`: the car shadows option, copied from `dword_755660`.
    pub car_shadows: Dword,
    /// `dword_75AD14`: the car detail option, copied from `dword_755664`.
    pub car_detail: Dword,
    /// `dword_75AD18`: the headlights option (all cars' lamps), copied from `dword_755668`.
    pub headlights: Dword,
    /// The two players' settings: [0] the current player's (the "second set", 0x75AD1C),
    /// [1] player 1's (the "first set", 0x75AE88).
    pub cars: [RaceCar; 2],
}

/// The race's results (IDA 0x75AFF4 .. 0x75B10C).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceResults {
    /// `dword_75AFF4`: the arcade race's stage (Shaun's `dword_6238D0`).
    pub race_stage: Dword,
    /// `dword_75AFF8`: the arcade race's level (Shaun's `dword_6238CC`).
    pub race_level: Dword,
    /// `dword_75AFFC`: a figure of the race (the record's +0x654 side; `race::race_shell`).
    pub figure_ffc: Dword,
    /// `dword_75B000`: a figure of the race (Shaun's `dword_6051BC`).
    pub figure_000: Dword,
    /// `dword_75B004`: the current player's record +0x288.
    pub service_spent_current: Dword,
    /// `dword_75B008`: player 1's record +0x288.
    pub service_spent_first: Dword,
    /// `dword_75B00C`: the current player's record +0x284 (the HUD's bits); goes back to it.
    pub hud_bits_current: Dword,
    /// `dword_75B010`: player 1's record +0x284.
    pub hud_bits_first: Dword,
    /// `dword_75B014`: the current player's record +0x278.
    pub flags_current: Dword,
    /// `dword_75B018`: player 1's record +0x278.
    pub flags_first: Dword,
    /// The two cars' results: [0] the current car's (0x75B01C), [1] the first car's (0x75B08C).
    pub results: [RaceResult; 2],
    /// `dword_75B0FC`: the arcade race's index into the stage table (`drive::finish_race`).
    pub stage_index: Dword,
    /// `dword_75B100`: the finishing player (+0xAA8 of the car block).
    pub finish_player: Dword,
    /// `dword_75B104`: the cars' kind (into `g_raceCarKinds`).
    pub finish_kind: Dword,
    /// `dword_75B108`: 1 at the championship's finish / the arcade final.
    pub finished: Dword,
}

/// The race's settings and results.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct RaceState {
    /// The race's settings as the front end keeps them (IDA 0x75ACCC .. 0x75AFF4; "Graham's"
    pub settings: RaceSettings,
    /// The race's results (IDA 0x75AFF4 .. 0x75B10C).
    pub results: RaceResults,
    /// Unreferenced bytes (IDA 0X75B10C .. 0X75B340).
    pub _gap_75b10c: Bytes<564>,
}

impl RaceState {
    /// First values the image's.
    pub const fn new() -> Self {
        RaceState {
            settings: RaceSettings {
                a8_game: Dword::new(0),
                rivals: Dword::new(0),
                stage: Dword::new(0),
                run_kind: Dword::new(0),
                no_replay: Dword::new(0),
                race_mode: Dword::new(1),
                kept: Dword::new(0),
                car_kind_current: Dword::new(0),
                car_kind_first: Dword::new(0),
                direction: Dword::new(0),
                unk_75acf4: Dword::new(0),
                track_1c: Dword::new(0x2d),
                track_20: Dword::new(0x19),
                lens_flare: Dword::new(0),
                unused_percent_a: Dword::new(0x64),
                unused_percent_b: Dword::new(0x64),
                reflections: Dword::new(0),
                car_shadows: Dword::new(0),
                car_detail: Dword::new(0),
                headlights: Dword::new(0),
                cars: [
                    RaceCar {
                        tyre: Dword::new(0),
                        brake_balance: Dword::new(0x32),
                        ride_height: Dword::new(0x32),
                        suspension_stiffness: Dword::new(0x32),
                        steering_sensitivity: Dword::new(0x32),
                        gear_ratio: Dword::new(2),
                        damage: [const { Dword::new(0) }; 13],
                        wear: [const { Dword::new(0) }; 36],
                        map_area: Dword::new(0),
                        rear_mirror: Dword::new(0),
                        rev_counter: Dword::new(0),
                        metric_units: Dword::new(0),
                        message_display: Dword::new(0),
                        car_info: Dword::new(1),
                        progress_bar: Dword::new(1),
                        driver_info: Dword::new(1),
                        pace_notes: Dword::new(1),
                        race_number: Dword::new(0x61),
                        codriver_sex: Dword::new(0),
                        view: Dword::new(1),
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
                                0x2c, 0x2d, 0x66, 0x6b, 0x39, 0x69, 0x68, 0x26, 0x27, 0x2e, 0x19,
                                0x28,
                            ],
                        )])),
                    },
                    RaceCar {
                        tyre: Dword::new(0),
                        brake_balance: Dword::new(0x32),
                        ride_height: Dword::new(0x32),
                        suspension_stiffness: Dword::new(0x32),
                        steering_sensitivity: Dword::new(0x32),
                        gear_ratio: Dword::new(2),
                        damage: [const { Dword::new(0) }; 13],
                        wear: [const { Dword::new(0) }; 36],
                        map_area: Dword::new(0),
                        rear_mirror: Dword::new(0),
                        rev_counter: Dword::new(0),
                        metric_units: Dword::new(0),
                        message_display: Dword::new(0),
                        car_info: Dword::new(1),
                        progress_bar: Dword::new(1),
                        driver_info: Dword::new(1),
                        pace_notes: Dword::new(1),
                        race_number: Dword::new(0x60),
                        codriver_sex: Dword::new(0),
                        view: Dword::new(1),
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
                                0x4f, 0x50, 0x4d, 0x51, 0x52, 0x4e, 0x53, 0x4a, 0x48, 0x49, 0x4c,
                                0x4b,
                            ],
                        )])),
                    },
                ],
            },
            results: RaceResults {
                race_stage: Dword::new(0),
                race_level: Dword::new(0),
                figure_ffc: Dword::new(0),
                figure_000: Dword::new(0),
                service_spent_current: Dword::new(0),
                service_spent_first: Dword::new(0),
                hud_bits_current: Dword::new(0xffffffff),
                hud_bits_first: Dword::new(0xffffffff),
                flags_current: Dword::new(0),
                flags_first: Dword::new(0),
                results: [
                    RaceResult {
                        time: Dword::new(0),
                        word: Dword::new(0),
                        secs: Float::from_bits(0),
                        result: Dword::new(0),
                        data: Bytes::new([0xff; 64]),
                        impact_marks: [const { Dword::new(0) }; 8],
                    },
                    RaceResult {
                        time: Dword::new(0),
                        word: Dword::new(0),
                        secs: Float::from_bits(0),
                        result: Dword::new(0),
                        data: Bytes::new([0xff; 64]),
                        impact_marks: [const { Dword::new(0) }; 8],
                    },
                ],
                stage_index: Dword::new(0),
                finish_player: Dword::new(1),
                finish_kind: Dword::new(0),
                finished: Dword::new(0),
            },
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

/// IDA 0X75ACCC, the first byte of the piece.
pub const START: usize = 0x75ACCC;
/// IDA 0X75B340, the end of the piece.
pub const END: usize = 0x75B340;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn race_state_starts_as_the_games() {
        let _g = crate::store::test_lock();
        let size = size_of::<RaceState>();
        assert!(END - START <= size && size <= (END - START).next_multiple_of(4));
        let mine = unsafe {
            std::slice::from_raw_parts((&raw const race().shared.state).cast::<u8>(), END - START)
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(RaceState, settings), 0x75ACCC);
        at(offset_of!(RaceState, results), 0x75AFF4);
        at(
            offset_of!(RaceState, settings) + offset_of!(RaceSettings, cars),
            0x75AD1C,
        );
        at(
            offset_of!(RaceState, settings) + offset_of!(RaceSettings, cars) + size_of::<RaceCar>(),
            0x75AE88,
        );
        at(
            offset_of!(RaceState, results) + offset_of!(RaceResults, results),
            0x75B01C,
        );
        at(
            offset_of!(RaceState, results)
                + offset_of!(RaceResults, results)
                + size_of::<RaceResult>(),
            0x75B08C,
        );
        // The record's order inside a `RaceCar` (offsets from 0x75AD1C less the first word).
        assert_eq!(offset_of!(RaceCar, controller), 0x140 - 0x34);
        assert_eq!(offset_of!(RaceCar, gearing), 0x144 - 0x34);
        assert_eq!(offset_of!(RaceCar, sensitivity_level), 0x148 - 0x34);
        assert_eq!(offset_of!(RaceCar, axis_codes), 0x14C - 0x34);
        assert_eq!(offset_of!(RaceCar, controller_mode), 0x150 - 0x34);
        assert_eq!(offset_of!(RaceCar, end_zone_pct), 0x154 - 0x34);
        assert_eq!(offset_of!(RaceCar, centre_dead_zone_pct), 0x158 - 0x34);
        assert_eq!(offset_of!(RaceCar, strength), 0x15C - 0x34);
        assert_eq!(offset_of!(RaceCar, bindings), 0x164 - 0x34);
        assert_eq!(offset_of!(RaceCar, bindings) + 4 * 10, 0x18C - 0x34);
        assert_eq!(offset_of!(RaceCar, bindings) + 4 * 11, 0x190 - 0x34);
        assert_eq!(offset_of!(RaceCar, keys), 0x194 - 0x34);
        assert_eq!(offset_of!(RaceResult, impact_marks), 0x50);
    }
}
