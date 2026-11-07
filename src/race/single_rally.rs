//! `race::single_rally_race` (0x4843EC, `single_rally`'s race, IDA 2026-10-17): the chosen
//! track looked up (`g_trakPakHeader`), the settings and the players' records
//! (`g_players`) copied into the front end's race variables, the race run
//! (`race::race_from_menu`, with the sound objects and the screen modes round it), and the
//! results copied back into the players' records.
//!
//! Plain Rust: copies between a record and the game's variables, the calls (`Call`, done
//! by `step`), the track search as a loop, two divisions by 10 and one float-to-integer
//! conversion. It was checked against an interpreter of the original's instructions on
//! synthetic memory and players, with calls that record themselves (`table` and the test at the
//! end of the file, `cfg(test)` only).

use crate::errors::{BadInput, FatalError, Refused};
use crate::race::marks_settings_store::MARKS_SETTINGS;
use crate::race::owner::race;
use crate::race::state_store::{RaceCar, RaceResult};
use crate::session::{Bytes, Dword};
use crate::system::owner::system;
use rc2k_core::menu::player::{Player, TAKES_PART};

/// A player's or a track's record: a game address.
type Record = crate::menu::standings::PlayerCell;

// g_trakPakHeader (0x734BC6, `records::stages_store`): +0x34 the entries, 0x40 apart. An
// entry (at `edi`) holds its track number at +0x34; the list ends at a dword -1 where the
// next entry would start.
/// g_players (0x75647C): the players' records, 4 bytes each.
/// dword_7556F4: whether the standings summary is taken (1, `saves::best_times::SUMMARY`),
/// copied to `RACE_INFO`.
const SUMMARY: &Dword = &crate::menu::owner::MENUS.play.champ_view.summary;
/// dword_7558DC, copied to dword_5C2FD0 after the loading picture.
const AFTER_LOADING: &Dword = &crate::menu::owner::MENUS.screens.event.sparks_mode;
/// dword_7554C8: kept over the race (`RACE_KEPT`).
const KEPT: &Dword = &crate::menu::owner::MENUS.screens.flow.kept;
/// dword_755984: the number of players.
const PLAYER_COUNT: &Dword = &crate::menu::owner::MENUS.screens.event.player_count;
/// dword_755988: the event mode (5, 4, 0xF, 0xA: the events' one-car races).
const EVENT_MODE: &Dword = &crate::menu::owner::MENUS.screens.event.event_kind;
/// dword_75598C: the current player's number.
const CURRENT: &Dword = &crate::menu::owner::MENUS.screens.event.current_player;
/// dword_7587E8: the run's start argument (1: no loading picture after).
const SWITCH: &Dword = &crate::menu::owner::MENUS.play.arcade.scene_let_go;
/// dword_75ACD0: a copy of `SUMMARY` (the summary's block id, as the race reads it:
/// `race::copy_league_table`, `physics::fill_rival_table`).
const RACE_INFO: &Dword = &crate::race::owner::RACE.shared.state.settings.rivals;
/// dword_75ACD4: the stage chosen.
const STAGE: &Dword = &crate::race::owner::RACE.shared.state.settings.stage;
/// dword_75ACD8: the run's kind argument.
const RACE_KIND: &Dword = &crate::race::owner::RACE.shared.state.settings.run_kind;
/// dword_75ACDC: 1 when the race is not an event's and has more than one player.
const NO_REPLAY: &Dword = &crate::race::owner::RACE.shared.state.settings.no_replay;
/// dword_75ACE0: 1, or 2 in the events' one-car modes.
const RACE_MODE: &Dword = &crate::race::owner::RACE.shared.state.settings.race_mode;
/// dword_75ACE4: a copy of `KEPT`, put back after the race.
const RACE_KEPT: &Dword = &crate::race::owner::RACE.shared.state.settings.kept;
/// dword_75B028 / dword_75B098: the current / the first car's result (-1 none, -2 left).
const RESULT: &Dword = &crate::race::owner::RACE.shared.state.results.results[0].result;
const RESULT_2: &Dword = &crate::race::owner::RACE.shared.state.results.results[1].result;
/// One player's side of the race variables: the car's settings and the results it
/// leaves. `FIRST_SIDE` is player 1's, `CURRENT_SIDE` the current player's.
struct Side {
    car: &'static RaceCar,
    /// dword_75ACE8 / dword_75ACEC: the record's `car_number` (+8).
    car_kind: &'static Dword,
    /// The record's `service_spent`.
    service_spent: &'static Dword,
    /// The record's `state_flags`.
    flags: &'static Dword,
    /// dword_75B00C (the current's): the record's `hud_bits`.
    hud_bits: &'static Dword,
    /// The car's result, and the 0x20 bytes of its `field_24c`.
    result: &'static RaceResult,
}
const FIRST_SIDE: Side = Side {
    car: &crate::race::owner::RACE.shared.state.settings.cars[1],
    car_kind: &crate::race::owner::RACE
        .shared
        .state
        .settings
        .car_kind_first,
    service_spent: &crate::race::owner::RACE
        .shared
        .state
        .results
        .service_spent_first,
    flags: &crate::race::owner::RACE.shared.state.results.flags_first,
    hud_bits: &crate::race::owner::RACE.shared.state.results.hud_bits_first,
    result: &crate::race::owner::RACE.shared.state.results.results[1],
};
const CURRENT_SIDE: Side = Side {
    car: &crate::race::owner::RACE.shared.state.settings.cars[0],
    car_kind: &crate::race::owner::RACE
        .shared
        .state
        .settings
        .car_kind_current,
    service_spent: &crate::race::owner::RACE
        .shared
        .state
        .results
        .service_spent_current,
    flags: &crate::race::owner::RACE.shared.state.results.flags_current,
    hud_bits: &crate::race::owner::RACE
        .shared
        .state
        .results
        .hud_bits_current,
    result: &crate::race::owner::RACE.shared.state.results.results[0],
};
/// dword_75AE24: the second set's word that mode 2 takes from the record's `second_view`.
const MODE_WORD: &Dword = &crate::race::owner::RACE.shared.state.settings.cars[0].view;

#[cfg(test)]
const BLOCK_FIRST: &[Dword; 8] =
    &crate::race::owner::RACE.shared.state.results.results[1].field_24c;
#[cfg(test)]
const BLOCK_CURRENT: &[Dword; 8] =
    &crate::race::owner::RACE.shared.state.results.results[0].field_24c;
#[cfg(test)]
const TIMES_FIRST: [&Dword; 4] = [
    &crate::race::owner::RACE.shared.state.settings.cars[1].wear[10],
    &crate::race::owner::RACE.shared.state.settings.cars[1].wear[11],
    &crate::race::owner::RACE.shared.state.settings.cars[1].wear[12],
    &crate::race::owner::RACE.shared.state.settings.cars[1].wear[13],
];
#[cfg(test)]
const EXTRA_FIRST: &Dword = &crate::race::owner::RACE.shared.state.settings.cars[1].wear[14];
#[cfg(test)]
const TIMES_CURRENT: [&Dword; 4] = [
    &crate::race::owner::RACE.shared.state.settings.cars[0].wear[10],
    &crate::race::owner::RACE.shared.state.settings.cars[0].wear[11],
    &crate::race::owner::RACE.shared.state.settings.cars[0].wear[12],
    &crate::race::owner::RACE.shared.state.settings.cars[0].wear[13],
];
#[cfg(test)]
const EXTRA_CURRENT: &Dword = &crate::race::owner::RACE.shared.state.settings.cars[0].wear[14];
#[cfg(test)]
const OUTCOME_FIRST: &RaceResult = &crate::race::owner::RACE.shared.state.results.results[1];
#[cfg(test)]
const OUTCOME_CURRENT: &RaceResult = &crate::race::owner::RACE.shared.state.results.results[0];

/// The fatal error of a track number not in the list.
const UNKNOWN_TRACK: FatalError = FatalError {
    code: 0x8073_9281,
    message: "Unknown Shaun TrakNum",
};

/// Five settings words copied to the race variables (from, to).
const SETTINGS_TO_RACE: [(&Dword, &Dword); 5] = [
    (
        &crate::menu::owner::MENUS.screens.option_work.setting_0,
        &crate::race::owner::RACE.shared.state.settings.setting_0,
    ),
    (
        &crate::menu::owner::MENUS.screens.option_work.setting_1,
        &crate::race::owner::RACE.shared.state.settings.setting_1,
    ),
    (
        &crate::menu::owner::MENUS.screens.option_work.setting_2,
        &crate::race::owner::RACE.shared.state.settings.setting_2,
    ),
    (
        &crate::menu::owner::MENUS.screens.option_work.setting_3,
        &crate::race::owner::RACE.shared.state.settings.setting_3,
    ),
    (
        &crate::menu::owner::MENUS.screens.option_work.setting_4,
        &crate::race::owner::RACE.shared.state.settings.setting_4,
    ),
];

#[cfg(test)]
fn set_field(rec: Record, off: u32, v: u32) -> Result<(), BadInput> {
    rec.update(|p| p.store(&[(off as usize, v)]));
    Ok(())
}

/// `g_players`[n].
pub fn player(n: u32) -> Result<Record, BadInput> {
    crate::menu::player::player(n).ok_or(BadInput("a player past the four"))
}

/// A record's controller settings into a car's: the controller, gearing, sensitivity, axes,
/// mode, dead zones, strength, and the 0x2C bytes of `bindings` as eleven words (the last
/// two are `controls_74` and `controls_78`).
fn take_controls(p: &Player, car: &RaceCar) {
    car.controller.set(p.controller);
    car.gearing.set(p.gearing);
    car.sensitivity_level.set(p.sensitivity_level);
    car.axis_codes.set(u32::from_le_bytes(p.axis_codes));
    car.controller_mode.set(p.controller_mode);
    car.dead_zone_1.set(p.dead_zone_1);
    car.dead_zone_2.set(p.dead_zone_2);
    car.strength.set(p.strength);
    let cells = car
        .controls_50
        .iter()
        .chain([&car.controls_74, &car.controls_78]);
    let (words, _) = p.bindings.as_chunks::<4>();
    for (cell, &word) in cells.zip(words) {
        cell.set(u32::from_le_bytes(word));
    }
}

/// The 12 key bytes of a record into a car's `keys`.
fn take_keys(p: &Player, keys: &Bytes<12>) {
    for (k, &b) in p.keys.iter().enumerate() {
        keys.set(k, b);
    }
}

/// A record into a side's variables: the car, set-up, HUD choices, state, and (with
/// `condition`) the damage and wear. The original also writes the car's `wear[9]` from the
/// record's `damage[12]`, and the first side's `view` from `view` and then `second_view`
/// (the caller writes the second); each is replaced by the write after it, so only the
/// last is made.
fn take_player(p: &Player, side: &Side, condition: bool) {
    let car = side.car;
    side.car_kind.set(p.car_number);
    side.service_spent.set(p.service_spent);
    side.flags.set(p.state_flags);
    side.hud_bits.set(p.hud_bits as u32);
    car.tyre.set(p.tyre);
    car.brake_balance.set(p.brake_balance);
    car.ride_height.set(p.ride_height);
    car.suspension_stiffness.set(p.suspension_stiffness);
    car.steering_sensitivity.set(p.steering_sensitivity);
    car.gear_ratio.set(p.gear_ratio);
    car.map_area.set(p.map_area);
    car.rear_mirror.set(p.rear_mirror);
    car.rev_counter.set(p.rev_counter);
    car.message_display.set(p.message_display);
    car.car_info.set(p.car_info);
    car.driver_info.set(p.driver_info);
    car.progress_bar.set(p.progress_bar);
    car.pace_notes.set(p.pace_notes);
    car.metric_units.set(p.metric_units);
    car.race_number.set(p.race_number);
    car.codriver_sex.set(p.codriver_sex);
    car.view.set(p.view);
    take_controls(p, car);
    if condition {
        // Record damage words 0 .. 12 and 13 (the word 12 has no variable).
        for (cell, &word) in car.damage.iter().zip(&p.damage[..12]) {
            cell.set(word);
        }
        car.damage[12].set(p.damage[13]);
        let wear = p.parts_wear.iter().chain(&p.tyre_wear);
        for (cell, &word) in car.wear.iter().zip(wear) {
            cell.set(word);
        }
    }
}

/// The 0x20 bytes (`field_24c`) of a record into a result.
fn take_block(p: &Player, result: &RaceResult) {
    for (cell, &word) in result.field_24c.iter().zip(&p.field_24c) {
        cell.set(word);
    }
}

/// The total a record keeps in `damage[1]` (+0xD8): a tenth of the sum of the car's four
/// times, and a tenth of six times the extra word. The original divides with `div ebp` (edx
/// 0, ebp 10), after a test of the dividend for 0 that skips the division, so its fatal
/// error 0x80701010 is out of reach.
fn total(car: &RaceCar) -> u32 {
    let sum = car.wear[10]
        .get()
        .wrapping_add(car.wear[11].get())
        .wrapping_add(car.wear[12].get())
        .wrapping_add(car.wear[13].get());
    (sum / 10).wrapping_add(car.wear[14].get().wrapping_mul(6) / 10)
}

/// A side's variables back into its record. As the original's lists do: the damage words are
/// written from the car's damage first, then `damage[1]` is replaced by the total and
/// `damage[5]`, `[9]`, `[11]` and `[13]` by the wear words 22, 27, 30 and 35 (the original's
/// slip, kept); `damage[12]` takes `wear[9]`.
fn give_player(p: &mut Player, side: &Side) {
    let car = side.car;
    p.hud_bits = side.hud_bits.get() as i32;
    p.state_flags = side.flags.get();
    for (word, cell) in p.damage.iter_mut().zip(&car.damage[..12]) {
        *word = cell.get();
    }
    p.damage[12] = car.wear[9].get();
    p.damage[13] = car.damage[12].get();
    let wear = p.parts_wear.iter_mut().chain(&mut p.tyre_wear);
    for (word, cell) in wear.zip(&car.wear) {
        *word = cell.get();
    }
    for (word, cell) in p.field_24c.iter_mut().zip(&side.result.field_24c) {
        *word = cell.get();
    }
    p.damage[1] = total(car);
    p.damage[9] = car.wear[27].get();
    p.damage[13] = car.wear[35].get();
    p.damage[11] = car.wear[30].get();
    p.damage[5] = car.wear[22].get();
    p.codriver_sex = car.codriver_sex.get();
    p.view = car.view.get();
}

/// A car's outcome into its record: a result of -1 or -2 puts the time at 0x36EE80 and the
/// entry at 6 (out); any other copies the time, the result word, the seconds rounded times
/// 10 (`figure`) and the 0x40 bytes of data (`best_line`).
fn give_outcome(p: &mut Player, result: &RaceResult) {
    let r = result.result.get();
    if r == -2i32 as u32 || r == -1i32 as u32 {
        p.time = 0x36EE80;
        p.entry = 6;
    } else {
        p.time = result.time.get();
        p.result_word = result.word.get();
        let secs = crate::math::x87::to_int(result.secs.bits()) as u32;
        p.figure = secs.wrapping_mul(10);
        p.best_line = result.data.read();
    }
}

/// `race::single_rally_race` (0x4843EC): `start` (eax) the run's start (1: the loading
/// picture is not shown again after), `kind` (ebx) the kind (3: the rally's times copied
/// first). Refuses, before anything, a missing player record (`g_players`[1] or
/// [`dword_75598C`]; the original reads and writes through it).
pub fn single_rally_race(start: u32, kind: u32) -> Result<(), Refused> {
    guarded(start, kind, &mut step)
}

/// The refusal, then the race, its calls done by `step`.
fn guarded(
    start: u32,
    kind: u32,
    step: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    player(1)?;
    player(CURRENT.get())?;
    single_rally(start, kind, step)
}

/// The last calls of the original (from 0x897 of the table): the kept word back, the tune.
fn finish(step: &mut dyn FnMut(Call) -> Result<(), Refused>) -> Result<(), Refused> {
    KEPT.set(RACE_KEPT.get());
    step(Call::PlayTune3)
}

/// The debug UI's test races: the players' records into the race's settings as a race does
/// (the HUD, the keys, the set-up, ...), without their damage and wear.
#[cfg(debug_ui)]
pub fn player_settings_in() -> Result<(), Refused> {
    // As `single_rally`'s start: the last race's outcome, else the car starts out of it.
    RESULT.set(0);
    RESULT_2.set(0);
    let first = player(1)?.get();
    take_controls(&first, FIRST_SIDE.car);
    take_keys(&first, &FIRST_SIDE.car.keys);
    let current = player(CURRENT.get())?.get();
    take_player(&current, &CURRENT_SIDE, false);
    take_keys(&current, &CURRENT_SIDE.car.keys);
    Ok(())
}

/// The original's body; `step(call)` does the call `call`.
fn single_rally(
    start: u32,
    kind: u32,
    step: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let shared = &race().shared;
    let system = system();
    // Closed on a menu whose item leads here (a window's close, MENU + START): the screen is
    // left as if chosen, with no stage yet. Not the original's, whose close stopped nothing.
    if system.session.flags.quitting.get() == 1 {
        crate::standalone::leave_on_close();
    }
    if kind == 3 {
        step(Call::CopyRallyTimes)?;
    }
    RACE_INFO.set(SUMMARY.get());
    SWITCH.set(start);
    RACE_KIND.set(kind);
    RESULT.set(0);
    RESULT_2.set(0);
    RACE_KEPT.set(0);
    RACE_MODE.set(1);
    step(Call::StopMusic)?;
    step(Call::SetRaceScreenMode)?;
    step(Call::ShowLoadingPicture)?;
    MARKS_SETTINGS.after_loading_copy.set(AFTER_LOADING.get());

    // The stage's entry in the track list: the tracks, then the pack's closing -1.
    let stage = STAGE.get();
    let pack = &system.records.stage_tables.trak_pak;
    let mut at = 0;
    let entry = loop {
        let entry = pack
            .tracks
            .get(at)
            .ok_or(BadInput("a track past the pack"))?;
        if entry.stage.get() == stage {
            break entry;
        }
        at += 1;
        let next_first = match pack.tracks.get(at) {
            Some(next) => next.name.get(),
            None => pack.end.get(),
        };
        if next_first == u32::MAX {
            return Err(UNKNOWN_TRACK.into());
        }
    };
    // The entry's +0x1C (temperature) and +0x20 (wind) into the race variables.
    shared.state.settings.track_1c.set(entry.temperature.get());
    shared.state.settings.track_20.set(entry.wind.get());
    for &(from, to) in &SETTINGS_TO_RACE {
        to.set(from.get());
    }
    let first = player(1)?.get();
    take_controls(&first, FIRST_SIDE.car);
    take_keys(&first, &FIRST_SIDE.car.keys);

    let one_car = matches!(EVENT_MODE.get(), 5 | 4 | 0xF | 0xA);
    if one_car {
        RACE_KEPT.set(KEPT.get());
        RACE_MODE.set(2);
        if first.entry != TAKES_PART {
            RESULT_2.set(u32::MAX);
        }
        take_player(&first, &FIRST_SIDE, true);
        take_block(&first, FIRST_SIDE.result);
        FIRST_SIDE.car.view.set(first.second_view);
        CURRENT.set(0);
    } else if player(CURRENT.get())?.get().entry != TAKES_PART {
        // The current player's record is not in use: no race.
        RESULT.set(u32::MAX);
        return finish(step);
    }

    // The current player's record (player 0 after an event's one-car race).
    let rec = player(CURRENT.get())?.get();
    if rec.entry != TAKES_PART {
        RESULT.set(u32::MAX);
    }
    take_player(&rec, &CURRENT_SIDE, true);
    take_block(&rec, CURRENT_SIDE.result);
    if RACE_MODE.get() == 2 {
        MODE_WORD.set(rec.second_view);
    }
    take_keys(&rec, &CURRENT_SIDE.car.keys);

    // The race.
    NO_REPLAY.set(0);
    if !one_car && PLAYER_COUNT.get() != 1 {
        NO_REPLAY.set(1);
    }
    step(Call::SetUpRaceSoundObjects)?;
    step(Call::SetUpRaceSounds)?;
    step(Call::RaceFromMenu)?;
    step(Call::StopMusic)?;
    step(Call::SetMenuScreenMode)?;
    if SWITCH.get() != 1 {
        step(Call::ShowLoadingPicture)?;
    }
    step(Call::ClearRaceSoundObjects)?;
    step(Call::ApplySettings)?;

    // The outcome back into the records.
    if one_car {
        player(1)?.update(|p| {
            give_player(p, &FIRST_SIDE);
            give_outcome(p, FIRST_SIDE.result);
        });
        CURRENT.set(0);
    }
    player(CURRENT.get())?.update(|p| {
        give_player(p, &CURRENT_SIDE);
        give_outcome(p, CURRENT_SIDE.result);
    });

    if SWITCH.get() != 1 {
        step(Call::SetUpEngine)?;
        step(Call::SetUpFrontEndScene)?;
        step(Call::SetUpEngine)?;
    }
    step(Call::TakeKey)?;
    finish(step)
}

/// The calls into the rest of the game that the race makes, in the order they are numbered
/// in the tests' record of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// `race::copy_rally_times`.
    CopyRallyTimes = 1,
    /// `sound::stop_music`.
    StopMusic,
    /// `race::set_race_screen_mode`.
    SetRaceScreenMode,
    /// `menu::show_loading_picture`.
    ShowLoadingPicture,
    /// `race::set_up_race_sound_objects`.
    SetUpRaceSoundObjects,
    /// `sound::set_up_race_sounds`.
    SetUpRaceSounds,
    /// `race::race_from_menu`.
    RaceFromMenu,
    /// `gfx::set_menu_screen_mode`.
    SetMenuScreenMode,
    /// `race::clear_race_sound_objects`.
    ClearRaceSoundObjects,
    /// `sound::apply_settings`.
    ApplySettings,
    /// `scene::set_up_engine`.
    SetUpEngine,
    /// `scene::set_up_front_end_scene`.
    SetUpFrontEndScene,
    /// `input::take_key`.
    TakeKey,
    /// `sound::play_tune_3`.
    PlayTune3,
}

/// Does one of the race's calls. What the original's calls leave in registers is never read.
pub fn step(call: Call) -> Result<(), Refused> {
    let race = race();
    match call {
        Call::CopyRallyTimes => crate::race::rally::copy_rally_times()?,
        Call::StopMusic => crate::sound::stop_music::stop_music(),
        Call::SetRaceScreenMode => crate::race::race_shell::set_race_screen_mode(),
        Call::ShowLoadingPicture => {
            crate::menu::show_loading_picture::show_loading_picture()?;
        }
        Call::SetUpRaceSoundObjects => crate::race::sound_tables::set_up_race_sound_objects(
            race,
            &crate::race::sound_tables::ObjectVars::game(),
        )?,
        Call::SetUpRaceSounds => crate::sound::set_up_race_sounds::set_up_race_sounds()?,
        Call::RaceFromMenu => crate::race::race_end::race_from_menu(race)?,
        Call::SetMenuScreenMode => {
            crate::platform::set_menu_mode_now();
        }
        Call::ClearRaceSoundObjects => crate::race::sound_tables::clear_race_sound_objects()?,
        Call::ApplySettings => crate::sound::apply_settings::apply_settings()?,
        Call::SetUpEngine => crate::scene::set_up_engine::set_up_engine()?,
        Call::SetUpFrontEndScene => {
            crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
        }
        Call::TakeKey => {
            crate::input::take_key::take_key();
        }
        Call::PlayTune3 => crate::sound::play_tune_3::play_tune_3(),
    }
    Ok(())
}

/// The original's instructions as a table and the small interpreter that does them (made from
/// the disassembly by script, used as the game's code until the plain version replaced it):
/// the reference the test compares the plain version with.
#[cfg(test)]
mod table {
    use crate::errors::{FatalError, Refused};
    use crate::framework::{Regs, game_addr, rva};

    /// A register of the table's.
    #[derive(Clone, Copy, Debug)]
    pub enum R {
        Eax,
        Ebx,
        Ecx,
        Edx,
        Esi,
        Edi,
        Ebp,
        /// eax's low byte.
        Al,
    }

    /// One instruction (addresses as IDA shows them; jump targets are op numbers).
    #[derive(Clone, Copy, Debug)]
    pub enum Op {
        Pusha,
        Popa,
        Push(R),
        Pop(R),
        /// `mov [m], r`.
        Store(R, usize),
        /// `mov r, [m]`.
        Load(R, usize),
        /// `mov r, [edi + n]` (al: a byte).
        FromEdi(R, u32),
        /// `mov [edi + n], r`.
        ToEdi(R, u32),
        /// `mov dword [m], value`.
        Put(usize, u32),
        /// `mov dword [edi + n], value`.
        EdiPut(u32, u32),
        /// `mov edi, [table + eax * 4]`.
        EdiFromTable(usize),
        /// `mov r, value`.
        Imm(R, u32),
        /// `lea r, m`.
        Lea(R, usize),
        /// `lea r, [edi + n]`.
        LeaEdi(R, u32),
        /// `mov a, b`.
        Move(R, R),
        Nop,
        /// `cmp dword [m], value`.
        CmpMem(usize, u32),
        /// `cmp r, value`.
        CmpReg(R, u32),
        /// `cmp dword [edi + n], value`.
        CmpEdi(u32, u32),
        /// `cmp [edi + n], eax`.
        CmpEdiEax(u32),
        Jz(usize),
        Jnz(usize),
        Jmp(usize),
        /// `jz` to a fatal stub (0 "Unknown Shaun TrakNum", 1 the division's).
        JzFatal(usize),
        Call(usize),
        /// `rep movsb`.
        RepMovsb,
        /// `add r, [m]`.
        AddMem(R, usize),
        /// `add r, value`.
        AddImm(R, u32),
        /// `add a, b`.
        AddReg(R, R),
        /// `imul r, value`.
        MulImm(R, u32),
        /// `div ebp` (edx:eax).
        DivEbp,
        /// `push eax; fld [esp]; fistp [esp]; pop ebx`: ebx the float eax rounded.
        RoundEaxToEbx,
        Return,
    }

    use Op::*;

    const FATALS: [FatalError; 2] = [
        FatalError {
            code: 0x8073_9281,
            message: "Unknown Shaun TrakNum",
        },
        FatalError {
            code: 0x8070_1010,
            message: "",
        },
    ];

    /// The calls the table makes, by place (its index + 1), as IDA shows them.
    pub const CALLS: [usize; 14] = [
        0x48C997, // race::copy_rally_times
        0x40F826, // sound::stop_music
        0x404830, // race::set_race_screen_mode
        0x47CEA8, // menu::show_loading_picture
        0x415C93, // race::set_up_race_sound_objects
        0x40F02E, // sound::set_up_race_sounds
        0x485570, // race::race_from_menu
        0x404823, // gfx::set_menu_screen_mode (platform)
        0x415C6D, // race::clear_race_sound_objects
        0x40EFF3, // sound::apply_settings
        0x442AF6, // scene::set_up_engine
        0x474E04, // scene::set_up_front_end_scene
        0x407F95, // input::take_key
        0x40F780, // sound::play_tune_3
    ];

    fn mem(a: usize) -> *mut u32 {
        crate::pointers::host(game_addr(rva(a))).cast::<u32>()
    }

    fn reg(g: &mut Regs, r: R) -> &mut u32 {
        match r {
            R::Eax | R::Al => &mut g.eax,
            R::Ebx => &mut g.ebx,
            R::Ecx => &mut g.ecx,
            R::Edx => &mut g.edx,
            R::Esi => &mut g.esi,
            R::Edi => &mut g.edi,
            R::Ebp => &mut g.ebp,
        }
    }

    /// The table from the first op, with eax and ebx the run's start and kind. The registers
    /// are the table's own scratch (a local `Regs`: the original is `pusha` .. `popa`, it
    /// leaves every register of its caller as it was).
    pub unsafe fn run(
        ops: &[Op],
        start: u32,
        kind: u32,
        step: &mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>,
    ) -> Result<(), Refused> {
        let mut g = Regs {
            eax: start,
            ebx: kind,
            ..Regs::default()
        };
        let mut stack: Vec<u32> = Vec::new();
        let mut saved: Vec<Regs> = Vec::new();
        let mut zero = false;
        let mut pc = 0;
        unsafe {
            loop {
                let op = ops[pc];
                pc += 1;
                match op {
                    Pusha => saved.push(g),
                    Popa => {
                        let s = saved.pop().expect("a popa after its pusha");
                        g = Regs {
                            eflags: g.eflags,
                            ..s
                        };
                    }
                    Push(r) => stack.push(*reg(&mut g, r)),
                    Pop(r) => *reg(&mut g, r) = stack.pop().expect("a pop after its push"),
                    Store(R::Al, a) => *mem(a).cast::<u8>() = g.eax as u8,
                    Store(r, a) => mem(a).write_unaligned(*reg(&mut g, r)),
                    Load(r, a) => *reg(&mut g, r) = mem(a).read_unaligned(),
                    FromEdi(R::Al, n) => {
                        g.eax = g.eax & !0xFF
                            | *crate::pointers::host(g.edi.wrapping_add(n)).cast_const() as u32;
                    }
                    FromEdi(r, n) => {
                        *reg(&mut g, r) = (crate::pointers::host(g.edi.wrapping_add(n))
                            as *const u32)
                            .read_unaligned();
                    }
                    ToEdi(r, n) => {
                        let v = *reg(&mut g, r);
                        crate::pointers::host(g.edi.wrapping_add(n))
                            .cast::<u32>()
                            .write_unaligned(v);
                    }
                    Put(a, v) => mem(a).write_unaligned(v),
                    EdiPut(n, v) => crate::pointers::host(g.edi.wrapping_add(n))
                        .cast::<u32>()
                        .write_unaligned(v),
                    EdiFromTable(t) => {
                        let at = (game_addr(rva(t))).wrapping_add(g.eax.wrapping_mul(4));
                        g.edi = (crate::pointers::host(at) as *const u32).read_unaligned();
                    }
                    Imm(r, v) => *reg(&mut g, r) = v,
                    Lea(r, a) => *reg(&mut g, r) = game_addr(rva(a)),
                    LeaEdi(r, n) => *reg(&mut g, r) = g.edi.wrapping_add(n),
                    Move(a, b) => *reg(&mut g, a) = *reg(&mut g, b),
                    Nop => {}
                    CmpMem(a, v) => zero = mem(a).read_unaligned() == v,
                    CmpReg(r, v) => zero = *reg(&mut g, r) == v,
                    CmpEdi(n, v) => {
                        zero = (crate::pointers::host(g.edi.wrapping_add(n)) as *const u32)
                            .read_unaligned()
                            == v;
                    }
                    CmpEdiEax(n) => {
                        zero = (crate::pointers::host(g.edi.wrapping_add(n)) as *const u32)
                            .read_unaligned()
                            == g.eax;
                    }
                    Jz(t) => {
                        if zero {
                            pc = t;
                        }
                    }
                    Jnz(t) => {
                        if !zero {
                            pc = t;
                        }
                    }
                    Jmp(t) => pc = t,
                    JzFatal(n) => {
                        if zero {
                            return Err(FATALS[n].into());
                        }
                    }
                    Call(a) => {
                        let n = CALLS
                            .iter()
                            .position(|&c| c == a)
                            .expect("a call the table has");
                        step(n as u32 + 1, &mut g)?;
                    }
                    RepMovsb => {
                        std::ptr::copy(
                            crate::pointers::host(g.esi).cast_const(),
                            crate::pointers::host(g.edi),
                            g.ecx as usize,
                        );
                        g.esi = g.esi.wrapping_add(g.ecx);
                        g.edi = g.edi.wrapping_add(g.ecx);
                        g.ecx = 0;
                    }
                    AddMem(r, a) => {
                        let v = mem(a).read_unaligned();
                        let x = reg(&mut g, r);
                        *x = x.wrapping_add(v);
                    }
                    AddImm(r, v) => {
                        let x = reg(&mut g, r);
                        *x = x.wrapping_add(v);
                    }
                    AddReg(a, b) => {
                        let v = *reg(&mut g, b);
                        let x = reg(&mut g, a);
                        *x = x.wrapping_add(v);
                    }
                    MulImm(r, v) => {
                        let x = reg(&mut g, r);
                        *x = x.wrapping_mul(v);
                    }
                    DivEbp => {
                        let n = ((g.edx as u64) << 32) | g.eax as u64;
                        let q = n / g.ebp as u64;
                        g.edx = (n % g.ebp as u64) as u32;
                        g.eax = q as u32;
                    }
                    RoundEaxToEbx => g.ebx = crate::math::x87::to_int(g.eax) as u32,
                    Return => return Ok(()),
                }
            }
        }
    }

    // The table (made from the disassembly by script).
    pub const SINGLE_RALLY: [Op; 902] = [
        Pusha,
        CmpReg(R::Ebx, 0x3),
        Jnz(4),
        Call(0x48C997),
        Load(R::Esi, 0x7556F4),
        Store(R::Esi, 0x75ACD0),
        Store(R::Eax, 0x7587E8),
        Store(R::Ebx, 0x75ACD8),
        Put(0x75B028, 0x0),
        Put(0x75B098, 0x0),
        Put(0x75ACE4, 0x0),
        Put(0x75ACE0, 0x1),
        Call(0x40F826),
        Call(0x404830),
        Call(0x47CEA8),
        Load(R::Eax, 0x7558DC),
        Store(R::Eax, 0x5C2FD0),
        Lea(R::Esi, 0x75ACD0),
        Load(R::Eax, 0x75ACD4),
        Lea(R::Edi, 0x734BC6),
        AddImm(R::Edi, 0x34),
        CmpEdiEax(0x34),
        Jz(27),
        AddImm(R::Edi, 0x40),
        CmpEdi(0x0, 0xFFFFFFFF),
        JzFatal(0),
        Jmp(21),
        FromEdi(R::Eax, 0x1C),
        Store(R::Eax, 0x75ACF8),
        FromEdi(R::Eax, 0x20),
        Store(R::Eax, 0x75ACFC),
        Load(R::Eax, 0x755650),
        Store(R::Eax, 0x75AD00),
        Load(R::Eax, 0x75565C),
        Store(R::Eax, 0x75AD0C),
        Load(R::Eax, 0x755660),
        Store(R::Eax, 0x75AD10),
        Load(R::Eax, 0x755664),
        Store(R::Eax, 0x75AD14),
        Load(R::Eax, 0x755668),
        Store(R::Eax, 0x75AD18),
        Push(R::Edi),
        Push(R::Eax),
        Imm(R::Eax, 0x1),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        FromEdi(R::Eax, 0x2C),
        Store(R::Eax, 0x75AF94),
        FromEdi(R::Eax, 0x30),
        Store(R::Eax, 0x75AF98),
        FromEdi(R::Eax, 0x34),
        Store(R::Eax, 0x75AF9C),
        FromEdi(R::Eax, 0x38),
        Store(R::Eax, 0x75AFA0),
        FromEdi(R::Eax, 0x3C),
        Store(R::Eax, 0x75AFA4),
        FromEdi(R::Eax, 0x40),
        Store(R::Eax, 0x75AFA8),
        FromEdi(R::Eax, 0x44),
        Store(R::Eax, 0x75AFAC),
        FromEdi(R::Eax, 0x48),
        Store(R::Eax, 0x75AFB0),
        FromEdi(R::Eax, 0x50),
        Store(R::Eax, 0x75AFB8),
        FromEdi(R::Eax, 0x54),
        Store(R::Eax, 0x75AFBC),
        FromEdi(R::Eax, 0x58),
        Store(R::Eax, 0x75AFC0),
        FromEdi(R::Eax, 0x5C),
        Store(R::Eax, 0x75AFC4),
        FromEdi(R::Eax, 0x60),
        Store(R::Eax, 0x75AFC8),
        FromEdi(R::Eax, 0x64),
        Store(R::Eax, 0x75AFCC),
        FromEdi(R::Eax, 0x68),
        Store(R::Eax, 0x75AFD0),
        FromEdi(R::Eax, 0x6C),
        Store(R::Eax, 0x75AFD4),
        FromEdi(R::Eax, 0x70),
        Store(R::Eax, 0x75AFD8),
        FromEdi(R::Eax, 0x74),
        Store(R::Eax, 0x75AFE0),
        FromEdi(R::Eax, 0x78),
        Store(R::Eax, 0x75AFE4),
        FromEdi(R::Al, 0x1C),
        Store(R::Al, 0x75AFE8),
        FromEdi(R::Al, 0x1D),
        Store(R::Al, 0x75AFE9),
        FromEdi(R::Al, 0x1E),
        Store(R::Al, 0x75AFEA),
        FromEdi(R::Al, 0x1F),
        Store(R::Al, 0x75AFEB),
        FromEdi(R::Al, 0x20),
        Store(R::Al, 0x75AFEC),
        FromEdi(R::Al, 0x21),
        Store(R::Al, 0x75AFED),
        FromEdi(R::Al, 0x22),
        Store(R::Al, 0x75AFEE),
        FromEdi(R::Al, 0x23),
        Store(R::Al, 0x75AFEF),
        FromEdi(R::Al, 0x24),
        Store(R::Al, 0x75AFF0),
        FromEdi(R::Al, 0x25),
        Store(R::Al, 0x75AFF1),
        FromEdi(R::Al, 0x26),
        Store(R::Al, 0x75AFF2),
        FromEdi(R::Al, 0x27),
        Store(R::Al, 0x75AFF3),
        Pop(R::Edi),
        CmpMem(0x755988, 0x5),
        Jz(126),
        CmpMem(0x755988, 0x4),
        Jz(126),
        CmpMem(0x755988, 0xF),
        Jz(126),
        CmpMem(0x755988, 0xA),
        Jz(126),
        Push(R::Eax),
        Load(R::Eax, 0x75598C),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        CmpEdi(0xD0, 0x1),
        Jz(290),
        Put(0x75B028, 0xFFFFFFFF),
        Jmp(897),
        Jmp(290),
        Load(R::Edx, 0x7554C8),
        Store(R::Edx, 0x75ACE4),
        Put(0x75ACE0, 0x2),
        Push(R::Eax),
        Imm(R::Eax, 0x1),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        CmpEdi(0xD0, 0x1),
        Jz(136),
        Put(0x75B098, 0xFFFFFFFF),
        FromEdi(R::Eax, 0x8),
        Store(R::Eax, 0x75ACEC),
        Load(R::Eax, 0x75ACEC),
        FromEdi(R::Eax, 0x10C),
        Store(R::Eax, 0x75AE88),
        FromEdi(R::Eax, 0x110),
        Store(R::Eax, 0x75AE8C),
        FromEdi(R::Eax, 0x114),
        Store(R::Eax, 0x75AE90),
        FromEdi(R::Eax, 0x118),
        Store(R::Eax, 0x75AE94),
        FromEdi(R::Eax, 0x11C),
        Store(R::Eax, 0x75AE98),
        FromEdi(R::Eax, 0x120),
        Store(R::Eax, 0x75AE9C),
        FromEdi(R::Eax, 0x288),
        Store(R::Eax, 0x75B008),
        Pusha,
        Imm(R::Ecx, 0x20),
        LeaEdi(R::Esi, 0x24C),
        Lea(R::Edi, 0x75B0DC),
        RepMovsb,
        Popa,
        FromEdi(R::Eax, 0x278),
        Store(R::Eax, 0x75B018),
        FromEdi(R::Eax, 0xD4),
        Store(R::Eax, 0x75AEA0),
        FromEdi(R::Eax, 0xD8),
        Store(R::Eax, 0x75AEA4),
        FromEdi(R::Eax, 0xDC),
        Store(R::Eax, 0x75AEA8),
        FromEdi(R::Eax, 0xE0),
        Store(R::Eax, 0x75AEAC),
        FromEdi(R::Eax, 0xE4),
        Store(R::Eax, 0x75AEB0),
        FromEdi(R::Eax, 0xE8),
        Store(R::Eax, 0x75AEB4),
        FromEdi(R::Eax, 0xEC),
        Store(R::Eax, 0x75AEB8),
        FromEdi(R::Eax, 0xF0),
        Store(R::Eax, 0x75AEBC),
        FromEdi(R::Eax, 0xF4),
        Store(R::Eax, 0x75AEC0),
        FromEdi(R::Eax, 0xF8),
        Store(R::Eax, 0x75AEC4),
        FromEdi(R::Eax, 0xFC),
        Store(R::Eax, 0x75AEC8),
        FromEdi(R::Eax, 0x100),
        Store(R::Eax, 0x75AECC),
        FromEdi(R::Eax, 0x104),
        Store(R::Eax, 0x75AEF8),
        FromEdi(R::Eax, 0x108),
        Store(R::Eax, 0x75AED0),
        FromEdi(R::Eax, 0x158),
        Store(R::Eax, 0x75AF64),
        FromEdi(R::Eax, 0x15C),
        Store(R::Eax, 0x75AF68),
        FromEdi(R::Eax, 0x160),
        Store(R::Eax, 0x75AF6C),
        FromEdi(R::Eax, 0x168),
        Store(R::Eax, 0x75AF74),
        FromEdi(R::Eax, 0x164),
        Store(R::Eax, 0x75AF78),
        FromEdi(R::Eax, 0x170),
        Store(R::Eax, 0x75AF80),
        FromEdi(R::Eax, 0x16C),
        Store(R::Eax, 0x75AF7C),
        FromEdi(R::Eax, 0x174),
        Store(R::Eax, 0x75AF84),
        FromEdi(R::Eax, 0x178),
        Store(R::Eax, 0x75AF70),
        FromEdi(R::Eax, 0x284),
        Store(R::Eax, 0x75B010),
        FromEdi(R::Eax, 0x17C),
        Store(R::Eax, 0x75AED4),
        FromEdi(R::Eax, 0x180),
        Store(R::Eax, 0x75AED8),
        FromEdi(R::Eax, 0x184),
        Store(R::Eax, 0x75AEDC),
        FromEdi(R::Eax, 0x188),
        Store(R::Eax, 0x75AEE0),
        FromEdi(R::Eax, 0x18C),
        Store(R::Eax, 0x75AEE4),
        FromEdi(R::Eax, 0x190),
        Store(R::Eax, 0x75AEE8),
        FromEdi(R::Eax, 0x194),
        Store(R::Eax, 0x75AEEC),
        FromEdi(R::Eax, 0x198),
        Store(R::Eax, 0x75AEF0),
        FromEdi(R::Eax, 0x19C),
        Store(R::Eax, 0x75AEF4),
        FromEdi(R::Eax, 0x1A0),
        Store(R::Eax, 0x75AEF8),
        FromEdi(R::Eax, 0x1A4),
        Store(R::Eax, 0x75AEFC),
        FromEdi(R::Eax, 0x1A8),
        Store(R::Eax, 0x75AF00),
        FromEdi(R::Eax, 0x1AC),
        Store(R::Eax, 0x75AF04),
        FromEdi(R::Eax, 0x1B0),
        Store(R::Eax, 0x75AF08),
        FromEdi(R::Eax, 0x1B4),
        Store(R::Eax, 0x75AF0C),
        FromEdi(R::Eax, 0x1B8),
        Store(R::Eax, 0x75AF10),
        FromEdi(R::Eax, 0x1BC),
        Store(R::Eax, 0x75AF14),
        FromEdi(R::Eax, 0x1C0),
        Store(R::Eax, 0x75AF18),
        FromEdi(R::Eax, 0x1C4),
        Store(R::Eax, 0x75AF1C),
        FromEdi(R::Eax, 0x1C8),
        Store(R::Eax, 0x75AF20),
        FromEdi(R::Eax, 0x1CC),
        Store(R::Eax, 0x75AF24),
        FromEdi(R::Eax, 0x1D0),
        Store(R::Eax, 0x75AF28),
        FromEdi(R::Eax, 0x1D4),
        Store(R::Eax, 0x75AF2C),
        FromEdi(R::Eax, 0x1D8),
        Store(R::Eax, 0x75AF30),
        FromEdi(R::Eax, 0x1DC),
        Store(R::Eax, 0x75AF34),
        FromEdi(R::Eax, 0x1E0),
        Store(R::Eax, 0x75AF38),
        FromEdi(R::Eax, 0x1E4),
        Store(R::Eax, 0x75AF3C),
        FromEdi(R::Eax, 0x1E8),
        Store(R::Eax, 0x75AF40),
        FromEdi(R::Eax, 0x1EC),
        Store(R::Eax, 0x75AF44),
        FromEdi(R::Eax, 0x1F0),
        Store(R::Eax, 0x75AF48),
        FromEdi(R::Eax, 0x1F4),
        Store(R::Eax, 0x75AF4C),
        FromEdi(R::Eax, 0x1F8),
        Store(R::Eax, 0x75AF50),
        FromEdi(R::Eax, 0x1FC),
        Store(R::Eax, 0x75AF54),
        FromEdi(R::Eax, 0x200),
        Store(R::Eax, 0x75AF58),
        FromEdi(R::Eax, 0x204),
        Store(R::Eax, 0x75AF5C),
        FromEdi(R::Eax, 0x208),
        Store(R::Eax, 0x75AF60),
        FromEdi(R::Eax, 0x28),
        Store(R::Eax, 0x75AF88),
        FromEdi(R::Eax, 0x154),
        Store(R::Eax, 0x75AF8C),
        FromEdi(R::Eax, 0xC),
        Store(R::Eax, 0x75AF90),
        FromEdi(R::Eax, 0x10),
        Store(R::Eax, 0x75AF90),
        Put(0x75598C, 0x0),
        Push(R::Eax),
        Load(R::Eax, 0x75598C),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        FromEdi(R::Eax, 0x8),
        Store(R::Eax, 0x75ACE8),
        Load(R::Eax, 0x75ACE8),
        CmpEdi(0xD0, 0x1),
        Jz(300),
        Put(0x75B028, 0xFFFFFFFF),
        FromEdi(R::Eax, 0x10C),
        Store(R::Eax, 0x75AD1C),
        FromEdi(R::Eax, 0x110),
        Store(R::Eax, 0x75AD20),
        FromEdi(R::Eax, 0x114),
        Store(R::Eax, 0x75AD24),
        FromEdi(R::Eax, 0x118),
        Store(R::Eax, 0x75AD28),
        FromEdi(R::Eax, 0x11C),
        Store(R::Eax, 0x75AD2C),
        FromEdi(R::Eax, 0x120),
        Store(R::Eax, 0x75AD30),
        FromEdi(R::Eax, 0x288),
        Store(R::Eax, 0x75B004),
        Pusha,
        Imm(R::Ecx, 0x20),
        LeaEdi(R::Esi, 0x24C),
        Lea(R::Edi, 0x75B06C),
        RepMovsb,
        Popa,
        FromEdi(R::Eax, 0x278),
        Store(R::Eax, 0x75B014),
        FromEdi(R::Eax, 0xD4),
        Store(R::Eax, 0x75AD34),
        FromEdi(R::Eax, 0xD8),
        Store(R::Eax, 0x75AD38),
        FromEdi(R::Eax, 0xDC),
        Store(R::Eax, 0x75AD3C),
        FromEdi(R::Eax, 0xE0),
        Store(R::Eax, 0x75AD40),
        FromEdi(R::Eax, 0xE4),
        Store(R::Eax, 0x75AD44),
        FromEdi(R::Eax, 0xE8),
        Store(R::Eax, 0x75AD48),
        FromEdi(R::Eax, 0xEC),
        Store(R::Eax, 0x75AD4C),
        FromEdi(R::Eax, 0xF0),
        Store(R::Eax, 0x75AD50),
        FromEdi(R::Eax, 0xF4),
        Store(R::Eax, 0x75AD54),
        FromEdi(R::Eax, 0xF8),
        Store(R::Eax, 0x75AD58),
        FromEdi(R::Eax, 0xFC),
        Store(R::Eax, 0x75AD5C),
        FromEdi(R::Eax, 0x100),
        Store(R::Eax, 0x75AD60),
        FromEdi(R::Eax, 0x104),
        Store(R::Eax, 0x75AD8C),
        FromEdi(R::Eax, 0x108),
        Store(R::Eax, 0x75AD64),
        FromEdi(R::Eax, 0x158),
        Store(R::Eax, 0x75ADF8),
        FromEdi(R::Eax, 0x15C),
        Store(R::Eax, 0x75ADFC),
        FromEdi(R::Eax, 0x160),
        Store(R::Eax, 0x75AE00),
        FromEdi(R::Eax, 0x168),
        Store(R::Eax, 0x75AE08),
        FromEdi(R::Eax, 0x164),
        Store(R::Eax, 0x75AE0C),
        FromEdi(R::Eax, 0x170),
        Store(R::Eax, 0x75AE14),
        FromEdi(R::Eax, 0x16C),
        Store(R::Eax, 0x75AE10),
        FromEdi(R::Eax, 0x174),
        Store(R::Eax, 0x75AE18),
        FromEdi(R::Eax, 0x178),
        Store(R::Eax, 0x75AE04),
        FromEdi(R::Eax, 0x284),
        Store(R::Eax, 0x75B00C),
        FromEdi(R::Eax, 0x17C),
        Store(R::Eax, 0x75AD68),
        FromEdi(R::Eax, 0x180),
        Store(R::Eax, 0x75AD6C),
        FromEdi(R::Eax, 0x184),
        Store(R::Eax, 0x75AD70),
        FromEdi(R::Eax, 0x188),
        Store(R::Eax, 0x75AD74),
        FromEdi(R::Eax, 0x18C),
        Store(R::Eax, 0x75AD78),
        FromEdi(R::Eax, 0x190),
        Store(R::Eax, 0x75AD7C),
        FromEdi(R::Eax, 0x194),
        Store(R::Eax, 0x75AD80),
        FromEdi(R::Eax, 0x198),
        Store(R::Eax, 0x75AD84),
        FromEdi(R::Eax, 0x19C),
        Store(R::Eax, 0x75AD88),
        FromEdi(R::Eax, 0x1A0),
        Store(R::Eax, 0x75AD8C),
        FromEdi(R::Eax, 0x1A4),
        Store(R::Eax, 0x75AD90),
        FromEdi(R::Eax, 0x1A8),
        Store(R::Eax, 0x75AD94),
        FromEdi(R::Eax, 0x1AC),
        Store(R::Eax, 0x75AD98),
        FromEdi(R::Eax, 0x1B0),
        Store(R::Eax, 0x75AD9C),
        FromEdi(R::Eax, 0x1B4),
        Store(R::Eax, 0x75ADA0),
        FromEdi(R::Eax, 0x1B8),
        Store(R::Eax, 0x75ADA4),
        FromEdi(R::Eax, 0x1BC),
        Store(R::Eax, 0x75ADA8),
        FromEdi(R::Eax, 0x1C0),
        Store(R::Eax, 0x75ADAC),
        FromEdi(R::Eax, 0x1C4),
        Store(R::Eax, 0x75ADB0),
        FromEdi(R::Eax, 0x1C8),
        Store(R::Eax, 0x75ADB4),
        FromEdi(R::Eax, 0x1CC),
        Store(R::Eax, 0x75ADB8),
        FromEdi(R::Eax, 0x1D0),
        Store(R::Eax, 0x75ADBC),
        FromEdi(R::Eax, 0x1D4),
        Store(R::Eax, 0x75ADC0),
        FromEdi(R::Eax, 0x1D8),
        Store(R::Eax, 0x75ADC4),
        FromEdi(R::Eax, 0x1DC),
        Store(R::Eax, 0x75ADC8),
        FromEdi(R::Eax, 0x1E0),
        Store(R::Eax, 0x75ADCC),
        FromEdi(R::Eax, 0x1E4),
        Store(R::Eax, 0x75ADD0),
        FromEdi(R::Eax, 0x1E8),
        Store(R::Eax, 0x75ADD4),
        FromEdi(R::Eax, 0x1EC),
        Store(R::Eax, 0x75ADD8),
        FromEdi(R::Eax, 0x1F0),
        Store(R::Eax, 0x75ADDC),
        FromEdi(R::Eax, 0x1F4),
        Store(R::Eax, 0x75ADE0),
        FromEdi(R::Eax, 0x1F8),
        Store(R::Eax, 0x75ADE4),
        FromEdi(R::Eax, 0x1FC),
        Store(R::Eax, 0x75ADE8),
        FromEdi(R::Eax, 0x200),
        Store(R::Eax, 0x75ADEC),
        FromEdi(R::Eax, 0x204),
        Store(R::Eax, 0x75ADF0),
        FromEdi(R::Eax, 0x208),
        Store(R::Eax, 0x75ADF4),
        FromEdi(R::Eax, 0x28),
        Store(R::Eax, 0x75AE1C),
        FromEdi(R::Eax, 0x154),
        Store(R::Eax, 0x75AE20),
        FromEdi(R::Eax, 0xC),
        Store(R::Eax, 0x75AE24),
        CmpMem(0x75ACE0, 0x2),
        Jnz(452),
        FromEdi(R::Eax, 0x10),
        Store(R::Eax, 0x75AE24),
        FromEdi(R::Eax, 0x2C),
        Store(R::Eax, 0x75AE28),
        FromEdi(R::Eax, 0x30),
        Store(R::Eax, 0x75AE2C),
        FromEdi(R::Eax, 0x34),
        Store(R::Eax, 0x75AE30),
        FromEdi(R::Eax, 0x38),
        Store(R::Eax, 0x75AE34),
        FromEdi(R::Eax, 0x3C),
        Store(R::Eax, 0x75AE38),
        FromEdi(R::Eax, 0x40),
        Store(R::Eax, 0x75AE3C),
        FromEdi(R::Eax, 0x44),
        Store(R::Eax, 0x75AE40),
        FromEdi(R::Eax, 0x48),
        Store(R::Eax, 0x75AE44),
        FromEdi(R::Eax, 0x50),
        Store(R::Eax, 0x75AE4C),
        FromEdi(R::Eax, 0x54),
        Store(R::Eax, 0x75AE50),
        FromEdi(R::Eax, 0x58),
        Store(R::Eax, 0x75AE54),
        FromEdi(R::Eax, 0x5C),
        Store(R::Eax, 0x75AE58),
        FromEdi(R::Eax, 0x60),
        Store(R::Eax, 0x75AE5C),
        FromEdi(R::Eax, 0x64),
        Store(R::Eax, 0x75AE60),
        FromEdi(R::Eax, 0x68),
        Store(R::Eax, 0x75AE64),
        FromEdi(R::Eax, 0x6C),
        Store(R::Eax, 0x75AE68),
        FromEdi(R::Eax, 0x70),
        Store(R::Eax, 0x75AE6C),
        FromEdi(R::Eax, 0x74),
        Store(R::Eax, 0x75AE74),
        FromEdi(R::Eax, 0x78),
        Store(R::Eax, 0x75AE78),
        FromEdi(R::Al, 0x1C),
        Store(R::Al, 0x75AE7C),
        FromEdi(R::Al, 0x1D),
        Store(R::Al, 0x75AE7D),
        FromEdi(R::Al, 0x1E),
        Store(R::Al, 0x75AE7E),
        FromEdi(R::Al, 0x1F),
        Store(R::Al, 0x75AE7F),
        FromEdi(R::Al, 0x20),
        Store(R::Al, 0x75AE80),
        FromEdi(R::Al, 0x21),
        Store(R::Al, 0x75AE81),
        FromEdi(R::Al, 0x22),
        Store(R::Al, 0x75AE82),
        FromEdi(R::Al, 0x23),
        Store(R::Al, 0x75AE83),
        FromEdi(R::Al, 0x24),
        Store(R::Al, 0x75AE84),
        FromEdi(R::Al, 0x25),
        Store(R::Al, 0x75AE85),
        FromEdi(R::Al, 0x26),
        Store(R::Al, 0x75AE86),
        FromEdi(R::Al, 0x27),
        Store(R::Al, 0x75AE87),
        Put(0x75ACDC, 0x0),
        CmpMem(0x755988, 0x5),
        Jz(526),
        CmpMem(0x755988, 0x4),
        Jz(526),
        CmpMem(0x755988, 0xF),
        Jz(526),
        CmpMem(0x755988, 0xA),
        Jz(526),
        CmpMem(0x755984, 0x1),
        Jz(526),
        Put(0x75ACDC, 0x1),
        Call(0x415C93),
        Call(0x40F02E),
        Call(0x485570),
        Call(0x40F826),
        Call(0x404823),
        Load(R::Eax, 0x7587E8),
        CmpReg(R::Eax, 0x1),
        Jz(535),
        Call(0x47CEA8),
        Call(0x415C6D),
        Call(0x40EFF3),
        Lea(R::Esi, 0x75ACD0),
        CmpMem(0x755988, 0x5),
        Jz(547),
        CmpMem(0x755988, 0x4),
        Jz(547),
        CmpMem(0x755988, 0xF),
        Jz(547),
        CmpMem(0x755988, 0xA),
        Jz(547),
        Jmp(720),
        Push(R::Eax),
        Imm(R::Eax, 0x1),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        Load(R::Eax, 0x75B010),
        ToEdi(R::Eax, 0x284),
        Load(R::Eax, 0x75B018),
        ToEdi(R::Eax, 0x278),
        Load(R::Eax, 0x75AEA0),
        ToEdi(R::Eax, 0xD4),
        Load(R::Eax, 0x75AEA4),
        ToEdi(R::Eax, 0xD8),
        Load(R::Eax, 0x75AEA8),
        ToEdi(R::Eax, 0xDC),
        Load(R::Eax, 0x75AEAC),
        ToEdi(R::Eax, 0xE0),
        Load(R::Eax, 0x75AEB0),
        ToEdi(R::Eax, 0xE4),
        Load(R::Eax, 0x75AEB4),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AEB8),
        ToEdi(R::Eax, 0xEC),
        Load(R::Eax, 0x75AEBC),
        ToEdi(R::Eax, 0xF0),
        Load(R::Eax, 0x75AEC0),
        ToEdi(R::Eax, 0xF4),
        Load(R::Eax, 0x75AEC4),
        ToEdi(R::Eax, 0xF8),
        Load(R::Eax, 0x75AEC8),
        ToEdi(R::Eax, 0xFC),
        Load(R::Eax, 0x75AECC),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75AEF8),
        ToEdi(R::Eax, 0x104),
        Load(R::Eax, 0x75AED0),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75AED4),
        ToEdi(R::Eax, 0x17C),
        Load(R::Eax, 0x75AED8),
        ToEdi(R::Eax, 0x180),
        Load(R::Eax, 0x75AEDC),
        ToEdi(R::Eax, 0x184),
        Load(R::Eax, 0x75AEE0),
        ToEdi(R::Eax, 0x188),
        Load(R::Eax, 0x75AEE4),
        ToEdi(R::Eax, 0x18C),
        Load(R::Eax, 0x75AEE8),
        ToEdi(R::Eax, 0x190),
        Load(R::Eax, 0x75AEEC),
        ToEdi(R::Eax, 0x194),
        Load(R::Eax, 0x75AEF0),
        ToEdi(R::Eax, 0x198),
        Load(R::Eax, 0x75AEF4),
        ToEdi(R::Eax, 0x19C),
        Load(R::Eax, 0x75AEF8),
        ToEdi(R::Eax, 0x1A0),
        Load(R::Eax, 0x75AEFC),
        ToEdi(R::Eax, 0x1A4),
        Load(R::Eax, 0x75AF00),
        ToEdi(R::Eax, 0x1A8),
        Load(R::Eax, 0x75AF04),
        ToEdi(R::Eax, 0x1AC),
        Load(R::Eax, 0x75AF08),
        ToEdi(R::Eax, 0x1B0),
        Load(R::Eax, 0x75AF0C),
        ToEdi(R::Eax, 0x1B4),
        Load(R::Eax, 0x75AF10),
        ToEdi(R::Eax, 0x1B8),
        Load(R::Eax, 0x75AF14),
        ToEdi(R::Eax, 0x1BC),
        Load(R::Eax, 0x75AF18),
        ToEdi(R::Eax, 0x1C0),
        Load(R::Eax, 0x75AF1C),
        ToEdi(R::Eax, 0x1C4),
        Load(R::Eax, 0x75AF20),
        ToEdi(R::Eax, 0x1C8),
        Load(R::Eax, 0x75AF24),
        ToEdi(R::Eax, 0x1CC),
        Load(R::Eax, 0x75AF28),
        ToEdi(R::Eax, 0x1D0),
        Load(R::Eax, 0x75AF2C),
        ToEdi(R::Eax, 0x1D4),
        Load(R::Eax, 0x75AF30),
        ToEdi(R::Eax, 0x1D8),
        Load(R::Eax, 0x75AF34),
        ToEdi(R::Eax, 0x1DC),
        Load(R::Eax, 0x75AF38),
        ToEdi(R::Eax, 0x1E0),
        Load(R::Eax, 0x75AF3C),
        ToEdi(R::Eax, 0x1E4),
        Load(R::Eax, 0x75AF40),
        ToEdi(R::Eax, 0x1E8),
        Load(R::Eax, 0x75AF44),
        ToEdi(R::Eax, 0x1EC),
        Load(R::Eax, 0x75AF48),
        ToEdi(R::Eax, 0x1F0),
        Load(R::Eax, 0x75AF4C),
        ToEdi(R::Eax, 0x1F4),
        Load(R::Eax, 0x75AF50),
        ToEdi(R::Eax, 0x1F8),
        Load(R::Eax, 0x75AF54),
        ToEdi(R::Eax, 0x1FC),
        Load(R::Eax, 0x75AF58),
        ToEdi(R::Eax, 0x200),
        Load(R::Eax, 0x75AF5C),
        ToEdi(R::Eax, 0x204),
        Load(R::Eax, 0x75AF60),
        ToEdi(R::Eax, 0x208),
        Pusha,
        Imm(R::Ecx, 0x20),
        Lea(R::Esi, 0x75B0DC),
        LeaEdi(R::Edi, 0x24C),
        RepMovsb,
        Popa,
        Load(R::Ebx, 0x75AEFC),
        AddMem(R::Ebx, 0x75AF00),
        AddMem(R::Ebx, 0x75AF04),
        AddMem(R::Ebx, 0x75AF08),
        CmpReg(R::Ebx, 0x0),
        Jz(674),
        Move(R::Eax, R::Ebx),
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        Move(R::Ebx, R::Eax),
        Load(R::Eax, 0x75AF0C),
        MulImm(R::Eax, 0x6),
        CmpReg(R::Eax, 0x0),
        Jz(684),
        Nop,
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        AddReg(R::Ebx, R::Eax),
        ToEdi(R::Ebx, 0xD8),
        Load(R::Eax, 0x75AF40),
        ToEdi(R::Eax, 0xF8),
        Load(R::Eax, 0x75AF60),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75AF4C),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75AF2C),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AF8C),
        ToEdi(R::Eax, 0x154),
        Load(R::Eax, 0x75AF90),
        ToEdi(R::Eax, 0xC),
        CmpMem(0x75B098, 0xFFFFFFFE),
        Jz(702),
        CmpMem(0x75B098, 0xFFFFFFFF),
        Jnz(705),
        EdiPut(0x88, 0x36EE80),
        EdiPut(0xD0, 0x6),
        Jmp(719),
        Load(R::Eax, 0x75B08C),
        ToEdi(R::Eax, 0x88),
        Load(R::Eax, 0x75B090),
        ToEdi(R::Eax, 0x27C),
        Load(R::Eax, 0x75B094),
        RoundEaxToEbx,
        MulImm(R::Ebx, 0xA),
        ToEdi(R::Ebx, 0x94),
        Pusha,
        Imm(R::Ecx, 0x40),
        Lea(R::Esi, 0x75B09C),
        LeaEdi(R::Edi, 0x20C),
        RepMovsb,
        Popa,
        Put(0x75598C, 0x0),
        Push(R::Eax),
        Load(R::Eax, 0x75598C),
        EdiFromTable(0x75647C),
        Pop(R::Eax),
        Load(R::Eax, 0x75B00C),
        ToEdi(R::Eax, 0x284),
        Pusha,
        Imm(R::Ecx, 0x20),
        Lea(R::Esi, 0x75B06C),
        LeaEdi(R::Edi, 0x24C),
        RepMovsb,
        Popa,
        Load(R::Eax, 0x75B014),
        ToEdi(R::Eax, 0x278),
        Load(R::Eax, 0x75AD34),
        ToEdi(R::Eax, 0xD4),
        Load(R::Eax, 0x75AD38),
        ToEdi(R::Eax, 0xD8),
        Load(R::Eax, 0x75AD3C),
        ToEdi(R::Eax, 0xDC),
        Load(R::Eax, 0x75AD40),
        ToEdi(R::Eax, 0xE0),
        Load(R::Eax, 0x75AD44),
        ToEdi(R::Eax, 0xE4),
        Load(R::Eax, 0x75AD48),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AD4C),
        ToEdi(R::Eax, 0xEC),
        Load(R::Eax, 0x75AD50),
        ToEdi(R::Eax, 0xF0),
        Load(R::Eax, 0x75AD54),
        ToEdi(R::Eax, 0xF4),
        Load(R::Eax, 0x75AD5C),
        ToEdi(R::Eax, 0xFC),
        Load(R::Eax, 0x75AD60),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75AD8C),
        ToEdi(R::Eax, 0x104),
        Load(R::Eax, 0x75AD64),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75AD68),
        ToEdi(R::Eax, 0x17C),
        Load(R::Eax, 0x75AD6C),
        ToEdi(R::Eax, 0x180),
        Load(R::Eax, 0x75AD70),
        ToEdi(R::Eax, 0x184),
        Load(R::Eax, 0x75AD74),
        ToEdi(R::Eax, 0x188),
        Load(R::Eax, 0x75AD78),
        ToEdi(R::Eax, 0x18C),
        Load(R::Eax, 0x75AD7C),
        ToEdi(R::Eax, 0x190),
        Load(R::Eax, 0x75AD80),
        ToEdi(R::Eax, 0x194),
        Load(R::Eax, 0x75AD84),
        ToEdi(R::Eax, 0x198),
        Load(R::Eax, 0x75AD88),
        ToEdi(R::Eax, 0x19C),
        Load(R::Eax, 0x75AD8C),
        ToEdi(R::Eax, 0x1A0),
        Load(R::Eax, 0x75AD90),
        ToEdi(R::Eax, 0x1A4),
        Load(R::Eax, 0x75AD94),
        ToEdi(R::Eax, 0x1A8),
        Load(R::Eax, 0x75AD98),
        ToEdi(R::Eax, 0x1AC),
        Load(R::Eax, 0x75AD9C),
        ToEdi(R::Eax, 0x1B0),
        Load(R::Eax, 0x75ADA0),
        ToEdi(R::Eax, 0x1B4),
        Load(R::Eax, 0x75ADA4),
        ToEdi(R::Eax, 0x1B8),
        Load(R::Eax, 0x75ADA8),
        ToEdi(R::Eax, 0x1BC),
        Load(R::Eax, 0x75ADAC),
        ToEdi(R::Eax, 0x1C0),
        Load(R::Eax, 0x75ADB0),
        ToEdi(R::Eax, 0x1C4),
        Load(R::Eax, 0x75ADB4),
        ToEdi(R::Eax, 0x1C8),
        Load(R::Eax, 0x75ADB8),
        ToEdi(R::Eax, 0x1CC),
        Load(R::Eax, 0x75ADBC),
        ToEdi(R::Eax, 0x1D0),
        Load(R::Eax, 0x75ADC0),
        ToEdi(R::Eax, 0x1D4),
        Load(R::Eax, 0x75ADC4),
        ToEdi(R::Eax, 0x1D8),
        Load(R::Eax, 0x75ADC8),
        ToEdi(R::Eax, 0x1DC),
        Load(R::Eax, 0x75ADCC),
        ToEdi(R::Eax, 0x1E0),
        Load(R::Eax, 0x75ADD0),
        ToEdi(R::Eax, 0x1E4),
        Load(R::Eax, 0x75ADD4),
        ToEdi(R::Eax, 0x1E8),
        Load(R::Eax, 0x75ADD8),
        ToEdi(R::Eax, 0x1EC),
        Load(R::Eax, 0x75ADDC),
        ToEdi(R::Eax, 0x1F0),
        Load(R::Eax, 0x75ADE0),
        ToEdi(R::Eax, 0x1F4),
        Load(R::Eax, 0x75ADE4),
        ToEdi(R::Eax, 0x1F8),
        Load(R::Eax, 0x75ADE8),
        ToEdi(R::Eax, 0x1FC),
        Load(R::Eax, 0x75ADEC),
        ToEdi(R::Eax, 0x200),
        Load(R::Eax, 0x75ADF0),
        ToEdi(R::Eax, 0x204),
        Load(R::Eax, 0x75ADF4),
        ToEdi(R::Eax, 0x208),
        Load(R::Ebx, 0x75AD90),
        AddMem(R::Ebx, 0x75AD94),
        AddMem(R::Ebx, 0x75AD98),
        AddMem(R::Ebx, 0x75AD9C),
        CmpReg(R::Ebx, 0x0),
        Jz(845),
        Move(R::Eax, R::Ebx),
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        Move(R::Ebx, R::Eax),
        Load(R::Eax, 0x75ADA0),
        MulImm(R::Eax, 0x6),
        CmpReg(R::Eax, 0x0),
        Jz(855),
        Nop,
        CmpReg(R::Eax, 0x0),
        JzFatal(1),
        Imm(R::Edx, 0),
        Imm(R::Ebp, 0xA),
        DivEbp,
        AddReg(R::Ebx, R::Eax),
        ToEdi(R::Ebx, 0xD8),
        Load(R::Eax, 0x75ADD4),
        ToEdi(R::Eax, 0xF8),
        Load(R::Eax, 0x75ADF4),
        ToEdi(R::Eax, 0x108),
        Load(R::Eax, 0x75ADE0),
        ToEdi(R::Eax, 0x100),
        Load(R::Eax, 0x75ADC0),
        ToEdi(R::Eax, 0xE8),
        Load(R::Eax, 0x75AE20),
        ToEdi(R::Eax, 0x154),
        Load(R::Eax, 0x75AE24),
        ToEdi(R::Eax, 0xC),
        CmpMem(0x75B028, 0xFFFFFFFE),
        Jz(873),
        CmpMem(0x75B028, 0xFFFFFFFF),
        Jnz(876),
        EdiPut(0x88, 0x36EE80),
        EdiPut(0xD0, 0x6),
        Jmp(890),
        Load(R::Eax, 0x75B01C),
        ToEdi(R::Eax, 0x88),
        Load(R::Eax, 0x75B020),
        ToEdi(R::Eax, 0x27C),
        Load(R::Eax, 0x75B024),
        RoundEaxToEbx,
        MulImm(R::Ebx, 0xA),
        ToEdi(R::Ebx, 0x94),
        Pusha,
        Imm(R::Ecx, 0x40),
        Lea(R::Esi, 0x75B02C),
        LeaEdi(R::Edi, 0x20C),
        RepMovsb,
        Popa,
        Load(R::Eax, 0x7587E8),
        CmpReg(R::Eax, 0x1),
        Jz(896),
        Call(0x442AF6),
        Call(0x474E04),
        Call(0x442AF6),
        Call(0x407F95),
        Load(R::Edx, 0x75ACE4),
        Store(R::Edx, 0x7554C8),
        Call(0x40F780),
        Popa,
        Return,
    ];
}

#[cfg(test)]
mod tests {
    use super::table::{Op, R, SINGLE_RALLY};
    use super::*;
    use crate::errors::BadInput;
    use crate::framework::{Regs, game_addr, rva};
    use crate::menu::owner::menus;
    use crate::pointers::GamePtr;

    /// The tests share the game's memory.
    /// The size of the stand-in for the game's image: every variable of the function is below.
    const IMAGE_LEN: usize = 0x400000;
    const RECORD_LEN: usize = 0x28C;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 16) as u32
        }
        fn pick<T: Copy>(&mut self, list: &[T]) -> T {
            list[self.next() as usize % list.len()]
        }
    }

    /// The game's memory for the tests: the image's stand-in (random bytes, `BASE` at its
    /// start, for the variables that have not moved to statics) and the players' records.
    struct World {
        image: usize,
        /// Every range the table (and its calls' stand-ins) reads or writes: (address, length).
        ranges: Vec<(usize, usize)>,
    }

    fn world() -> World {
        use std::sync::atomic::Ordering::Relaxed;
        let play = &menus().play;
        let image = Box::leak(vec![0u8; IMAGE_LEN].into_boxed_slice());
        let base = crate::pointers::lend(image.as_mut_ptr(), IMAGE_LEN);
        crate::framework::BASE
            .compare_exchange(0, base as usize, Relaxed, Relaxed)
            .expect("no other test sets the base");
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for i in 0..IMAGE_LEN {
            unsafe {
                crate::pointers::host_at(base as usize)
                    .add(i)
                    .write(rng.next() as u8);
            };
        }
        let mut ranges = Vec::new();
        for op in SINGLE_RALLY.iter() {
            match *op {
                Op::Store(R::Al, a) => ranges.push((game_addr(rva(a)) as usize, 1)),
                Op::Store(_, a) | Op::Load(_, a) | Op::Put(a, _) | Op::CmpMem(a, _) => {
                    ranges.push((game_addr(rva(a)) as usize, 4));
                }
                Op::AddMem(_, a) => ranges.push((game_addr(rva(a)) as usize, 4)),
                _ => {}
            }
        }
        for (a, len) in [
            (BLOCK_FIRST[0].addr(), 0x20),
            (BLOCK_CURRENT[0].addr(), 0x20),
            (OUTCOME_FIRST.data.addr(), 0x40),
            (OUTCOME_CURRENT.data.addr(), 0x40),
            (play.players.pointers[0].addr(), 16),
        ] {
            ranges.push((a as usize, len));
        }
        for r in &play.players.players {
            ranges.push((r.addr() as usize, RECORD_LEN));
        }
        ranges.sort_unstable();
        ranges.dedup();
        World {
            image: base as usize,
            ranges,
        }
    }

    impl World {
        fn image_bytes(&self) -> Vec<u8> {
            unsafe {
                std::slice::from_raw_parts(
                    crate::pointers::host_at(self.image).cast_const(),
                    IMAGE_LEN,
                )
                .to_vec()
            }
        }
        /// The bytes of every range (the variables that have moved out of the image too).
        fn range_bytes(&self) -> Vec<u8> {
            let mut out = Vec::new();
            for &(a, len) in &self.ranges {
                out.extend_from_slice(unsafe {
                    std::slice::from_raw_parts(crate::pointers::host_at(a).cast_const(), len)
                });
            }
            out
        }
        fn set_range_bytes(&self, bytes: &[u8]) {
            let mut at = 0;
            for &(a, len) in &self.ranges {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        bytes[at..].as_ptr(),
                        crate::pointers::host_at(a),
                        len,
                    );
                };
                at += len;
            }
        }
        fn set_image(&self, bytes: &[u8]) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    crate::pointers::host_at(self.image),
                    IMAGE_LEN,
                );
            }
        }
    }

    #[derive(Clone, Debug)]
    struct Case {
        start: u32,
        kind: u32,
        mode: u32,
        count: u32,
        current: u32,
        /// The +0xD0 word of each record.
        d0: [u32; 4],
        /// Records `g_players` has no entry for (0).
        stage: u32,
        /// The entry of the track list holding the stage, or the entry the list ends at.
        entry: usize,
        found: bool,
        result: Option<u32>,
        result_2: Option<u32>,
        secs: u32,
        secs_2: u32,
        /// What the call at place 9 leaves in `SWITCH`.
        switch_after: Option<u32>,
        /// How the race leaves the sums of times: 0 random, 1 zero, 2 wrapping to zero, 3 small.
        sums: u32,
        sums_2: u32,
        extra: u32,
        extra_2: u32,
        fail_at: Option<u32>,
        seed: u64,
    }

    /// The calls' stand-ins: they log (place, the memory then), may fail, and write what the
    /// real calls would change that the function reads after.
    struct Calls<'a> {
        case: &'a Case,
        ranges: &'a [(usize, usize)],
        log: Vec<(u32, u64)>,
        rng: Rng,
    }

    fn digest(ranges: &[(usize, usize)]) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for &(a, len) in ranges {
            for i in 0..len {
                h ^= unsafe { crate::pointers::host_at(a + i).cast_const().read() } as u64;
                h = h.wrapping_mul(0x100_0000_01b3);
            }
        }
        h
    }

    impl Calls<'_> {
        fn call(&mut self, place: u32) -> Result<(), Refused> {
            let c = self.case;
            self.log.push((place, digest(self.ranges)));
            if c.fail_at == Some(place) {
                return Err(BadInput("injected").into());
            }
            match place {
                1 => SUMMARY.set(self.rng.next()),
                3 => STAGE.set(c.stage),
                4 => AFTER_LOADING.set(self.rng.next()),
                7 => {
                    // The race changes everything the results are copied from.
                    let (kept, kept_2) = (RESULT.get(), RESULT_2.get());
                    for a in (0x75AD00..0x75B0FC).step_by(4) {
                        GamePtr::new(game_addr(rva(a)))
                            .write(self.rng.next(), "a test variable")
                            .unwrap();
                    }
                    for (times, extra, sums, e) in [
                        (TIMES_FIRST, EXTRA_FIRST, c.sums, c.extra),
                        (TIMES_CURRENT, EXTRA_CURRENT, c.sums_2, c.extra_2),
                    ] {
                        let v: [u32; 4] = match sums {
                            1 => [0; 4],
                            2 => [1, u32::MAX, 0, 0],
                            3 => [3, 0, 2, 1],
                            _ => [(); 4].map(|_| self.rng.next()),
                        };
                        for (a, v) in times.iter().zip(v) {
                            a.set(v);
                        }
                        extra.set(e);
                    }
                    RESULT.set(c.result.unwrap_or(kept));
                    RESULT_2.set(c.result_2.unwrap_or(kept_2));
                    OUTCOME_CURRENT.secs.set_bits(c.secs);
                    OUTCOME_FIRST.secs.set_bits(c.secs_2);
                }
                9 => {
                    if let Some(s) = c.switch_after {
                        SWITCH.set(s);
                    }
                }
                13 => RACE_KEPT.set(self.rng.next()),
                _ => {}
            }
            Ok(())
        }
    }

    /// Puts the case's memory in place (the same bytes for both runs).
    unsafe fn setup(w: &mut World, c: &Case) {
        let play = &menus().play;
        unsafe {
            let mut rng = Rng(c.seed | 1);
            for &(a, len) in &w.ranges {
                for i in 0..len {
                    crate::pointers::host_at(a + i).write(rng.next() as u8);
                }
            }
            for (k, r) in play.players.players.iter().enumerate() {
                play.players.pointers[k].set(r.addr());
                set_field(r, 0xD0, c.d0[k]).expect("the record is there");
            }
            EVENT_MODE.set(c.mode);
            PLAYER_COUNT.set(c.count);
            CURRENT.set(c.current);
            // The track list: entries 0x40 apart from +0x34; the stage's number at +0x34 of
            // the entry; a dword -1 at the start of the entry the list ends at.
            let entry =
                |k: usize| system().records.stage_tables.trak_pak() + 0x34 + 0x40 * k as u32;
            for k in 0..6 {
                let mut id = rng.next();
                if id == c.stage {
                    id ^= 1;
                }
                GamePtr::new(entry(k).wrapping_add(0x34))
                    .write(id, "a test variable")
                    .unwrap();
                if k > 0 {
                    let mut s = rng.next();
                    if s == u32::MAX {
                        s = 0;
                    }
                    GamePtr::new(entry(k).wrapping_add(0))
                        .write(s, "a test variable")
                        .unwrap();
                }
            }
            if c.found {
                GamePtr::new(entry(c.entry).wrapping_add(0x34))
                    .write(c.stage, "a test variable")
                    .unwrap();
            } else {
                GamePtr::new(entry(c.entry).wrapping_add(0))
                    .write(u32::MAX, "a test variable")
                    .unwrap();
            }
        }
    }

    type Outcome3 = (Result<(), Refused>, Vec<(u32, u64)>, Vec<u8>, Vec<u8>);

    fn after(w: &World, result: Result<(), Refused>, log: Vec<(u32, u64)>) -> Outcome3 {
        (result, log, w.image_bytes(), w.range_bytes())
    }

    /// The table, with its own registers; the stand-ins leave junk in the ones the real
    /// calls leave (eax, ebx, ecx, edx, al).
    unsafe fn run_table(w: &World, c: &Case) -> Outcome3 {
        unsafe {
            let mut calls = Calls {
                case: c,
                ranges: &w.ranges,
                log: Vec::new(),
                rng: Rng(c.seed ^ 0x55AA),
            };
            let result = {
                super::table::run(
                    &SINGLE_RALLY,
                    c.start,
                    c.kind,
                    &mut |place, g: &mut Regs| {
                        let junk = 0xDEAD_0000 | place;
                        match place {
                            4 => g.eax = junk,
                            11 => {
                                [g.eax, g.ebx, g.ecx, g.edx] = [junk, junk + 1, junk + 2, junk + 3];
                            }
                            13 => g.eax = (g.eax & !0xFF) | 0x41,
                            _ => {}
                        }
                        calls.call(place)
                    },
                )
            };
            after(w, result, calls.log)
        }
    }

    fn run_new(w: &World, c: &Case) -> Outcome3 {
        let mut calls = Calls {
            case: c,
            ranges: &w.ranges,
            log: Vec::new(),
            rng: Rng(c.seed ^ 0x55AA),
        };
        let result = guarded(c.start, c.kind, &mut |call| calls.call(call as u32));
        after(w, result, calls.log)
    }

    fn cases() -> Vec<Case> {
        let mut rng = Rng(0x1234_5678_9ABC_DEF1);
        let floats = [
            0.0f32.to_bits(),
            1.5f32.to_bits(),
            2.5f32.to_bits(),
            12.75f32.to_bits(),
            (-3.5f32).to_bits(),
            123456.7f32.to_bits(),
            3e9f32.to_bits(),
            0x7FC0_0000,
        ];
        let results = [None, Some(-2i32 as u32), Some(u32::MAX), Some(0), Some(7)];
        let mut out = Vec::new();
        let make = |rng: &mut Rng, kind, start, mode, count, current| {
            let found = !rng.next().is_multiple_of(8);
            Case {
                start,
                kind,
                mode,
                count,
                current,
                d0: [(); 4].map(|_| rng.pick(&[1, 1, 0, 2])),
                stage: rng.next(),
                entry: if found {
                    rng.pick(&[0, 1, 2, 4])
                } else {
                    rng.pick(&[1, 2, 3])
                },
                found,
                result: rng.pick(&results),
                result_2: rng.pick(&results),
                secs: rng.pick(&floats),
                secs_2: rng.pick(&floats),
                switch_after: rng.pick(&[None, Some(0), Some(1)]),
                sums: rng.pick(&[0, 1, 2, 3]),
                sums_2: rng.pick(&[0, 1, 2, 3]),
                extra: {
                    let r = rng.next();
                    rng.pick(&[0, 1, 2, 0x2AAA_AAAB, r])
                },
                extra_2: {
                    let r = rng.next();
                    rng.pick(&[0, 1, 2, 0x2AAA_AAAB, r])
                },
                fail_at: None,
                seed: rng.next() as u64 | 1 << 40,
            }
        };
        for kind in [3, 1, 7] {
            for start in [1, 0, 2] {
                for mode in [5, 4, 0xF, 0xA, 9, 0x12] {
                    for count in [1, 2] {
                        for current in [0, 1, 2, 3] {
                            for _ in 0..3 {
                                out.push(make(&mut rng, kind, start, mode, count, current));
                            }
                        }
                    }
                }
            }
        }
        // Each call failing, in both families of modes, with a track found.
        for place in 1..=14 {
            for mode in [5, 9] {
                for start in [1, 0] {
                    let mut c = make(&mut rng, 3, start, mode, 2, 1);
                    c.found = true;
                    c.entry = 1;
                    c.d0 = [1; 4];
                    c.fail_at = Some(place);
                    out.push(c);
                }
            }
        }
        out
    }

    /// The table and the plain Rust version, on the same memory and calls, agree on the
    /// result, the order of the calls with the memory at each, and every byte left.
    #[test]
    fn plain_rust_equals_the_table() {
        let play = &menus().play;
        let _g = crate::store::test_lock();
        let mut w = world();
        // The game's variables as they were, put back at the end (other tests read them).
        let first = crate::moved::snapshot();
        // A first check that the stand-in memory is there.
        RACE_INFO.set(0x1234_5678);
        assert_eq!(RACE_INFO.get(), 0x1234_5678);
        let kept_players: Vec<_> = play
            .players
            .players
            .iter()
            .map(crate::session::Shared::get)
            .collect();
        let (mut done, mut fatal, mut failed, mut engine, mut early, mut results) =
            (0, 0, 0, 0, 0, 0);
        let cases = cases();
        for (n, c) in cases.iter().enumerate() {
            unsafe {
                setup(&mut w, c);
                let init_image = w.image_bytes();
                let init_ranges = w.range_bytes();
                let reference = run_table(&w, c);
                // Back to the start: the image, and every range (the records, the variables
                // that have left the image).
                w.set_image(&init_image);
                w.set_range_bytes(&init_ranges);
                let ours = run_new(&w, c);
                assert_eq!(ours.0, reference.0, "case {n}: result {c:?}");
                assert_eq!(ours.1, reference.1, "case {n}: calls {c:?}");
                assert!(ours.2 == reference.2, "case {n}: image {c:?}");
                assert!(ours.3 == reference.3, "case {n}: ranges {c:?}");
                match reference.0 {
                    Ok(()) => done += 1,
                    Err(Refused::Fatal(_)) => fatal += 1,
                    Err(Refused::BadInput(BadInput("injected"))) => failed += 1,
                    Err(e) => panic!("case {n}: an unexpected refusal: {e:?}"),
                }
                let places: Vec<u32> = reference.1.iter().map(|&(p, _)| p).collect();
                engine += places.contains(&11) as u32;
                early += (places.last() == Some(&14) && !places.contains(&5)) as u32;
                results += (places.contains(&7) && c.result == Some(u32::MAX)) as u32;
            }
        }
        assert!(
            done > 100 && fatal > 10 && failed > 20 && engine > 20 && early > 5 && results > 10,
            "coverage: {done} {fatal} {failed} {engine} {early} {results}"
        );
        crate::moved::restore(&first);
        for (cell, kept) in play.players.players.iter().zip(kept_players) {
            cell.set(kept);
        }
    }
}
