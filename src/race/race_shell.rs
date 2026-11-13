//! The race's shell (`race::single_rally_race`, `single_rally`'s race, and `race::run_race`): what runs round a stage's loading and its run. Here its small leaves.
use crate::net::store::NetShared;
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;
use crate::race::owner_shared::RaceShared;
use crate::race::trap::Trap;

pub const KIND_ORDER_LEN: u32 = rc2k_core::race::cars::KIND_ORDER_LEN;

/// `race::release_controls` (0x409425): `Devices.input_command` 2.
pub fn release_controls(input: &crate::input::owner::Input) {
    input.devices.input_command.set(2); // g_inputCommand
}

/// `race::clear_player_absences` (0x449D44): the eight blocks' +0xB0 0.
pub fn clear_player_absences(race: &Race) {
    // Each car block's +0xB0: the player absent.
    for car in &race.car_blocks().cars[..8] {
        car.stop_state.set(0);
    }
}

/// `race::kind_unusable` (0x449D34): carry for a kind past the table or 6; zero as the last `cmp`
/// leaves it (against 0x17 when past, else against 6)..
pub fn kind_unusable(eax: u32) -> (bool, bool) {
    // Converted: `rc2k_core::race::cars::kind_unusable`.
    let f = rc2k_core::race::cars::kind_unusable(eax);
    (f.carry, f.zero)
}

/// `race::index_of_kind` (0x437720): the place of kind `kind` in `kind_order` (the first of
/// equals), 0 for none.
pub fn index_of_kind(shared: &RaceShared, kind: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::cars::index_of_kind`; the table read entry by entry.
    // `kind_order`: 0x22 kinds, but `kind_order` holds 32: the last two are the two dwords
    // after it (`view_share`, `camera_turn_step`), read as the original does.
    let trap = Trap::new();
    let order = (0..KIND_ORDER_LEN).map(|n| {
        trap.keep_read(
            shared
                .tables()
                .dword_from(crate::race::tables_store::Listed::KindOrder, n as usize),
        )
    });
    let place = rc2k_core::race::cars::index_of_kind(kind, order).unwrap_or(0);
    trap.finish()?;
    Ok(place)
}

/// `race::set_race_screen_mode` (0x404830): `gfx::set_screen_mode` (the platform's) with
/// `g_raceScreenMode`.
pub fn set_race_screen_mode(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) {
    crate::platform::set_mode_now(gfx.display.mode.race_mode.get(), gfx, system);
}

// ---- batch 12 ----

use crate::errors::{BadInput, Refused};
use crate::session::PlainDword;

/// `race::release_stage_textures` (0x44E79F): `gfx::release_all_textures` (the platform's; a `call`
/// and a `retn`).
pub fn release_stage_textures(gfx: &crate::gfx::owner::Gfx) {
    crate::platform_3d::release_all_textures(gfx);
}

/// `race::seal_stage_time` (0x41F29D): the first block's time (+0x88, held to 0x22550FF unsigned;
/// its +0xB0 if that is negative) `xor` `dword_4F66F9` and turned right by its low five bits, into
/// the dword at 0x5BECCC.
pub fn seal_stage_time(race: &Race) -> Result<(), BadInput> {
    let replay = &race.replay;
    // Converted: `rc2k_core::race::seal::seal_time`.
    {
        let key = replay.seal_key().get();
        let sealed = rc2k_core::race::seal::seal_time(
            race.car_blocks().cars[0].stop_state.get(),
            || race.car_blocks().cars[0].stage_time.get(),
            key,
        );
        race.shared.replay_file().index.seal.set(sealed);
    }
    Ok(())
}

/// `race::copy_league_table` (0x4650B7): with a source (`RaceState.settings.rivals`), its ten records' +8 into
/// `names` and their sixteen dwords from +0x10 into column n of the sixteen rows at
/// 0x604134.
pub fn copy_league_table(race: &Race) -> Result<(), BadInput> {
    let course = &race.course;
    // Converted: `rc2k_core::race::results::league`.
    use rc2k_core::race::results::league;
    {
        // The standings summary (`settings.rivals` 1 while it is there).
        if race.shared.state.settings.rivals.get() == 0 {
            return Ok(());
        }
        let rows = crate::saves::best_times::SUMMARY.with(|t| t.rows.map(|r| r.dwords()))?;
        // The core reads a row's dwords by offset (0x50 a row).
        let failed = std::cell::Cell::new(false);
        let l = league(|n, off| {
            rows.get(n)
                .and_then(|r| r.get(off as usize / 4))
                .copied()
                .unwrap_or_else(|| {
                    failed.set(true);
                    0
                })
        });
        if failed.get() {
            return Err(BadInput("a league record past the summary"));
        }
        crate::session::write_bits(&course.league().names, l.firsts);
        for (k, row) in l.rows.iter().enumerate() {
            crate::session::write_first(&course.league().rows[10 * k..10 * k + 10], row);
        }
    }
    Ok(())
}

/// A kind's value for `set_start_states`: the record of the kind (held to 0x21) its
/// +0x18 when its limit (`CarSpecs.cars[0].engine_top`) is 0x1B58 or less, else 0.
fn start_value(physics: &Physics, kind: u32) -> u32 {
    use rc2k_core::race::cars::{start_record, start_state_value};
    let record = &physics.car_specs().cars[start_record(kind) as usize];
    start_state_value(record.engine_top.get(), || record.drive_kind.get())
}

/// Whether a kind is one with its flag (`kind_has_views`) set (below 0x22).
fn flagged(race: &Race, kind: u32) -> bool {
    kind < 0x22 && race.kind_tables().kind_has_views[kind as usize].get() != 0 // ViewerAndKinds.kind_tables.kind_has_views[0]: 0x22
}

/// A block's camera view (+0x684) of 2 or 3 made 4.
fn settle_state(view: &PlainDword) {
    if let Some(v) = rc2k_core::race::cars::settled_state(view.get()) {
        view.set(v);
    }
}

/// `race::set_start_states` (0x446D2E): `start_values` / `CE` the start values of the first car's
/// kind (the chosen car's, `player`, with `session` 1) and of `RaceVars.race_car_kinds[1]`; then with
/// `session` 1: `split` 0 and, unless the chosen car's kind is flagged, its block's
/// +0x684 2 or 3 made 4; else `split` 0 (one player) or held to 1 .. 2 (two), and, when the
/// first car's kind is not flagged or there are two players, the first two blocks' +0x684 2 or 3
/// made 4. it keeps the rest). Refuses, with `session` 1, a chosen car past 7 (the original
/// reads past the tables), before anything.
pub fn set_start_states(race: &Race, vars: &NetShared) -> Result<(), Refused> {
    let shared = &race.shared;
    let frame = &race.frame;
    let physics = &race.physics;
    // Converted: `rc2k_core::race::cars::{start_record, start_state_value, held_split,
    // settled_state}`.
    {
        let one = vars.session.get() == 1;
        let chosen: u32 = vars.player.get();
        if one && chosen > 7 {
            return Err(BadInput("a chosen car past the eight").into());
        }
        let first_kind: u32 = if one {
            shared.vars.get().race_car_kinds[chosen as usize].get()
        } else {
            shared.vars.get().race_car_kinds[0].get()
        };
        let v = start_value(physics, first_kind);
        frame.panel_and_overlay.get().panel.start_values[0].set(v);
        let v = start_value(physics, shared.vars.get().race_car_kinds[1].get());
        frame.panel_and_overlay.get().panel.start_values[1].set(v);
        if one {
            shared.vars.get().split.set(0u32);
            let block = shared.race_car(chosen as usize)?;
            if !flagged(race, block.car_kind.get()) {
                settle_state(&block.camera_view);
            }
            return Ok(());
        }
        let two = shared.view.get().players.get() == 2;
        let held = rc2k_core::race::cars::held_split(two, shared.vars.get().split.get() as i32);
        shared.vars.get().split.set(held as u32);
        let kind_0: u32 = race.car_blocks().cars[0].car_kind.get();
        if !flagged(race, kind_0) || shared.view.get().players.get() >= 2 {
            settle_state(&race.car_blocks().cars[0].camera_view);
            settle_state(&race.car_blocks().cars[1].camera_view);
        }
    }
    Ok(())
}

/// `race::store_results_by_mode` (0x449957): by `run_kind`: 2 the first block's +0x8C to
/// `RaceState.results.figure_ffc`, its +0x654 to `RaceState.results.figure_000` and `start_place`, its +0x88 to `RaceState.results.results[0].time`; 0
/// the first two blocks' +0x88 to `RaceState.results.results[0].time` / `RaceState.results.results[1].time` and `g_stageCarTotal` x 100 less
/// each, held to 0 .. 300000, to `RaceState.results.results[0].word` / `RaceState.results.results[1].word`; 1 and 3 the two +0x88s; others
/// nothing.
pub fn store_results_by_mode(race: &Race) {
    let physics = &race.physics;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::cars::margin`.
    use rc2k_core::race::cars::margin;
    match shared.vars.get().run_kind.get() {
        2 => {
            shared
                .state
                .results
                .figure_ffc
                .set(race.car_blocks().cars[0].time_left.get());
            let v = race.car_blocks().cars[0].mode2_place.get();
            shared.state.results.figure_000.set(v);
            shared.tables().start_place.set(v);
            shared.state.results.results[0]
                .time
                .set(race.car_blocks().cars[0].stage_time.get());
        }
        0 => {
            shared.state.results.results[0]
                .time
                .set(race.car_blocks().cars[0].stage_time.get());
            shared.state.results.results[1]
                .time
                .set(race.car_blocks().cars[1].stage_time.get());
            let total = physics.split_tables().stage_car_total.get();
            shared.state.results.results[0]
                .word
                .set(margin(total, race.car_blocks().cars[0].stage_time.get()));
            shared.state.results.results[1]
                .word
                .set(margin(total, race.car_blocks().cars[1].stage_time.get()));
        }
        1 | 3 => {
            shared.state.results.results[0]
                .time
                .set(race.car_blocks().cars[0].stage_time.get());
            shared.state.results.results[1]
                .time
                .set(race.car_blocks().cars[1].stage_time.get());
        }
        _ => {}
    }
}
