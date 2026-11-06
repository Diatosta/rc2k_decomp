//! The race's shell (`sub_4843EC`, `single_rally`'s race, and `sub_443F0B`, IDA
//! 2026-10-17): what runs round a stage's loading and its run. Here its small leaves.
use crate::input::store::DEVICES;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::kind_store::KIND_TABLES;
use crate::race::owner::race;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::state_store::RACE_STATE;
use crate::race::tables_store::RACE_TABLES;
use crate::race::trap::Trap;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::session::Dword;

pub const KIND_ORDER_LEN: u32 = rc2k_core::race::cars::KIND_ORDER_LEN;

/// `race::release_controls` (0x409425): `dword_4F7DE4` 2. Every register kept.
pub fn release_controls() {
    DEVICES.input_command.set(2); // g_inputCommand
}

/// `race::clear_player_absences` (0x449D44): the eight blocks' +0xB0 0. Every register
/// kept.
pub fn clear_player_absences() {
    // Each car block's +0xB0: the player absent.
    for car in &CAR_BLOCKS.cars[..8] {
        car.stop_state.set(0);
    }
}

/// `race::kind_unusable` (0x449D34): carry for a kind past the table or 6; zero as the
/// last `cmp` leaves it (against 0x17 when past, else against 6). (carry, zero).
pub fn kind_unusable(eax: u32) -> (bool, bool) {
    // Converted: `rc2k_core::race::cars::kind_unusable`.
    let f = rc2k_core::race::cars::kind_unusable(eax);
    (f.carry, f.zero)
}

/// `race::index_of_kind` (0x437720): the place of kind `kind` in `dword_6051E0` (the
/// first of equals), 0 for none.
pub fn index_of_kind(kind: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::race::cars::index_of_kind`; the table read entry by entry.
    // dword_6051E0: 0x22 kinds, but `unknown_ba4` holds 32: the last two are the two dwords
    // after it (`view_share`, `camera_turn_step`), read as the original does.
    // Bounded by the whole store: the last two entries are past the table.
    let first = crate::pointers::GamePtr::to(&RACE_TABLES)
        .offset(core::mem::offset_of!(crate::race::tables_store::RaceTables, unknown_ba4) as u32);
    let trap = Trap::new();
    let order =
        (0..KIND_ORDER_LEN).map(|n| trap.read_at::<u32>(first.offset(4 * n), "the kind order"));
    let place = rc2k_core::race::cars::index_of_kind(kind, order).unwrap_or(0);
    trap.finish()?;
    Ok(place)
}

/// `race::set_race_screen_mode` (0x404830): `gfx::set_screen_mode` (the platform's) with
/// `g_raceScreenMode`; every register and flag kept (`pusha` / `popa`; its carry is read
/// by nobody).
pub fn set_race_screen_mode() {
    crate::platform::set_mode_now(crate::gfx::owner::gfx().display.mode.race_mode.get());
}

// ---- batch 12 ----

use crate::errors::{BadInput, Refused};

/// `race::release_stage_textures` (0x44E79F): `gfx::release_all_textures` (the
/// platform's; a `call` and a `retn`).
pub fn release_stage_textures() {
    crate::platform_3d::release_textures();
}

/// `race::seal_stage_time` (0x41F29D): the first block's time (+0x88, held to
/// 0x22550FF unsigned; its +0xB0 if that is negative) `xor` `dword_4F66F9` and turned
/// right by its low five bits, into the dword at 0x5BECCC. Every register kept.
pub fn seal_stage_time() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::seal::seal_time`.
    {
        let key = crate::race::owner::race().replay.seal_key.get();
        let sealed = rc2k_core::race::seal::seal_time(
            CAR_BLOCKS.cars[0].stop_state.bits(),
            || CAR_BLOCKS.cars[0].stage_time.bits(),
            key,
        );
        REPLAY_FILE.index_seal.set(sealed);
    }
    Ok(())
}

/// `race::copy_league_table` (0x4650B7): with a source (`dword_75ACD0`), its ten records'
/// +8 into `dword_60410C` and their sixteen dwords from +0x10 into column n of the
/// sixteen rows at 0x604134. The registers it leaves are read by no caller (taint at its
/// one call site, 0x4486DB; conversion phase 3, 2026-10-24).
pub fn copy_league_table() -> Result<(), BadInput> {
    // Converted: `rc2k_core::race::results::league`.
    use rc2k_core::race::results::league;
    {
        // The standings summary (`settings.rivals` 1 while it is there).
        if RACE_STATE.settings.rivals.get() == 0 {
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
        crate::session::write_bits(&crate::race::owner::race().course.league.names, l.firsts);
        for (k, row) in l.rows.iter().enumerate() {
            crate::session::write_first(
                &crate::race::owner::race().course.league.rows[10 * k..10 * k + 10],
                row,
            );
        }
    }
    Ok(())
}

/// A kind's value for `set_start_states`: the record of the kind (held to 0x21) its
/// +0x18 when its limit (`dword_5C7980`) is 0x1B58 or less, else 0.
fn start_value(kind: u32) -> u32 {
    use rc2k_core::race::cars::{start_record, start_state_value};
    let record = &crate::race::owner::race().physics.car_specs.cars[start_record(kind) as usize];
    start_state_value(record.engine_top.get(), || record.drive_kind.get())
}

/// Whether a kind is one with its flag (`dword_5C5AC0`) set (below 0x22).
fn flagged(kind: u32) -> bool {
    kind < 0x22 && KIND_TABLES.kind_has_views[kind as usize].get() != 0 // dword_5C5AC0: 0x22
}

/// A block's camera view (+0x684) of 2 or 3 made 4.
fn settle_state(view: &Dword) {
    if let Some(v) = rc2k_core::race::cars::settled_state(view.get()) {
        view.set(v);
    }
}

/// `race::set_start_states` (0x446D2E): `dword_6D14CA` / `CE` the start values of the
/// first car's kind (the chosen car's, `dword_52A920`, with `dword_52A910` 1) and of
/// `dword_62385C`; then with `dword_52A910` 1: `dword_6238A4` 0 and, unless the chosen
/// car's kind is flagged, its block's +0x684 2 or 3 made 4; else `dword_6238A4` 0 (one
/// player) or held to 1 .. 2 (two), and, when the first car's kind is not flagged or
/// there are two players, the first two blocks' +0x684 2 or 3 made 4. The eax and ebx it
/// leaves are read by none of its four callers (taint at each site; it keeps the rest).
/// Refuses, with `dword_52A910` 1, a chosen car past 7 (the original reads past the
/// tables), before anything.
pub fn set_start_states() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::cars::{start_record, start_state_value, held_split,
    // settled_state}`.
    {
        let one = crate::net::store::NET.session.get() == 1;
        let chosen: u32 = crate::net::store::NET.player.get();
        if one && chosen > 7 {
            return Err(BadInput("a chosen car past the eight").into());
        }
        let first_kind: u32 = if one {
            RACE_VARS.race_car_kinds[chosen as usize].get()
        } else {
            RACE_VARS.race_car_kinds[0].bits()
        };
        let v = start_value(first_kind);
        race().frame.panel_and_overlay.panel.start_values[0].set(v);
        let v = start_value(RACE_VARS.race_car_kinds[1].bits());
        race().frame.panel_and_overlay.panel.start_values[1].set(v);
        if one {
            let block: u32 = crate::race::work_store::RACE_WORK.car_block(chosen as usize);
            RACE_VARS.split.set_bits(0u32);
            let block = crate::race::trap::car_at(block)?;
            if !flagged(block.car_kind.get()) {
                settle_state(&block.camera_view);
            }
            return Ok(());
        }
        let two = RACE_VIEW.players.bits() == 2;
        let held = rc2k_core::race::cars::held_split(two, RACE_VARS.split.bits() as i32);
        RACE_VARS.split.set_bits(held as u32);
        let kind_0: u32 = CAR_BLOCKS.cars[0].car_kind.bits();
        if !flagged(kind_0) || RACE_VIEW.players.bits() >= 2 {
            settle_state(&CAR_BLOCKS.cars[0].camera_view);
            settle_state(&CAR_BLOCKS.cars[1].camera_view);
        }
    }
    Ok(())
}

/// `race::store_results_by_mode` (0x449957): by `dword_62389C`: 2 the first block's +0x8C
/// to `dword_75AFFC`, its +0x654 to `dword_75B000` and `dword_6051BC`, its +0x88 to
/// `dword_75B01C`; 0 the first two blocks' +0x88 to `dword_75B01C` / `dword_75B08C` and
/// `g_stageCarTotal` x 100 less each, held to 0 .. 300000, to `dword_75B020` /
/// `dword_75B090` (ebx the stage held to 0 .. 0x4C); 1 and 3 the two +0x88s; others
/// nothing. The registers it leaves are read by no caller (taint at 0x448C02).
pub fn store_results_by_mode() {
    // Converted: `rc2k_core::race::cars::margin`.
    use rc2k_core::race::cars::margin;
    match RACE_VARS.run_kind.bits() {
        2 => {
            RACE_STATE
                .results
                .figure_ffc
                .set(CAR_BLOCKS.cars[0].time_left.get());
            let v = CAR_BLOCKS.cars[0].mode2_place.get();
            RACE_STATE.results.figure_000.set(v);
            RACE_TABLES.start_place.set_bits(v);
            RACE_STATE.results.results[0]
                .time
                .set(CAR_BLOCKS.cars[0].stage_time.get());
        }
        0 => {
            RACE_STATE.results.results[0]
                .time
                .set(CAR_BLOCKS.cars[0].stage_time.get());
            RACE_STATE.results.results[1]
                .time
                .set(CAR_BLOCKS.cars[1].stage_time.get());
            let total = crate::race::owner::race()
                .physics
                .split_tables
                .stage_car_total
                .get();
            RACE_STATE.results.results[0]
                .word
                .set(margin(total, CAR_BLOCKS.cars[0].stage_time.get()));
            RACE_STATE.results.results[1]
                .word
                .set(margin(total, CAR_BLOCKS.cars[1].stage_time.get()));
        }
        1 | 3 => {
            RACE_STATE.results.results[0]
                .time
                .set(CAR_BLOCKS.cars[0].stage_time.get());
            RACE_STATE.results.results[1]
                .time
                .set(CAR_BLOCKS.cars[1].stage_time.get());
        }
        _ => {}
    }
}
