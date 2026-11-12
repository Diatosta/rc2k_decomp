//! `race::single_rally_race` (0x4843EC, `single_rally`'s race): the chosen
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
use crate::menu::owner::Menus;
use crate::race::owner::Race;
use crate::race::state_store::{RaceCar, RaceResult};
use crate::session::DwordLike;
use crate::session::PlainDword;
use crate::session::{Bytes, Dword};
#[cfg(test)]
use crate::system::owner::system;
use rc2k_core::menu::player::{Player, TAKES_PART};

/// A player's or a track's record: a game address.
type Record<'a> = crate::menu::standings::PlayerCell<'a>;

// g_trakPakHeader (0x734BC6, `records::stages_store`): +0x34 the entries, 0x40 apart. An
// entry (at `edi`) holds its track number at +0x34; the list ends at a dword -1 where the
// next entry would start.
/// g_players (0x75647C): the players' records, 4 bytes each.
// The front end's words the race reads and writes (`Menus`):
// - ChampView.summary: whether the standings summary is taken (1, `saves::best_times::SUMMARY`),
//   copied to `RaceState.settings.rivals`.
// - Event.sparks_mode, copied to `after_loading_copy` after the loading picture.
// - ScreenFlow.kept: kept over the race (`RaceState.settings.kept`).
// - Event.player_count: the number of players.
// - Event.event_kind: the event mode (5, 4, 0xF, 0xA: the events' one-car races).
// - Event.current_player: the current player's number.
// - Arcade.scene_let_go: the run's start argument (1: no loading picture after).
// The race's (`RaceState.settings`, `RaceState.results`):
// - settings.rivals: a copy of the summary word (the summary's block id, as the race reads
//   it: `race::copy_league_table`, `physics::fill_rival_table`).
// - settings.stage: the stage chosen; settings.run_kind: the run's kind argument.
// - settings.no_replay: 1 when the race is not an event's and has more than one player.
// - settings.race_mode: 1, or 2 in the events' one-car modes.
// - settings.kept: a copy of ScreenFlow.kept, put back after the race.
// - results.results[0].result / results.results[1].result: the current / the first car's
//   result (-1 none, -2 left).
// - settings.cars[0].view: the second set's word that mode 2 takes from the record's
//   `second_view`.
#[cfg(test)]
const SUMMARY: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .play
    .champ_view
    .summary;
#[cfg(test)]
const AFTER_LOADING: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .event
    .sparks_mode;
#[cfg(test)]
const PLAYER_COUNT: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .event
    .player_count;
#[cfg(test)]
const EVENT_MODE: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .event
    .event_kind;
#[cfg(test)]
const CURRENT: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .event
    .current_player;
#[cfg(test)]
const SWITCH: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .play
    .arcade
    .scene_let_go;
#[cfg(test)]
const STAGE: &Dword = &crate::race::owner::RACE.shared.state.settings.stage;
#[cfg(test)]
const RACE_KEPT: &Dword = &crate::race::owner::RACE.shared.state.settings.kept;
#[cfg(test)]
const RESULT: &Dword = &crate::race::owner::RACE.shared.state.results.results[0].result;
#[cfg(test)]
const RESULT_2: &Dword = &crate::race::owner::RACE.shared.state.results.results[1].result;
/// One player's side of the race variables: the car's settings and the results it
/// leaves. `first_side` is player 1's, `current_side` the current player's.
struct Side<'a> {
    car: &'a RaceCar,
    /// RaceState.settings.car_kind_current / RaceState.settings.car_kind_first: the record's `car_number` (+8).
    car_kind: &'a dyn DwordLike,
    /// The record's `service_spent`.
    service_spent: &'a dyn DwordLike,
    /// The record's `state_flags`.
    flags: &'a dyn DwordLike,
    /// RaceState.results.hud_bits_current (the current's): the record's `hud_bits`.
    hud_bits: &'a dyn DwordLike,
    /// The car's result, and the 0x20 bytes of its `impact_marks`.
    result: &'a RaceResult,
}
/// Player 1's side.
fn first_side(race: &Race) -> Side<'_> {
    let (settings, results) = (&race.shared.state.settings, &race.shared.state.results);
    Side {
        car: &settings.cars[1],
        car_kind: &settings.car_kind_first,
        service_spent: &results.service_spent_first,
        flags: &results.flags_first,
        hud_bits: &results.hud_bits_first,
        result: &results.results[1],
    }
}
/// The current player's side.
fn current_side(race: &Race) -> Side<'_> {
    let (settings, results) = (&race.shared.state.settings, &race.shared.state.results);
    Side {
        car: &settings.cars[0],
        car_kind: &settings.car_kind_current,
        service_spent: &results.service_spent_current,
        flags: &results.flags_current,
        hud_bits: &results.hud_bits_current,
        result: &results.results[0],
    }
}

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
fn settings_to_race<'a>(menus: &'a Menus, race: &'a Race) -> [(&'a PlainDword, &'a Dword); 5] {
    let (from, to) = (&menus.screens.option_work, &race.shared.state.settings);
    [
        (&from.lens_flare, &to.lens_flare),
        (&from.reflections, &to.reflections),
        (&from.car_shadows, &to.car_shadows),
        (&from.car_detail, &to.car_detail),
        (&from.headlights, &to.headlights),
    ]
}

#[cfg(test)]
fn set_field(rec: Record, off: u32, v: u32) -> Result<(), BadInput> {
    rec.update(|p| p.store(&[(off as usize, v)]));
    Ok(())
}

/// `g_players`[n].
pub fn player(menus: &Menus, n: u32) -> Result<Record<'_>, BadInput> {
    crate::menu::player::player(menus, n).ok_or(BadInput("a player past the four"))
}

/// A record's controller settings into a car's: the controller, gearing, sensitivity, axes,
/// mode, dead zones, strength, and the 0x2C bytes of `bindings` as eleven words
/// (`RaceCar::recorded_bindings`).
fn take_controls(p: &Player, car: &RaceCar) {
    car.controller.set(p.controller);
    car.gearing.set(p.gearing);
    car.sensitivity_level.set(p.sensitivity_level);
    car.axis_codes.set(u32::from_le_bytes(p.axis_codes));
    car.controller_mode.set(p.controller_mode);
    car.end_zone_pct.set(p.end_zone_pct);
    car.centre_dead_zone_pct.set(p.centre_dead_zone_pct);
    car.strength.set(p.strength);
    for (cell, word) in car.recorded_bindings().zip(p.bindings.words()) {
        cell.set(word);
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

/// The 0x20 bytes (`impact_marks`) of a record into a result.
fn take_block(p: &Player, result: &RaceResult) {
    for (cell, &word) in result.impact_marks.iter().zip(&p.impact_marks) {
        cell.set(word);
    }
}

/// The total a record keeps in `damage[1]` (+0xD8): a tenth of the sum of the car's four times, and
/// a tenth of six times the extra word. The original divides with `div ebp`, after a test of the
/// dividend for 0 that skips the division, so its fatal error 0x80701010 is out of reach.
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
    for (word, cell) in p.impact_marks.iter_mut().zip(&side.result.impact_marks) {
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
        let secs = crate::math::x87::to_int(&result.secs) as u32;
        p.figure = secs.wrapping_mul(10);
        p.best_line = result.data.read();
    }
}

/// `race::single_rally_race` (0x4843EC): `start` the run's start (1: the loading picture is not
/// shown again after), `kind` the kind (3: the rally's times copied first). Refuses, before
/// anything, a missing player record (`g_players`[1] or [`Event.current_player`]; the original reads and
/// writes through it).
pub fn single_rally_race(
    o: crate::owners::Owners<'static>,
    start: u32,
    kind: u32,
) -> Result<(), Refused> {
    guarded(o, start, kind, &mut |call| step(o, call))
}

/// The refusal, then the race, its calls done by `step`.
fn guarded(
    o: crate::owners::Owners<'static>,
    start: u32,
    kind: u32,
    step: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    player(o.menus, 1)?;
    player(o.menus, o.menus.screens.event.current_player.get())?;
    single_rally(o, start, kind, step)
}

/// The last calls of the original (from 0x897 of the table): the kept word back, the tune.
fn finish(
    menus: &Menus,
    race: &Race,
    step: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let kept = &menus.screens.flow.kept;
    kept.set(race.shared.state.settings.kept.get());
    step(Call::PlayTune3)
}

/// The debug UI's test races: the players' records into the race's settings as a race does
/// (the HUD, the keys, the set-up, ...), without their damage and wear.
#[cfg(debug_ui)]
pub fn player_settings_in(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (menus, race) = (o.menus, o.race);
    let results = &race.shared.state.results.results;
    let (first_side, current_side) = (first_side(race), current_side(race));
    // As `single_rally`'s start: the last race's outcome, else the car starts out of it.
    results[0].result.set(0);
    results[1].result.set(0);
    let first = player(menus, 1)?.get();
    take_controls(&first, first_side.car);
    take_keys(&first, &first_side.car.keys);
    let current = player(menus, menus.screens.event.current_player.get())?.get();
    take_player(&current, &current_side, false);
    take_keys(&current, &current_side.car.keys);
    Ok(())
}

/// The original's body; `step(call)` does the call `call`.
fn single_rally(
    o: crate::owners::Owners<'static>,
    start: u32,
    kind: u32,
    step: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let (race, menus, system) = (o.race, o.menus, o.system);
    let (settings, results) = (
        &race.shared.state.settings,
        &race.shared.state.results.results,
    );
    let (event, current) = (&menus.screens.event, &menus.screens.event.current_player);
    let switch = &menus.play.arcade.scene_let_go;
    let (first_side, current_side) = (first_side(race), current_side(race));
    // Closed on a menu whose item leads here (a window's close, MENU + START): the screen is
    // left as if chosen, with no stage yet. Not the original's, whose close stopped nothing.
    if system.session.flags.quitting.is_set() {
        crate::standalone::leave_on_close(o);
    }
    if kind == 3 {
        step(Call::CopyRallyTimes)?;
    }
    settings.rivals.set(menus.play.champ_view.summary.get());
    switch.set(start);
    settings.run_kind.set(kind);
    results[0].result.set(0);
    results[1].result.set(0);
    settings.kept.set(0);
    settings.race_mode.set(1);
    step(Call::StopMusic)?;
    step(Call::SetRaceScreenMode)?;
    step(Call::ShowLoadingPicture)?;
    race.effects_settings()
        .after_loading_copy
        .set(event.sparks_mode.get());

    // The stage's entry in the track list: the tracks, then the pack's closing -1.
    let stage = settings.stage.get();
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
    settings.track_1c.set(entry.temperature.get());
    settings.track_20.set(entry.wind.get());
    for (from, to) in settings_to_race(menus, race) {
        to.set(from.get());
    }
    let first = player(menus, 1)?.get();
    take_controls(&first, first_side.car);
    take_keys(&first, &first_side.car.keys);

    let one_car = matches!(event.event_kind.get(), 5 | 4 | 0xF | 0xA);
    if one_car {
        settings.kept.set(menus.screens.flow.kept.get());
        settings.race_mode.set(2);
        if first.entry != TAKES_PART {
            results[1].result.set(u32::MAX);
        }
        take_player(&first, &first_side, true);
        take_block(&first, first_side.result);
        first_side.car.view.set(first.second_view);
        current.set(0);
    } else if player(menus, current.get())?.get().entry != TAKES_PART {
        // The current player's record is not in use: no race.
        results[0].result.set(u32::MAX);
        return finish(menus, race, step);
    }

    // The current player's record (player 0 after an event's one-car race).
    let rec = player(menus, current.get())?.get();
    if rec.entry != TAKES_PART {
        results[0].result.set(u32::MAX);
    }
    take_player(&rec, &current_side, true);
    take_block(&rec, current_side.result);
    if settings.race_mode.get() == 2 {
        settings.cars[0].view.set(rec.second_view);
    }
    take_keys(&rec, &current_side.car.keys);

    // The race.
    settings.no_replay.set(0);
    if !one_car && event.player_count.get() != 1 {
        settings.no_replay.set(1);
    }
    step(Call::SetUpRaceSoundObjects)?;
    step(Call::SetUpRaceSounds)?;
    step(Call::RaceFromMenu)?;
    step(Call::StopMusic)?;
    step(Call::SetMenuScreenMode)?;
    if switch.get() != 1 {
        step(Call::ShowLoadingPicture)?;
    }
    step(Call::ClearRaceSoundObjects)?;
    step(Call::ApplySettings)?;

    // The outcome back into the records.
    if one_car {
        player(menus, 1)?.update(|p| {
            give_player(p, &first_side);
            give_outcome(p, first_side.result);
        });
        current.set(0);
    }
    player(menus, current.get())?.update(|p| {
        give_player(p, &current_side);
        give_outcome(p, current_side.result);
    });

    if switch.get() != 1 {
        step(Call::SetUpEngine)?;
        step(Call::SetUpFrontEndScene)?;
        step(Call::SetUpEngine)?;
    }
    step(Call::TakeKey)?;
    finish(menus, race, step)
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

/// Does one of the race's calls.
pub fn step(o: crate::owners::Owners<'static>, call: Call) -> Result<(), Refused> {
    let (sound, race) = (o.sound, o.race);
    match call {
        Call::CopyRallyTimes => crate::race::rally::copy_rally_times(o.system, o.menus)?,
        Call::StopMusic => crate::sound::stop_music::stop_music(o.audio, o.system),
        Call::SetRaceScreenMode => crate::race::race_shell::set_race_screen_mode(o.gfx),
        Call::ShowLoadingPicture => {
            crate::menu::show_loading_picture::show_loading_picture_in(o.menus, o.gfx, o.system)?;
        }
        Call::SetUpRaceSoundObjects => crate::race::sound_tables::set_up_race_sound_objects(
            race,
            &crate::race::sound_tables::ObjectVars::of(sound),
            sound,
            o.audio,
            o.net,
            o.system,
        )?,
        Call::SetUpRaceSounds => crate::sound::set_up_race_sounds::set_up_race_sounds(o)?,
        Call::RaceFromMenu => crate::race::race_end::race_from_menu(o)?,
        Call::SetMenuScreenMode => {
            crate::platform::set_menu_mode_now(o.gfx);
        }
        Call::ClearRaceSoundObjects => {
            crate::race::sound_tables::clear_race_sound_objects(sound, o.audio, o.system)?;
        }
        Call::ApplySettings => crate::sound::apply_settings::apply_settings(o)?,
        Call::SetUpEngine => crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?,
        Call::SetUpFrontEndScene => {
            crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
        }
        Call::TakeKey => {
            crate::input::take_key::take_key();
        }
        Call::PlayTune3 => crate::sound::play_tune_3::play_tune_3(o.sound, o.audio, o.system),
    }
    Ok(())
}

/// The plain version against the outputs of the original's instructions (the table an
/// interpreter ran), recorded (`test_support::race_vectors`): for each case the result,
/// the calls in order with the memory at each, and the memory left.
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::menu::owner::menus;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::{Snapshot, store};

    /// The memory the function and its calls' stand-ins read and write: (store, offset,
    /// bytes), in the order of the original's addresses.
    const RANGES: [(&str, usize, usize); 247] = [
        ("menu.owner.MENUS.screens.event", 0x18, 0x4),
        ("menu.owner.MENUS.screens.event", 0xC0, 0x4),
        ("menu.owner.MENUS.screens.event", 0xC4, 0x4),
        ("menu.owner.MENUS.screens.event", 0xC8, 0x4),
        ("menu.owner.MENUS.screens.flow", 0x8, 0x4),
        ("menu.owner.MENUS.screens.option_work", 0x10, 0x4),
        ("menu.owner.MENUS.screens.option_work", 0x1C, 0x4),
        ("menu.owner.MENUS.screens.option_work", 0x20, 0x4),
        ("menu.owner.MENUS.screens.option_work", 0x24, 0x4),
        ("menu.owner.MENUS.screens.option_work", 0x28, 0x4),
        ("menu.owner.MENUS.play.players", 0x0, 0x28C),
        ("menu.owner.MENUS.play.players", 0x28C, 0x28C),
        ("menu.owner.MENUS.play.players", 0x518, 0x28C),
        ("menu.owner.MENUS.play.players", 0x7A4, 0x28C),
        ("menu.owner.MENUS.play.players", 0xA30, 0x10),
        ("menu.owner.MENUS.play.arcade", 0x0, 0x4),
        ("menu.owner.MENUS.play.champ_view", 0x40, 0x4),
        ("race.owner.RACE.replay.viewer_and_kinds", 0x6B4, 0x4),
        ("race.owner.RACE.shared.state", 0x4, 0x4),
        ("race.owner.RACE.shared.state", 0x8, 0x4),
        ("race.owner.RACE.shared.state", 0xC, 0x4),
        ("race.owner.RACE.shared.state", 0x10, 0x4),
        ("race.owner.RACE.shared.state", 0x14, 0x4),
        ("race.owner.RACE.shared.state", 0x18, 0x4),
        ("race.owner.RACE.shared.state", 0x1C, 0x4),
        ("race.owner.RACE.shared.state", 0x20, 0x4),
        ("race.owner.RACE.shared.state", 0x2C, 0x4),
        ("race.owner.RACE.shared.state", 0x30, 0x4),
        ("race.owner.RACE.shared.state", 0x34, 0x4),
        ("race.owner.RACE.shared.state", 0x40, 0x4),
        ("race.owner.RACE.shared.state", 0x44, 0x4),
        ("race.owner.RACE.shared.state", 0x48, 0x4),
        ("race.owner.RACE.shared.state", 0x4C, 0x4),
        ("race.owner.RACE.shared.state", 0x50, 0x4),
        ("race.owner.RACE.shared.state", 0x54, 0x4),
        ("race.owner.RACE.shared.state", 0x58, 0x4),
        ("race.owner.RACE.shared.state", 0x5C, 0x4),
        ("race.owner.RACE.shared.state", 0x60, 0x4),
        ("race.owner.RACE.shared.state", 0x64, 0x4),
        ("race.owner.RACE.shared.state", 0x68, 0x4),
        ("race.owner.RACE.shared.state", 0x6C, 0x4),
        ("race.owner.RACE.shared.state", 0x70, 0x4),
        ("race.owner.RACE.shared.state", 0x74, 0x4),
        ("race.owner.RACE.shared.state", 0x78, 0x4),
        ("race.owner.RACE.shared.state", 0x7C, 0x4),
        ("race.owner.RACE.shared.state", 0x80, 0x4),
        ("race.owner.RACE.shared.state", 0x84, 0x4),
        ("race.owner.RACE.shared.state", 0x88, 0x4),
        ("race.owner.RACE.shared.state", 0x8C, 0x4),
        ("race.owner.RACE.shared.state", 0x90, 0x4),
        ("race.owner.RACE.shared.state", 0x94, 0x4),
        ("race.owner.RACE.shared.state", 0x98, 0x4),
        ("race.owner.RACE.shared.state", 0x9C, 0x4),
        ("race.owner.RACE.shared.state", 0xA0, 0x4),
        ("race.owner.RACE.shared.state", 0xA4, 0x4),
        ("race.owner.RACE.shared.state", 0xA8, 0x4),
        ("race.owner.RACE.shared.state", 0xAC, 0x4),
        ("race.owner.RACE.shared.state", 0xB0, 0x4),
        ("race.owner.RACE.shared.state", 0xB4, 0x4),
        ("race.owner.RACE.shared.state", 0xB8, 0x4),
        ("race.owner.RACE.shared.state", 0xBC, 0x4),
        ("race.owner.RACE.shared.state", 0xC0, 0x4),
        ("race.owner.RACE.shared.state", 0xC4, 0x4),
        ("race.owner.RACE.shared.state", 0xC8, 0x4),
        ("race.owner.RACE.shared.state", 0xCC, 0x4),
        ("race.owner.RACE.shared.state", 0xD0, 0x4),
        ("race.owner.RACE.shared.state", 0xD4, 0x4),
        ("race.owner.RACE.shared.state", 0xD8, 0x4),
        ("race.owner.RACE.shared.state", 0xDC, 0x4),
        ("race.owner.RACE.shared.state", 0xE0, 0x4),
        ("race.owner.RACE.shared.state", 0xE4, 0x4),
        ("race.owner.RACE.shared.state", 0xE8, 0x4),
        ("race.owner.RACE.shared.state", 0xEC, 0x4),
        ("race.owner.RACE.shared.state", 0xF0, 0x4),
        ("race.owner.RACE.shared.state", 0xF4, 0x4),
        ("race.owner.RACE.shared.state", 0xF8, 0x4),
        ("race.owner.RACE.shared.state", 0xFC, 0x4),
        ("race.owner.RACE.shared.state", 0x100, 0x4),
        ("race.owner.RACE.shared.state", 0x104, 0x4),
        ("race.owner.RACE.shared.state", 0x108, 0x4),
        ("race.owner.RACE.shared.state", 0x10C, 0x4),
        ("race.owner.RACE.shared.state", 0x110, 0x4),
        ("race.owner.RACE.shared.state", 0x114, 0x4),
        ("race.owner.RACE.shared.state", 0x118, 0x4),
        ("race.owner.RACE.shared.state", 0x11C, 0x4),
        ("race.owner.RACE.shared.state", 0x120, 0x4),
        ("race.owner.RACE.shared.state", 0x124, 0x4),
        ("race.owner.RACE.shared.state", 0x128, 0x4),
        ("race.owner.RACE.shared.state", 0x12C, 0x4),
        ("race.owner.RACE.shared.state", 0x130, 0x4),
        ("race.owner.RACE.shared.state", 0x134, 0x4),
        ("race.owner.RACE.shared.state", 0x138, 0x4),
        ("race.owner.RACE.shared.state", 0x13C, 0x4),
        ("race.owner.RACE.shared.state", 0x140, 0x4),
        ("race.owner.RACE.shared.state", 0x144, 0x4),
        ("race.owner.RACE.shared.state", 0x148, 0x4),
        ("race.owner.RACE.shared.state", 0x14C, 0x4),
        ("race.owner.RACE.shared.state", 0x150, 0x4),
        ("race.owner.RACE.shared.state", 0x154, 0x4),
        ("race.owner.RACE.shared.state", 0x158, 0x4),
        ("race.owner.RACE.shared.state", 0x15C, 0x4),
        ("race.owner.RACE.shared.state", 0x160, 0x4),
        ("race.owner.RACE.shared.state", 0x164, 0x4),
        ("race.owner.RACE.shared.state", 0x168, 0x4),
        ("race.owner.RACE.shared.state", 0x16C, 0x4),
        ("race.owner.RACE.shared.state", 0x170, 0x4),
        ("race.owner.RACE.shared.state", 0x174, 0x4),
        ("race.owner.RACE.shared.state", 0x178, 0x4),
        ("race.owner.RACE.shared.state", 0x180, 0x4),
        ("race.owner.RACE.shared.state", 0x184, 0x4),
        ("race.owner.RACE.shared.state", 0x188, 0x4),
        ("race.owner.RACE.shared.state", 0x18C, 0x4),
        ("race.owner.RACE.shared.state", 0x190, 0x4),
        ("race.owner.RACE.shared.state", 0x194, 0x4),
        ("race.owner.RACE.shared.state", 0x198, 0x4),
        ("race.owner.RACE.shared.state", 0x19C, 0x4),
        ("race.owner.RACE.shared.state", 0x1A0, 0x4),
        ("race.owner.RACE.shared.state", 0x1A8, 0x4),
        ("race.owner.RACE.shared.state", 0x1AC, 0x4),
        ("race.owner.RACE.shared.state", 0x1B0, 0x1),
        ("race.owner.RACE.shared.state", 0x1B1, 0x1),
        ("race.owner.RACE.shared.state", 0x1B2, 0x1),
        ("race.owner.RACE.shared.state", 0x1B3, 0x1),
        ("race.owner.RACE.shared.state", 0x1B4, 0x1),
        ("race.owner.RACE.shared.state", 0x1B5, 0x1),
        ("race.owner.RACE.shared.state", 0x1B6, 0x1),
        ("race.owner.RACE.shared.state", 0x1B7, 0x1),
        ("race.owner.RACE.shared.state", 0x1B8, 0x1),
        ("race.owner.RACE.shared.state", 0x1B9, 0x1),
        ("race.owner.RACE.shared.state", 0x1BA, 0x1),
        ("race.owner.RACE.shared.state", 0x1BB, 0x1),
        ("race.owner.RACE.shared.state", 0x1BC, 0x4),
        ("race.owner.RACE.shared.state", 0x1C0, 0x4),
        ("race.owner.RACE.shared.state", 0x1C4, 0x4),
        ("race.owner.RACE.shared.state", 0x1C8, 0x4),
        ("race.owner.RACE.shared.state", 0x1CC, 0x4),
        ("race.owner.RACE.shared.state", 0x1D0, 0x4),
        ("race.owner.RACE.shared.state", 0x1D4, 0x4),
        ("race.owner.RACE.shared.state", 0x1D8, 0x4),
        ("race.owner.RACE.shared.state", 0x1DC, 0x4),
        ("race.owner.RACE.shared.state", 0x1E0, 0x4),
        ("race.owner.RACE.shared.state", 0x1E4, 0x4),
        ("race.owner.RACE.shared.state", 0x1E8, 0x4),
        ("race.owner.RACE.shared.state", 0x1EC, 0x4),
        ("race.owner.RACE.shared.state", 0x1F0, 0x4),
        ("race.owner.RACE.shared.state", 0x1F4, 0x4),
        ("race.owner.RACE.shared.state", 0x1F8, 0x4),
        ("race.owner.RACE.shared.state", 0x1FC, 0x4),
        ("race.owner.RACE.shared.state", 0x200, 0x4),
        ("race.owner.RACE.shared.state", 0x204, 0x4),
        ("race.owner.RACE.shared.state", 0x208, 0x4),
        ("race.owner.RACE.shared.state", 0x20C, 0x4),
        ("race.owner.RACE.shared.state", 0x210, 0x4),
        ("race.owner.RACE.shared.state", 0x214, 0x4),
        ("race.owner.RACE.shared.state", 0x218, 0x4),
        ("race.owner.RACE.shared.state", 0x21C, 0x4),
        ("race.owner.RACE.shared.state", 0x220, 0x4),
        ("race.owner.RACE.shared.state", 0x224, 0x4),
        ("race.owner.RACE.shared.state", 0x228, 0x4),
        ("race.owner.RACE.shared.state", 0x22C, 0x4),
        ("race.owner.RACE.shared.state", 0x230, 0x4),
        ("race.owner.RACE.shared.state", 0x234, 0x4),
        ("race.owner.RACE.shared.state", 0x238, 0x4),
        ("race.owner.RACE.shared.state", 0x23C, 0x4),
        ("race.owner.RACE.shared.state", 0x240, 0x4),
        ("race.owner.RACE.shared.state", 0x244, 0x4),
        ("race.owner.RACE.shared.state", 0x248, 0x4),
        ("race.owner.RACE.shared.state", 0x24C, 0x4),
        ("race.owner.RACE.shared.state", 0x250, 0x4),
        ("race.owner.RACE.shared.state", 0x254, 0x4),
        ("race.owner.RACE.shared.state", 0x258, 0x4),
        ("race.owner.RACE.shared.state", 0x25C, 0x4),
        ("race.owner.RACE.shared.state", 0x260, 0x4),
        ("race.owner.RACE.shared.state", 0x264, 0x4),
        ("race.owner.RACE.shared.state", 0x268, 0x4),
        ("race.owner.RACE.shared.state", 0x26C, 0x4),
        ("race.owner.RACE.shared.state", 0x270, 0x4),
        ("race.owner.RACE.shared.state", 0x274, 0x4),
        ("race.owner.RACE.shared.state", 0x278, 0x4),
        ("race.owner.RACE.shared.state", 0x27C, 0x4),
        ("race.owner.RACE.shared.state", 0x280, 0x4),
        ("race.owner.RACE.shared.state", 0x284, 0x4),
        ("race.owner.RACE.shared.state", 0x288, 0x4),
        ("race.owner.RACE.shared.state", 0x28C, 0x4),
        ("race.owner.RACE.shared.state", 0x290, 0x4),
        ("race.owner.RACE.shared.state", 0x294, 0x4),
        ("race.owner.RACE.shared.state", 0x298, 0x4),
        ("race.owner.RACE.shared.state", 0x29C, 0x4),
        ("race.owner.RACE.shared.state", 0x2A0, 0x4),
        ("race.owner.RACE.shared.state", 0x2A4, 0x4),
        ("race.owner.RACE.shared.state", 0x2A8, 0x4),
        ("race.owner.RACE.shared.state", 0x2AC, 0x4),
        ("race.owner.RACE.shared.state", 0x2B0, 0x4),
        ("race.owner.RACE.shared.state", 0x2B4, 0x4),
        ("race.owner.RACE.shared.state", 0x2B8, 0x4),
        ("race.owner.RACE.shared.state", 0x2BC, 0x4),
        ("race.owner.RACE.shared.state", 0x2C0, 0x4),
        ("race.owner.RACE.shared.state", 0x2C4, 0x4),
        ("race.owner.RACE.shared.state", 0x2C8, 0x4),
        ("race.owner.RACE.shared.state", 0x2CC, 0x4),
        ("race.owner.RACE.shared.state", 0x2D0, 0x4),
        ("race.owner.RACE.shared.state", 0x2D4, 0x4),
        ("race.owner.RACE.shared.state", 0x2D8, 0x4),
        ("race.owner.RACE.shared.state", 0x2DC, 0x4),
        ("race.owner.RACE.shared.state", 0x2E0, 0x4),
        ("race.owner.RACE.shared.state", 0x2E4, 0x4),
        ("race.owner.RACE.shared.state", 0x2EC, 0x4),
        ("race.owner.RACE.shared.state", 0x2F0, 0x4),
        ("race.owner.RACE.shared.state", 0x2F4, 0x4),
        ("race.owner.RACE.shared.state", 0x2F8, 0x4),
        ("race.owner.RACE.shared.state", 0x2FC, 0x4),
        ("race.owner.RACE.shared.state", 0x300, 0x4),
        ("race.owner.RACE.shared.state", 0x304, 0x4),
        ("race.owner.RACE.shared.state", 0x308, 0x4),
        ("race.owner.RACE.shared.state", 0x30C, 0x4),
        ("race.owner.RACE.shared.state", 0x314, 0x4),
        ("race.owner.RACE.shared.state", 0x318, 0x4),
        ("race.owner.RACE.shared.state", 0x31C, 0x1),
        ("race.owner.RACE.shared.state", 0x31D, 0x1),
        ("race.owner.RACE.shared.state", 0x31E, 0x1),
        ("race.owner.RACE.shared.state", 0x31F, 0x1),
        ("race.owner.RACE.shared.state", 0x320, 0x1),
        ("race.owner.RACE.shared.state", 0x321, 0x1),
        ("race.owner.RACE.shared.state", 0x322, 0x1),
        ("race.owner.RACE.shared.state", 0x323, 0x1),
        ("race.owner.RACE.shared.state", 0x324, 0x1),
        ("race.owner.RACE.shared.state", 0x325, 0x1),
        ("race.owner.RACE.shared.state", 0x326, 0x1),
        ("race.owner.RACE.shared.state", 0x327, 0x1),
        ("race.owner.RACE.shared.state", 0x338, 0x4),
        ("race.owner.RACE.shared.state", 0x33C, 0x4),
        ("race.owner.RACE.shared.state", 0x340, 0x4),
        ("race.owner.RACE.shared.state", 0x344, 0x4),
        ("race.owner.RACE.shared.state", 0x348, 0x4),
        ("race.owner.RACE.shared.state", 0x34C, 0x4),
        ("race.owner.RACE.shared.state", 0x350, 0x4),
        ("race.owner.RACE.shared.state", 0x354, 0x4),
        ("race.owner.RACE.shared.state", 0x358, 0x4),
        ("race.owner.RACE.shared.state", 0x35C, 0x4),
        ("race.owner.RACE.shared.state", 0x360, 0x40),
        ("race.owner.RACE.shared.state", 0x3A0, 0x20),
        ("race.owner.RACE.shared.state", 0x3C0, 0x4),
        ("race.owner.RACE.shared.state", 0x3C4, 0x4),
        ("race.owner.RACE.shared.state", 0x3C8, 0x4),
        ("race.owner.RACE.shared.state", 0x3CC, 0x4),
        ("race.owner.RACE.shared.state", 0x3D0, 0x40),
        ("race.owner.RACE.shared.state", 0x410, 0x20),
    ];

    /// Where `g_players` is in its store.
    const POINTERS_AT: usize =
        core::mem::offset_of!(crate::menu::players_store::PlayerTable, pointers);

    /// The stores the test writes.
    const STORES: [&str; 9] = [
        "menu.owner.MENUS.play.arcade",
        "menu.owner.MENUS.play.champ_view",
        "menu.owner.MENUS.play.players",
        "menu.owner.MENUS.screens.event",
        "menu.owner.MENUS.screens.flow",
        "menu.owner.MENUS.screens.option_work",
        "race.owner.RACE.replay.viewer_and_kinds",
        "race.owner.RACE.shared.state",
        "system.owner.SYSTEM.records.stage_tables",
    ];

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
        log: Vec<(u32, u64)>,
        rng: Rng,
    }

    /// The ranges' bytes, hashed.
    fn digest() -> u64 {
        crate::test_support::first_values::hash(&ranges_bytes())
    }

    /// The ranges' bytes; `g_players` (read by nothing) holds the records' addresses, which
    /// differ from run to run: each such dword as the record's number.
    fn ranges_bytes() -> Vec<u8> {
        let players = &menus().play.players;
        let mut out = Vec::new();
        for (name, at, len) in RANGES {
            let mut bytes = vec![0; len];
            store(name).load(at, &mut bytes).unwrap();
            if name == "menu.owner.MENUS.play.players" && at == POINTERS_AT {
                for word in bytes.as_chunks_mut::<4>().0 {
                    let v = u32::from_le_bytes(*word);
                    if let Some(k) = players.players.iter().position(|r| r.addr() == v) {
                        word.copy_from_slice(&(k as u32).to_le_bytes());
                    }
                }
            }
            out.extend_from_slice(&bytes);
        }
        out
    }

    impl Calls<'_> {
        fn call(&mut self, place: u32) -> Result<(), Refused> {
            let c = self.case;
            self.log.push((place, digest()));
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
                    let state = store("race.owner.RACE.shared.state");
                    for at in (0x34..0x430).step_by(4) {
                        state.store(at, &self.rng.next().to_le_bytes()).unwrap();
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

    /// Puts the case's memory in place: the ranges' bytes random, then what the case says.
    fn setup(c: &Case) {
        let play = &menus().play;
        let mut rng = Rng(c.seed | 1);
        for (name, at, len) in RANGES {
            let bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
            store(name).store(at, &bytes).unwrap();
        }
        for (k, r) in play.players.players.iter().enumerate() {
            play.players.pointers[k].set(r.addr());
            set_field(r, 0xD0, c.d0[k]).expect("the record is there");
        }
        EVENT_MODE.set(c.mode);
        PLAYER_COUNT.set(c.count);
        CURRENT.set(c.current);
        // The track list: the stage's number in its entry; -1 at the start of the entry the
        // list ends at.
        let tracks = &system().records.stage_tables.trak_pak.tracks;
        for (k, track) in tracks.iter().take(6).enumerate() {
            let mut id = rng.next();
            if id == c.stage {
                id ^= 1;
            }
            track.stage.set(id);
            if k > 0 {
                let mut s = rng.next();
                if s == u32::MAX {
                    s = 0;
                }
                track.name.set(s);
            }
        }
        if c.found {
            tracks[c.entry].stage.set(c.stage);
        } else {
            tracks[c.entry].name.set(u32::MAX);
        }
    }

    /// A case's outcome as words: the result (0 done, 1 a fatal error with its code and its
    /// message's hash, 2 the injected failure), the calls (their count and a hash of the
    /// places with the memory at each) and the hash of the memory left.
    fn outcome(result: &Result<(), Refused>, log: &[(u32, u64)]) -> [u32; 8] {
        let (kind, code, message) = match result {
            Ok(()) => (0, 0, 0),
            Err(Refused::Fatal(f)) => {
                let h = crate::test_support::first_values::hash(f.message.as_bytes());
                (1, f.code, h as u32)
            }
            Err(Refused::BadInput(BadInput("injected"))) => (2, 0, 0),
            Err(e) => panic!("an unexpected refusal: {e:?}"),
        };
        let mut calls = Vec::new();
        for &(place, memory) in log {
            calls.extend_from_slice(&place.to_le_bytes());
            calls.extend_from_slice(&memory.to_le_bytes());
        }
        let calls = crate::test_support::first_values::hash(&calls);
        let left = digest();
        [
            kind,
            code,
            message,
            log.len() as u32,
            calls as u32,
            (calls >> 32) as u32,
            left as u32,
            (left >> 32) as u32,
        ]
    }

    fn run_new(c: &Case) -> (Result<(), Refused>, Vec<(u32, u64)>) {
        let mut calls = Calls {
            case: c,
            log: Vec::new(),
            rng: Rng(c.seed ^ 0x55AA),
        };
        let result = guarded(crate::owners::Owners::the(), c.start, c.kind, &mut |call| {
            calls.call(call as u32)
        });
        (result, calls.log)
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

    /// The plain version gives the original's result, calls with the memory at each, and
    /// memory left, case by case.
    #[test]
    fn single_rally_is_the_originals() {
        let _g = crate::store::test_lock();
        let kept = Snapshot::of(&STORES);
        let mut vectors = Vectors::open("race.single_rally.cases", Kind::Bits, 8);
        let (mut done, mut fatal, mut failed, mut engine, mut early, mut results) =
            (0, 0, 0, 0, 0, 0);
        for (n, c) in cases().iter().enumerate() {
            setup(c);
            let (result, log) = run_new(c);
            let got = outcome(&result, &log);
            vectors.check(&got, |a, b| a == b, || format!("case {n}: {c:?}"));
            match result {
                Ok(()) => done += 1,
                Err(Refused::Fatal(_)) => fatal += 1,
                _ => failed += 1,
            }
            let places: Vec<u32> = log.iter().map(|&(p, _)| p).collect();
            engine += places.contains(&11) as u32;
            early += (places.last() == Some(&14) && !places.contains(&5)) as u32;
            results += (places.contains(&7) && c.result == Some(u32::MAX)) as u32;
        }
        kept.restore();
        vectors.finish();
        assert!(
            done > 100 && fatal > 10 && failed > 20 && engine > 20 && early > 5 && results > 10,
            "coverage: {done} {fatal} {failed} {engine} {early} {results}"
        );
    }
}
