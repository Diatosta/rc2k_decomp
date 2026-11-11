//! A player's inspection figures from its car's table (from
//! `menu::driver_stats` 0x4A73D8): each part's value by its wear, the parts' limits, the
//! weights of the inspected parts and the penalties of those at or past their limits.
//!
//! The three float steps (`fild`, `fdiv` / `fidiv`, `fimul` / `fmul`, `fistp`) are kept in
//! `f64` and rounded once to a whole number: at the x87's double precision (the game's
//! 0x027F) each step rounds as an `f64` step does, so the results are the same; at
//! extended precision a result at a rounding edge may differ by one.

use crate::math::geometry::fistp64;
use crate::menu::inspection::INSPECTED;

/// The parts of a car.
pub const PARTS: usize = 14;

/// The limit a part gets when its fitting is not all there or a switch says so.
pub const SHORT_LIMIT: u32 = 0xA;

/// The bits of the player's +0x278 parts 1 and 13 need for the table's limit.
pub const PART_1_FITTED: u32 = 0x1E0;
pub const PART_13_FITTED: u32 = 0x1E;

/// What is read from the car's table (`CarTable.cars[0]`[car]'s +0x74).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table {
    /// Each part's (low, high): at +8 (k / 2), +0x38 for odd k.
    pub pairs: [(u32, u32); PARTS],
    /// +0x78 + 4k.
    pub limits: [u32; PARTS],
    /// +0x30, +0x68, +0x70, +0x74.
    pub extra: [u32; 4],
}

impl Table {
    /// Where part `k`'s pair starts.
    pub fn pair_at(k: usize) -> usize {
        8 * (k / 2) + if k % 2 == 1 { 0x38 } else { 0 }
    }

    /// The table read through `word` (a dword at an offset), in the original's order.
    pub fn read(word: impl Fn(usize) -> u32) -> Table {
        Table {
            pairs: std::array::from_fn(|k| (word(Self::pair_at(k)), word(Self::pair_at(k) + 4))),
            limits: std::array::from_fn(|k| word(0x78 + 4 * k)),
            extra: [word(0x30), word(0x68), word(0x70), word(0x74)],
        }
    }
}

/// What is read from the player's record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Car {
    /// +0xD4 + 4k.
    pub wear: [u32; PARTS],
    /// +0x278.
    pub fitted: u32,
    /// +0x114, +0x118.
    pub setup: [u32; 2],
}

/// The game's values the figures use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constants {
    /// A part's span is cut in this many steps of wear.
    pub steps: f32,
    /// The weights' and penalties' scale.
    pub scale: f32,
    /// `MenuState.driver_stats.switch_a`: 1 keeps part 1's limit.
    pub keep_part_1: u32,
    /// `MenuState.driver_stats.switch_b`: 1 shortens part 5's limit.
    pub short_part_5: u32,
}

/// An inspected part's log line: its wear and the limit it is held to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Logged {
    pub wear: u32,
    pub limit: u32,
}

/// The figures, as the original leaves them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Figures {
    /// The fourteen values, then (over 12 and 13 and past them) the
    /// table's +0x30, +0x68, +0x70, +0x74.
    pub values: [u32; 16],
    /// Each part's high end.
    pub highs: [u32; PARTS],
    /// Each part's limit.
    pub limits: [u32; PARTS],
    /// The inspected parts' weights.
    pub weights: [Option<u32>; PARTS],
    /// The penalties of the inspected parts at or past their limits.
    pub penalties: [Option<u32>; PARTS],
    /// The inspected parts' high ends added.
    pub total: u32,
    /// `MenuState.driver_stats.setups[2].chosen`, `ACAC`, `ACB4`, `ACB8`: the player's +0x114 twice, +0x118 twice.
    pub kept: [u32; 4],
    /// `DrawWork.figure` as the last step leaves it.
    pub figure: u32,
    /// `hhmm`: the last span used, if any.
    pub span: Option<u32>,
    /// The log lines, by inspected part.
    pub logged: [Logged; 13],
}

/// `fild span; fdiv steps; fimul n; fistp`.
pub fn wear_share(span: u32, steps: f32, n: u32) -> u32 {
    fistp64(span as i32 as f64 / steps as f64 * n as i32 as f64) as u32
}

/// `fild high; fidiv total; fmul scale; fistp`: a part's weight.
pub fn weight(high: u32, total: u32, scale: f32) -> u32 {
    fistp64(high as i32 as f64 / total as i32 as f64 * scale as f64) as u32
}

/// `fild wear; fdiv scale; fimul weight; fistp`: a part's penalty.
pub fn penalty(wear: u32, scale: f32, weight: u32) -> u32 {
    fistp64(wear as i32 as f64 / scale as f64 * weight as i32 as f64) as u32
}

/// `menu::driver_stats`' figures for a car.
pub fn figures(t: &Table, car: &Car, c: &Constants) -> Figures {
    let mut f = Figures {
        values: [0; 16],
        highs: [0; PARTS],
        limits: [0; PARTS],
        weights: [None; PARTS],
        penalties: [None; PARTS],
        total: 0,
        kept: [0; 4],
        figure: 0,
        span: None,
        logged: [Logged { wear: 0, limit: 0 }; 13],
    };
    for k in 0..PARTS {
        let (low, high) = t.pairs[k];
        let wear = car.wear[k];
        f.figure = wear;
        f.values[k] = if wear == 0 {
            0
        } else {
            f.figure = wear.wrapping_sub(1);
            let span = high.wrapping_sub(low);
            if span == 0 {
                low
            } else {
                f.span = Some(span);
                f.figure = wear_share(span, c.steps, wear.wrapping_sub(1));
                low.wrapping_add(f.figure)
            }
        };
        f.highs[k] = high;
        let wanted = match k {
            1 => Some(PART_1_FITTED),
            13 => Some(PART_13_FITTED),
            _ => None,
        };
        f.limits[k] = match wanted {
            Some(bits) if car.fitted & bits != bits => SHORT_LIMIT,
            _ => t.limits[k],
        };
    }
    f.total = INSPECTED
        .iter()
        .fold(0u32, |s, &k| s.wrapping_add(f.highs[k]));
    f.values[14] = t.extra[2];
    f.values[15] = t.extra[3];
    f.values[12] = t.extra[0];
    f.values[13] = t.extra[1];
    f.kept = [car.setup[0], car.setup[0], car.setup[1], car.setup[1]];
    for (slot, &k) in INSPECTED.iter().enumerate() {
        let w = weight(f.highs[k], f.total, c.scale);
        f.figure = w;
        f.weights[k] = Some(w);
        let limit = if (k == 1 && c.keep_part_1 != 1) || (k == 5 && c.short_part_5 == 1) {
            SHORT_LIMIT
        } else {
            f.limits[k]
        };
        let wear = car.wear[k];
        f.logged[slot] = Logged { wear, limit };
        if wear as i32 >= limit as i32 {
            let p = penalty(wear, c.scale, w);
            f.figure = p;
            f.penalties[k] = Some(p);
        }
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Table {
        Table {
            pairs: std::array::from_fn(|k| {
                (10 * k as u32, 10 * k as u32 + if k == 2 { 0 } else { 40 })
            }),
            limits: std::array::from_fn(|k| 20 + k as u32),
            extra: [0x30, 0x68, 0x70, 0x74],
        }
    }

    #[test]
    fn steps() {
        assert_eq!(wear_share(40, 8.0, 3), 15);
        assert_eq!(weight(1, 4, 100.0), 25);
        assert_eq!(weight(1, 0, 100.0), 0x8000_0000);
        assert_eq!(penalty(30, 10.0, 7), 21);
        assert_eq!(Table::pair_at(0), 0);
        assert_eq!(Table::pair_at(1), 0x38);
        assert_eq!(Table::pair_at(5), 0x48);
    }

    #[test]
    fn figures_of_a_car() {
        let mut wear = [0u32; PARTS];
        wear[0] = 5;
        wear[2] = 3;
        wear[5] = 30;
        let car = Car {
            wear,
            fitted: 0x1E0,
            setup: [7, 9],
        };
        let c = Constants {
            steps: 8.0,
            scale: 10.0,
            keep_part_1: 1,
            short_part_5: 1,
        };
        let f = figures(&table(), &car, &c);
        // Part 0: span 40, (40 / 8) * 4 = 20 over its low end 0.
        assert_eq!(f.values[0], 20);
        // Part 2: a flat pair: its low end.
        assert_eq!(f.values[2], 20);
        assert_eq!(f.values[1], 0);
        assert_eq!(f.span, Some(40));
        assert_eq!(f.limits[1], 21);
        assert_eq!(f.limits[13], SHORT_LIMIT);
        assert_eq!(f.values[12..], [0x30, 0x68, 0x70, 0x74]);
        assert_eq!(f.kept, [7, 7, 9, 9]);
        assert_eq!(f.weights[4], None);
        assert!(f.weights[0].is_some());
        // Part 5 held to 0xA by the switch: 30 is past it.
        assert_eq!(
            f.logged[4],
            Logged {
                wear: 30,
                limit: SHORT_LIMIT
            }
        );
        assert!(f.penalties[5].is_some() && f.penalties[0].is_none());
        let total: u32 = INSPECTED.iter().map(|&k| f.highs[k]).sum();
        assert_eq!(f.total, total);
        assert_eq!(f.figure, f.weights[13].unwrap());
    }
}
