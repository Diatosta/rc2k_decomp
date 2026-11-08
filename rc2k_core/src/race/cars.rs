//! The race's cars: their entries in the per-car tables, their kinds, their grades, their
//! starts and the players' states (converted 2026-10-21 from `race::car_file_number`
//! 0x41F4FD, `car_kind_10` 0x41F4A1, `car_kind_11` 0x41F485, `kind_past_table` 0x4651AC,
//! `kind_unusable` 0x449D34, `index_of_kind` 0x437720, `players_all_out` 0x445506,
//! `reset_car_starts` 0x444BBC, `nonzero_random` 0x444C14, `seed_car_randoms` 0x444BF6,
//! `drop_absent_player` 0x436402, `grade_car_values` 0x43FE24, `grade_all_cars`
//! 0x43FDFE, `set_car_counter` 0x43A96C, `set_both_counters` 0x43A987,
//! `set_start_states` 0x446D2E and `store_results_by_mode` 0x449957; the set-up's
//! choice converted 2026-10-21 from `auto_set_up_car` 0x436F08, the opponents' draws from
//! `choose_arcade_opponents` 0x449B79; the blocks' reset converted 2026-10-21 from
//! `race::reset_car_blocks` 0x43658A; the modes' steps, the brakes and the arcade's draws
//! converted 2026-10-21 from `race::set_up_cars` 0x43603C, `set_up_mode_0_cars`
//! 0x4364BD and `set_up_mode_2_cars` 0x43613A; the race state's reset from
//! `race::reset_car_race_state` 0x444C1D; the collision points' floats and their two
//! new corners from `race::set_up_car_points` 0x426B67).

use crate::Invalid;
use crate::math::random::Random;

/// The cars the per-car tables hold (the file numbers `dword_5C5928`, the kinds
/// `dword_5C5818` / `dword_5C58A0`, the records at 0x5C790C).
pub const TABLE_CARS: u32 = 0x22;

/// A car's entry in the per-car tables: the car held to 0 ..= 0x21 (signed).
pub fn table_entry(car: u32) -> usize {
    (car as i32).clamp(0, TABLE_CARS as i32 - 1) as usize
}

/// The kinds the kind tables hold.
pub const KINDS: u32 = 0x17;

/// The kind no race uses (`kind_unusable`).
pub const UNUSABLE_KIND: u32 = 6;

/// A test's answer as the original leaves it in the flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flags {
    pub carry: bool,
    pub zero: bool,
}

/// `race::kind_past_table`: carry for a kind of 0x17 or more; zero as `cmp eax, 17h`
/// leaves it.
pub fn kind_past_table(kind: u32) -> Flags {
    Flags {
        carry: kind >= KINDS,
        zero: kind == KINDS,
    }
}

/// `race::kind_unusable`: carry for a kind past the table or 6; zero as the last `cmp`
/// leaves it (against 0x17 when past, else against 6).
pub fn kind_unusable(kind: u32) -> Flags {
    let past = kind_past_table(kind);
    if past.carry {
        return past;
    }
    let six = kind == UNUSABLE_KIND;
    Flags {
        carry: six,
        zero: six,
    }
}

/// The kinds in the order `dword_6051E0` keeps (0x22).
pub const KIND_ORDER_LEN: u32 = 0x22;

/// `race::index_of_kind`: the place of `kind` in the kind order (the first of equals),
/// None for none; the order is read entry by entry, up to the one found.
pub fn index_of_kind(kind: u32, order: impl IntoIterator<Item = u32>) -> Option<u32> {
    order
        .into_iter()
        .take(KIND_ORDER_LEN as usize)
        .position(|k| k == kind)
        .map(|n| n as u32)
}

/// Whose state `players_all_out` looks at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Players {
    /// The one chosen car (`dword_52A910` 1, the car `dword_52A920`).
    Chosen(u32),
    /// The first n cars (`dword_63DB94`).
    First(u32),
}

/// `race::players_all_out`: whether every player is out (`out` asked of each car, in
/// order, up to the first still in). None in: false. Refuses a chosen car past 7 or more
/// than 8 players (the original reads past the blocks).
pub fn players_all_out(players: Players, out: impl Fn(u32) -> bool) -> Result<bool, Invalid> {
    match players {
        Players::Chosen(n) if n > 7 => Err(Invalid("a chosen car past the eight")),
        Players::Chosen(n) => Ok(out(n)),
        Players::First(n) if n > 8 => Err(Invalid("more than eight players")),
        Players::First(0) => Ok(false),
        Players::First(n) => Ok((0..n).all(out)),
    }
}

/// `race::drop_absent_player`: the player who is out of a two-player race: unless the
/// race's mode (`dword_62389C`) is 2 or 4, with two players (asked then), the first of the
/// two whose block's +0xB0 is set (`absent`, asked in order).
pub fn absent_player(
    mode: u32,
    players: impl FnOnce() -> u32,
    absent: impl Fn(u32) -> bool,
) -> Option<u32> {
    if mode == 2 || mode == 4 || players() != 2 {
        return None;
    }
    (0..2).find(|&n| absent(n))
}

/// `race::reset_car_starts`: the value each block's +0x14 and +8 start at: 0x320 while a
/// replay runs (`dword_71C3E0`), else 0.
pub fn start_value(replaying: bool) -> u32 {
    if replaying { 0x320 } else { 0 }
}

/// `race::nonzero_random`: the next random number, 1 in place of 0.
pub fn nonzero_random(random: &mut Random) -> u32 {
    match random.draw() {
        0 => 1,
        n => n,
    }
}

/// `race::seed_car_randoms`: the eight blocks' seeds (+0x10 and +0xC), one
/// `nonzero_random` each, in the blocks' order.
pub fn car_seeds(random: &mut Random) -> [u32; 8] {
    std::array::from_fn(|_| nonzero_random(random))
}

/// A value's grade: how many of the thresholds, taken in order, lie below it (`fild t;
/// fcomp x` then `jnb` on C0: a NaN, unordered, passes them all); and the last threshold
/// tried (the original's way in to the x87, `dword_62400C`).
pub fn grade(value: f32, thresholds: &[u32]) -> (u32, u32) {
    let mut grade = 0u32;
    let mut tried = 0;
    for &t in thresholds {
        tried = t;
        // Exact: the thresholds are small whole numbers.
        if !((t as f32) < value || value.is_nan()) {
            break;
        }
        grade += 1;
    }
    (grade, tried)
}

/// What a car's message (the panel's line at its block's +0x71C, shown 0x12C frames from
/// +0x72C's start, +0x730 its age) becomes when `race::set_car_counter` sets it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    /// +0x71C: the message (a string id).
    pub id: u32,
    /// +0x72C: how long it stays.
    pub time_left: u32,
    /// +0x730.
    pub age: u32,
}

impl Message {
    /// How long a new message stays.
    pub const SHOWN_FOR: u32 = 0x12C;

    /// A new message.
    pub fn new(id: u32) -> Self {
        Self {
            id,
            time_left: Self::SHOWN_FOR,
            age: 0,
        }
    }
}

/// A car record's limit (`dword_5C7980`) past which its start value is 0.
pub const START_LIMIT: u32 = 0x1B58;

/// `race::set_start_states`: the car record a kind's start value comes from (the kind
/// held to 0x21, unsigned).
pub fn start_record(kind: u32) -> u32 {
    kind.min(TABLE_CARS - 1)
}

/// `race::set_start_states`: a record's start value: its value (+0x18) unless its limit
/// is past `START_LIMIT`, then 0. `value` is asked only when used.
pub fn start_state_value(limit: u32, value: impl FnOnce() -> u32) -> u32 {
    if limit > START_LIMIT { 0 } else { value() }
}

/// `race::set_start_states`: the split choice (`dword_6238A4`) for one player (0) or
/// two (held to 1 ..= 2 by the original's two sign tests, `cmp` / `jns` / `js`, wrapping
/// as they do).
pub fn held_split(two: bool, choice: i32) -> i32 {
    if !two {
        return 0;
    }
    let mut h = choice;
    if h.wrapping_sub(1) < 0 {
        h = 1;
    }
    if h.wrapping_sub(2) >= 0 {
        h = 2;
    }
    h
}

/// `race::set_start_states`: a block's start state (+0x684) settled: 2 or 3 become 4,
/// None (left) otherwise.
pub fn settled_state(state: u32) -> Option<u32> {
    matches!(state, 2 | 3).then_some(4)
}

/// The longest result `store_results_by_mode` keeps in mode 0 (300000).
pub const MOST_MARGIN: i32 = 300_000;

/// The last stage `store_results_by_mode` holds to (0x4C).
pub const LAST_STAGE: i32 = 0x4C;

/// `race::store_results_by_mode` (mode 0): the stage held to 0 ..= 0x4C (the original's
/// sign tests, wrapping).
pub fn held_stage(stage: i32) -> u32 {
    let mut s = stage.max(0);
    if s.wrapping_sub(LAST_STAGE) >= 0 {
        s = LAST_STAGE;
    }
    s as u32
}

/// `race::store_results_by_mode` (mode 0): a car's margin: the stage's total
/// (`g_stageCarTotal`) x 100 less its time, held to 0 ..= 300000 (signed, the sign tests
/// wrapping).
pub fn margin(total: u32, time: u32) -> u32 {
    let mut v = (total.wrapping_mul(100).wrapping_sub(time) as i32).max(0);
    if v.wrapping_sub(MOST_MARGIN) >= 0 {
        v = MOST_MARGIN;
    }
    v as u32
}

/// The first stage with an automatic set-up (`race::auto_set_up_car`).
pub const FIRST_SET_UP_STAGE: u32 = 0x15;
/// The stages with one end before this.
pub const SET_UP_STAGES_END: u32 = 0x50;

/// The set-up `race::auto_set_up_car` picks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetUpPick {
    /// Its number in `dword_6043FC` (16 bytes each): (stage - 0x15) / 10 x 6 + the rest.
    pub index: u32,
    /// (stage - 0x15) % 10.
    pub rest: u32,
}

/// `race::auto_set_up_car`: the set-up for `stage` (0x15 .. 0x4F; None for the others).
/// Refused for stage 0x14, which the original's range test lets in: its `sub` makes an
/// index of 0x9999999B (a read far outside).
pub fn set_up_pick(stage: u32) -> Result<Option<SetUpPick>, Invalid> {
    if stage == FIRST_SET_UP_STAGE - 1 {
        return Err(Invalid("stage 0x14: the set-up index runs outside"));
    }
    if !(FIRST_SET_UP_STAGE - 1..SET_UP_STAGES_END).contains(&stage) {
        return Ok(None);
    }
    let v = stage - FIRST_SET_UP_STAGE;
    Ok(Some(SetUpPick {
        index: (v / 10) * 6 + v % 10,
        rest: v % 10,
    }))
}

/// `race::choose_arcade_opponents`: a kind drawn from a random number (x 0x22, the high
/// half).
pub fn drawn_kind(random: u32) -> u32 {
    ((random as u64 * TABLE_CARS as u64) >> 32) as u32
}

/// How far apart in the kinds' order an opponent may be from the first car (below).
pub const OPPONENT_REACH: u32 = 5;

/// `race::choose_arcade_opponents`: the places' distance (`cmp` / `jns` / `neg`) and
/// whether an opponent at `place` is close enough to the first car's (unsigned, below 5).
pub fn opponent_distance(place: u32, first: u32) -> (u32, bool) {
    let d = place.wrapping_sub(first);
    let d = if (d as i32) < 0 { d.wrapping_neg() } else { d };
    (d, d < OPPONENT_REACH)
}

/// The car blocks reset for a race.
pub const CAR_BLOCKS: u32 = 8;

/// What `race::reset_car_race_state` writes into each car block (offset, value, in the
/// original's order): +0x730 0, +0x72C 0x12C, +0x724 0, +0x71C 0xCA, +0x720 0, +0x728 0,
/// the five dwords +0xA0C .. -1.
pub const RACE_STATE: [(u32, u32); 11] = [
    (0x730, 0),
    (0x72C, 0x12C),
    (0x724, 0),
    (0x71C, 0xCA),
    (0x720, 0),
    (0x728, 0),
    (0xA0C, u32::MAX),
    (0xA10, u32::MAX),
    (0xA14, u32::MAX),
    (0xA18, u32::MAX),
    (0xA1C, u32::MAX),
];
/// The race's four text dwords' reset (0x6D1838 ..): four spaces.
pub const BLANK_TEXT: u32 = 0x2020_2020;
/// The four at 0x6D1850 ..: none (-1).
pub const UNSET_TEXT: u32 = u32::MAX;

/// A car block's collision points: x, y and z lists of 0x40 floats' bits (+0xAF0,
/// +0xBF0, +0xCF0).
pub type PointLists = [[u32; 0x40]; 3];

/// `race::set_up_car_points`: a point's place (16.16) as a float (`fild` / `fdiv` by
/// 65536 / `fstp`: one rounding).
pub fn point_float(whole: u32, scale: f32) -> u32 {
    ((whole as i32 as f64 / scale as f64) as f32).to_bits()
}

/// `race::set_up_car_points`: points 4 .. 0x3D moved two on (from the last down), the
/// new 4 at 0 and the new 5 at 0 with the y of the new 6.
pub fn open_two_points(lists: &mut PointLists) {
    for n in (4..=0x3Dusize).rev() {
        for list in lists.iter_mut() {
            list[n + 2] = list[n];
        }
    }
    lists[0][4] = 0;
    lists[1][4] = 0;
    lists[2][4] = 0;
    lists[0][5] = 0;
    lists[1][5] = lists[1][6];
    lists[2][5] = 0;
}

/// One of race init's "cars" steps for a game mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarsStep {
    /// The stage's numbers read (`race::read_stage_car_numbers`).
    Numbers,
    /// The cars' times (`race::set_car_times`).
    Times,
    /// The mode's set-up: 2 the arcade's, 1, 0, 4 theirs, 3 `race::set_up_race_cars`.
    SetUp(u32),
    /// The arcade's 0x40 dwords at `dword_601C68` cleared.
    ClearArcadeTable,
}

/// `race::set_up_cars`: the steps for a mode (`dword_62389C`); none for others.
pub fn cars_steps(mode: u32) -> &'static [CarsStep] {
    use CarsStep::*;
    match mode {
        2 => &[Numbers, Times, SetUp(2), ClearArcadeTable],
        1 => &[SetUp(1)],
        3 => &[SetUp(3)],
        0 => &[Numbers, SetUp(0)],
        4 => &[SetUp(4)],
        _ => &[],
    }
}

/// A player's brake setting (`dword_75AD2C` / `dword_75AE98`) for the block's +0x3C:
/// below 0 is 0, 100 or more 0x100, else `scaled` (the x87's x 2.56, rounded).
pub fn held_brake(v: i32, scaled: impl FnOnce(i32) -> u32) -> u32 {
    if v < 0 {
        0
    } else if v.wrapping_sub(100) >= 0 {
        0x100
    } else {
        scaled(v)
    }
}

/// `race::set_up_mode_0_cars`: with every car set up automatically (`dword_52A910`) the
/// race's cars (`g_raceCars`) must be 1 .. 8 (the original's `jnz` runs on otherwise).
pub fn every_car_set_up(all: bool, cars: u32) -> Result<bool, Invalid> {
    if all && (cars == 0 || cars > CAR_BLOCKS) {
        return Err(Invalid("no cars or more than eight"));
    }
    Ok(all)
}

/// The arcade's start place: the cars' count with `dword_60101C` 2 or at the rally's
/// first stage; else it stays.
pub fn arcade_start_is_last(kind: u32, stage_in_rally: u32) -> bool {
    kind == 2 || stage_in_rally == 0
}

/// `race::set_up_mode_2_cars`: the next number a car takes, the players' numbers
/// (+0xAA8; the second only with two players) skipped in turn.
pub fn arcade_number(number: u32, first: u32, second: Option<u32>) -> u32 {
    let mut n = number;
    if n == first {
        n += 1;
    }
    if second == Some(n) {
        n += 1;
    }
    n
}

/// `race::set_up_mode_2_cars`: a car's type drawn from noise (turned right by the car's
/// index, x 3): (the product's low half, its high half: the type less one, 0 .. 2).
pub fn arcade_type_draw(noise: u32, car: u32) -> (u32, u32) {
    let p = noise.rotate_right(car & 0x1F) as u64 * 3;
    (p as u32, (p >> 32) as u32)
}

/// `race::set_up_mode_2_cars`: the place a car's number is swapped with (noise x the
/// count): (the product's low half, its high half).
pub fn arcade_swap_draw(noise: u32, cars: u32) -> (u32, u32) {
    let p = noise as u64 * cars as u64;
    (p as u32, (p >> 32) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_ups_and_opponents() {
        assert!(set_up_pick(0x14).is_err());
        assert_eq!(set_up_pick(0x13), Ok(None));
        assert_eq!(set_up_pick(0x50), Ok(None));
        assert_eq!(set_up_pick(0x15), Ok(Some(SetUpPick { index: 0, rest: 0 })));
        assert_eq!(
            set_up_pick(0x4F),
            Ok(Some(SetUpPick { index: 38, rest: 8 }))
        );
        assert_eq!(set_up_pick(0x1E), Ok(Some(SetUpPick { index: 9, rest: 9 })));
        assert_eq!(drawn_kind(u32::MAX), 0x21);
        assert_eq!(drawn_kind(0x8000_0000), 0x11);
        assert_eq!(opponent_distance(3, 7), (4, true));
        assert_eq!(opponent_distance(12, 7), (5, false));
        assert_eq!(opponent_distance(0x8000_0000, 0), (0x8000_0000, false));
    }

    #[test]
    fn tables_and_kinds() {
        assert_eq!(table_entry(u32::MAX), 0);
        assert_eq!(table_entry(5), 5);
        assert_eq!(table_entry(0x30), 0x21);
        assert_eq!(table_entry(0x8000_0000), 0);
        let f = |carry, zero| Flags { carry, zero };
        assert_eq!(kind_past_table(0x16), f(false, false));
        assert_eq!(kind_past_table(0x17), f(true, true));
        assert_eq!(kind_past_table(u32::MAX), f(true, false));
        assert_eq!(kind_unusable(6), f(true, true));
        assert_eq!(kind_unusable(5), f(false, false));
        assert_eq!(kind_unusable(0x17), f(true, true));
        assert_eq!(index_of_kind(3, [1, 3, 3]), Some(1));
        assert_eq!(index_of_kind(9, [1, 3]), None);
        assert_eq!(
            index_of_kind(9, std::iter::repeat_n(1, 0x21).chain([9])),
            Some(0x21)
        );
        assert_eq!(
            index_of_kind(9, std::iter::repeat_n(1, 0x22).chain([9])),
            None
        );
    }

    #[test]
    fn players() {
        let out = |n: u32| n != 2;
        assert_eq!(players_all_out(Players::Chosen(1), out), Ok(true));
        assert_eq!(players_all_out(Players::Chosen(2), out), Ok(false));
        assert!(players_all_out(Players::Chosen(8), out).is_err());
        assert_eq!(players_all_out(Players::First(0), out), Ok(false));
        assert_eq!(players_all_out(Players::First(2), out), Ok(true));
        assert_eq!(players_all_out(Players::First(3), out), Ok(false));
        assert!(players_all_out(Players::First(9), out).is_err());
        assert_eq!(absent_player(2, || unreachable!(), |_| true), None);
        assert_eq!(absent_player(0, || 1, |_| true), None);
        assert_eq!(absent_player(0, || 2, |n| n == 1), Some(1));
        assert_eq!(absent_player(3, || 2, |_| false), None);
    }

    #[test]
    fn randoms() {
        // A state whose next number is 0, then the generator's own run.
        let mut r = Random { state: 0 };
        assert_eq!(nonzero_random(&mut r), 1);
        let mut a = Random { state: 0x1234_5678 };
        let mut b = a;
        let seeds = car_seeds(&mut a);
        for s in seeds {
            assert_eq!(s, nonzero_random(&mut b));
        }
        assert_eq!(a, b);
        assert_eq!((start_value(true), start_value(false)), (0x320, 0));
    }

    #[test]
    fn grades() {
        let t = &[50, 75, 90, 100];
        assert_eq!(grade(0.0, t), (0, 50));
        assert_eq!(grade(50.0, t), (0, 50));
        assert_eq!(grade(50.5, t), (1, 75));
        assert_eq!(grade(100.5, t), (4, 100));
        assert_eq!(grade(f32::NAN, t), (4, 100));
        assert_eq!(grade(f32::NEG_INFINITY, &[100]), (0, 100));
    }

    #[test]
    fn starts_and_results() {
        assert_eq!(
            Message::new(7),
            Message {
                id: 7,
                time_left: 0x12C,
                age: 0
            }
        );
        assert_eq!(start_record(0x40), 0x21);
        assert_eq!(start_state_value(0x1B59, || unreachable!()), 0);
        assert_eq!(start_state_value(0x1B58, || 9), 9);
        assert_eq!(held_split(false, 5), 0);
        assert_eq!(held_split(true, 0), 1);
        assert_eq!(held_split(true, 7), 2);
        assert_eq!(held_split(true, i32::MIN), 2);
        assert_eq!(settled_state(3), Some(4));
        assert_eq!(settled_state(4), None);
        assert_eq!(held_stage(-3), 0);
        assert_eq!(held_stage(0x50), 0x4C);
        assert_eq!(held_stage(i32::MAX), 0x4C);
        assert_eq!(margin(10, 400), 600);
        assert_eq!(margin(10, 2000), 0);
        assert_eq!(margin(4000, 0), 300_000);
    }

    #[test]
    fn modes_and_draws() {
        assert_eq!(cars_steps(0), [CarsStep::Numbers, CarsStep::SetUp(0)]);
        assert_eq!(cars_steps(2).len(), 4);
        assert!(cars_steps(5).is_empty());
        assert_eq!(held_brake(-1, |_| unreachable!()), 0);
        assert_eq!(held_brake(100, |_| unreachable!()), 0x100);
        assert_eq!(held_brake(50, |v| v as u32 * 2), 100);
        assert!(every_car_set_up(true, 0).is_err() && every_car_set_up(true, 9).is_err());
        assert_eq!(every_car_set_up(false, 0), Ok(false));
        assert_eq!(every_car_set_up(true, 8), Ok(true));
        assert!(arcade_start_is_last(2, 3) && arcade_start_is_last(0, 0));
        assert!(!arcade_start_is_last(1, 1));
        assert_eq!(arcade_number(3, 3, Some(4)), 5);
        assert_eq!(arcade_number(3, 4, Some(3)), 4);
        assert_eq!(arcade_number(3, 3, None), 4);
        assert_eq!(arcade_type_draw(0x8000_0000, 0), (0x8000_0000, 1));
        assert_eq!(arcade_type_draw(1, 1), (0x8000_0000, 1));
        assert_eq!(arcade_swap_draw(0xFFFF_FFFF, 4), (0xFFFF_FFFC, 3));
    }

    #[test]
    fn collision_points() {
        assert_eq!(point_float(0x18000, 65536.0), 1.5f32.to_bits());
        assert_eq!(point_float(0xFFFF_0000, 65536.0), (-1.0f32).to_bits());
        let mut l: PointLists = [[0; 0x40]; 3];
        for (k, list) in l.iter_mut().enumerate() {
            for (n, v) in list.iter_mut().enumerate() {
                *v = (k as u32) << 8 | n as u32;
            }
        }
        open_two_points(&mut l);
        assert_eq!(l[0][..8], [0, 1, 2, 3, 0, 0, 4, 5]);
        assert_eq!(l[1][5], 0x104);
        assert_eq!(l[2][0x3F], 0x23D);
    }
}
