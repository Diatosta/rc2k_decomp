//! The race's cars: their entries in the per-car tables, their kinds, their grades, their
//! starts and the players' states (converted 2026-10-21 from `race::car_file_number`
//! 0x41F4FD, `car_kind_10` 0x41F4A1, `car_kind_11` 0x41F485, `kind_past_table` 0x4651AC,
//! `kind_unusable` 0x449D34, `index_of_kind` 0x437720, `players_all_out` 0x445506,
//! `reset_car_starts` 0x444BBC, `nonzero_random` 0x444C14, `seed_car_randoms` 0x444BF6,
//! `drop_absent_player` 0x436402, `grade_car_values` 0x43FE24, `grade_all_cars`
//! 0x43FDFE, `set_car_counter` 0x43A96C, `set_both_counters` 0x43A987,
//! `set_start_states` 0x446D2E and `store_results_by_mode` 0x449957).

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

/// A car's grade: (the value's offset in its block, the grade's, the thresholds), in the
/// original's order (`race::grade_car_values`).
pub type Grade = (u32, u32, &'static [u32]);

/// The grades `race::grade_car_values` works out.
pub const GRADES: [Grade; 34] = [
    (0x2B4, 0x3C0, &[50, 75, 90, 100]),
    (0x2AC, 0x3C4, &[75]),
    (0x2B8, 0x368, &[50, 75, 90, 100]),
    (0x2BC, 0x36C, &[50, 75, 90, 100]),
    (0x2C0, 0x370, &[50, 75, 90, 100]),
    (0x2C4, 0x374, &[50, 75, 90, 100]),
    (0x2C8, 0x378, &[50, 75, 90, 100]),
    (0x2CC, 0x37C, &[50, 75, 90, 100]),
    (0x2F4, 0x380, &[50, 75, 90, 100]),
    (0x30C, 0x384, &[50, 75, 90, 100]),
    (0x310, 0x388, &[50, 75, 90, 100]),
    (0x330, 0x38C, &[50, 75, 90, 100]),
    (0x350, 0x390, &[50, 75, 90, 100]),
    (0x344, 0x39C, &[50, 75, 90, 100]),
    (0x32C, 0x394, &[50, 75, 90]),
    (0x314, 0x3CC, &[100]),
    (0x318, 0x3D0, &[100]),
    (0x31C, 0x3D4, &[100]),
    (0x320, 0x3D8, &[100]),
    (0x324, 0x3DC, &[100]),
    (0x328, 0x3E0, &[100]),
    (0x2F8, 0x3E4, &[100]),
    (0x2FC, 0x3E8, &[100]),
    (0x300, 0x3EC, &[100]),
    (0x304, 0x3F0, &[100]),
    (0x364, 0x398, &[50, 75, 90]),
    (0x2D0, 0x3A0, &[100]),
    (0x2D8, 0x3A4, &[100]),
    (0x2DC, 0x3A8, &[100]),
    (0x2E0, 0x3AC, &[100]),
    (0x2E4, 0x3B0, &[100]),
    (0x2E8, 0x3B4, &[100]),
    (0x2EC, 0x3B8, &[100]),
    (0x2F0, 0x3BC, &[100]),
];

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
