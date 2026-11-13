//! The race's settings copied between the front end's ("Graham's", 0x75ACxx ..) and the
//! race's ("Shaun's") variables and car blocks, and logged: `race::copy_settings_in`
//! (0x446EBE, "COPY GRAHAM TO SHAUN", before a race) and `race::copy_settings_out`
//! (0x4486FB, "COPY SHAUN TO GRAHAM", after it). The original's code is two long runs of
//! the same few kinds of step; here each is a plain function that loops over the two cars
//! and the `GAUGES` table and does the few steps that differ by hand, in the original's
//! order (the log's lines are in that order too). The old step-list machine is kept as a
//! test reference (`old_machine`), against which `new_copies_match_the_machine` compares
//! the cells written and the log text.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::math::x87;
use crate::owners::Owners;
use crate::physics::car_store::Car;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::race::state_store::RaceCar;
#[cfg(test)]
use crate::session::Dword;
use crate::session::DwordLike;
#[cfg(test)]
#[cfg(test)]
mod old_machine;

/// A named cell: one of the physics block's cells, a field of a car's block, or a store's
/// dword (the front end's items and the credits race's settings name theirs by it).
#[derive(Clone, Copy)]
pub enum StepOperand {
    /// A cell of the physics block (the reference machine's tables).
    #[cfg(test)]
    Cell(&'static Dword),
    /// A word of player `n`'s settings in the race's variables (`settings.cars[n]`), by field.
    #[cfg(test)]
    Setting(usize, SettingWord),
    /// A car's result float (`results[n].secs`), by its bits.
    #[cfg(test)]
    ResultSecs(usize),
    /// A car's result impact mark (`results[n].impact_marks[k]`).
    #[cfg(test)]
    ResultMark(usize, usize),
    /// A dword of car block n (`CAR_BLOCKS.cars[n]`), by field (`car!`).
    CarCell(usize, fn(&Car) -> &PlainDword),
    /// A float of car block n, by its bits (`car_bits!`).
    CarBits(usize, fn(&Car) -> &PlainFloat),
    /// A dword of an overlay store on the race block (`var!`: `RACE_VARS.stage_number`).
    #[cfg(test)]
    Var(fn() -> &'static dyn DwordLike),
    /// A dword of one of the owners (`var!(@o menus.screens.option_work.eax)`).
    Owned(for<'a> fn(&'a Owners<'a>) -> &'a dyn DwordLike),
    /// A dword of the race's (`var!(@race shared.vars.direction)`).
    RaceVar(fn(&Race) -> &dyn DwordLike),
}
use crate::session::{PlainDword, PlainFloat};
#[cfg(test)]
use StepOperand::Var;
use StepOperand::{CarBits, CarCell, Owned, RaceVar};
#[cfg(test)]
use StepOperand::{Cell, ResultMark, ResultSecs, Setting};

impl StepOperand {
    /// Its dword (a float's bits), in the game's owners (the tests').
    #[cfg(test)]
    pub fn get(self) -> u32 {
        self.get_with(&Owners::the())
    }

    /// Its dword set (a float's bits), in the game's owners (the tests').
    #[cfg(test)]
    pub fn set(self, v: u32) {
        self.set_with(&Owners::the(), v);
    }

    /// Its dword (a float's bits), the owners' being `o`'s.
    pub fn get_with(self, o: &Owners<'static>) -> u32 {
        match self {
            #[cfg(test)]
            Cell(cell) => cell.get(),
            #[cfg(test)]
            Setting(n, f) => f(&o.race.shared.state.settings.cars[n]).get(),
            #[cfg(test)]
            ResultSecs(n) => o.race.shared.state.results.results[n].secs.bits(),
            #[cfg(test)]
            ResultMark(n, k) => o.race.shared.state.results.results[n].impact_marks[k].get(),
            CarCell(n, f) => f(&o.race.car_blocks().cars[n]).get(),
            CarBits(n, f) => f(&o.race.car_blocks().cars[n]).bits(),
            #[cfg(test)]
            Var(f) => f().get(),
            Owned(f) => f(o).get(),
            RaceVar(f) => f(o.race).get(),
        }
    }

    /// Its dword set (a float's bits), the owners' being `o`'s.
    pub fn set_with(self, o: &Owners<'static>, v: u32) {
        match self {
            #[cfg(test)]
            Cell(cell) => cell.set(v),
            #[cfg(test)]
            Setting(n, f) => f(&o.race.shared.state.settings.cars[n]).set(v),
            #[cfg(test)]
            ResultSecs(n) => o.race.shared.state.results.results[n].secs.set_bits(v),
            #[cfg(test)]
            ResultMark(n, k) => o.race.shared.state.results.results[n].impact_marks[k].set(v),
            CarCell(n, f) => f(&o.race.car_blocks().cars[n]).set(v),
            CarBits(n, f) => f(&o.race.car_blocks().cars[n]).set_bits(v),
            #[cfg(test)]
            Var(f) => f().set(v),
            Owned(f) => f(o).set(v),
            RaceVar(f) => f(o.race).set(v),
        }
    }
}

/// `car!(n, field)`: car block n's dword `field` as a `StepOperand` (a `fn` item, so the tables
/// stay `const`).
macro_rules! car {
    ($n:literal, $($f:tt)+) => {
        $crate::race::settings_copy::StepOperand::CarCell($n, {
            fn f(c: &$crate::physics::car_store::Car) -> &$crate::session::PlainDword {
                &c.$($f)+
            }
            f
        })
    };
}

/// `car_bits!(n, field)`: car block n's float `field`, by its bits.
macro_rules! car_bits {
    ($n:literal, $($f:tt)+) => {
        $crate::race::settings_copy::StepOperand::CarBits($n, {
            fn f(c: &$crate::physics::car_store::Car) -> &$crate::session::PlainFloat {
                &c.$($f)+
            }
            f
        })
    };
}

/// `var!(STATIC.field)`: a dword of an overlay store as a `StepOperand` (a `fn` item, as `car!`);
/// `var!(@gfx field ..)` one of the graphics owner's, `var!(@race field ..)` one of the race's.
macro_rules! var {
    (@o $($f:tt)+) => {
        $crate::race::settings_copy::StepOperand::Owned({
            fn f<'a>(o: &'a $crate::owners::Owners<'a>) -> &'a dyn $crate::session::DwordLike {
                &o.$($f)+
            }
            f
        })
    };
    (@race $($f:tt)+) => {
        $crate::race::settings_copy::StepOperand::RaceVar({
            fn f(race: &$crate::race::owner::Race) -> &dyn $crate::session::DwordLike {
                &race.$($f)+
            }
            f
        })
    };
    (@gfx $($f:tt)+) => {
        $crate::race::settings_copy::StepOperand::Owned({
            fn f<'a>(o: &'a $crate::owners::Owners<'a>) -> &'a dyn $crate::session::DwordLike {
                &o.gfx.$($f)+
            }
            f
        })
    };
    ($($s:ident)::+ . $($f:tt)+) => {
        $crate::race::settings_copy::StepOperand::Var({
            fn f() -> &'static dyn $crate::session::DwordLike {
                &$($s)::+.$($f)+
            }
            f
        })
    };
}
pub(crate) use {car, car_bits, var};

/// The effects of the copies besides moving cells: the log and the three routines the
/// end of a copy runs. The game's is `GameEffects`; the test captures them.
trait Effects {
    /// A log text (`log_write`), with a line break after it if `newline`.
    fn text(&mut self, text: &str, newline: bool);
    /// A signed number and a line break (`log_int`).
    fn int(&mut self, value: u32);
    /// Eight hex digits and a line break (`log_hex8`).
    fn hex8(&mut self, value: u32);
    /// A float with two decimals and a line break (`log_float`).
    fn float(&mut self, value: f32);
    /// `race::clear_player_absences` (0x449D44).
    fn clear_player_absences(&mut self, race: &Race);
    /// `race::copy_league_table` (0x4650B7).
    fn copy_league_table(&mut self, race: &Race) -> Result<(), Refused>;
    /// `race::store_results_by_mode` (0x449957).
    fn store_results_by_mode(&mut self, race: &Race);
}

/// The game's log and routines.
struct GameEffects;

impl Effects for GameEffects {
    fn text(&mut self, text: &str, newline: bool) {
        crate::logfile::log_write::log_nul(text.as_bytes(), newline, 1);
    }
    fn int(&mut self, value: u32) {
        crate::logfile::log_int::log_int(value as i32, true, 1);
    }
    fn hex8(&mut self, value: u32) {
        crate::logfile::log_hex8::log_hex8(value, true, 1);
    }
    fn float(&mut self, value: f32) {
        crate::logfile::log_float::log_float(value, 2, true, 1);
    }
    fn clear_player_absences(&mut self, race: &Race) {
        crate::race::race_shell::clear_player_absences(race);
    }
    fn copy_league_table(&mut self, race: &Race) -> Result<(), Refused> {
        Ok(crate::race::race_shell::copy_league_table(race)?)
    }
    fn store_results_by_mode(&mut self, race: &Race) {
        crate::race::race_shell::store_results_by_mode(race);
    }
}

/// `race::copy_settings_in` (0x446EBE): "COPY GRAHAM TO SHAUN".
pub fn copy_settings_in(race: &Race, gfx: &Gfx) -> Result<(), Refused> {
    copy_in(race, gfx, &mut GameEffects)
}

/// `race::copy_settings_out` (0x4486FB): "COPY SHAUN TO GRAHAM"; skipped (and logged so)
/// while a replay is shown.
pub fn copy_settings_out(race: &Race) -> Result<(), Refused> {
    copy_out(race, &mut GameEffects)
}

const RULE: &str = "-------------------------------------";

/// A label and its number logged on one line (`log_write`, then `log_int`).
fn logged_int(fx: &mut impl Effects, label: &str, value: u32) {
    fx.text(label, false);
    fx.int(value);
}

/// Both cars' logged values `[carN.name]`: the three identity words, the 43 gauges, the
/// eight impact marks; car 2's only when there are two players.
fn log_cars(race: &Race, fx: &mut impl Effects) {
    let shared = &race.shared;
    for car in 0..2 {
        let c = &race.car_blocks().cars[car];
        let label = |name: &str| format!("[car{}.{name}]", car + 1);
        logged_int(fx, &label("shauncarnum1"), c.car_kind.get());
        fx.text(&label("carpartsleft"), false);
        fx.hex8(c.parts_present.get());
        logged_int(fx, &label("shauntyretype1"), c.tyre_kind.get());
        for ((gauge, _), name) in GAUGES.iter().zip(GAUGE_NAMES) {
            fx.text(&label(name), false);
            fx.float(gauge(c).get());
        }
        for (n, mark) in c.impact_octant_marks.iter().enumerate() {
            fx.text(&label(&format!("caroctdamage{n}")), false);
            fx.float(mark.get());
        }
        // The original jumps over car 2's lines unless the race has exactly two players.
        if car == 0 && shared.view.get().players.get() != 2 {
            break;
        }
    }
}

/// The arcade positions, logged at the end of both copies.
fn log_arcade_positions(race: &Race, fx: &mut impl Effects) {
    let shared = &race.shared;
    logged_int(
        fx,
        "[arcadestartposition]",
        shared.tables().start_place.get(),
    );
    logged_int(
        fx,
        "[car1.arcadeposition]",
        race.car_blocks().cars[0].mode2_place.get(),
    );
    logged_int(
        fx,
        "[onerace.car1arcadeposition]",
        shared.state.results.figure_000.get(),
    );
}

/// The front end's settings into the race's (`copy_settings_in`), in the original's order.
fn copy_in(race: &Race, gfx: &Gfx, fx: &mut impl Effects) -> Result<(), Refused> {
    let shared = &race.shared;
    let (settings, results, vars, view) = (
        &shared.state.settings,
        &shared.state.results,
        shared.vars.get(),
        shared.view.get(),
    );
    fx.text(RULE, true);
    fx.text("COPY GRAHAM TO SHAUN", true);

    // The race's variables take the front end's settings first.
    vars.no_replay.set(settings.no_replay.get());
    vars.rivals.set(settings.rivals.get());
    vars.race_car_kinds[0].set(settings.car_kind_current.get());
    vars.race_car_kinds[1].set(settings.car_kind_first.get());
    view.players.set(settings.race_mode.get());
    vars.split.set(settings.kept.get());
    vars.stage_number.set(settings.stage.get());
    vars.direction.set(settings.direction.get());
    vars.race_level.set(results.race_level.get());
    vars.race_stage.set(results.race_stage.get());
    // The race's stage counts from 0, the front end's from 1.
    vars.race_stage.set(vars.race_stage.get().wrapping_sub(1));
    vars.run_kind.set(settings.run_kind.get());
    view.race_cars.set(view.players.get());
    vars.car_tyres[0].set(settings.cars[0].tyre.get());
    vars.car_tyres[1].set(settings.cars[1].tyre.get());
    vars.mip_setting.set(gfx.display.choices.mipmapping.get());
    vars.settings_words[0].set(settings.lens_flare.get());
    vars.settings_words[1].set(settings.reflections.get());
    vars.settings_words[2].set(settings.car_shadows.get());
    vars.settings_words[3].set(settings.car_detail.get());
    vars.settings_words[4].set(settings.headlights.get());

    let cells: [(&str, &dyn crate::session::DwordLike); 14] = [
        ("[shaunnumplayers]", &view.players),
        ("[shaunsplitmode]", &vars.split),
        ("[shauntracknum]", &vars.stage_number),
        ("[shauncrsdirec]", &vars.direction),
        ("[shaunrallynumber]", &vars.race_level),
        ("[shaunstagenumber]", &vars.race_stage),
        ("[shaungmode]", &vars.run_kind),
        ("[shaunnumcars]", &view.race_cars),
        ("[shaunmipmapping]", &vars.mip_setting),
        ("[shaunlensflare]", &vars.settings_words[0]),
        ("[shaunreflections]", &vars.settings_words[1]),
        ("[shauncarshadow]", &vars.settings_words[2]),
        ("[shauncardetail]", &vars.settings_words[3]),
        ("[shaunheadlights]", &vars.settings_words[4]),
    ];
    for (label, cell) in cells {
        logged_int(fx, label, cell.get());
    }

    // The car blocks' settings. The brake-like settings are percentages: below 0 is 0,
    // 100 or more is 0x100 (full), else the setting x 2.56 (the x87's product, rounded).
    let brake_scale = race.course.numbers.get().brake_scale.get();
    for car in 0..2 {
        let c = &race.car_blocks().cars[car];
        let s = &settings.cars[car];
        for (field, setting) in SETTING_BRAKES {
            let held = rc2k_core::race::cars::held_brake(setting(s).get() as i32, |v| {
                x87::int_times_to_int(v, brake_scale) as u32
            });
            field(c).set(held);
        }
        c.gear_ratio_setting.set(s.gear_ratio.get());
    }
    // The first car's brake balance is turned round: 0x100 less itself.
    let balance = &race.car_blocks().cars[0].brake_balance;
    balance.set(0x100u32.wrapping_sub(balance.get()));
    for car in 0..2 {
        vars.car_colours[car].set(settings.cars[car].controls.gearing.get());
    }
    for car in 0..2 {
        race.car_blocks().cars[car]
            .start_slot_value
            .set(settings.cars[car].race_number.get());
    }
    for car in 0..2 {
        let c = &race.car_blocks().cars[car];
        c.camera_view.set(settings.cars[car].view.get());
        c.camera_blend_view.set(settings.cars[car].view.get());
    }

    let hud_bits = [&results.hud_bits_current, &results.hud_bits_first];
    logged_int(fx, "[onerace.car1overbits]", hud_bits[0].get());
    logged_int(fx, "[onerace.car2overbits]", hud_bits[1].get());
    // The head-up display's option bits: the results' word, unless it is -1 (all ones);
    // then the bits are made from the player's own on/off settings.
    for (car, hud) in hud_bits.into_iter().enumerate() {
        let mut bits = hud.get();
        if bits == 0xFFFF_FFFF {
            bits = 0;
            for (setting, bit) in OPTION_BITS {
                if setting(&settings.cars[car]).get() != 0 {
                    bits |= bit;
                }
            }
        }
        race.car_blocks().cars[car].option_bits.set(bits);
        view.player_options[car].set(bits);
    }

    // The gauges: the front end keeps them as whole numbers, the cars as floats (`fild`,
    // `fstp`: the x87's exact conversion, so large numbers round as the original's do).
    for car in 0..2 {
        let c = &race.car_blocks().cars[car];
        for (gauge, setting) in GAUGES {
            gauge(c).set(x87::from_int(setting(&settings.cars[car]).get() as i32));
        }
        for (mark, saved) in c.impact_octant_marks.iter().zip(result_marks(shared, car)) {
            mark.set(f32::from_bits(saved.get()));
        }
    }

    race.car_blocks().cars[0]
        .parts_present
        .set(results.flags_current.get());
    race.car_blocks().cars[1]
        .parts_present
        .set(results.flags_first.get());
    shared.tables().start_place.set(results.figure_000.get());
    race.car_blocks().cars[0]
        .mode2_place
        .set(results.figure_000.get());

    log_cars(race, fx);

    fx.clear_player_absences(race);
    for car in 0..2 {
        race.car_blocks().cars[car]
            .stop_state
            .set(results.results[car].result.get());
    }
    log_arcade_positions(race, fx);
    fx.copy_league_table(race)?;
    fx.text("COPY GRAHAM TO SHAUN", true);
    fx.text(RULE, true);
    Ok(())
}

/// The race's settings back to the front end's (`copy_settings_out`), in the original's
/// order; while a replay is shown (`hold_car_revs` not 0) nothing is copied.
fn copy_out(race: &Race, fx: &mut impl Effects) -> Result<(), Refused> {
    let shared = &race.shared;
    let (settings, results) = (&shared.state.settings, &shared.state.results);
    fx.text(RULE, true);
    fx.text("COPY SHAUN TO GRAHAM", true);
    if shared.replay_file().hold_car_revs.get() != 0 {
        fx.text("SKIPPING - REPLAY!", true);
    } else {
        results
            .hud_bits_current
            .set(race.car_blocks().cars[0].option_bits.get());
        results
            .hud_bits_first
            .set(race.car_blocks().cars[1].option_bits.get());
        // The gauges back to whole numbers (`fld`, `fistp`: rounded as the x87's control
        // word says, to the nearest by default), the impact marks' bits as they are.
        for car in 0..2 {
            let c = &race.car_blocks().cars[car];
            for (gauge, setting) in GAUGES {
                setting(&settings.cars[car]).set(x87::to_int(gauge(c)) as u32);
            }
            for (mark, saved) in c.impact_octant_marks.iter().zip(result_marks(shared, car)) {
                saved.set(mark.get().to_bits());
            }
        }
        fx.store_results_by_mode(race);
        for car in 0..2 {
            results.results[car]
                .result
                .set(race.car_blocks().cars[car].stop_state.get());
        }
        for car in 0..2 {
            results.results[car]
                .secs
                .set(x87::loaded(&race.car_blocks().cars[car].top_speed));
        }
        results
            .flags_current
            .set(race.car_blocks().cars[0].parts_present.get());
        results
            .flags_first
            .set(race.car_blocks().cars[1].parts_present.get());
        log_cars(race, fx);
        log_arcade_positions(race, fx);
    }
    fx.text("COPY SHAUN TO GRAHAM", true);
    fx.text(RULE, true);
    Ok(())
}

/// Car `car`'s eight impact marks in the results record (`results[car].impact_marks`).
fn result_marks(shared: &RaceShared, car: usize) -> &[PlainDword; 8] {
    &shared.state.results.results[car].impact_marks
}

/// A float of a car block, picked by field.
type CarFloat = fn(&Car) -> &PlainFloat;
/// A dword of a car block, picked by field.
type CarWord = fn(&Car) -> &PlainDword;
/// A word of a player's settings record in the race's variables, picked by field.
type SettingWord = fn(&RaceCar) -> &PlainDword;

/// The race's car blocks and the front end's per-car settings records: the 43 gauges of a
/// car (damage and wear), each with the settings word it is kept in.
const GAUGES: [(CarFloat, SettingWord); 43] = [
    (|c| &c.engine_damage, |r| &r.damage[0]),
    (|c| &c.exhaust_damage, |r| &r.damage[2]),
    (|c| &c.electrics_damage, |r| &r.damage[3]),
    (|c| &c.turbo_damage, |r| &r.damage[4]),
    (|c| &c.cooling_damage, |r| &r.damage[6]),
    (|c| &c.steering_damage, |r| &r.damage[7]),
    (|c| &c.clutch_damage, |r| &r.damage[8]),
    (|c| &c.differential_damage, |r| &r.damage[10]),
    (|c| &c.overall_suspension_damage, |r| &r.wear[27]),
    (|c| &c.tyre_wear, |r| &r.wear[35]),
    (|c| &c.brake_damage, |r| &r.wear[30]),
    (|c| &c.lights_damage, |r| &r.wear[22]),
    (|c| &c.gearbox_damage, |r| &r.wear[9]),
    (|c| &c.body_damage, |r| &r.damage[1]),
    (|c| &c.gear_wear[0], |r| &r.wear[0]),
    (|c| &c.gear_wear[2], |r| &r.wear[2]),
    (|c| &c.gear_wear[3], |r| &r.wear[3]),
    (|c| &c.gear_wear[4], |r| &r.wear[4]),
    (|c| &c.gear_wear[5], |r| &r.wear[5]),
    (|c| &c.gear_wear[6], |r| &r.wear[6]),
    (|c| &c.gear_wear[7], |r| &r.wear[7]),
    (|c| &c.gear_wear[8], |r| &r.wear[8]),
    (|c| &c.wheel_hit_damage[0], |r| &r.wear[10]),
    (|c| &c.wheel_hit_damage[1], |r| &r.wear[11]),
    (|c| &c.wheel_hit_damage[2], |r| &r.wear[12]),
    (|c| &c.wheel_hit_damage[3], |r| &r.wear[13]),
    (|c| &c.body_hit_damage, |r| &r.wear[14]),
    (|c| &c.light_damage[0], |r| &r.wear[16]),
    (|c| &c.light_damage[1], |r| &r.wear[17]),
    (|c| &c.light_damage[2], |r| &r.wear[18]),
    (|c| &c.light_damage[3], |r| &r.wear[19]),
    (|c| &c.light_damage[4], |r| &r.wear[20]),
    (|c| &c.light_damage[5], |r| &r.wear[21]),
    (|c| &c.suspension_damage[0], |r| &r.wear[23]),
    (|c| &c.suspension_damage[1], |r| &r.wear[24]),
    (|c| &c.suspension_damage[2], |r| &r.wear[25]),
    (|c| &c.suspension_damage[3], |r| &r.wear[26]),
    (|c| &c.brake_wear_a, |r| &r.wear[28]),
    (|c| &c.brake_wear_b, |r| &r.wear[29]),
    (|c| &c.tyre_wears[0], |r| &r.wear[31]),
    (|c| &c.tyre_wears[1], |r| &r.wear[32]),
    (|c| &c.tyre_wears[2], |r| &r.wear[33]),
    (|c| &c.tyre_wears[3], |r| &r.wear[34]),
];

/// The log names of the gauges in `GAUGES` (`[carN.name]`).
const GAUGE_NAMES: [&str; 43] = [
    "dam_engine",
    "dam_exhaust",
    "dam_elect",
    "dam_turbo",
    "dam_cooling",
    "dam_steer",
    "dam_clutch",
    "dam_diff",
    "dam_susp",
    "dam_tyres",
    "dam_brakes",
    "dam_lights",
    "dam_gearbox",
    "dam_body",
    "dam_gearrvs",
    "dam_gear1",
    "dam_gear2",
    "dam_gear3",
    "dam_gear4",
    "dam_gear5",
    "dam_gear6",
    "dam_gear7",
    "dam_whl1",
    "dam_whl2",
    "dam_whl3",
    "dam_whl4",
    "dam_bdywrk",
    "dam_lights1",
    "dam_lights2",
    "dam_lights3",
    "dam_lights4",
    "dam_lightsf",
    "dam_lightsb",
    "dam_susp1",
    "dam_susp2",
    "dam_susp3",
    "dam_susp4",
    "dam_brakef",
    "dam_brakeb",
    "dam_tyre1",
    "dam_tyre2",
    "dam_tyre3",
    "dam_tyre4",
];

/// The settings words of a car that are held to 0 .. 0x100 (`Brake`), then the gear ratio.
const SETTING_BRAKES: [(CarWord, SettingWord); 4] = [
    (|c| &c.steering_sensitivity, |r| &r.steering_sensitivity),
    (|c| &c.brake_balance, |r| &r.brake_balance),
    (|c| &c.ride_height, |r| &r.ride_height),
    (|c| &c.suspension_stiffness, |r| &r.suspension_stiffness),
];

/// The head-up display's option bits: the settings word and the bit it sets.
const OPTION_BITS: [(SettingWord, u32); 9] = [
    (|r| &r.map_area, 0x10),
    (|r| &r.rear_mirror, 0x8),
    (|r| &r.rev_counter, 0x1),
    (|r| &r.message_display, 0x4),
    (|r| &r.driver_info, 0x2),
    (|r| &r.metric_units, 0x100),
    (|r| &r.progress_bar, 0x40),
    (|r| &r.car_info, 0x20),
    (|r| &r.pace_notes, 0x80),
];

#[cfg(test)]
mod tests {
    use super::old_machine::{self, Step};
    use super::*;
    use crate::race::owner::race;

    /// A log and routine calls captured as text (floats by their bits).
    #[derive(Default, PartialEq, Debug)]
    struct Capture(Vec<String>);

    impl Effects for Capture {
        fn text(&mut self, text: &str, newline: bool) {
            self.0.push(format!("text {text} {newline}"));
        }
        fn int(&mut self, value: u32) {
            self.0.push(format!("int {}", value as i32));
        }
        fn hex8(&mut self, value: u32) {
            self.0.push(format!("hex8 {value:08X}"));
        }
        fn float(&mut self, value: f32) {
            self.0.push(format!("float {:08X}", value.to_bits()));
        }
        fn clear_player_absences(&mut self, _: &Race) {
            self.0.push("call clear_player_absences".into());
        }
        fn copy_league_table(&mut self, _: &Race) -> Result<(), Refused> {
            self.0.push("call copy_league_table".into());
            Ok(())
        }
        fn store_results_by_mode(&mut self, _: &Race) {
            self.0.push("call store_results_by_mode".into());
        }
    }

    /// xorshift64*: a seeded generator, no crates.
    struct Rng(u64);

    impl Rng {
        fn next(&mut self) -> u32 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
        }

        /// A word the settings might hold, or the floats of the gauges: small numbers
        /// (the 0 .. 3 the compares look at), percentages incl. out of range, -1, floats
        /// (also huge ones and NaNs, for the conversions), anything.
        fn word(&mut self) -> u32 {
            let r = self.next();
            match r % 9 {
                0 | 7 => (r >> 8) % 4,
                1 => 0,
                2 => ((r >> 8) % 300).wrapping_sub(20),
                3 => 0xFFFF_FFFF,
                4 => (((r >> 8) % 32_000) as f32 / 100.0 - 20.0).to_bits(),
                5 => ((r >> 8) as f32 * 700.0).to_bits(),
                6 => self.next(),
                _ => (((r >> 8) % 1000) as f32 + 0.5).to_bits(),
            }
        }
    }

    /// Every cell the old step lists name.
    fn operands(steps: &[Step<StepOperand>]) -> Vec<StepOperand> {
        let mut out = Vec::new();
        for step in steps {
            match *step {
                Step::Value(a, _)
                | Step::Float(a)
                | Step::Dec(a)
                | Step::BitIf(a, _)
                | Step::Load(a)
                | Step::Store(a)
                | Step::SubEax(a)
                | Step::JumpIfNot(a, ..) => out.push(a),
                Step::Copy(a, b)
                | Step::IntToFloat(a, b)
                | Step::FloatToInt(a, b)
                | Step::FloatCopy(a, b)
                | Step::Brake(a, b) => out.extend([a, b]),
                Step::Text(..)
                | Step::LoadImm(_)
                | Step::JumpIfEaxNot(..)
                | Step::Jump(_)
                | Step::Call(_)
                | Step::Return => {}
            }
        }
        out
    }

    fn snapshot(cells: &[StepOperand]) -> Vec<u32> {
        cells.iter().map(|c| c.get()).collect()
    }

    fn restore(cells: &[StepOperand], values: &[u32]) {
        for (c, v) in cells.iter().zip(values) {
            c.set(*v);
        }
    }

    /// Both copies over randomised states: the step-list machine (`old_machine`) and the
    /// plain functions start from the same cells and must leave the same values in every
    /// cell the machine names, with the same log lines (floats by their bits) and
    /// routine calls, in the same order, and return the same result.
    #[test]
    fn new_copies_match_the_machine() {
        let _lock = crate::test_support::test_lock();
        let race = race();
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        let scale = &race.course.numbers.get().brake_scale;
        let scale_kept = scale.get();
        let scales = [scale_kept, 2.56, 1.0, 3.7];
        let in_steps = old_machine::graham_to_shaun();
        let out_steps = old_machine::shaun_to_graham();
        for (copy_in, steps) in [(true, &in_steps), (false, &out_steps)] {
            let cells = operands(steps);
            let kept = snapshot(&cells);
            let mut branches = [0u32; 4];
            for round in 0..1500 {
                for c in &cells {
                    c.set(rng.word());
                }
                scale.set(scales[round % scales.len()]);
                let start = snapshot(&cells);
                branches[0] += u32::from(race.shared.view.get().players.get() == 2);
                branches[1] += u32::from(race.shared.replay_file().hold_car_revs.get() != 0);

                let mut old_log = Capture::default();
                let old_result = old_machine::run(race, &mut old_log, steps);
                let old_cells = snapshot(&cells);

                restore(&cells, &start);
                let mut new_log = Capture::default();
                let new_result = if copy_in {
                    super::copy_in(race, crate::gfx::owner::gfx(), &mut new_log)
                } else {
                    super::copy_out(race, &mut new_log)
                };
                let new_cells = snapshot(&cells);

                assert_eq!(old_result.is_ok(), new_result.is_ok());
                assert_eq!(old_log, new_log, "log, round {round}, in: {copy_in}");
                for (n, (o, w)) in old_cells.iter().zip(&new_cells).enumerate() {
                    assert_eq!(o, w, "cell {n}, round {round}, in: {copy_in}");
                }
                restore(&cells, &start);
            }
            // The compares were both ways in a good share of the rounds.
            assert!(branches[0] > 30 || copy_in, "two players");
            assert!(branches[1] > 30 || copy_in, "a replay shown");
            restore(&cells, &kept);
        }
        scale.set(scale_kept);
    }
}
