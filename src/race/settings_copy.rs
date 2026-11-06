//! The race's settings copied between the front end's ("Graham's", 0x75ACxx ..) and the
//! race's ("Shaun's") variables and car blocks, and logged: `race::copy_settings_in`
//! (0x446EBE, "COPY GRAHAM TO SHAUN", before a race) and `race::copy_settings_out`
//! (0x4486FB, "COPY SHAUN TO GRAHAM", after it). Both are long runs of the same few
//! kinds of step (`Step`): the copies and logs of each of the two cars are built by loops
//! over the gauge and setting tables, and `run` does the steps in order as the original's
//! code does.

use crate::errors::Refused;
use crate::gfx::owner::GFX;
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::race;
use crate::race::owner_replay::Replay;
use crate::race::state_store::RaceCar;
use crate::session::{Dword, Float};
use std::ffi::{CStr, CString};

use Step::*;
pub use rc2k_core::race::settings_copy::Kind;
use rc2k_core::race::settings_copy::Step;

/// A routine a `Call` step runs (the original's `call` at its address).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Routine {
    /// `race::clear_player_absences` (0x449D44).
    ClearPlayerAbsences,
    /// `race::copy_league_table` (0x4650B7).
    CopyLeagueTable,
    /// `race::store_results_by_mode` (0x449957).
    StoreResultsByMode,
}

/// An operand of the steps: one of the physics block's cells, a field of a car's block, a
/// store's dword, or (for `Call` only) a routine.
#[derive(Clone, Copy)]
pub enum Place {
    Routine(Routine),
    Cell(&'static Dword),
    /// A float of the physics block, by its bits.
    Bits(&'static Float),
    /// A dword of car block n (`CAR_BLOCKS.cars[n]`), by field (`car!`).
    CarCell(usize, fn(&Car) -> &Dword),
    /// A float of car block n, by its bits (`car_bits!`).
    CarBits(usize, fn(&Car) -> &Float),
    /// A dword of an overlay store on the race block (`var!`: `RACE_VARS.stage_number`).
    Var(fn() -> &'static Dword),
}
use Place::{Bits, CarBits, CarCell, Cell, Var};

impl Place {
    /// Its dword (a float's bits).
    pub fn get(self) -> u32 {
        match self {
            Place::Routine(r) => unreachable!("{r:?} is a routine, read as a variable"),
            Cell(cell) => cell.get(),
            Bits(cell) => cell.bits(),
            CarCell(n, f) => f(&CAR_BLOCKS.cars[n]).get(),
            CarBits(n, f) => f(&CAR_BLOCKS.cars[n]).bits(),
            Var(f) => f().get(),
        }
    }

    /// Its dword set (a float's bits).
    pub fn set(self, v: u32) {
        match self {
            Place::Routine(r) => unreachable!("{r:?} is a routine, written as a variable"),
            Cell(cell) => cell.set(v),
            Bits(cell) => cell.set_bits(v),
            CarCell(n, f) => f(&CAR_BLOCKS.cars[n]).set(v),
            CarBits(n, f) => f(&CAR_BLOCKS.cars[n]).set_bits(v),
            Var(f) => f().set(v),
        }
    }
}

/// `car!(n, field)`: car block n's dword `field` as a `Place` (a `fn` item, so the tables
/// stay `const`).
macro_rules! car {
    ($n:literal, $($f:tt)+) => {
        $crate::race::settings_copy::Place::CarCell($n, {
            fn f(c: &$crate::physics::car_store::Car) -> &$crate::session::Dword {
                &c.$($f)+
            }
            f
        })
    };
}

/// `car_bits!(n, field)`: car block n's float `field`, by its bits.
macro_rules! car_bits {
    ($n:literal, $($f:tt)+) => {
        $crate::race::settings_copy::Place::CarBits($n, {
            fn f(c: &$crate::physics::car_store::Car) -> &$crate::session::Float {
                &c.$($f)+
            }
            f
        })
    };
}

/// `var!(STATIC.field)`: a dword of an overlay store as a `Place` (a `fn` item, as `car!`).
macro_rules! var {
    ($($s:ident)::+ . $($f:tt)+) => {
        $crate::race::settings_copy::Place::Var({
            fn f() -> &'static $crate::session::Dword {
                &$($s)::+.$($f)+
            }
            f
        })
    };
}
pub(crate) use {car, car_bits, var};

/// The game as the steps' machine: its dwords at IDA addresses, the steps' own eax and ebx
/// (the original reads no register at entry: they start from nothing), the x87's
/// conversions, and the logs and calls.
#[derive(Default)]
struct Game {
    eax: u32,
    ebx: u32,
}

impl rc2k_core::race::settings_copy::Machine<Place> for Game {
    type Error = Refused;
    fn get(&mut self, a: Place) -> u32 {
        a.get()
    }
    fn set(&mut self, a: Place, v: u32) {
        a.set(v);
    }
    fn eax(&self) -> u32 {
        self.eax
    }
    fn set_eax(&mut self, v: u32) {
        self.eax = v;
    }
    fn set_ebx(&mut self, v: u32) {
        self.ebx = v;
    }
    fn int_to_float(&mut self, v: u32) -> u32 {
        x87::from_int(v as i32)
    }
    fn float_to_int(&mut self, v: u32) -> u32 {
        x87::to_int(v) as u32
    }
    fn float_copy(&mut self, v: u32) -> u32 {
        x87::loaded(v)
    }
    fn scaled_brake(&mut self, v: i32) -> u32 {
        x87::int_times_to_int(v, race().course.numbers.brake_scale.bits()) as u32
    }
    fn text(&mut self, text: &'static std::ffi::CStr, nl: bool) -> Result<(), Refused> {
        crate::logfile::log_write::log_nul(&text, nl, 1);
        Ok(())
    }
    fn value(&mut self, v: u32, kind: Kind) -> Result<(), Refused> {
        if kind == Kind::Int {
            crate::logfile::log_int::log_int(v as i32, true, 1);
        } else {
            crate::logfile::log_hex8::log_hex8(v, true, 1);
        }
        Ok(())
    }
    fn float(&mut self, v: u32) -> Result<(), Refused> {
        crate::logfile::log_float::log_float(f32::from_bits(v), 2, true, 1);
        Ok(())
    }
    fn call(&mut self, a: Place) -> Result<(), Refused> {
        let course = &race().course;
        let physics = &race().physics;
        let Place::Routine(routine) = a else {
            unreachable!("a call of a variable");
        };
        use crate::race::race_shell::{
            clear_player_absences, copy_league_table, store_results_by_mode,
        };
        // Each keeps the registers the steps read (eax is set again before any use).
        {
            match routine {
                Routine::ClearPlayerAbsences => clear_player_absences(),
                Routine::CopyLeagueTable => copy_league_table(course)?,
                Routine::StoreResultsByMode => store_results_by_mode(physics),
            }
        }
        Ok(())
    }
}

/// The steps from the first, in the original's order.
pub fn run(steps: &[Step<Place>]) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::settings_copy::run` (the x87's conversions and the
    // brake's product kept, through `Machine`).
    rc2k_core::race::settings_copy::run(steps, &mut Game::default())
}

/// `race::copy_settings_in` (0x446EBE): `GRAHAM_TO_SHAUN`.
pub fn copy_settings_in() -> Result<(), Refused> {
    run(&race().replay.graham_to_shaun)
}

/// `race::copy_settings_out` (0x4486FB): `SHAUN_TO_GRAHAM`.
pub fn copy_settings_out(replay: &Replay) -> Result<(), Refused> {
    run(&replay.shaun_to_graham)
}

/// A float of a car block, picked by field.
type CarFloat = fn(&Car) -> &Float;
/// A dword of a car block, picked by field.
type CarWord = fn(&Car) -> &Dword;
/// A word of a player's settings record in the race's variables, picked by field.
type SettingWord = fn(&RaceCar) -> &Dword;

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
    (|c| &c.part_hit_damage[0], |r| &r.wear[31]),
    (|c| &c.part_hit_damage[1], |r| &r.wear[32]),
    (|c| &c.part_hit_damage[2], |r| &r.wear[33]),
    (|c| &c.part_hit_damage[3], |r| &r.wear[34]),
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

/// The race's settings that are logged after the copy: the log text and the variable.
const SHAUN_LOGS: [(&CStr, Place); 14] = [
    (
        c"[shaunnumplayers]",
        var!(crate::race::owner::RACE.shared.view.players),
    ),
    (
        c"[shaunsplitmode]",
        var!(crate::race::owner::RACE.shared.vars.split),
    ),
    (
        c"[shauntracknum]",
        var!(crate::race::owner::RACE.shared.vars.stage_number),
    ),
    (
        c"[shauncrsdirec]",
        var!(crate::race::owner::RACE.shared.vars.direction),
    ),
    (
        c"[shaunrallynumber]",
        var!(crate::race::owner::RACE.shared.vars.race_level),
    ),
    (
        c"[shaunstagenumber]",
        var!(crate::race::owner::RACE.shared.vars.race_stage),
    ),
    (
        c"[shaungmode]",
        var!(crate::race::owner::RACE.shared.vars.run_kind),
    ),
    (
        c"[shaunnumcars]",
        var!(crate::race::owner::RACE.shared.view.race_cars),
    ),
    (
        c"[shaunmipmapping]",
        var!(crate::race::owner::RACE.shared.vars.mip_setting),
    ),
    (
        c"[shaunlensflare]",
        var!(crate::race::owner::RACE.shared.vars.settings_words[0]),
    ),
    (
        c"[shaunreflections]",
        var!(crate::race::owner::RACE.shared.vars.settings_words[1]),
    ),
    (
        c"[shauncarshadow]",
        var!(crate::race::owner::RACE.shared.vars.settings_words[2]),
    ),
    (
        c"[shauncardetail]",
        var!(crate::race::owner::RACE.shared.vars.settings_words[3]),
    ),
    (
        c"[shaunheadlights]",
        var!(crate::race::owner::RACE.shared.vars.settings_words[4]),
    ),
];

/// The steps in building: pushes, and jumps patched to where they land.
struct Builder(Vec<Step<Place>>);

impl Builder {
    fn push(&mut self, step: Step<Place>) {
        self.0.push(step);
    }

    /// Where the next step goes (a jump's target).
    fn here(&self) -> usize {
        self.0.len()
    }

    /// The log text `[carN.name]` (N from 1), kept for the program's life.
    fn car_label(car: usize, name: &str) -> &'static CStr {
        let text = CString::new(format!("[car{}.{name}]", car + 1)).expect("no NUL in a name");
        Box::leak(text.into_boxed_c_str())
    }

    /// Both cars' logged values: the three identity words, the 43 gauges, the eight impact
    /// marks; after car 1's, a jump over car 2's unless there are two players.
    fn car_logs(&mut self) {
        let mut skip = None;
        for car in 0..2 {
            self.push(Text(Self::car_label(car, "shauncarnum1"), false));
            self.push(Value(CarCell(car, |c| &c.car_kind), Kind::Int));
            self.push(Text(Self::car_label(car, "carpartsleft"), false));
            self.push(Value(CarCell(car, |c| &c.parts_present), Kind::Hex8));
            self.push(Text(Self::car_label(car, "shauntyretype1"), false));
            self.push(Value(CarCell(car, |c| &c.tyre_kind), Kind::Int));
            for ((field, _), name) in GAUGES.iter().zip(GAUGE_NAMES) {
                self.push(Text(Self::car_label(car, name), false));
                self.push(Float(CarBits(car, *field)));
            }
            for (mark, field) in IMPACT_MARKS.into_iter().enumerate() {
                self.push(Text(
                    Self::car_label(car, &format!("caroctdamage{mark}")),
                    false,
                ));
                self.push(Float(CarBits(car, field)));
            }
            if car == 0 {
                skip = Some(self.here());
                self.push(JumpIfNot(
                    var!(crate::race::owner::RACE.shared.view.players),
                    0x2,
                    0,
                ));
            }
        }
        if let Some(at) = skip {
            self.0[at] = JumpIfNot(
                var!(crate::race::owner::RACE.shared.view.players),
                0x2,
                self.here(),
            );
        }
    }
}

/// A car block's eight impact marks.
const IMPACT_MARKS: [CarFloat; 8] = [
    |c| &c.impact_octant_marks[0],
    |c| &c.impact_octant_marks[1],
    |c| &c.impact_octant_marks[2],
    |c| &c.impact_octant_marks[3],
    |c| &c.impact_octant_marks[4],
    |c| &c.impact_octant_marks[5],
    |c| &c.impact_octant_marks[6],
    |c| &c.impact_octant_marks[7],
];

/// Car `car`'s eight impact marks in the results record (`results[car].field_24c`).
fn result_marks(car: usize) -> &'static [Dword; 8] {
    &crate::race::owner::race().shared.state.results.results[car].field_24c
}

/// The steps of `copy_settings_in` (the front end's settings into the race's), in the
/// original's order.
pub fn graham_to_shaun() -> Vec<Step<Place>> {
    let mut b = Builder(Vec::with_capacity(446));
    let rule = c"-------------------------------------";
    b.push(Text(rule, true));
    b.push(Text(c"COPY GRAHAM TO SHAUN", true));
    for step in HEAD_IN {
        b.push(step);
    }
    for (text, place) in SHAUN_LOGS {
        b.push(Text(text, false));
        b.push(Value(place, Kind::Int));
    }
    for car in 0..2 {
        let settings = |f: fn(&RaceCar) -> &Dword| {
            Cell(f(
                &crate::race::owner::race().shared.state.settings.cars[car]
            ))
        };
        for (field, setting) in SETTING_BRAKES {
            b.push(Brake(CarCell(car, field), settings(setting)));
        }
        b.push(Copy(
            CarCell(car, |c| &c.gear_ratio_setting),
            settings(|r| &r.gear_ratio),
        ));
    }
    // The first car's brake balance is turned round: 0x100 less itself.
    b.push(LoadImm(0x100));
    b.push(SubEax(CarCell(0, |c| &c.brake_balance)));
    b.push(Store(CarCell(0, |c| &c.brake_balance)));
    for (car, colour) in CAR_COLOURS.into_iter().enumerate() {
        b.push(Load(Cell(
            &crate::race::owner::race().shared.state.settings.cars[car].gearing,
        )));
        b.push(Store(Var(colour)));
    }
    for car in 0..2 {
        b.push(Copy(
            CarCell(car, |c| &c.start_slot_value),
            Cell(&crate::race::owner::race().shared.state.settings.cars[car].race_number),
        ));
    }
    for car in 0..2 {
        b.push(Load(Cell(
            &crate::race::owner::race().shared.state.settings.cars[car].view,
        )));
        b.push(Store(CarCell(car, |c| &c.camera_view)));
        b.push(Store(CarCell(car, |c| &c.camera_blend_view)));
    }
    let hud_bits = [
        &crate::race::owner::race()
            .shared
            .state
            .results
            .hud_bits_current,
        &crate::race::owner::race()
            .shared
            .state
            .results
            .hud_bits_first,
    ];
    b.push(Text(c"[onerace.car1overbits]", false));
    b.push(Value(Cell(hud_bits[0]), Kind::Int));
    b.push(Text(c"[onerace.car2overbits]", false));
    b.push(Value(Cell(hud_bits[1]), Kind::Int));
    // The option bits: from the player's HUD settings, or -1 (all) as the race's override.
    for car in 0..2 {
        b.push(Load(Cell(hud_bits[car])));
        let jump = b.here();
        b.push(JumpIfEaxNot(0xFFFF_FFFF, 0));
        b.push(LoadImm(0));
        for (field, bit) in OPTION_BITS {
            b.push(BitIf(
                Cell(field(
                    &crate::race::owner::race().shared.state.settings.cars[car],
                )),
                bit,
            ));
        }
        b.0[jump] = JumpIfEaxNot(0xFFFF_FFFF, b.here());
        b.push(Store(CarCell(car, |c| &c.option_bits)));
        b.push(Store(Var(PLAYER_OPTIONS[car])));
    }
    for car in 0..2 {
        for (field, setting) in GAUGES {
            b.push(IntToFloat(
                CarBits(car, field),
                Cell(setting(
                    &crate::race::owner::race().shared.state.settings.cars[car],
                )),
            ));
        }
        for (mark, cell) in IMPACT_MARKS.into_iter().zip(result_marks(car)) {
            b.push(Copy(CarBits(car, mark), Cell(cell)));
        }
    }
    for step in FLAGS_IN {
        b.push(step);
    }
    b.car_logs();
    for step in TAIL_IN {
        b.push(step);
    }
    b.0
}

/// The front end's settings that the race's variables overwrite first (`GRAHAM_TO_SHAUN`).
static HEAD_IN: [Step<Place>; 21] = [
    Copy(
        var!(crate::race::owner::RACE.shared.vars.no_replay),
        Cell(&crate::race::owner::RACE.shared.state.settings.no_replay),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.rivals),
        Cell(&crate::race::owner::RACE.shared.state.settings.rivals),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.race_car_kinds[0]),
        Cell(
            &crate::race::owner::RACE
                .shared
                .state
                .settings
                .car_kind_current,
        ),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.race_car_kinds[1]),
        Cell(
            &crate::race::owner::RACE
                .shared
                .state
                .settings
                .car_kind_first,
        ),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.view.players),
        Cell(&crate::race::owner::RACE.shared.state.settings.race_mode),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.split),
        Cell(&crate::race::owner::RACE.shared.state.settings.kept),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.stage_number),
        Cell(&crate::race::owner::RACE.shared.state.settings.stage),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.direction),
        Cell(&crate::race::owner::RACE.shared.state.settings.direction),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.race_level),
        Cell(&crate::race::owner::RACE.shared.state.results.race_level),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.race_stage),
        Cell(&crate::race::owner::RACE.shared.state.results.race_stage),
    ),
    Dec(var!(crate::race::owner::RACE.shared.vars.race_stage)),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.run_kind),
        Cell(&crate::race::owner::RACE.shared.state.settings.run_kind),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.view.race_cars),
        var!(crate::race::owner::RACE.shared.view.players),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.car_tyres[0]),
        Cell(&crate::race::owner::RACE.shared.state.settings.cars[0].tyre),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.car_tyres[1]),
        Cell(&crate::race::owner::RACE.shared.state.settings.cars[1].tyre),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.mip_setting),
        var!(GFX.display.choices.mipmapping),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[0]),
        Cell(&crate::race::owner::RACE.shared.state.settings.setting_0),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[1]),
        Cell(&crate::race::owner::RACE.shared.state.settings.setting_1),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[2]),
        Cell(&crate::race::owner::RACE.shared.state.settings.setting_2),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[3]),
        Cell(&crate::race::owner::RACE.shared.state.settings.setting_3),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[4]),
        Cell(&crate::race::owner::RACE.shared.state.settings.setting_4),
    ),
];

/// The cars' part flags and the arcade start place into the results.
static FLAGS_IN: [Step<Place>; 4] = [
    Copy(
        car!(0, parts_present),
        Cell(&crate::race::owner::RACE.shared.state.results.flags_current),
    ),
    Copy(
        car!(1, parts_present),
        Cell(&crate::race::owner::RACE.shared.state.results.flags_first),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.tables.start_place),
        Cell(&crate::race::owner::RACE.shared.state.results.figure_000),
    ),
    Copy(
        car!(0, mode2_place),
        Cell(&crate::race::owner::RACE.shared.state.results.figure_000),
    ),
];

/// The end of the copy: the players' absences cleared, the results into the cars' stop
/// states, the arcade positions logged, the league table copied.
static TAIL_IN: [Step<Place>; 15] = [
    Call(Place::Routine(Routine::ClearPlayerAbsences)),
    Load(Cell(
        &crate::race::owner::RACE.shared.state.results.results[0].result,
    )),
    Store(car!(0, stop_state)),
    Load(Cell(
        &crate::race::owner::RACE.shared.state.results.results[1].result,
    )),
    Store(car!(1, stop_state)),
    Text(c"[arcadestartposition]", false),
    Value(
        var!(crate::race::owner::RACE.shared.tables.start_place),
        Kind::Int,
    ),
    Text(c"[car1.arcadeposition]", false),
    Value(car!(0, mode2_place), Kind::Int),
    Text(c"[onerace.car1arcadeposition]", false),
    Value(
        Cell(&crate::race::owner::RACE.shared.state.results.figure_000),
        Kind::Int,
    ),
    Call(Place::Routine(Routine::CopyLeagueTable)),
    Text(c"COPY GRAHAM TO SHAUN", true),
    Text(c"-------------------------------------", true),
    Return,
];

/// The arcade positions logged at the end of the copy back.
static TAIL_OUT: [Step<Place>; 6] = [
    Text(c"[arcadestartposition]", false),
    Value(
        var!(crate::race::owner::RACE.shared.tables.start_place),
        Kind::Int,
    ),
    Text(c"[car1.arcadeposition]", false),
    Value(car!(0, mode2_place), Kind::Int),
    Text(c"[onerace.car1arcadeposition]", false),
    Value(
        Cell(&crate::race::owner::RACE.shared.state.results.figure_000),
        Kind::Int,
    ),
];

/// The steps of `copy_settings_out` (the race's settings back to the front end's), in the
/// original's order; skipped while a replay is shown.
pub fn shaun_to_graham() -> Vec<Step<Place>> {
    let mut b = Builder(Vec::with_capacity(344));
    b.push(Text(c"-------------------------------------", true));
    b.push(Text(c"COPY SHAUN TO GRAHAM", true));
    let skip = b.here();
    b.push(JumpIfNot(
        var!(crate::race::owner::RACE.shared.replay_file.hold_car_revs),
        0x0,
        0,
    ));
    b.push(Copy(
        Cell(
            &crate::race::owner::race()
                .shared
                .state
                .results
                .hud_bits_current,
        ),
        CarCell(0, |c| &c.option_bits),
    ));
    b.push(Copy(
        Cell(
            &crate::race::owner::race()
                .shared
                .state
                .results
                .hud_bits_first,
        ),
        CarCell(1, |c| &c.option_bits),
    ));
    for car in 0..2 {
        for (field, setting) in GAUGES {
            b.push(FloatToInt(
                Cell(setting(
                    &crate::race::owner::race().shared.state.settings.cars[car],
                )),
                CarBits(car, field),
            ));
        }
        for (mark, cell) in IMPACT_MARKS.into_iter().zip(result_marks(car)) {
            b.push(Copy(Cell(cell), CarBits(car, mark)));
        }
    }
    b.push(Call(Place::Routine(Routine::StoreResultsByMode)));
    for car in 0..2 {
        b.push(Load(CarCell(car, |c| &c.stop_state)));
        b.push(Store(Cell(
            &crate::race::owner::RACE.shared.state.results.results[car].result,
        )));
    }
    for car in 0..2 {
        b.push(FloatCopy(
            Bits(&crate::race::owner::RACE.shared.state.results.results[car].secs),
            CarBits(car, |c| &c.top_speed),
        ));
    }
    b.push(Copy(
        Cell(&crate::race::owner::RACE.shared.state.results.flags_current),
        CarCell(0, |c| &c.parts_present),
    ));
    b.push(Copy(
        Cell(&crate::race::owner::RACE.shared.state.results.flags_first),
        CarCell(1, |c| &c.parts_present),
    ));
    b.car_logs();
    for step in TAIL_OUT {
        b.push(step);
    }
    let over = b.here();
    b.push(Jump(0));
    b.0[skip] = JumpIfNot(
        var!(crate::race::owner::RACE.shared.replay_file.hold_car_revs),
        0x0,
        b.here(),
    );
    b.push(Text(c"SKIPPING - REPLAY!", true));
    b.0[over] = Jump(b.here());
    b.push(Text(c"COPY SHAUN TO GRAHAM", true));
    b.push(Text(c"-------------------------------------", true));
    b.push(Return);
    b.0
}

/// The race variable of each of the two cars' colours, and of each player's options.
const CAR_COLOURS: [fn() -> &'static Dword; 2] = [
    || &crate::race::owner::RACE.shared.vars.car_colours[0],
    || &crate::race::owner::RACE.shared.vars.car_colours[1],
];
const PLAYER_OPTIONS: [fn() -> &'static Dword; 2] = [
    || &crate::race::owner::RACE.shared.view.player_options[0],
    || &crate::race::owner::RACE.shared.view.player_options[1],
];
