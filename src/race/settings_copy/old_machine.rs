//! Test-only reference: the step-list machine (`Step`, `Machine`, `run_steps`, the step tables of
//! `graham_to_shaun` and `shaun_to_graham`) that `copy_settings_in` / `copy_settings_out` were
//! before they became plain loops, kept verbatim (the log and the three routines go through
//! `Effects` so a test can capture them; the routine operands are marker cells). The test
//! `new_copies_match_the_machine` runs both over randomised states.

use super::*;
use crate::race::owner::race;
use crate::session::DwordLike;
use Step::*;
use std::ffi::{CStr, CString};

/// How a value is logged after its word (each a `log_..._nl`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Int,
    Hex8,
}

/// One step; jump targets are step numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step<A = usize> {
    /// The text logged (`log_write_l1`, `_nl` with true).
    Text(&'static core::ffi::CStr, bool),
    /// The dword logged (`log_int_l1_nl` / `log_hex8_l1_nl`).
    Value(A, Kind),
    /// The float logged (`log_float_l1_nl`, two decimals).
    Float(A),
    /// `push from` / `pop to`.
    Copy(A, A),
    /// `dec` of the dword.
    Dec(A),
    /// `fild from` / `fstp to`.
    IntToFloat(A, A),
    /// `fld from` / `fistp to`.
    FloatToInt(A, A),
    /// `fld from` / `fstp to`.
    FloatCopy(A, A),
    /// The dword from held: below 0 is 0, 100 or more 0x100, else x 2.56 rounded, into
    /// to.
    Brake(A, A),
    /// `mov ebx, [var]`; `cmp ebx, 0`; `jz`; `or eax, bit`.
    BitIf(A, u32),
    /// eax the dword.
    Load(A),
    /// eax the value.
    LoadImm(u32),
    /// The dword eax.
    Store(A),
    /// eax less the dword.
    SubEax(A),
    /// `cmp eax, value` / `jnz`.
    JumpIfEaxNot(u32, usize),
    /// `cmp [var], value` / `jnz`.
    JumpIfNot(A, u32, usize),
    Jump(usize),
    /// A call.
    Call(A),
    Return,
}

/// What the steps run on.
pub trait Machine<A> {
    type Error;
    fn get(&mut self, at: A) -> u32;
    fn set(&mut self, at: A, value: u32);
    fn eax(&self) -> u32;
    fn set_eax(&mut self, value: u32);
    fn set_ebx(&mut self, value: u32);
    /// `fild` / `fstp` of a whole number.
    fn int_to_float(&mut self, value: u32) -> u32;
    /// `fld` / `fistp` of a float (the control word's rounding).
    fn float_to_int(&mut self, value: u32) -> u32;
    /// `fld` / `fstp` of a float.
    fn float_copy(&mut self, value: u32) -> u32;
    /// A brake value 0 .. 99 x 2.56, rounded (`fild` / `fmul` / `fistp`).
    fn scaled_brake(&mut self, value: i32) -> u32;
    /// The logs (inside `pusha` / `popa`: the registers kept).
    fn text(&mut self, text: &'static core::ffi::CStr, newline: bool) -> Result<(), Self::Error>;
    fn value(&mut self, value: u32, kind: Kind) -> Result<(), Self::Error>;
    fn float(&mut self, bits: u32) -> Result<(), Self::Error>;
    fn call(&mut self, at: A) -> Result<(), Self::Error>;
}

/// The steps from the first, as the original runs them, up to the `Return`.
pub fn run_steps<A: std::marker::Copy, M: Machine<A>>(
    steps: &[Step<A>],
    m: &mut M,
) -> Result<(), M::Error> {
    use Step::*;
    let mut pc = 0;
    loop {
        let step = steps[pc];
        pc += 1;
        match step {
            Text(text, newline) => m.text(text, newline)?,
            Value(v, kind) => {
                let v = m.get(v);
                m.value(v, kind)?;
            }
            Float(v) => {
                let v = m.get(v);
                m.float(v)?;
            }
            Copy(to, from) => {
                let v = m.get(from);
                m.set(to, v);
            }
            Dec(at) => {
                let v = m.get(at).wrapping_sub(1);
                m.set(at, v);
            }
            IntToFloat(to, from) => {
                let v = m.get(from);
                let v = m.int_to_float(v);
                m.set(to, v);
            }
            FloatToInt(to, from) => {
                let v = m.get(from);
                let v = m.float_to_int(v);
                m.set(to, v);
            }
            FloatCopy(to, from) => {
                let v = m.get(from);
                let v = m.float_copy(v);
                m.set(to, v);
            }
            Brake(to, from) => {
                let v = m.get(from) as i32;
                let v = rc2k_core::race::cars::held_brake(v, |v| m.scaled_brake(v));
                m.set(to, v);
            }
            BitIf(var, bit) => {
                let v = m.get(var);
                m.set_ebx(v);
                if v != 0 {
                    let eax = m.eax() | bit;
                    m.set_eax(eax);
                }
            }
            Load(at) => {
                let v = m.get(at);
                m.set_eax(v);
            }
            LoadImm(v) => m.set_eax(v),
            Store(at) => {
                let v = m.eax();
                m.set(at, v);
            }
            SubEax(at) => {
                let v = m.eax().wrapping_sub(m.get(at));
                m.set_eax(v);
            }
            JumpIfEaxNot(v, to) => {
                if m.eax() != v {
                    pc = to;
                }
            }
            JumpIfNot(var, v, to) => {
                if m.get(var) != v {
                    pc = to;
                }
            }
            Jump(to) => pc = to,
            Call(at) => m.call(at)?,
            Return => return Ok(()),
        }
    }
}

/// A routine a `Call` step runs (the original's `call` at its address).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Routine {
    ClearPlayerAbsences,
    CopyLeagueTable,
    StoreResultsByMode,
}

static ROUTINE_MARKS: [Dword; 3] = [Dword::new(0), Dword::new(0), Dword::new(0)];

/// The operand a `Call` step carries for a routine: a marker cell, found again by address.
const fn routine(r: Routine) -> StepOperand {
    StepOperand::Cell(&ROUTINE_MARKS[r as usize])
}

fn routine_of(a: StepOperand) -> Routine {
    for r in [
        Routine::ClearPlayerAbsences,
        Routine::CopyLeagueTable,
        Routine::StoreResultsByMode,
    ] {
        if let StepOperand::Cell(c) = a
            && std::ptr::eq(c, std::ptr::from_ref(&ROUTINE_MARKS[r as usize]))
        {
            return r;
        }
    }
    unreachable!("a call of a variable");
}

/// The game as the steps' machine: its dwords at IDA addresses, the steps' own eax and ebx
/// (the original reads no register at entry: they start from nothing), the x87's
/// conversions, and the logs and calls.
struct Game<'a, E: Effects> {
    race: &'a Race,
    fx: &'a mut E,
    eax: u32,
    ebx: u32,
}

impl<E: Effects> Machine<StepOperand> for Game<'_, E> {
    type Error = Refused;
    fn get(&mut self, a: StepOperand) -> u32 {
        a.get()
    }
    fn set(&mut self, a: StepOperand, v: u32) {
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
        x87::from_int(v as i32).to_bits()
    }
    fn float_to_int(&mut self, v: u32) -> u32 {
        x87::to_int(v) as u32
    }
    fn float_copy(&mut self, v: u32) -> u32 {
        x87::loaded(v).to_bits()
    }
    fn scaled_brake(&mut self, v: i32) -> u32 {
        x87::int_times_to_int(v, &self.race.course.numbers.brake_scale) as u32
    }
    fn text(&mut self, text: &'static std::ffi::CStr, nl: bool) -> Result<(), Refused> {
        self.fx.text(text.to_str().expect("ASCII"), nl);
        Ok(())
    }
    fn value(&mut self, v: u32, kind: Kind) -> Result<(), Refused> {
        if kind == Kind::Int {
            self.fx.int(v);
        } else {
            self.fx.hex8(v);
        }
        Ok(())
    }
    fn float(&mut self, v: u32) -> Result<(), Refused> {
        self.fx.float(v);
        Ok(())
    }
    fn call(&mut self, a: StepOperand) -> Result<(), Refused> {
        match routine_of(a) {
            Routine::ClearPlayerAbsences => self.fx.clear_player_absences(self.race),
            Routine::CopyLeagueTable => self.fx.copy_league_table(self.race)?,
            Routine::StoreResultsByMode => self.fx.store_results_by_mode(self.race),
        }
        Ok(())
    }
}

/// The steps from the first, in the original's order.
pub fn run(race: &Race, fx: &mut impl Effects, steps: &[Step<StepOperand>]) -> Result<(), Refused> {
    run_steps(
        steps,
        &mut Game {
            race,
            fx,
            eax: 0,
            ebx: 0,
        },
    )
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

/// The race's settings that are logged after the copy: the log text and the variable.
const SHAUN_LOGS: [(&CStr, StepOperand); 14] = [
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
struct Builder(Vec<Step<StepOperand>>);

impl Builder {
    fn push(&mut self, step: Step<StepOperand>) {
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

/// The steps of `copy_settings_in` (the front end's settings into the race's), in the
/// original's order.
pub fn graham_to_shaun() -> Vec<Step<StepOperand>> {
    let shared = &race().shared;
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
        let settings = |f: fn(&RaceCar) -> &Dword| Cell(f(&shared.state.settings.cars[car]));
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
        b.push(Load(Cell(&shared.state.settings.cars[car].gearing)));
        b.push(Store(Var(colour)));
    }
    for car in 0..2 {
        b.push(Copy(
            CarCell(car, |c| &c.start_slot_value),
            Cell(&shared.state.settings.cars[car].race_number),
        ));
    }
    for car in 0..2 {
        b.push(Load(Cell(&shared.state.settings.cars[car].view)));
        b.push(Store(CarCell(car, |c| &c.camera_view)));
        b.push(Store(CarCell(car, |c| &c.camera_blend_view)));
    }
    let hud_bits = [
        &shared.state.results.hud_bits_current,
        &shared.state.results.hud_bits_first,
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
            b.push(BitIf(Cell(field(&shared.state.settings.cars[car])), bit));
        }
        b.0[jump] = JumpIfEaxNot(0xFFFF_FFFF, b.here());
        b.push(Store(CarCell(car, |c| &c.option_bits)));
        b.push(Store(Var(PLAYER_OPTIONS[car])));
    }
    for car in 0..2 {
        for (field, setting) in GAUGES {
            b.push(IntToFloat(
                CarBits(car, field),
                Cell(setting(&shared.state.settings.cars[car])),
            ));
        }
        for (mark, cell) in IMPACT_MARKS.into_iter().zip(result_marks(shared, car)) {
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
static HEAD_IN: [Step<StepOperand>; 21] = [
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
        var!(@gfx display.choices.mipmapping),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[0]),
        Cell(&crate::race::owner::RACE.shared.state.settings.lens_flare),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[1]),
        Cell(&crate::race::owner::RACE.shared.state.settings.reflections),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[2]),
        Cell(&crate::race::owner::RACE.shared.state.settings.car_shadows),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[3]),
        Cell(&crate::race::owner::RACE.shared.state.settings.car_detail),
    ),
    Copy(
        var!(crate::race::owner::RACE.shared.vars.settings_words[4]),
        Cell(&crate::race::owner::RACE.shared.state.settings.headlights),
    ),
];

/// The cars' part flags and the arcade start place into the results.
static FLAGS_IN: [Step<StepOperand>; 4] = [
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
static TAIL_IN: [Step<StepOperand>; 15] = [
    Call(routine(Routine::ClearPlayerAbsences)),
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
    Call(routine(Routine::CopyLeagueTable)),
    Text(c"COPY GRAHAM TO SHAUN", true),
    Text(c"-------------------------------------", true),
    Return,
];

/// The arcade positions logged at the end of the copy back.
static TAIL_OUT: [Step<StepOperand>; 6] = [
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
pub fn shaun_to_graham() -> Vec<Step<StepOperand>> {
    let shared = &race().shared;
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
        Cell(&shared.state.results.hud_bits_current),
        CarCell(0, |c| &c.option_bits),
    ));
    b.push(Copy(
        Cell(&shared.state.results.hud_bits_first),
        CarCell(1, |c| &c.option_bits),
    ));
    for car in 0..2 {
        for (field, setting) in GAUGES {
            b.push(FloatToInt(
                Cell(setting(&shared.state.settings.cars[car])),
                CarBits(car, field),
            ));
        }
        for (mark, cell) in IMPACT_MARKS.into_iter().zip(result_marks(shared, car)) {
            b.push(Copy(Cell(cell), CarBits(car, mark)));
        }
    }
    b.push(Call(routine(Routine::StoreResultsByMode)));
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
const CAR_COLOURS: [fn() -> &'static dyn DwordLike; 2] = [
    || &crate::race::owner::RACE.shared.vars.car_colours[0],
    || &crate::race::owner::RACE.shared.vars.car_colours[1],
];
const PLAYER_OPTIONS: [fn() -> &'static dyn DwordLike; 2] = [
    || &crate::race::owner::RACE.shared.view.player_options[0],
    || &crate::race::owner::RACE.shared.view.player_options[1],
];
