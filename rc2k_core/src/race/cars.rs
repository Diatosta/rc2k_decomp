//! The race's cars: their entries in the per-car tables, kinds, grades, starts and players'
//! states; each game mode's set-up, the automatic set-ups, the arcade draws and collision points.

use crate::Invalid;
use crate::math::random::Random;

/// The cars the per-car tables hold (file numbers, kinds, car records).
pub const TABLE_CARS: u32 = 0x22;

/// A car's entry in the per-car tables: the car held to 0 ..= 0x21 (signed).
pub fn table_entry(car: u32) -> usize {
    (car as i32).clamp(0, TABLE_CARS as i32 - 1) as usize
}

/// The kinds the kind tables hold.
pub const KINDS: u32 = 0x17;

/// The kind no race uses.
pub const UNUSABLE_KIND: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KindCheckFlags {
    pub carry: bool,
    pub zero: bool,
}

/// Carry for a kind of 0x17 or more; zero when it equals 0x17.
pub fn kind_past_table(kind: u32) -> KindCheckFlags {
    KindCheckFlags {
        carry: kind >= KINDS,
        zero: kind == KINDS,
    }
}

/// Carry for a kind past the table or 6; zero as a comparison against 0x17 when past, else 6.
pub fn kind_unusable(kind: u32) -> KindCheckFlags {
    let past = kind_past_table(kind);
    if past.carry {
        return past;
    }
    let six = kind == UNUSABLE_KIND;
    KindCheckFlags {
        carry: six,
        zero: six,
    }
}

/// The kinds the kind order lists.
pub const KIND_ORDER_LEN: u32 = 0x22;

/// The place of `kind` in the kind order (the first of equals), None when absent.
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
    /// The one chosen car.
    Chosen(u32),
    /// The first n cars.
    First(u32),
}

/// Whether every player is out (`out` asked in order up to the first still in); no players:
/// false. Refuses a chosen car past 7 or more than 8 players.
pub fn players_all_out(players: Players, out: impl Fn(u32) -> bool) -> Result<bool, Invalid> {
    match players {
        Players::Chosen(n) if n > 7 => Err(Invalid("a chosen car past the eight")),
        Players::Chosen(n) => Ok(out(n)),
        Players::First(n) if n > 8 => Err(Invalid("more than eight players")),
        Players::First(0) => Ok(false),
        Players::First(n) => Ok((0..n).all(out)),
    }
}

/// The player who is out of a two-player race: not in modes 2 and 4, and only with two players;
/// the first of the two for whom `absent` is set.
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

/// The value each block's start fields begin at: 0x320 while a replay runs, else 0.
pub fn start_value(replaying: bool) -> u32 {
    if replaying { 0x320 } else { 0 }
}

/// The next random number, 1 in place of 0.
pub fn nonzero_random(random: &mut Random) -> u32 {
    match random.draw() {
        0 => 1,
        n => n,
    }
}

/// The eight car blocks' seeds: one `nonzero_random` each, in the blocks' order.
pub fn car_seeds(random: &mut Random) -> [u32; 8] {
    std::array::from_fn(|_| nonzero_random(random))
}

/// A value's grade: how many of the thresholds, in order, lie below it (NaN passes all);
/// and the last threshold tried.
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

/// A car's panel message as set: its string id, how long it stays and its age.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    /// The message's string id.
    pub id: u32,
    /// How long the message stays, in frames.
    pub time_left: u32,
    /// Frames since the message was set.
    pub age: u32,
}

impl Message {
    /// How long a new message stays, in frames.
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

/// A car record's limit past which its start value is 0.
pub const START_LIMIT: u32 = 0x1B58;

/// The car record a kind's start value comes from: the kind held to 0x21.
pub fn start_record(kind: u32) -> u32 {
    kind.min(TABLE_CARS - 1)
}

/// A record's start value: its value unless its limit is past `START_LIMIT`, then 0.
pub fn start_state_value(limit: u32, value: impl FnOnce() -> u32) -> u32 {
    if limit > START_LIMIT { 0 } else { value() }
}

/// The split choice for one player (0) or two (held to 1 ..= 2 by two signed tests).
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

/// A block's start state settled: 2 or 3 become 4, None (left as is) otherwise.
pub fn settled_state(state: u32) -> Option<u32> {
    matches!(state, 2 | 3).then_some(4)
}

/// The longest result kept in mode 0 (300000).
pub const MOST_MARGIN: i32 = 300_000;

/// The last stage mode 0's results hold to.
pub const LAST_STAGE: i32 = 0x4C;

/// The stage held to 0 ..= 0x4C (mode 0).
pub fn held_stage(stage: i32) -> u32 {
    let mut s = stage.max(0);
    if s.wrapping_sub(LAST_STAGE) >= 0 {
        s = LAST_STAGE;
    }
    s as u32
}

/// A mode 0 car's margin: the stage's total x 100 less its time, held to 0 ..= 300000.
pub fn margin(total: u32, time: u32) -> u32 {
    let mut v = (total.wrapping_mul(100).wrapping_sub(time) as i32).max(0);
    if v.wrapping_sub(MOST_MARGIN) >= 0 {
        v = MOST_MARGIN;
    }
    v as u32
}

/// The first stage with an automatic set-up.
pub const FIRST_SET_UP_STAGE: u32 = 0x15;
/// The stages with an automatic set-up end before this.
pub const SET_UP_STAGES_END: u32 = 0x50;

/// The set-up picked for a stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SetUpPick {
    /// Its number in `auto_setups` (16 bytes each): (stage - 0x15) / 10 x 6 + the rest.
    pub index: u32,
    /// (stage - 0x15) % 10.
    pub rest: u32,
}

/// The set-up for `stage` (0x15 ..= 0x4F; None for the others). Refused for stage 0x14.
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

/// A kind drawn from a random number (x 0x22, the high half).
pub fn drawn_kind(random: u32) -> u32 {
    ((random as u64 * TABLE_CARS as u64) >> 32) as u32
}

/// How far apart in the kinds' order an opponent may be from the first car (below).
pub const OPPONENT_REACH: u32 = 5;

/// The places' distance and whether an opponent at `place` is close enough (below 5).
pub fn opponent_distance(place: u32, first: u32) -> (u32, bool) {
    let d = place.wrapping_sub(first);
    let d = if (d as i32) < 0 { d.wrapping_neg() } else { d };
    (d, d < OPPONENT_REACH)
}

/// The car blocks reset for a race.
pub const CAR_BLOCKS: u32 = 8;

/// The car block's race-state reset: (offset, value) pairs, in order.
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
/// The race's first four text dwords' reset: four spaces.
pub const BLANK_TEXT: u32 = 0x2020_2020;
/// The second four text dwords' reset: none (-1).
pub const UNSET_TEXT: u32 = u32::MAX;

/// A car block's collision points: x, y and z lists of 0x40 floats' bits.
pub type PointLists = [[u32; 0x40]; 3];

/// A point's place (16.16) as a float: one rounding.
pub fn point_float(whole: u32, scale: f32) -> f32 {
    (whole as i32 as f64 / scale as f64) as f32
}

/// Points 4 ..= 0x3D moved two on, the new 4 at 0, the new 5 at 0 with the new 6's y.
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
    /// The stage's car numbers read.
    Numbers,
    /// The cars' times set.
    Times,
    /// The mode's set-up: 2 the arcade's, 0, 1 and 4 their own, 3 the race cars'.
    SetUp(u32),
    /// The arcade's kept times cleared.
    ClearArcadeTable,
}

/// The steps for a mode; none for others.
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

/// A player's brake setting for the block: below 0 is 0, 100 or more 0x100, else x 2.56.
pub fn held_brake(v: i32, scaled: impl FnOnce(i32) -> u32) -> u32 {
    if v < 0 {
        0
    } else if v.wrapping_sub(100) >= 0 {
        0x100
    } else {
        scaled(v)
    }
}

/// With every car set up automatically the race's cars must be 1 ..= 8.
pub fn every_car_set_up(all: bool, cars: u32) -> Result<bool, Invalid> {
    if all && (cars == 0 || cars > CAR_BLOCKS) {
        return Err(Invalid("no cars or more than eight"));
    }
    Ok(all)
}

/// The arcade's start place: the cars' count with `arcade_kind` 2 or at the rally's first stage.
pub fn arcade_start_is_last(kind: u32, stage_in_rally: u32) -> bool {
    kind == 2 || stage_in_rally == 0
}

/// The next number an arcade car takes, skipping the players' numbers.
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

/// An arcade car's type drawn from noise: (the product's low half, its high half 0 ..= 2).
pub fn arcade_type_draw(noise: u32, car: u32) -> (u32, u32) {
    let p = noise.rotate_right(car & 0x1F) as u64 * 3;
    (p as u32, (p >> 32) as u32)
}

/// The place an arcade car's number is swapped with: (low half, high half) of noise x count.
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
        let f = |carry, zero| KindCheckFlags { carry, zero };
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
        assert_eq!(point_float(0x18000, 65536.0).to_bits(), 1.5f32.to_bits());
        assert_eq!(
            point_float(0xFFFF_0000, 65536.0).to_bits(),
            (-1.0f32).to_bits()
        );
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
