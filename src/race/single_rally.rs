//! `race::single_rally_race` (`single_rally`'s race): the chosen track is looked up in the track
//! list, the settings and the players' records are copied into the race variables, the race runs
//! (`race::race_from_menu`, with the sound objects and screen modes set round it), and the
//! results are copied back into the players' records.
//!
//! Plain Rust: copies between a record and the race variables, the calls (`Call`, done by
//! `step`), the track search as a loop, two divisions by 10 and one float-to-integer conversion.
//! The test at the end of the file compares it with recorded results (`cfg(test)` only).

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::owner::Menus;
#[cfg(test)]
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::race::state_store::{PlayerControls, RaceCar, RaceResult};
use crate::session::Bytes;
use crate::session::DwordLike;
#[cfg(test)]
use crate::session::PlainDword;
use rc2k_core::menu::player::{Player, TAKES_PART};

/// A player's record.
type Record<'a> = crate::menu::standings::PlayerCell<'a>;

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
fn summary(o: Owners<'static>) -> &'static PlainDword {
    &o.menus.play.champ_view.summary
}
#[cfg(test)]
fn after_loading(o: Owners<'static>) -> &'static PlainDword {
    &o.menus.screens.event.sparks_mode
}
#[cfg(test)]
fn player_count(o: Owners<'static>) -> &'static PlainDword {
    &o.menus.screens.event.player_count
}
#[cfg(test)]
fn event_mode(o: Owners<'static>) -> &'static PlainDword {
    &o.menus.screens.event.event_kind
}
#[cfg(test)]
fn current(o: Owners<'static>) -> &'static PlainDword {
    &o.menus.screens.event.current_player
}
#[cfg(test)]
fn switch(o: Owners<'static>) -> &'static PlainDword {
    &o.menus.play.arcade.scene_let_go
}
#[cfg(test)]
fn stage(o: Owners<'static>) -> &'static PlainDword {
    &o.race.shared.state.settings.stage
}
#[cfg(test)]
fn race_kept(o: Owners<'static>) -> &'static PlainDword {
    &o.race.shared.state.settings.kept
}
#[cfg(test)]
fn result(o: Owners<'static>) -> &'static PlainDword {
    &o.race.shared.state.results.results[0].result
}
#[cfg(test)]
fn result_2(o: Owners<'static>) -> &'static PlainDword {
    &o.race.shared.state.results.results[1].result
}
/// One player's side of the race variables: the car's settings and the results it leaves.
/// `first_side` is player 1's, `current_side` the current player's.
struct Side<'a> {
    car: &'a RaceCar,
    /// `RaceState.settings.car_kind_current` or `car_kind_first`: the record's `car_number`.
    car_kind: &'a dyn DwordLike,
    /// The record's `service_spent`.
    service_spent: &'a dyn DwordLike,
    /// The record's `state_flags`.
    flags: &'a dyn DwordLike,
    /// `RaceState.results.hud_bits_current` or `hud_bits_first`: the record's `hud_bits`.
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

/// Car `car`'s four rally-time words and the word after them.
#[cfg(test)]
fn rally_time_words(
    o: Owners<'static>,
    car: usize,
) -> ([&'static PlainDword; 4], &'static PlainDword) {
    let wear = &o.race.shared.state.settings.cars[car].wear;
    ([&wear[10], &wear[11], &wear[12], &wear[13]], &wear[14])
}
#[cfg(test)]
fn outcome_first(o: Owners<'static>) -> &'static RaceResult {
    &o.race.shared.state.results.results[1]
}
#[cfg(test)]
fn outcome_current(o: Owners<'static>) -> &'static RaceResult {
    &o.race.shared.state.results.results[0]
}

/// The fatal error of a track number not in the list.
const UNKNOWN_TRACK: FatalError = FatalError {
    code: 0x8073_9281,
    message: "Unknown Shaun TrakNum",
};

#[cfg(test)]
fn set_field(rec: Record, off: u32, v: u32) -> Result<(), BadInput> {
    rec.update(|p| p.store(&[(off as usize, v)]));
    Ok(())
}

/// `players`[n].
pub fn player(menus: &Menus, n: u32) -> Result<Record<'_>, BadInput> {
    crate::menu::player::player(menus, n).ok_or(BadInput("a player past the four"))
}

/// A record's controller settings into a car's: the controller, gearing, sensitivity, axes, mode,
/// dead zones, strength, and the 0x2C bytes of `bindings` as eleven words
/// (`PlayerControls::recorded_bindings`).
fn take_controls(p: &Player, car: &PlayerControls) {
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

/// A record into a side's variables: the car, set-up, HUD choices, state, and (with `condition`)
/// the damage and wear. The car's `wear[9]` (from `damage[12]`) and the first side's `view` (from
/// `view`, then `second_view`) are not written here: a later write replaces them, so only that
/// one is made.
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
    take_controls(p, &car.controls);
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

/// The total a record keeps in `damage[1]`: a tenth of the sum of the car's four times, and a
/// tenth of six times the extra word. A division by 10 cannot fail, so there is no error path.
fn total(car: &RaceCar) -> u32 {
    let sum = car.wear[10]
        .get()
        .wrapping_add(car.wear[11].get())
        .wrapping_add(car.wear[12].get())
        .wrapping_add(car.wear[13].get());
    (sum / 10).wrapping_add(car.wear[14].get().wrapping_mul(6) / 10)
}

/// A side's variables back into its record. The damage words are written from the car's damage
/// first, then `damage[1]` is replaced by the total and `damage[5]`, `[9]`, `[11]` and `[13]` by
/// the wear words 22, 27, 30 and 35; `damage[12]` takes `wear[9]`. The earlier writes of `[13]`
/// are overwritten, as they always were.
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

/// A car's outcome into its record: a result of -1 or -2 sets the time to 0x36EE80 and the entry
/// to 6 (out); any other copies the time, the result word, the seconds rounded times 10
/// (`figure`) and the 0x40 bytes of data (`best_line`).
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

/// `race::single_rally_race`: `start` the run's start (1: the loading picture is not shown again
/// after), `kind` the kind (3: the rally's times copied first). Refuses, before anything, a missing
/// player record (player 1 or `Event.current_player`).
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

/// The end of the race: the kept word put back, then the tune.
fn finish(
    menus: &Menus,
    race: &Race,
    step: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let kept = &menus.screens.flow.kept;
    kept.set(race.shared.state.settings.kept.get());
    step(Call::PlayTune3)
}

/// The debug UI's test races: the players' records into the race's settings as a race does (the
/// HUD, the keys, the set-up, ...), without their damage and wear.
#[cfg(debug_ui)]
pub fn player_settings_in(menus: &Menus, race: &Race) -> Result<(), Refused> {
    let results = &race.shared.state.results.results;
    let (first_side, current_side) = (first_side(race), current_side(race));
    // As `single_rally`'s start: the last race's outcome, else the car starts out of it.
    results[0].result.set(0);
    results[1].result.set(0);
    let first = player(menus, 1)?.get();
    take_controls(&first, &first_side.car.controls);
    take_keys(&first, &first_side.car.controls.keys);
    let current = player(menus, menus.screens.event.current_player.get())?.get();
    take_player(&current, &current_side, false);
    take_keys(&current, &current_side.car.controls.keys);
    Ok(())
}

/// The whole race; `step(call)` does the call `call`.
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
    // Closed on a menu whose item leads here (a window's close, MENU + START): the screen is left
    // as if chosen, with no stage yet.
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
    let pack = &system.records().stage_tables.trak_pak;
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
    // The entry's temperature and wind into the race variables.
    settings.track_1c.set(entry.temperature.get());
    settings.track_20.set(entry.wind.get());
    settings.set_visual_options(&menus.options.snapshot());
    let first = player(menus, 1)?.get();
    take_controls(&first, &first_side.car.controls);
    take_keys(&first, &first_side.car.controls.keys);

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
    take_keys(&rec, &current_side.car.controls.keys);

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

/// The calls into the rest of the game that the race makes, in the order they are numbered in the
/// tests' record of them.
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
        Call::StopMusic => crate::sound::stop_music::stop_music(o),
        Call::SetRaceScreenMode => crate::race::race_shell::set_race_screen_mode(o.gfx, o.system),
        Call::ShowLoadingPicture => {
            crate::menu::show_loading_picture::show_loading_picture_in(o)?;
        }
        Call::SetUpRaceSoundObjects => crate::race::sound_tables::set_up_race_sound_objects(
            o,
            &crate::race::sound_tables::ObjectVars::of(sound),
        )?,
        Call::SetUpRaceSounds => crate::sound::set_up_race_sounds::set_up_race_sounds(o)?,
        Call::RaceFromMenu => crate::race::race_end::race_from_menu(o)?,
        Call::SetMenuScreenMode => {
            crate::platform::set_menu_mode_now(o.gfx, o.system);
        }
        Call::ClearRaceSoundObjects => {
            crate::race::sound_tables::clear_race_sound_objects(o)?;
        }
        Call::ApplySettings => crate::sound::apply_settings::apply_settings(o)?,
        Call::SetUpEngine => crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?,
        Call::SetUpFrontEndScene => {
            crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
        }
        Call::TakeKey => {
            crate::input::take_key::take_key(o.input);
        }
        Call::PlayTune3 => crate::sound::play_tune_3::play_tune_3(o),
    }
    Ok(())
}

/// The plain version against recorded results (`test_support::race_vectors`): for each case the
/// result, the calls in order with the memory at each, and the memory left.
#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::owners::Owners;
    use crate::test_support::race_vectors::{Kind, Vectors};
    use crate::test_support::state::{Snapshot, walk};

    const STATE: &str = "owners.race.shared.state";

    /// The memory the function and its calls' stand-ins read and write: (store, field, byte of the
    /// field's cells to start at, bytes).
    #[rustfmt::skip]
    const RANGES: [(&str, &str, usize, usize); 247] = [
        ("owners.menus.screens.event", ".sparks_mode", 0, 0x4),
        ("owners.menus.screens.event", ".player_count", 0, 0x4),
        ("owners.menus.screens.event", ".event_kind", 0, 0x4),
        ("owners.menus.screens.event", ".current_player", 0, 0x4),
        ("owners.menus.screens.flow", ".kept", 0, 0x4),
        ("owners.menus.options", ".lens_flare", 0, 0x4),
        ("owners.menus.options", ".reflections", 0, 0x4),
        ("owners.menus.options", ".car_shadows", 0, 0x4),
        ("owners.menus.options", ".car_detail", 0, 0x4),
        ("owners.menus.options", ".headlights", 0, 0x4),
        ("owners.menus.play.slots", ".records[0]", 0, 0x28c),
        ("owners.menus.play.slots", ".records[1]", 0, 0x28c),
        ("owners.menus.play.slots", ".records[2]", 0, 0x28c),
        ("owners.menus.play.slots", ".records[3]", 0, 0x28c),
        ("owners.menus.play.slots", ".pointers[0]", 0, 0x10),
        ("owners.menus.play.arcade", ".scene_let_go", 0, 0x4),
        ("owners.menus.play.champ_view", ".summary", 0, 0x4),
        ("owners.race.replay.viewer_and_kinds", ".marks_settings.after_loading_copy", 0, 0x4),
        ("owners.race.shared.state", ".settings.rivals", 0, 0x4),
        ("owners.race.shared.state", ".settings.stage", 0, 0x4),
        ("owners.race.shared.state", ".settings.run_kind", 0, 0x4),
        ("owners.race.shared.state", ".settings.no_replay", 0, 0x4),
        ("owners.race.shared.state", ".settings.race_mode", 0, 0x4),
        ("owners.race.shared.state", ".settings.kept", 0, 0x4),
        ("owners.race.shared.state", ".settings.car_kind_current", 0, 0x4),
        ("owners.race.shared.state", ".settings.car_kind_first", 0, 0x4),
        ("owners.race.shared.state", ".settings.track_1c", 0, 0x4),
        ("owners.race.shared.state", ".settings.track_20", 0, 0x4),
        ("owners.race.shared.state", ".settings.lens_flare", 0, 0x4),
        ("owners.race.shared.state", ".settings.reflections", 0, 0x4),
        ("owners.race.shared.state", ".settings.car_shadows", 0, 0x4),
        ("owners.race.shared.state", ".settings.car_detail", 0, 0x4),
        ("owners.race.shared.state", ".settings.headlights", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.tyre", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.brake_balance", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.ride_height", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.suspension_stiffness", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.steering_sensitivity", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.gear_ratio", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[0]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[1]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[2]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[3]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[4]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[5]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[6]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[7]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[8]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[9]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[10]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[11]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.damage[12]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[0]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[1]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[2]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[3]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[4]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[5]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[6]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[7]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[8]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[9]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[10]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[11]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[12]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[13]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[14]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[15]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[16]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[17]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[18]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[19]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[20]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[21]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[22]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[23]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[24]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[25]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[26]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[27]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[28]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[29]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[30]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[31]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[32]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[33]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[34]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.wear[35]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.map_area", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.rear_mirror", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.rev_counter", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.metric_units", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.message_display", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.car_info", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.progress_bar", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.driver_info", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.pace_notes", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.race_number", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.codriver_sex", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].own.view", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.controller", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.gearing", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.sensitivity_level", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.axis_codes", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.controller_mode", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.end_zone_pct", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.centre_dead_zone_pct", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.strength", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[0]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[1]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[2]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[3]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[4]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[5]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[6]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[7]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[8]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[10]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.bindings[11]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 0, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 1, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 2, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 3, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 4, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 5, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 6, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 7, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 8, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 9, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 10, 0x1),
        ("owners.race.shared.state", ".settings.cars[0].controls.keys", 11, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].own.tyre", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.brake_balance", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.ride_height", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.suspension_stiffness", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.steering_sensitivity", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.gear_ratio", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[0]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[1]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[2]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[3]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[4]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[5]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[6]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[7]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[8]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[9]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[10]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[11]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.damage[12]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[0]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[1]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[2]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[3]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[4]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[5]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[6]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[7]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[8]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[9]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[10]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[11]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[12]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[13]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[14]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[15]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[16]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[17]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[18]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[19]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[20]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[21]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[22]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[23]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[24]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[25]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[26]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[27]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[28]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[29]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[30]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[31]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[32]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[33]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[34]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.wear[35]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.map_area", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.rear_mirror", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.rev_counter", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.metric_units", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.message_display", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.car_info", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.progress_bar", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.driver_info", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.pace_notes", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.race_number", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.codriver_sex", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].own.view", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.controller", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.gearing", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.sensitivity_level", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.axis_codes", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.controller_mode", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.end_zone_pct", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.centre_dead_zone_pct", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.strength", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[0]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[1]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[2]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[3]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[4]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[5]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[6]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[7]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[8]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[10]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.bindings[11]", 0, 0x4),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 0, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 1, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 2, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 3, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 4, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 5, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 6, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 7, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 8, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 9, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 10, 0x1),
        ("owners.race.shared.state", ".settings.cars[1].controls.keys", 11, 0x1),
        ("owners.race.shared.state", ".results.service_spent_current", 0, 0x4),
        ("owners.race.shared.state", ".results.service_spent_first", 0, 0x4),
        ("owners.race.shared.state", ".results.hud_bits_current", 0, 0x4),
        ("owners.race.shared.state", ".results.hud_bits_first", 0, 0x4),
        ("owners.race.shared.state", ".results.flags_current", 0, 0x4),
        ("owners.race.shared.state", ".results.flags_first", 0, 0x4),
        ("owners.race.shared.state", ".results.results[0].time", 0, 0x4),
        ("owners.race.shared.state", ".results.results[0].word", 0, 0x4),
        ("owners.race.shared.state", ".results.results[0].secs", 0, 0x4),
        ("owners.race.shared.state", ".results.results[0].result", 0, 0x4),
        ("owners.race.shared.state", ".results.results[0].data", 0, 0x40),
        ("owners.race.shared.state", ".results.results[0].impact_marks[0]", 0, 0x20),
        ("owners.race.shared.state", ".results.results[1].time", 0, 0x4),
        ("owners.race.shared.state", ".results.results[1].word", 0, 0x4),
        ("owners.race.shared.state", ".results.results[1].secs", 0, 0x4),
        ("owners.race.shared.state", ".results.results[1].result", 0, 0x4),
        ("owners.race.shared.state", ".results.results[1].data", 0, 0x40),
        ("owners.race.shared.state", ".results.results[1].impact_marks[0]", 0, 0x20),
    ];

    /// The stores the test writes.
    const STORES: [&str; 9] = [
        "owners.menus.play.arcade",
        "owners.menus.play.champ_view",
        "owners.menus.play.slots",
        "owners.menus.screens.event",
        "owners.menus.screens.flow",
        "owners.menus.options",
        "owners.race.replay.viewer_and_kinds",
        "owners.race.shared.state",
        "owners.system.records.stage_tables",
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
        /// Records `players` has no entry for (0).
        stage: u32,
        /// The entry of the track list holding the stage, or the entry the list ends at.
        entry: usize,
        found: bool,
        result: Option<u32>,
        result_2: Option<u32>,
        secs: u32,
        secs_2: u32,
        /// What the call at place 9 leaves in `switch()`.
        switch_after: Option<u32>,
        /// How the race leaves the sums of times: 0 random, 1 zero, 2 wrapping to zero, 3 small.
        sums: u32,
        sums_2: u32,
        extra: u32,
        extra_2: u32,
        fail_at: Option<u32>,
        seed: u64,
    }

    /// The calls' stand-ins: they log (place, the memory then), may fail, and write what the real
    /// calls would change that the function reads after.
    struct Calls<'a> {
        o: Owners<'static>,
        case: &'a Case,
        log: Vec<(u32, u64)>,
        rng: Rng,
    }

    /// The ranges' bytes, hashed.
    fn digest() -> u64 {
        crate::test_support::hash(&ranges_bytes())
    }

    /// The ranges' bytes (`players`, read by nothing, holds the records' numbers: the cases set
    /// them so).
    fn ranges_bytes() -> Vec<u8> {
        let mut out = Vec::new();
        for (name, path, skip, len) in RANGES {
            out.extend_from_slice(&read_range(name, path, skip, len));
        }
        out
    }

    /// `len` bytes of the cells from byte `skip` of the field `path` of the store `name`.
    fn read_range(name: &str, path: &str, skip: usize, len: usize) -> Vec<u8> {
        let mut cells = walk(name).range_bytes(path, skip + len);
        assert_eq!(cells.len(), skip + len, "{name}{path}");
        cells.drain(..skip);
        cells
    }

    /// The cells `read_range` names set from `bytes`.
    fn write_range(name: &str, path: &str, skip: usize, bytes: &[u8]) {
        let store = walk(name);
        let mut cells = store.range_bytes(path, skip + bytes.len());
        assert_eq!(cells.len(), skip + bytes.len(), "{name}{path}");
        cells[skip..].copy_from_slice(bytes);
        assert!(store.set_range_bytes(path, &cells), "{name}{path}");
    }

    impl Calls<'_> {
        fn call(&mut self, place: u32) -> Result<(), Refused> {
            let c = self.case;
            let o = self.o;
            self.log.push((place, digest()));
            if c.fail_at == Some(place) {
                return Err(BadInput("injected").into());
            }
            match place {
                1 => summary(o).set(self.rng.next()),
                3 => stage(o).set(c.stage),
                4 => after_loading(o).set(self.rng.next()),
                7 => {
                    // The race changes everything the results are copied from.
                    let (kept, kept_2) = (result(o).get(), result_2(o).get());
                    // The state's dwords from the lens flare setting to the results' stage.
                    let words: Vec<u8> = (0..(0x430 - 0x34) / 4)
                        .flat_map(|_| self.rng.next().to_le_bytes())
                        .collect();
                    write_range(STATE, ".settings.lens_flare", 0, &words);
                    for ((times, extra), sums, e) in [
                        (rally_time_words(o, 1), c.sums, c.extra),
                        (rally_time_words(o, 0), c.sums_2, c.extra_2),
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
                    result(o).set(c.result.unwrap_or(kept));
                    result_2(o).set(c.result_2.unwrap_or(kept_2));
                    outcome_current(o).secs.set(f32::from_bits(c.secs));
                    outcome_first(o).secs.set(f32::from_bits(c.secs_2));
                }
                9 => {
                    if let Some(s) = c.switch_after {
                        switch(o).set(s);
                    }
                }
                13 => race_kept(o).set(self.rng.next()),
                _ => {}
            }
            Ok(())
        }
    }

    /// Puts the case's memory in place: the ranges' bytes random, then what the case says.
    fn setup(o: Owners<'static>, c: &Case) {
        let play = &o.menus.play;
        let mut rng = Rng(c.seed | 1);
        for (name, path, skip, len) in RANGES {
            let bytes: Vec<u8> = (0..len).map(|_| rng.next() as u8).collect();
            write_range(name, path, skip, &bytes);
        }
        for (k, r) in play.slots.records.iter().enumerate() {
            play.slots.pointers[k].set(k as u32);
            set_field(r, 0xD0, c.d0[k]).expect("the record is there");
        }
        event_mode(o).set(c.mode);
        player_count(o).set(c.count);
        current(o).set(c.current);
        // The track list: the stage's number in its entry; -1 at the start of the entry the list
        // ends at.
        let tracks = &o.system.records().stage_tables.trak_pak.tracks;
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
    /// message's hash, 2 the injected failure), the calls (their count and a hash of the places
    /// with the memory at each) and the hash of the memory left.
    fn outcome(result: &Result<(), Refused>, log: &[(u32, u64)]) -> [u32; 8] {
        let (kind, code, message) = match result {
            Ok(()) => (0, 0, 0),
            Err(Refused::Fatal(f)) => {
                let h = crate::test_support::hash(f.message.as_bytes());
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
        let calls = crate::test_support::hash(&calls);
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

    fn run_new(o: Owners<'static>, c: &Case) -> (Result<(), Refused>, Vec<(u32, u64)>) {
        let mut calls = Calls {
            o,
            case: c,
            log: Vec::new(),
            rng: Rng(c.seed ^ 0x55AA),
        };
        let result = guarded(o, c.start, c.kind, &mut |call| calls.call(call as u32));
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

    /// The plain version gives the recorded result, calls with the memory at each, and memory
    /// left, case by case.
    #[test]
    fn single_rally_is_the_originals() {
        let _g = crate::test_support::test_lock();
        let kept = Snapshot::of(&STORES);
        let mut vectors = Vectors::open("race.single_rally.cases", Kind::Bits, 8);
        let (mut done, mut fatal, mut failed, mut engine, mut early, mut results) =
            (0, 0, 0, 0, 0, 0);
        let o = Owners::the();
        for (n, c) in cases().iter().enumerate() {
            setup(o, c);
            let (result, log) = run_new(o, c);
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
