//! The race's settings copied between the front end's ("Graham's", 0x75ACxx ..) and the
//! race's ("Shaun's") variables and car blocks, and logged: `race::copy_settings_in`
//! (0x446EBE, "COPY GRAHAM TO SHAUN", before a race) and `race::copy_settings_out`
//! (0x4486FB, "COPY SHAUN TO GRAHAM", after it). Both are long runs of the same few
//! kinds of step; their steps are tables (`Step`), made from the disassembly by script
//! (IDA 2026-10-17), and `run` does them in order as the original's code does.

use crate::errors::Refused;
use crate::framework::{game_addr, rva};
use crate::gfx::choices_store::SCREEN_CHOICES;
use crate::math::x87;
use crate::physics::car_store::Car;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::numbers_store::NUMBERS;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::state_store::RACE_STATE;
use crate::race::tables_store::RACE_TABLES;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::session::{Dword, Float};

use Step::*;
pub use rc2k_core::race::settings_copy::Kind;
use rc2k_core::race::settings_copy::Step;

/// A variable of the steps: one of the physics block's cells, a field of a car's block,
/// or the game's at an IDA address (the race's, the texts, the calls).
#[derive(Clone, Copy)]
pub enum Place {
    Addr(usize),
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
use Place::{Addr, Bits, CarBits, CarCell, Cell, Var};

impl Place {
    /// Its dword (a float's bits).
    pub fn get(self) -> u32 {
        match self {
            Addr(a) => get(a),
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
            Addr(a) => set(a, v),
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
    ($s:ident . $($f:tt)+) => {
        $crate::race::settings_copy::Place::Var({
            fn f() -> &'static $crate::session::Dword {
                &$s.$($f)+
            }
            f
        })
    };
}
pub(crate) use {car, car_bits, var};

/// `race::clear_player_absences`, `race::copy_league_table`, `race::store_results_by_mode`.
pub const CALLS: [usize; 3] = [0x449D44, 0x4650B7, 0x449957];

fn at(a: usize) -> *mut u32 {
    crate::pointers::host(game_addr(rva(a))) as *mut u32
}

fn get(a: usize) -> u32 {
    crate::pointers::read_or_end(
        crate::pointers::game_address(at(a)),
        "race::settings_copy's operand",
    )
}

fn set(a: usize, v: u32) {
    crate::pointers::write_or_end(
        crate::pointers::game_address(at(a)),
        v,
        "race::settings_copy's operand",
    )
}

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
        a.set(v)
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
        x87::int_times_to_int(v, NUMBERS.brake_scale.bits()) as u32
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
        let Addr(a) = a else {
            unreachable!("the calls are the image's");
        };
        use crate::race::race_shell::{
            clear_player_absences, copy_league_table, store_results_by_mode,
        };
        // Each keeps the registers the steps read (eax is set again before any use).
        {
            match CALLS.iter().position(|&c| c == a) {
                Some(0) => clear_player_absences(),
                Some(1) => copy_league_table()?,
                Some(2) => store_results_by_mode(),
                _ => unreachable!("a call the tables have"),
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
    run(&GRAHAM_TO_SHAUN)
}

/// `race::copy_settings_out` (0x4486FB): `SHAUN_TO_GRAHAM`.
pub fn copy_settings_out() -> Result<(), Refused> {
    run(&SHAUN_TO_GRAHAM)
}

pub static GRAHAM_TO_SHAUN: [Step<Place>; 446] = [
    Text(c"-------------------------------------", true),
    Text(c"COPY GRAHAM TO SHAUN", true),
    Copy(
        var!(RACE_VARS.no_replay),
        Cell(&RACE_STATE.settings.no_replay),
    ),
    Copy(var!(RACE_VARS.rivals), Cell(&RACE_STATE.settings.rivals)),
    Copy(
        var!(RACE_VARS.race_car_kinds[0]),
        Cell(&RACE_STATE.settings.car_kind_current),
    ),
    Copy(
        var!(RACE_VARS.race_car_kinds[1]),
        Cell(&RACE_STATE.settings.car_kind_first),
    ),
    Copy(
        var!(RACE_VIEW.players),
        Cell(&RACE_STATE.settings.race_mode),
    ),
    Copy(var!(RACE_VARS.split), Cell(&RACE_STATE.settings.kept)),
    Copy(
        var!(RACE_VARS.stage_number),
        Cell(&RACE_STATE.settings.stage),
    ),
    Copy(
        var!(RACE_VARS.direction),
        Cell(&RACE_STATE.settings.direction),
    ),
    Copy(
        var!(RACE_VARS.race_level),
        Cell(&RACE_STATE.results.race_level),
    ),
    Copy(
        var!(RACE_VARS.race_stage),
        Cell(&RACE_STATE.results.race_stage),
    ),
    Dec(var!(RACE_VARS.race_stage)),
    Copy(
        var!(RACE_VARS.run_kind),
        Cell(&RACE_STATE.settings.run_kind),
    ),
    Copy(var!(RACE_VIEW.race_cars), var!(RACE_VIEW.players)),
    Copy(
        var!(RACE_VARS.car_tyres[0]),
        Cell(&RACE_STATE.settings.cars[0].tyre),
    ),
    Copy(
        var!(RACE_VARS.car_tyres[1]),
        Cell(&RACE_STATE.settings.cars[1].tyre),
    ),
    Copy(var!(RACE_VARS.mip_setting), var!(SCREEN_CHOICES.mipmapping)),
    Copy(
        var!(RACE_VARS.settings_words[0]),
        Cell(&RACE_STATE.settings.setting_0),
    ),
    Copy(
        var!(RACE_VARS.settings_words[1]),
        Cell(&RACE_STATE.settings.setting_1),
    ),
    Copy(
        var!(RACE_VARS.settings_words[2]),
        Cell(&RACE_STATE.settings.setting_2),
    ),
    Copy(
        var!(RACE_VARS.settings_words[3]),
        Cell(&RACE_STATE.settings.setting_3),
    ),
    Copy(
        var!(RACE_VARS.settings_words[4]),
        Cell(&RACE_STATE.settings.setting_4),
    ),
    Text(c"[shaunnumplayers]", false),
    Value(var!(RACE_VIEW.players), Kind::Int),
    Text(c"[shaunsplitmode]", false),
    Value(var!(RACE_VARS.split), Kind::Int),
    Text(c"[shauntracknum]", false),
    Value(var!(RACE_VARS.stage_number), Kind::Int),
    Text(c"[shauncrsdirec]", false),
    Value(var!(RACE_VARS.direction), Kind::Int),
    Text(c"[shaunrallynumber]", false),
    Value(var!(RACE_VARS.race_level), Kind::Int),
    Text(c"[shaunstagenumber]", false),
    Value(var!(RACE_VARS.race_stage), Kind::Int),
    Text(c"[shaungmode]", false),
    Value(var!(RACE_VARS.run_kind), Kind::Int),
    Text(c"[shaunnumcars]", false),
    Value(var!(RACE_VIEW.race_cars), Kind::Int),
    Text(c"[shaunmipmapping]", false),
    Value(var!(RACE_VARS.mip_setting), Kind::Int),
    Text(c"[shaunlensflare]", false),
    Value(var!(RACE_VARS.settings_words[0]), Kind::Int),
    Text(c"[shaunreflections]", false),
    Value(var!(RACE_VARS.settings_words[1]), Kind::Int),
    Text(c"[shauncarshadow]", false),
    Value(var!(RACE_VARS.settings_words[2]), Kind::Int),
    Text(c"[shauncardetail]", false),
    Value(var!(RACE_VARS.settings_words[3]), Kind::Int),
    Text(c"[shaunheadlights]", false),
    Value(var!(RACE_VARS.settings_words[4]), Kind::Int),
    Brake(
        car!(0, steering_sensitivity),
        Cell(&RACE_STATE.settings.cars[0].steering_sensitivity),
    ),
    Brake(
        car!(0, brake_balance),
        Cell(&RACE_STATE.settings.cars[0].brake_balance),
    ),
    Brake(
        car!(0, ride_height),
        Cell(&RACE_STATE.settings.cars[0].ride_height),
    ),
    Brake(
        car!(0, suspension_stiffness),
        Cell(&RACE_STATE.settings.cars[0].suspension_stiffness),
    ),
    Copy(
        car!(0, gear_ratio_setting),
        Cell(&RACE_STATE.settings.cars[0].gear_ratio),
    ),
    Brake(
        car!(1, steering_sensitivity),
        Cell(&RACE_STATE.settings.cars[1].steering_sensitivity),
    ),
    Brake(
        car!(1, brake_balance),
        Cell(&RACE_STATE.settings.cars[1].brake_balance),
    ),
    Brake(
        car!(1, ride_height),
        Cell(&RACE_STATE.settings.cars[1].ride_height),
    ),
    Brake(
        car!(1, suspension_stiffness),
        Cell(&RACE_STATE.settings.cars[1].suspension_stiffness),
    ),
    Copy(
        car!(1, gear_ratio_setting),
        Cell(&RACE_STATE.settings.cars[1].gear_ratio),
    ),
    LoadImm(0x100),
    SubEax(car!(0, brake_balance)),
    Store(car!(0, brake_balance)),
    Load(Cell(&RACE_STATE.settings.cars[0].record_30)),
    Store(var!(RACE_VARS.car_colours[0])),
    Load(Cell(&RACE_STATE.settings.cars[1].record_30)),
    Store(var!(RACE_VARS.car_colours[1])),
    Copy(
        car!(0, start_slot_value),
        Cell(&RACE_STATE.settings.cars[0].race_number),
    ),
    Copy(
        car!(1, start_slot_value),
        Cell(&RACE_STATE.settings.cars[1].race_number),
    ),
    Load(Cell(&RACE_STATE.settings.cars[0].view)),
    Store(car!(0, camera_view)),
    Store(car!(0, camera_blend_view)),
    Load(Cell(&RACE_STATE.settings.cars[1].view)),
    Store(car!(1, camera_view)),
    Store(car!(1, camera_blend_view)),
    Text(c"[onerace.car1overbits]", false),
    Value(Cell(&RACE_STATE.results.hud_bits_current), Kind::Int),
    Text(c"[onerace.car2overbits]", false),
    Value(Cell(&RACE_STATE.results.hud_bits_first), Kind::Int),
    Load(Cell(&RACE_STATE.results.hud_bits_current)),
    JumpIfEaxNot(0xFFFFFFFF, 92),
    LoadImm(0),
    BitIf(Cell(&RACE_STATE.settings.cars[0].map_area), 0x10),
    BitIf(Cell(&RACE_STATE.settings.cars[0].rear_mirror), 0x8),
    BitIf(Cell(&RACE_STATE.settings.cars[0].rev_counter), 0x1),
    BitIf(Cell(&RACE_STATE.settings.cars[0].message_display), 0x4),
    BitIf(Cell(&RACE_STATE.settings.cars[0].driver_info), 0x2),
    BitIf(Cell(&RACE_STATE.settings.cars[0].metric_units), 0x100),
    BitIf(Cell(&RACE_STATE.settings.cars[0].progress_bar), 0x40),
    BitIf(Cell(&RACE_STATE.settings.cars[0].car_info), 0x20),
    BitIf(Cell(&RACE_STATE.settings.cars[0].pace_notes), 0x80),
    Store(car!(0, option_bits)),
    Store(var!(RACE_VIEW.player_options[0])),
    Load(Cell(&RACE_STATE.results.hud_bits_first)),
    JumpIfEaxNot(0xFFFFFFFF, 106),
    LoadImm(0),
    BitIf(Cell(&RACE_STATE.settings.cars[1].map_area), 0x10),
    BitIf(Cell(&RACE_STATE.settings.cars[1].rear_mirror), 0x8),
    BitIf(Cell(&RACE_STATE.settings.cars[1].rev_counter), 0x1),
    BitIf(Cell(&RACE_STATE.settings.cars[1].message_display), 0x4),
    BitIf(Cell(&RACE_STATE.settings.cars[1].driver_info), 0x2),
    BitIf(Cell(&RACE_STATE.settings.cars[1].metric_units), 0x100),
    BitIf(Cell(&RACE_STATE.settings.cars[1].progress_bar), 0x40),
    BitIf(Cell(&RACE_STATE.settings.cars[1].car_info), 0x20),
    BitIf(Cell(&RACE_STATE.settings.cars[1].pace_notes), 0x80),
    Store(car!(1, option_bits)),
    Store(var!(RACE_VIEW.player_options[1])),
    IntToFloat(
        car_bits!(0, engine_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[0]),
    ),
    IntToFloat(
        car_bits!(0, exhaust_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[2]),
    ),
    IntToFloat(
        car_bits!(0, electrics_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[3]),
    ),
    IntToFloat(
        car_bits!(0, turbo_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[4]),
    ),
    IntToFloat(
        car_bits!(0, cooling_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[6]),
    ),
    IntToFloat(
        car_bits!(0, steering_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[7]),
    ),
    IntToFloat(
        car_bits!(0, clutch_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[8]),
    ),
    IntToFloat(
        car_bits!(0, differential_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[10]),
    ),
    IntToFloat(
        car_bits!(0, overall_suspension_damage),
        Cell(&RACE_STATE.settings.cars[0].wear[27]),
    ),
    IntToFloat(
        car_bits!(0, tyre_wear),
        Cell(&RACE_STATE.settings.cars[0].wear[35]),
    ),
    IntToFloat(
        car_bits!(0, brake_damage),
        Cell(&RACE_STATE.settings.cars[0].wear[30]),
    ),
    IntToFloat(
        car_bits!(0, lights_damage),
        Cell(&RACE_STATE.settings.cars[0].wear[22]),
    ),
    IntToFloat(
        car_bits!(0, gearbox_damage),
        Cell(&RACE_STATE.settings.cars[0].wear[9]),
    ),
    IntToFloat(
        car_bits!(0, body_damage),
        Cell(&RACE_STATE.settings.cars[0].damage[1]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[0]),
        Cell(&RACE_STATE.settings.cars[0].wear[0]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[2]),
        Cell(&RACE_STATE.settings.cars[0].wear[2]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[3]),
        Cell(&RACE_STATE.settings.cars[0].wear[3]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[4]),
        Cell(&RACE_STATE.settings.cars[0].wear[4]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[5]),
        Cell(&RACE_STATE.settings.cars[0].wear[5]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[6]),
        Cell(&RACE_STATE.settings.cars[0].wear[6]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[7]),
        Cell(&RACE_STATE.settings.cars[0].wear[7]),
    ),
    IntToFloat(
        car_bits!(0, gear_wear[8]),
        Cell(&RACE_STATE.settings.cars[0].wear[8]),
    ),
    IntToFloat(
        car_bits!(0, wheel_hit_damage[0]),
        Cell(&RACE_STATE.settings.cars[0].wear[10]),
    ),
    IntToFloat(
        car_bits!(0, wheel_hit_damage[1]),
        Cell(&RACE_STATE.settings.cars[0].wear[11]),
    ),
    IntToFloat(
        car_bits!(0, wheel_hit_damage[2]),
        Cell(&RACE_STATE.settings.cars[0].wear[12]),
    ),
    IntToFloat(
        car_bits!(0, wheel_hit_damage[3]),
        Cell(&RACE_STATE.settings.cars[0].wear[13]),
    ),
    IntToFloat(
        car_bits!(0, body_hit_damage),
        Cell(&RACE_STATE.settings.cars[0].wear[14]),
    ),
    IntToFloat(
        car_bits!(0, light_damage[0]),
        Cell(&RACE_STATE.settings.cars[0].wear[16]),
    ),
    IntToFloat(
        car_bits!(0, light_damage[1]),
        Cell(&RACE_STATE.settings.cars[0].wear[17]),
    ),
    IntToFloat(
        car_bits!(0, light_damage[2]),
        Cell(&RACE_STATE.settings.cars[0].wear[18]),
    ),
    IntToFloat(
        car_bits!(0, light_damage[3]),
        Cell(&RACE_STATE.settings.cars[0].wear[19]),
    ),
    IntToFloat(
        car_bits!(0, light_damage[4]),
        Cell(&RACE_STATE.settings.cars[0].wear[20]),
    ),
    IntToFloat(
        car_bits!(0, light_damage[5]),
        Cell(&RACE_STATE.settings.cars[0].wear[21]),
    ),
    IntToFloat(
        car_bits!(0, suspension_damage[0]),
        Cell(&RACE_STATE.settings.cars[0].wear[23]),
    ),
    IntToFloat(
        car_bits!(0, suspension_damage[1]),
        Cell(&RACE_STATE.settings.cars[0].wear[24]),
    ),
    IntToFloat(
        car_bits!(0, suspension_damage[2]),
        Cell(&RACE_STATE.settings.cars[0].wear[25]),
    ),
    IntToFloat(
        car_bits!(0, suspension_damage[3]),
        Cell(&RACE_STATE.settings.cars[0].wear[26]),
    ),
    IntToFloat(
        car_bits!(0, brake_wear_a),
        Cell(&RACE_STATE.settings.cars[0].wear[28]),
    ),
    IntToFloat(
        car_bits!(0, brake_wear_b),
        Cell(&RACE_STATE.settings.cars[0].wear[29]),
    ),
    IntToFloat(
        car_bits!(0, part_hit_damage[0]),
        Cell(&RACE_STATE.settings.cars[0].wear[31]),
    ),
    IntToFloat(
        car_bits!(0, part_hit_damage[1]),
        Cell(&RACE_STATE.settings.cars[0].wear[32]),
    ),
    IntToFloat(
        car_bits!(0, part_hit_damage[2]),
        Cell(&RACE_STATE.settings.cars[0].wear[33]),
    ),
    IntToFloat(
        car_bits!(0, part_hit_damage[3]),
        Cell(&RACE_STATE.settings.cars[0].wear[34]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[0]),
        Cell(&RACE_STATE.results.results[0].block[0]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[1]),
        Cell(&RACE_STATE.results.results[0].block[1]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[2]),
        Cell(&RACE_STATE.results.results[0].block[2]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[3]),
        Cell(&RACE_STATE.results.results[0].block[3]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[4]),
        Cell(&RACE_STATE.results.results[0].block[4]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[5]),
        Cell(&RACE_STATE.results.results[0].block[5]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[6]),
        Cell(&RACE_STATE.results.results[0].block[6]),
    ),
    Copy(
        car_bits!(0, impact_octant_marks[7]),
        Cell(&RACE_STATE.results.results[0].block[7]),
    ),
    IntToFloat(
        car_bits!(1, engine_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[0]),
    ),
    IntToFloat(
        car_bits!(1, exhaust_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[2]),
    ),
    IntToFloat(
        car_bits!(1, electrics_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[3]),
    ),
    IntToFloat(
        car_bits!(1, turbo_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[4]),
    ),
    IntToFloat(
        car_bits!(1, cooling_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[6]),
    ),
    IntToFloat(
        car_bits!(1, steering_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[7]),
    ),
    IntToFloat(
        car_bits!(1, clutch_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[8]),
    ),
    IntToFloat(
        car_bits!(1, differential_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[10]),
    ),
    IntToFloat(
        car_bits!(1, overall_suspension_damage),
        Cell(&RACE_STATE.settings.cars[1].wear[27]),
    ),
    IntToFloat(
        car_bits!(1, tyre_wear),
        Cell(&RACE_STATE.settings.cars[1].wear[35]),
    ),
    IntToFloat(
        car_bits!(1, brake_damage),
        Cell(&RACE_STATE.settings.cars[1].wear[30]),
    ),
    IntToFloat(
        car_bits!(1, lights_damage),
        Cell(&RACE_STATE.settings.cars[1].wear[22]),
    ),
    IntToFloat(
        car_bits!(1, gearbox_damage),
        Cell(&RACE_STATE.settings.cars[1].wear[9]),
    ),
    IntToFloat(
        car_bits!(1, body_damage),
        Cell(&RACE_STATE.settings.cars[1].damage[1]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[0]),
        Cell(&RACE_STATE.settings.cars[1].wear[0]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[2]),
        Cell(&RACE_STATE.settings.cars[1].wear[2]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[3]),
        Cell(&RACE_STATE.settings.cars[1].wear[3]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[4]),
        Cell(&RACE_STATE.settings.cars[1].wear[4]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[5]),
        Cell(&RACE_STATE.settings.cars[1].wear[5]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[6]),
        Cell(&RACE_STATE.settings.cars[1].wear[6]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[7]),
        Cell(&RACE_STATE.settings.cars[1].wear[7]),
    ),
    IntToFloat(
        car_bits!(1, gear_wear[8]),
        Cell(&RACE_STATE.settings.cars[1].wear[8]),
    ),
    IntToFloat(
        car_bits!(1, wheel_hit_damage[0]),
        Cell(&RACE_STATE.settings.cars[1].wear[10]),
    ),
    IntToFloat(
        car_bits!(1, wheel_hit_damage[1]),
        Cell(&RACE_STATE.settings.cars[1].wear[11]),
    ),
    IntToFloat(
        car_bits!(1, wheel_hit_damage[2]),
        Cell(&RACE_STATE.settings.cars[1].wear[12]),
    ),
    IntToFloat(
        car_bits!(1, wheel_hit_damage[3]),
        Cell(&RACE_STATE.settings.cars[1].wear[13]),
    ),
    IntToFloat(
        car_bits!(1, body_hit_damage),
        Cell(&RACE_STATE.settings.cars[1].wear[14]),
    ),
    IntToFloat(
        car_bits!(1, light_damage[0]),
        Cell(&RACE_STATE.settings.cars[1].wear[16]),
    ),
    IntToFloat(
        car_bits!(1, light_damage[1]),
        Cell(&RACE_STATE.settings.cars[1].wear[17]),
    ),
    IntToFloat(
        car_bits!(1, light_damage[2]),
        Cell(&RACE_STATE.settings.cars[1].wear[18]),
    ),
    IntToFloat(
        car_bits!(1, light_damage[3]),
        Cell(&RACE_STATE.settings.cars[1].wear[19]),
    ),
    IntToFloat(
        car_bits!(1, light_damage[4]),
        Cell(&RACE_STATE.settings.cars[1].wear[20]),
    ),
    IntToFloat(
        car_bits!(1, light_damage[5]),
        Cell(&RACE_STATE.settings.cars[1].wear[21]),
    ),
    IntToFloat(
        car_bits!(1, suspension_damage[0]),
        Cell(&RACE_STATE.settings.cars[1].wear[23]),
    ),
    IntToFloat(
        car_bits!(1, suspension_damage[1]),
        Cell(&RACE_STATE.settings.cars[1].wear[24]),
    ),
    IntToFloat(
        car_bits!(1, suspension_damage[2]),
        Cell(&RACE_STATE.settings.cars[1].wear[25]),
    ),
    IntToFloat(
        car_bits!(1, suspension_damage[3]),
        Cell(&RACE_STATE.settings.cars[1].wear[26]),
    ),
    IntToFloat(
        car_bits!(1, brake_wear_a),
        Cell(&RACE_STATE.settings.cars[1].wear[28]),
    ),
    IntToFloat(
        car_bits!(1, brake_wear_b),
        Cell(&RACE_STATE.settings.cars[1].wear[29]),
    ),
    IntToFloat(
        car_bits!(1, part_hit_damage[0]),
        Cell(&RACE_STATE.settings.cars[1].wear[31]),
    ),
    IntToFloat(
        car_bits!(1, part_hit_damage[1]),
        Cell(&RACE_STATE.settings.cars[1].wear[32]),
    ),
    IntToFloat(
        car_bits!(1, part_hit_damage[2]),
        Cell(&RACE_STATE.settings.cars[1].wear[33]),
    ),
    IntToFloat(
        car_bits!(1, part_hit_damage[3]),
        Cell(&RACE_STATE.settings.cars[1].wear[34]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[0]),
        Cell(&RACE_STATE.results.results[1].block[0]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[1]),
        Cell(&RACE_STATE.results.results[1].block[1]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[2]),
        Cell(&RACE_STATE.results.results[1].block[2]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[3]),
        Cell(&RACE_STATE.results.results[1].block[3]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[4]),
        Cell(&RACE_STATE.results.results[1].block[4]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[5]),
        Cell(&RACE_STATE.results.results[1].block[5]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[6]),
        Cell(&RACE_STATE.results.results[1].block[6]),
    ),
    Copy(
        car_bits!(1, impact_octant_marks[7]),
        Cell(&RACE_STATE.results.results[1].block[7]),
    ),
    Copy(
        car!(0, parts_present),
        Cell(&RACE_STATE.results.flags_current),
    ),
    Copy(
        car!(1, parts_present),
        Cell(&RACE_STATE.results.flags_first),
    ),
    Copy(
        var!(RACE_TABLES.start_place),
        Cell(&RACE_STATE.results.figure_000),
    ),
    Copy(car!(0, mode2_place), Cell(&RACE_STATE.results.figure_000)),
    Text(c"[car1.shauncarnum1]", false),
    Value(car!(0, car_kind), Kind::Int),
    Text(c"[car1.carpartsleft]", false),
    Value(car!(0, parts_present), Kind::Hex8),
    Text(c"[car1.shauntyretype1]", false),
    Value(car!(0, tyre_kind), Kind::Int),
    Text(c"[car1.dam_engine]", false),
    Float(car_bits!(0, engine_damage)),
    Text(c"[car1.dam_exhaust]", false),
    Float(car_bits!(0, exhaust_damage)),
    Text(c"[car1.dam_elect]", false),
    Float(car_bits!(0, electrics_damage)),
    Text(c"[car1.dam_turbo]", false),
    Float(car_bits!(0, turbo_damage)),
    Text(c"[car1.dam_cooling]", false),
    Float(car_bits!(0, cooling_damage)),
    Text(c"[car1.dam_steer]", false),
    Float(car_bits!(0, steering_damage)),
    Text(c"[car1.dam_clutch]", false),
    Float(car_bits!(0, clutch_damage)),
    Text(c"[car1.dam_diff]", false),
    Float(car_bits!(0, differential_damage)),
    Text(c"[car1.dam_susp]", false),
    Float(car_bits!(0, overall_suspension_damage)),
    Text(c"[car1.dam_tyres]", false),
    Float(car_bits!(0, tyre_wear)),
    Text(c"[car1.dam_brakes]", false),
    Float(car_bits!(0, brake_damage)),
    Text(c"[car1.dam_lights]", false),
    Float(car_bits!(0, lights_damage)),
    Text(c"[car1.dam_gearbox]", false),
    Float(car_bits!(0, gearbox_damage)),
    Text(c"[car1.dam_body]", false),
    Float(car_bits!(0, body_damage)),
    Text(c"[car1.dam_gearrvs]", false),
    Float(car_bits!(0, gear_wear[0])),
    Text(c"[car1.dam_gear1]", false),
    Float(car_bits!(0, gear_wear[2])),
    Text(c"[car1.dam_gear2]", false),
    Float(car_bits!(0, gear_wear[3])),
    Text(c"[car1.dam_gear3]", false),
    Float(car_bits!(0, gear_wear[4])),
    Text(c"[car1.dam_gear4]", false),
    Float(car_bits!(0, gear_wear[5])),
    Text(c"[car1.dam_gear5]", false),
    Float(car_bits!(0, gear_wear[6])),
    Text(c"[car1.dam_gear6]", false),
    Float(car_bits!(0, gear_wear[7])),
    Text(c"[car1.dam_gear7]", false),
    Float(car_bits!(0, gear_wear[8])),
    Text(c"[car1.dam_whl1]", false),
    Float(car_bits!(0, wheel_hit_damage[0])),
    Text(c"[car1.dam_whl2]", false),
    Float(car_bits!(0, wheel_hit_damage[1])),
    Text(c"[car1.dam_whl3]", false),
    Float(car_bits!(0, wheel_hit_damage[2])),
    Text(c"[car1.dam_whl4]", false),
    Float(car_bits!(0, wheel_hit_damage[3])),
    Text(c"[car1.dam_bdywrk]", false),
    Float(car_bits!(0, body_hit_damage)),
    Text(c"[car1.dam_lights1]", false),
    Float(car_bits!(0, light_damage[0])),
    Text(c"[car1.dam_lights2]", false),
    Float(car_bits!(0, light_damage[1])),
    Text(c"[car1.dam_lights3]", false),
    Float(car_bits!(0, light_damage[2])),
    Text(c"[car1.dam_lights4]", false),
    Float(car_bits!(0, light_damage[3])),
    Text(c"[car1.dam_lightsf]", false),
    Float(car_bits!(0, light_damage[4])),
    Text(c"[car1.dam_lightsb]", false),
    Float(car_bits!(0, light_damage[5])),
    Text(c"[car1.dam_susp1]", false),
    Float(car_bits!(0, suspension_damage[0])),
    Text(c"[car1.dam_susp2]", false),
    Float(car_bits!(0, suspension_damage[1])),
    Text(c"[car1.dam_susp3]", false),
    Float(car_bits!(0, suspension_damage[2])),
    Text(c"[car1.dam_susp4]", false),
    Float(car_bits!(0, suspension_damage[3])),
    Text(c"[car1.dam_brakef]", false),
    Float(car_bits!(0, brake_wear_a)),
    Text(c"[car1.dam_brakeb]", false),
    Float(car_bits!(0, brake_wear_b)),
    Text(c"[car1.dam_tyre1]", false),
    Float(car_bits!(0, part_hit_damage[0])),
    Text(c"[car1.dam_tyre2]", false),
    Float(car_bits!(0, part_hit_damage[1])),
    Text(c"[car1.dam_tyre3]", false),
    Float(car_bits!(0, part_hit_damage[2])),
    Text(c"[car1.dam_tyre4]", false),
    Float(car_bits!(0, part_hit_damage[3])),
    Text(c"[car1.caroctdamage0]", false),
    Float(car_bits!(0, impact_octant_marks[0])),
    Text(c"[car1.caroctdamage1]", false),
    Float(car_bits!(0, impact_octant_marks[1])),
    Text(c"[car1.caroctdamage2]", false),
    Float(car_bits!(0, impact_octant_marks[2])),
    Text(c"[car1.caroctdamage3]", false),
    Float(car_bits!(0, impact_octant_marks[3])),
    Text(c"[car1.caroctdamage4]", false),
    Float(car_bits!(0, impact_octant_marks[4])),
    Text(c"[car1.caroctdamage5]", false),
    Float(car_bits!(0, impact_octant_marks[5])),
    Text(c"[car1.caroctdamage6]", false),
    Float(car_bits!(0, impact_octant_marks[6])),
    Text(c"[car1.caroctdamage7]", false),
    Float(car_bits!(0, impact_octant_marks[7])),
    JumpIfNot(var!(RACE_VIEW.players), 0x2, 431),
    Text(c"[car2.shauncarnum1]", false),
    Value(car!(1, car_kind), Kind::Int),
    Text(c"[car2.carpartsleft]", false),
    Value(car!(1, parts_present), Kind::Hex8),
    Text(c"[car2.shauntyretype1]", false),
    Value(car!(1, tyre_kind), Kind::Int),
    Text(c"[car2.dam_engine]", false),
    Float(car_bits!(1, engine_damage)),
    Text(c"[car2.dam_exhaust]", false),
    Float(car_bits!(1, exhaust_damage)),
    Text(c"[car2.dam_elect]", false),
    Float(car_bits!(1, electrics_damage)),
    Text(c"[car2.dam_turbo]", false),
    Float(car_bits!(1, turbo_damage)),
    Text(c"[car2.dam_cooling]", false),
    Float(car_bits!(1, cooling_damage)),
    Text(c"[car2.dam_steer]", false),
    Float(car_bits!(1, steering_damage)),
    Text(c"[car2.dam_clutch]", false),
    Float(car_bits!(1, clutch_damage)),
    Text(c"[car2.dam_diff]", false),
    Float(car_bits!(1, differential_damage)),
    Text(c"[car2.dam_susp]", false),
    Float(car_bits!(1, overall_suspension_damage)),
    Text(c"[car2.dam_tyres]", false),
    Float(car_bits!(1, tyre_wear)),
    Text(c"[car2.dam_brakes]", false),
    Float(car_bits!(1, brake_damage)),
    Text(c"[car2.dam_lights]", false),
    Float(car_bits!(1, lights_damage)),
    Text(c"[car2.dam_gearbox]", false),
    Float(car_bits!(1, gearbox_damage)),
    Text(c"[car2.dam_body]", false),
    Float(car_bits!(1, body_damage)),
    Text(c"[car2.dam_gearrvs]", false),
    Float(car_bits!(1, gear_wear[0])),
    Text(c"[car2.dam_gear1]", false),
    Float(car_bits!(1, gear_wear[2])),
    Text(c"[car2.dam_gear2]", false),
    Float(car_bits!(1, gear_wear[3])),
    Text(c"[car2.dam_gear3]", false),
    Float(car_bits!(1, gear_wear[4])),
    Text(c"[car2.dam_gear4]", false),
    Float(car_bits!(1, gear_wear[5])),
    Text(c"[car2.dam_gear5]", false),
    Float(car_bits!(1, gear_wear[6])),
    Text(c"[car2.dam_gear6]", false),
    Float(car_bits!(1, gear_wear[7])),
    Text(c"[car2.dam_gear7]", false),
    Float(car_bits!(1, gear_wear[8])),
    Text(c"[car2.dam_whl1]", false),
    Float(car_bits!(1, wheel_hit_damage[0])),
    Text(c"[car2.dam_whl2]", false),
    Float(car_bits!(1, wheel_hit_damage[1])),
    Text(c"[car2.dam_whl3]", false),
    Float(car_bits!(1, wheel_hit_damage[2])),
    Text(c"[car2.dam_whl4]", false),
    Float(car_bits!(1, wheel_hit_damage[3])),
    Text(c"[car2.dam_bdywrk]", false),
    Float(car_bits!(1, body_hit_damage)),
    Text(c"[car2.dam_lights1]", false),
    Float(car_bits!(1, light_damage[0])),
    Text(c"[car2.dam_lights2]", false),
    Float(car_bits!(1, light_damage[1])),
    Text(c"[car2.dam_lights3]", false),
    Float(car_bits!(1, light_damage[2])),
    Text(c"[car2.dam_lights4]", false),
    Float(car_bits!(1, light_damage[3])),
    Text(c"[car2.dam_lightsf]", false),
    Float(car_bits!(1, light_damage[4])),
    Text(c"[car2.dam_lightsb]", false),
    Float(car_bits!(1, light_damage[5])),
    Text(c"[car2.dam_susp1]", false),
    Float(car_bits!(1, suspension_damage[0])),
    Text(c"[car2.dam_susp2]", false),
    Float(car_bits!(1, suspension_damage[1])),
    Text(c"[car2.dam_susp3]", false),
    Float(car_bits!(1, suspension_damage[2])),
    Text(c"[car2.dam_susp4]", false),
    Float(car_bits!(1, suspension_damage[3])),
    Text(c"[car2.dam_brakef]", false),
    Float(car_bits!(1, brake_wear_a)),
    Text(c"[car2.dam_brakeb]", false),
    Float(car_bits!(1, brake_wear_b)),
    Text(c"[car2.dam_tyre1]", false),
    Float(car_bits!(1, part_hit_damage[0])),
    Text(c"[car2.dam_tyre2]", false),
    Float(car_bits!(1, part_hit_damage[1])),
    Text(c"[car2.dam_tyre3]", false),
    Float(car_bits!(1, part_hit_damage[2])),
    Text(c"[car2.dam_tyre4]", false),
    Float(car_bits!(1, part_hit_damage[3])),
    Text(c"[car2.caroctdamage0]", false),
    Float(car_bits!(1, impact_octant_marks[0])),
    Text(c"[car2.caroctdamage1]", false),
    Float(car_bits!(1, impact_octant_marks[1])),
    Text(c"[car2.caroctdamage2]", false),
    Float(car_bits!(1, impact_octant_marks[2])),
    Text(c"[car2.caroctdamage3]", false),
    Float(car_bits!(1, impact_octant_marks[3])),
    Text(c"[car2.caroctdamage4]", false),
    Float(car_bits!(1, impact_octant_marks[4])),
    Text(c"[car2.caroctdamage5]", false),
    Float(car_bits!(1, impact_octant_marks[5])),
    Text(c"[car2.caroctdamage6]", false),
    Float(car_bits!(1, impact_octant_marks[6])),
    Text(c"[car2.caroctdamage7]", false),
    Float(car_bits!(1, impact_octant_marks[7])),
    Call(Addr(0x449D44)),
    Load(Cell(&RACE_STATE.results.results[0].result)),
    Store(car!(0, stop_state)),
    Load(Cell(&RACE_STATE.results.results[1].result)),
    Store(car!(1, stop_state)),
    Text(c"[arcadestartposition]", false),
    Value(var!(RACE_TABLES.start_place), Kind::Int),
    Text(c"[car1.arcadeposition]", false),
    Value(car!(0, mode2_place), Kind::Int),
    Text(c"[onerace.car1arcadeposition]", false),
    Value(Cell(&RACE_STATE.results.figure_000), Kind::Int),
    Call(Addr(0x4650B7)),
    Text(c"COPY GRAHAM TO SHAUN", true),
    Text(c"-------------------------------------", true),
    Return,
];

pub static SHAUN_TO_GRAHAM: [Step<Place>; 344] = [
    Text(c"-------------------------------------", true),
    Text(c"COPY SHAUN TO GRAHAM", true),
    JumpIfNot(var!(REPLAY_FILE.hold_car_revs), 0x0, 340),
    Copy(
        Cell(&RACE_STATE.results.hud_bits_current),
        car!(0, option_bits),
    ),
    Copy(
        Cell(&RACE_STATE.results.hud_bits_first),
        car!(1, option_bits),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[0]),
        car_bits!(0, engine_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[2]),
        car_bits!(0, exhaust_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[3]),
        car_bits!(0, electrics_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[4]),
        car_bits!(0, turbo_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[6]),
        car_bits!(0, cooling_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[7]),
        car_bits!(0, steering_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[8]),
        car_bits!(0, clutch_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[10]),
        car_bits!(0, differential_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[27]),
        car_bits!(0, overall_suspension_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[35]),
        car_bits!(0, tyre_wear),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[30]),
        car_bits!(0, brake_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[22]),
        car_bits!(0, lights_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[9]),
        car_bits!(0, gearbox_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].damage[1]),
        car_bits!(0, body_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[0]),
        car_bits!(0, gear_wear[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[2]),
        car_bits!(0, gear_wear[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[3]),
        car_bits!(0, gear_wear[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[4]),
        car_bits!(0, gear_wear[4]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[5]),
        car_bits!(0, gear_wear[5]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[6]),
        car_bits!(0, gear_wear[6]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[7]),
        car_bits!(0, gear_wear[7]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[8]),
        car_bits!(0, gear_wear[8]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[10]),
        car_bits!(0, wheel_hit_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[11]),
        car_bits!(0, wheel_hit_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[12]),
        car_bits!(0, wheel_hit_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[13]),
        car_bits!(0, wheel_hit_damage[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[14]),
        car_bits!(0, body_hit_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[16]),
        car_bits!(0, light_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[17]),
        car_bits!(0, light_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[18]),
        car_bits!(0, light_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[19]),
        car_bits!(0, light_damage[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[20]),
        car_bits!(0, light_damage[4]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[21]),
        car_bits!(0, light_damage[5]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[23]),
        car_bits!(0, suspension_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[24]),
        car_bits!(0, suspension_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[25]),
        car_bits!(0, suspension_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[26]),
        car_bits!(0, suspension_damage[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[28]),
        car_bits!(0, brake_wear_a),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[29]),
        car_bits!(0, brake_wear_b),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[31]),
        car_bits!(0, part_hit_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[32]),
        car_bits!(0, part_hit_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[33]),
        car_bits!(0, part_hit_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[0].wear[34]),
        car_bits!(0, part_hit_damage[3]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[0]),
        car_bits!(0, impact_octant_marks[0]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[1]),
        car_bits!(0, impact_octant_marks[1]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[2]),
        car_bits!(0, impact_octant_marks[2]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[3]),
        car_bits!(0, impact_octant_marks[3]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[4]),
        car_bits!(0, impact_octant_marks[4]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[5]),
        car_bits!(0, impact_octant_marks[5]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[6]),
        car_bits!(0, impact_octant_marks[6]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[0].block[7]),
        car_bits!(0, impact_octant_marks[7]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[0]),
        car_bits!(1, engine_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[2]),
        car_bits!(1, exhaust_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[3]),
        car_bits!(1, electrics_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[4]),
        car_bits!(1, turbo_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[6]),
        car_bits!(1, cooling_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[7]),
        car_bits!(1, steering_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[8]),
        car_bits!(1, clutch_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[10]),
        car_bits!(1, differential_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[27]),
        car_bits!(1, overall_suspension_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[35]),
        car_bits!(1, tyre_wear),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[30]),
        car_bits!(1, brake_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[22]),
        car_bits!(1, lights_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[9]),
        car_bits!(1, gearbox_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].damage[1]),
        car_bits!(1, body_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[0]),
        car_bits!(1, gear_wear[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[2]),
        car_bits!(1, gear_wear[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[3]),
        car_bits!(1, gear_wear[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[4]),
        car_bits!(1, gear_wear[4]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[5]),
        car_bits!(1, gear_wear[5]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[6]),
        car_bits!(1, gear_wear[6]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[7]),
        car_bits!(1, gear_wear[7]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[8]),
        car_bits!(1, gear_wear[8]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[10]),
        car_bits!(1, wheel_hit_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[11]),
        car_bits!(1, wheel_hit_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[12]),
        car_bits!(1, wheel_hit_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[13]),
        car_bits!(1, wheel_hit_damage[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[14]),
        car_bits!(1, body_hit_damage),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[16]),
        car_bits!(1, light_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[17]),
        car_bits!(1, light_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[18]),
        car_bits!(1, light_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[19]),
        car_bits!(1, light_damage[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[20]),
        car_bits!(1, light_damage[4]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[21]),
        car_bits!(1, light_damage[5]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[23]),
        car_bits!(1, suspension_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[24]),
        car_bits!(1, suspension_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[25]),
        car_bits!(1, suspension_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[26]),
        car_bits!(1, suspension_damage[3]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[28]),
        car_bits!(1, brake_wear_a),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[29]),
        car_bits!(1, brake_wear_b),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[31]),
        car_bits!(1, part_hit_damage[0]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[32]),
        car_bits!(1, part_hit_damage[1]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[33]),
        car_bits!(1, part_hit_damage[2]),
    ),
    FloatToInt(
        Cell(&RACE_STATE.settings.cars[1].wear[34]),
        car_bits!(1, part_hit_damage[3]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[0]),
        car_bits!(1, impact_octant_marks[0]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[1]),
        car_bits!(1, impact_octant_marks[1]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[2]),
        car_bits!(1, impact_octant_marks[2]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[3]),
        car_bits!(1, impact_octant_marks[3]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[4]),
        car_bits!(1, impact_octant_marks[4]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[5]),
        car_bits!(1, impact_octant_marks[5]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[6]),
        car_bits!(1, impact_octant_marks[6]),
    ),
    Copy(
        Cell(&RACE_STATE.results.results[1].block[7]),
        car_bits!(1, impact_octant_marks[7]),
    ),
    Call(Addr(0x449957)),
    Load(car!(0, stop_state)),
    Store(Cell(&RACE_STATE.results.results[0].result)),
    Load(car!(1, stop_state)),
    Store(Cell(&RACE_STATE.results.results[1].result)),
    FloatCopy(
        Bits(&RACE_STATE.results.results[0].secs),
        car_bits!(0, top_speed),
    ),
    FloatCopy(
        Bits(&RACE_STATE.results.results[1].secs),
        car_bits!(1, top_speed),
    ),
    Copy(
        Cell(&RACE_STATE.results.flags_current),
        car!(0, parts_present),
    ),
    Copy(
        Cell(&RACE_STATE.results.flags_first),
        car!(1, parts_present),
    ),
    Text(c"[car1.shauncarnum1]", false),
    Value(car!(0, car_kind), Kind::Int),
    Text(c"[car1.carpartsleft]", false),
    Value(car!(0, parts_present), Kind::Hex8),
    Text(c"[car1.shauntyretype1]", false),
    Value(car!(0, tyre_kind), Kind::Int),
    Text(c"[car1.dam_engine]", false),
    Float(car_bits!(0, engine_damage)),
    Text(c"[car1.dam_exhaust]", false),
    Float(car_bits!(0, exhaust_damage)),
    Text(c"[car1.dam_elect]", false),
    Float(car_bits!(0, electrics_damage)),
    Text(c"[car1.dam_turbo]", false),
    Float(car_bits!(0, turbo_damage)),
    Text(c"[car1.dam_cooling]", false),
    Float(car_bits!(0, cooling_damage)),
    Text(c"[car1.dam_steer]", false),
    Float(car_bits!(0, steering_damage)),
    Text(c"[car1.dam_clutch]", false),
    Float(car_bits!(0, clutch_damage)),
    Text(c"[car1.dam_diff]", false),
    Float(car_bits!(0, differential_damage)),
    Text(c"[car1.dam_susp]", false),
    Float(car_bits!(0, overall_suspension_damage)),
    Text(c"[car1.dam_tyres]", false),
    Float(car_bits!(0, tyre_wear)),
    Text(c"[car1.dam_brakes]", false),
    Float(car_bits!(0, brake_damage)),
    Text(c"[car1.dam_lights]", false),
    Float(car_bits!(0, lights_damage)),
    Text(c"[car1.dam_gearbox]", false),
    Float(car_bits!(0, gearbox_damage)),
    Text(c"[car1.dam_body]", false),
    Float(car_bits!(0, body_damage)),
    Text(c"[car1.dam_gearrvs]", false),
    Float(car_bits!(0, gear_wear[0])),
    Text(c"[car1.dam_gear1]", false),
    Float(car_bits!(0, gear_wear[2])),
    Text(c"[car1.dam_gear2]", false),
    Float(car_bits!(0, gear_wear[3])),
    Text(c"[car1.dam_gear3]", false),
    Float(car_bits!(0, gear_wear[4])),
    Text(c"[car1.dam_gear4]", false),
    Float(car_bits!(0, gear_wear[5])),
    Text(c"[car1.dam_gear5]", false),
    Float(car_bits!(0, gear_wear[6])),
    Text(c"[car1.dam_gear6]", false),
    Float(car_bits!(0, gear_wear[7])),
    Text(c"[car1.dam_gear7]", false),
    Float(car_bits!(0, gear_wear[8])),
    Text(c"[car1.dam_whl1]", false),
    Float(car_bits!(0, wheel_hit_damage[0])),
    Text(c"[car1.dam_whl2]", false),
    Float(car_bits!(0, wheel_hit_damage[1])),
    Text(c"[car1.dam_whl3]", false),
    Float(car_bits!(0, wheel_hit_damage[2])),
    Text(c"[car1.dam_whl4]", false),
    Float(car_bits!(0, wheel_hit_damage[3])),
    Text(c"[car1.dam_bdywrk]", false),
    Float(car_bits!(0, body_hit_damage)),
    Text(c"[car1.dam_lights1]", false),
    Float(car_bits!(0, light_damage[0])),
    Text(c"[car1.dam_lights2]", false),
    Float(car_bits!(0, light_damage[1])),
    Text(c"[car1.dam_lights3]", false),
    Float(car_bits!(0, light_damage[2])),
    Text(c"[car1.dam_lights4]", false),
    Float(car_bits!(0, light_damage[3])),
    Text(c"[car1.dam_lightsf]", false),
    Float(car_bits!(0, light_damage[4])),
    Text(c"[car1.dam_lightsb]", false),
    Float(car_bits!(0, light_damage[5])),
    Text(c"[car1.dam_susp1]", false),
    Float(car_bits!(0, suspension_damage[0])),
    Text(c"[car1.dam_susp2]", false),
    Float(car_bits!(0, suspension_damage[1])),
    Text(c"[car1.dam_susp3]", false),
    Float(car_bits!(0, suspension_damage[2])),
    Text(c"[car1.dam_susp4]", false),
    Float(car_bits!(0, suspension_damage[3])),
    Text(c"[car1.dam_brakef]", false),
    Float(car_bits!(0, brake_wear_a)),
    Text(c"[car1.dam_brakeb]", false),
    Float(car_bits!(0, brake_wear_b)),
    Text(c"[car1.dam_tyre1]", false),
    Float(car_bits!(0, part_hit_damage[0])),
    Text(c"[car1.dam_tyre2]", false),
    Float(car_bits!(0, part_hit_damage[1])),
    Text(c"[car1.dam_tyre3]", false),
    Float(car_bits!(0, part_hit_damage[2])),
    Text(c"[car1.dam_tyre4]", false),
    Float(car_bits!(0, part_hit_damage[3])),
    Text(c"[car1.caroctdamage0]", false),
    Float(car_bits!(0, impact_octant_marks[0])),
    Text(c"[car1.caroctdamage1]", false),
    Float(car_bits!(0, impact_octant_marks[1])),
    Text(c"[car1.caroctdamage2]", false),
    Float(car_bits!(0, impact_octant_marks[2])),
    Text(c"[car1.caroctdamage3]", false),
    Float(car_bits!(0, impact_octant_marks[3])),
    Text(c"[car1.caroctdamage4]", false),
    Float(car_bits!(0, impact_octant_marks[4])),
    Text(c"[car1.caroctdamage5]", false),
    Float(car_bits!(0, impact_octant_marks[5])),
    Text(c"[car1.caroctdamage6]", false),
    Float(car_bits!(0, impact_octant_marks[6])),
    Text(c"[car1.caroctdamage7]", false),
    Float(car_bits!(0, impact_octant_marks[7])),
    JumpIfNot(var!(RACE_VIEW.players), 0x2, 333),
    Text(c"[car2.shauncarnum1]", false),
    Value(car!(1, car_kind), Kind::Int),
    Text(c"[car2.carpartsleft]", false),
    Value(car!(1, parts_present), Kind::Hex8),
    Text(c"[car2.shauntyretype1]", false),
    Value(car!(1, tyre_kind), Kind::Int),
    Text(c"[car2.dam_engine]", false),
    Float(car_bits!(1, engine_damage)),
    Text(c"[car2.dam_exhaust]", false),
    Float(car_bits!(1, exhaust_damage)),
    Text(c"[car2.dam_elect]", false),
    Float(car_bits!(1, electrics_damage)),
    Text(c"[car2.dam_turbo]", false),
    Float(car_bits!(1, turbo_damage)),
    Text(c"[car2.dam_cooling]", false),
    Float(car_bits!(1, cooling_damage)),
    Text(c"[car2.dam_steer]", false),
    Float(car_bits!(1, steering_damage)),
    Text(c"[car2.dam_clutch]", false),
    Float(car_bits!(1, clutch_damage)),
    Text(c"[car2.dam_diff]", false),
    Float(car_bits!(1, differential_damage)),
    Text(c"[car2.dam_susp]", false),
    Float(car_bits!(1, overall_suspension_damage)),
    Text(c"[car2.dam_tyres]", false),
    Float(car_bits!(1, tyre_wear)),
    Text(c"[car2.dam_brakes]", false),
    Float(car_bits!(1, brake_damage)),
    Text(c"[car2.dam_lights]", false),
    Float(car_bits!(1, lights_damage)),
    Text(c"[car2.dam_gearbox]", false),
    Float(car_bits!(1, gearbox_damage)),
    Text(c"[car2.dam_body]", false),
    Float(car_bits!(1, body_damage)),
    Text(c"[car2.dam_gearrvs]", false),
    Float(car_bits!(1, gear_wear[0])),
    Text(c"[car2.dam_gear1]", false),
    Float(car_bits!(1, gear_wear[2])),
    Text(c"[car2.dam_gear2]", false),
    Float(car_bits!(1, gear_wear[3])),
    Text(c"[car2.dam_gear3]", false),
    Float(car_bits!(1, gear_wear[4])),
    Text(c"[car2.dam_gear4]", false),
    Float(car_bits!(1, gear_wear[5])),
    Text(c"[car2.dam_gear5]", false),
    Float(car_bits!(1, gear_wear[6])),
    Text(c"[car2.dam_gear6]", false),
    Float(car_bits!(1, gear_wear[7])),
    Text(c"[car2.dam_gear7]", false),
    Float(car_bits!(1, gear_wear[8])),
    Text(c"[car2.dam_whl1]", false),
    Float(car_bits!(1, wheel_hit_damage[0])),
    Text(c"[car2.dam_whl2]", false),
    Float(car_bits!(1, wheel_hit_damage[1])),
    Text(c"[car2.dam_whl3]", false),
    Float(car_bits!(1, wheel_hit_damage[2])),
    Text(c"[car2.dam_whl4]", false),
    Float(car_bits!(1, wheel_hit_damage[3])),
    Text(c"[car2.dam_bdywrk]", false),
    Float(car_bits!(1, body_hit_damage)),
    Text(c"[car2.dam_lights1]", false),
    Float(car_bits!(1, light_damage[0])),
    Text(c"[car2.dam_lights2]", false),
    Float(car_bits!(1, light_damage[1])),
    Text(c"[car2.dam_lights3]", false),
    Float(car_bits!(1, light_damage[2])),
    Text(c"[car2.dam_lights4]", false),
    Float(car_bits!(1, light_damage[3])),
    Text(c"[car2.dam_lightsf]", false),
    Float(car_bits!(1, light_damage[4])),
    Text(c"[car2.dam_lightsb]", false),
    Float(car_bits!(1, light_damage[5])),
    Text(c"[car2.dam_susp1]", false),
    Float(car_bits!(1, suspension_damage[0])),
    Text(c"[car2.dam_susp2]", false),
    Float(car_bits!(1, suspension_damage[1])),
    Text(c"[car2.dam_susp3]", false),
    Float(car_bits!(1, suspension_damage[2])),
    Text(c"[car2.dam_susp4]", false),
    Float(car_bits!(1, suspension_damage[3])),
    Text(c"[car2.dam_brakef]", false),
    Float(car_bits!(1, brake_wear_a)),
    Text(c"[car2.dam_brakeb]", false),
    Float(car_bits!(1, brake_wear_b)),
    Text(c"[car2.dam_tyre1]", false),
    Float(car_bits!(1, part_hit_damage[0])),
    Text(c"[car2.dam_tyre2]", false),
    Float(car_bits!(1, part_hit_damage[1])),
    Text(c"[car2.dam_tyre3]", false),
    Float(car_bits!(1, part_hit_damage[2])),
    Text(c"[car2.dam_tyre4]", false),
    Float(car_bits!(1, part_hit_damage[3])),
    Text(c"[car2.caroctdamage0]", false),
    Float(car_bits!(1, impact_octant_marks[0])),
    Text(c"[car2.caroctdamage1]", false),
    Float(car_bits!(1, impact_octant_marks[1])),
    Text(c"[car2.caroctdamage2]", false),
    Float(car_bits!(1, impact_octant_marks[2])),
    Text(c"[car2.caroctdamage3]", false),
    Float(car_bits!(1, impact_octant_marks[3])),
    Text(c"[car2.caroctdamage4]", false),
    Float(car_bits!(1, impact_octant_marks[4])),
    Text(c"[car2.caroctdamage5]", false),
    Float(car_bits!(1, impact_octant_marks[5])),
    Text(c"[car2.caroctdamage6]", false),
    Float(car_bits!(1, impact_octant_marks[6])),
    Text(c"[car2.caroctdamage7]", false),
    Float(car_bits!(1, impact_octant_marks[7])),
    Text(c"[arcadestartposition]", false),
    Value(var!(RACE_TABLES.start_place), Kind::Int),
    Text(c"[car1.arcadeposition]", false),
    Value(car!(0, mode2_place), Kind::Int),
    Text(c"[onerace.car1arcadeposition]", false),
    Value(Cell(&RACE_STATE.results.figure_000), Kind::Int),
    Jump(341),
    Text(c"SKIPPING - REPLAY!", true),
    Text(c"COPY SHAUN TO GRAHAM", true),
    Text(c"-------------------------------------", true),
    Return,
];
