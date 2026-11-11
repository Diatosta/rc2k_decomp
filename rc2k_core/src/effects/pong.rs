//! The pong game over the race (from `effects::pong_score` 0x4B8382 and
//! `effects::pong_digit` 0x4B8418): the score, the hits and the best each as three digits,
//! each digit a 4 x 5 pattern of cells (`unk_768704`, 0x14 bytes a digit). The court
//! (from `effects::reset_pong` 0x4B7E64, `effects::pong_bat_back`
//! 0x4B8215, `effects::move_pong_bat` 0x4B82B0, `effects::pong_ball_hits` 0x4B8040,
//! `effects::move_pong_ball` 0x4B7F4F and `effects::pong_cell` 0x4B8557): the ball and the bat
//! as floats, their compares as the x87's status bits (C0, C1, C2: `& 0x700`). As in
//! `math::geometry`, what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores it. The compares are of sums of two or three floats: exact in `f64`
//! while their exponents are within 29 of each other (the game's places and sizes are),
//! so they decide as the x87's at either precision.

use crate::Invalid;
use crate::physics::x87::loaded;
use std::cmp::Ordering;

/// The digits' patterns: ten, 0x14 bytes each, four cells a row, five rows.
pub const DIGITS: u32 = 10;
pub const PATTERN_LEN: u32 = 0x14;
pub const COLUMNS: u32 = 4;
pub const ROWS: u32 = 5;
/// How far a digit moves the score's place on.
pub const DIGIT_STEP: u32 = 0x40;
/// Where the score starts and the gap between the hits
/// and the best.
pub const SCORE_AT: [u32; 2] = [0x100, 0x384];
pub const SCORE_GAP: u32 = 0x20;

/// A digit's pattern's offset; refuses a digit past 9 (the patterns' end).
pub fn pattern(digit: u32) -> Result<u32, Invalid> {
    if digit >= DIGITS {
        return Err(Invalid("a pong digit past the patterns"));
    }
    Ok(digit * PATTERN_LEN)
}

/// A digit's cells in the order drawn: (column, row), a row at a time.
pub fn cells() -> impl Iterator<Item = (u32, u32)> {
    (0..ROWS).flat_map(|row| (0..COLUMNS).map(move |col| (col, row)))
}

/// The best after a frame: raised to the hits when they reach it (signed).
pub fn best(hits: u32, best: u32) -> u32 {
    if hits as i32 >= best as i32 {
        hits
    } else {
        best
    }
}

/// A number of the score: its low byte (kept back in its place) as hundreds, tens and
/// ones.
pub fn score_digits(n: u32) -> (u32, [u32; 3]) {
    let n = n & 0xFF;
    (n, [n / 100, n % 100 / 10, n % 10])
}

/// The status bits `fcom` leaves (`& 0x700`) for `a` against `b`: C0 (0x100) below, C0
/// and C2 (0x500) unordered, none at or above.
pub fn status(a: f64, b: f64) -> u32 {
    match a.partial_cmp(&b) {
        Some(Ordering::Less) => 0x100,
        None => 0x500,
        _ => 0,
    }
}

/// The court's variables: the ball (x, y), its way, its
/// size; the bat's place before its move (as bits: moved through registers); the bat and
/// its size; the work places of a hit (the ball's centre `flt_7686DC`, the bat's
/// `flt_7686E8`, their difference `flt_7686F4`); the ball's steps a frame (`g_pongSteps`)
/// and the hits.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Court {
    pub ball: [f32; 2],
    pub way: [f32; 2],
    pub ball_size: [f32; 2],
    pub saved_bat: [u32; 2],
    pub bat: [f32; 2],
    pub bat_size: [f32; 2],
    pub ball_centre: [f32; 2],
    pub bat_centre: [f32; 2],
    pub difference: [f32; 2],
    pub steps: u32,
    pub hits: u32,
}

/// The court's constants: the centres' factors (ball x, y, bat x, y: `flt_7691C6` ..), the
/// nudge of an across of 0 bits, the new way's scales (`flt_7691DA`,
/// `flt_7691DE`), its length, the walls and
/// the start (`flt_769182` .., four (x, y) pairs as bits: the ball, its size, the bat, its
/// size).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rules {
    pub centre: [f32; 4],
    pub nudge: f32,
    pub way_scale: [f32; 2],
    pub length: f32,
    pub walls: [f32; 8],
    pub start: [u32; 8],
}

/// The ball's steps a frame after a reset.
pub const START_STEPS: u32 = 5;

/// `fld a; fsub b; fstp`.
fn less(a: f32, b: f32) -> f32 {
    (a as f64 - b as f64) as f32
}

impl Court {
    /// `effects::reset_pong`: the four pairs from the start (`fld; fstp`: a signalling NaN made
    /// quiet), the steps 5, the hits 0.
    pub fn reset(&mut self, rules: &Rules) {
        let s = rules
            .start
            .map(|b| f32::from_bits(crate::scene::places::loaded(b)));
        self.ball = [s[0], s[1]];
        self.ball_size = [s[2], s[3]];
        self.bat = [s[4], s[5]];
        self.bat_size = [s[6], s[7]];
        self.steps = START_STEPS;
        self.hits = 0;
    }

    /// The four compares of the ball against the bat (`fld a; fadd b; fcomp c`): whether
    /// they overlap (no compare gave C0 alone) and the last status.
    pub fn overlap(&self) -> (bool, u32) {
        let tests = [
            (self.ball[0], self.ball_size[0], self.bat[0]),
            (self.bat[0], self.bat_size[0], self.ball[0]),
            (self.ball[1], self.ball_size[1], self.bat[1]),
            (self.bat[1], self.bat_size[1], self.ball[1]),
        ];
        let mut s = 0;
        for (a, b, c) in tests {
            s = status(a as f64 + b as f64, c as f64);
            if s == 0x100 {
                return (false, s);
            }
        }
        (true, s)
    }

    /// `effects::pong_bat_back`: the bat put back to its place before the move when it
    /// overlaps the ball. Returns the last status, or with the bat put back its place's
    /// bits.
    pub fn bat_back(&mut self) -> Back {
        let (over, status) = self.overlap();
        if !over {
            return Back::Kept(status);
        }
        self.bat = self.saved_bat.map(f32::from_bits);
        Back::PutBack(self.saved_bat)
    }

    /// `effects::move_pong_bat`: the bat's place kept, then `bat_back` (the keys' moves
    /// between are jumped over in the original: the bat never moves). Returns the bat's
    /// place going in (as bits) and what `bat_back` returns.
    pub fn move_bat(&mut self) -> ([u32; 2], Back) {
        let place = self.bat.map(f32::to_bits);
        self.saved_bat = place;
        (place, self.bat_back())
    }

    /// `effects::pong_ball_hits`: when the ball overlaps the bat a hit, four more steps a
    /// frame, the ball a step back and its new way, from the bat's centre to the ball's
    /// made `length` long (an across of exactly 0 bits less the nudge), each part scaled.
    /// Returns the last status, or after a hit the new way.
    pub fn ball_hits(&mut self, rules: &Rules) -> Hit {
        let (over, status) = self.overlap();
        if !over {
            return Hit::Missed(status);
        }
        self.hits = self.hits.wrapping_add(1);
        self.steps = self.steps.wrapping_add(4);
        for n in 0..2 {
            self.ball[n] = less(self.ball[n], self.way[n]);
        }
        let centre = |size: f32, k: f32, at: f32| (size as f64 * k as f64 + at as f64) as f32;
        for n in 0..2 {
            self.ball_centre[n] = centre(self.ball_size[n], rules.centre[n], self.ball[n]);
            self.bat_centre[n] = centre(self.bat_size[n], rules.centre[2 + n], self.bat[n]);
        }
        for n in 0..2 {
            self.difference[n] = less(self.ball_centre[n], self.bat_centre[n]);
        }
        if self.difference[0].to_bits() == 0 {
            self.difference[0] = less(self.difference[0], rules.nudge);
        }
        let [a, b] = self.difference.map(|v| v as f64);
        let k = rules.length as f64 / (a * a + b * b).sqrt();
        self.difference = [(a * k) as f32, (b * k) as f32];
        for n in 0..2 {
            self.difference[n] = (self.difference[n] as f64 * rules.way_scale[n] as f64) as f32;
        }
        self.way = self.difference;
        Hit::Hit(self.way.map(f32::to_bits))
    }

    /// `effects::move_pong_ball`: one step: the ball plus its way, `ball_hits`; below the
    /// left wall (or unordered) put at its place and the across way turned; not below the
    /// right wall less its width and the margin a reset (`reset`); below the top put at
    /// its place, below the bottom less its height put there less its height, the along
    /// way turned each time. Returns `ball_hits`' result and the last status.
    pub fn move_ball(&mut self, rules: &Rules) -> (Hit, u32) {
        for n in 0..2 {
            self.ball[n] = (self.ball[n] as f64 + self.way[n] as f64) as f32;
        }
        let hit = self.ball_hits(rules);
        let w = rules.walls.map(|v| v as f64);
        let turned = |v: f32| f32::from_bits(v.to_bits() ^ 0x8000_0000);
        let mut s = status(self.ball[0] as f64, w[0]);
        if s != 0 {
            self.ball[0] = loaded(rules.walls[1]);
            self.way[0] = turned(self.way[0]);
        }
        s = status(w[2] - self.ball_size[0] as f64 - w[3], self.ball[0] as f64);
        if s != 0 {
            self.reset(rules);
        }
        s = status(self.ball[1] as f64, w[4]);
        if s != 0 {
            self.ball[1] = loaded(rules.walls[5]);
            self.way[1] = turned(self.way[1]);
        }
        s = status(w[6] - self.ball_size[1] as f64, self.ball[1] as f64);
        if s != 0 {
            self.ball[1] = less(rules.walls[7], self.ball_size[1]);
            self.way[1] = turned(self.way[1]);
        }
        (hit, s)
    }
}

/// What `bat_back` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Back {
    /// No overlap: the last status.
    Kept(u32),
    /// The bat put back to this place.
    PutBack([u32; 2]),
}

/// What `ball_hits` did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    /// No overlap: the last status.
    Missed(u32),
    /// A hit: the new way's bits.
    Hit([u32; 2]),
}

/// `effects::pong_cell`'s box place: column times the step plus the score's place, and row
/// times its step plus its place (`fild; fmul; fiadd`, kept on the x87's stack).
pub fn cell_place(column: i32, row: i32, step: [f32; 2], at: [i32; 2]) -> [f64; 2] {
    [
        column as f64 * step[0] as f64 + at[0] as f64,
        row as f64 * step[1] as f64 + at[1] as f64,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn court() -> (Court, Rules) {
        let court = Court {
            ball: [290.0, 95.0],
            way: [3.0, 1.5],
            ball_size: [6.0, 6.0],
            saved_bat: [299.0f32.to_bits(), 98.0f32.to_bits()],
            bat: [300.0, 100.0],
            bat_size: [8.0, 60.0],
            steps: 1,
            ..Court::default()
        };
        let rules = Rules {
            centre: [0.5; 4],
            nudge: 0.25,
            way_scale: [2.0, 1.0],
            length: 5.0,
            walls: [10.0, 12.0, 620.0, 4.0, 20.0, 22.0, 460.0, 450.0],
            start: [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0].map(f32::to_bits),
        };
        (court, rules)
    }

    #[test]
    fn statuses() {
        assert_eq!(status(1.0, 2.0), 0x100);
        assert_eq!(status(2.0, 2.0), 0);
        assert_eq!(status(3.0, 2.0), 0);
        assert_eq!(status(f64::NAN, 2.0), 0x500);
    }

    #[test]
    fn bat_and_hits() {
        let (mut c, rules) = court();
        // apart: the ball's x plus its size below the bat's
        assert_eq!(c.overlap(), (false, 0x100));
        assert_eq!(c.bat_back(), Back::Kept(0x100));
        c.ball = [296.0, 110.0];
        assert_eq!(c.overlap(), (true, 0));
        let (place, back) = c.move_bat();
        assert_eq!(place, [300.0f32.to_bits(), 100.0f32.to_bits()]);
        assert_eq!(back, Back::PutBack(place));
        let Hit::Hit(way) = c.ball_hits(&rules) else {
            panic!("no hit");
        };
        assert_eq!((c.hits, c.steps), (1, 5));
        assert_eq!(c.ball, [293.0, 108.5]);
        // centres (296, 111.5) and (304, 130): (-8, -18.5) made 5 long, x doubled
        let l = (8.0f64 * 8.0 + 18.5 * 18.5).sqrt();
        assert_eq!(
            way,
            [
                ((-8.0 * (5.0 / l)) as f32 * 2.0).to_bits(),
                ((-18.5 * (5.0 / l)) as f32).to_bits()
            ]
        );
        // an across of 0 bits is nudged
        c.ball = [300.0, 110.0];
        c.bat = [300.0, 100.0];
        c.ball_size = [8.0, 6.0];
        c.way = [0.0, 0.0];
        c.ball_hits(&rules);
        let k = 5.0 / (0.0625f64 + 17.0 * 17.0).sqrt();
        assert_eq!(c.difference[0], (-0.25 * k) as f32 * 2.0);
        assert_eq!(c.difference[1], (-17.0 * k) as f32);
    }

    #[test]
    fn walls_and_reset() {
        let (mut c, rules) = court();
        c.ball = [5.0, 10.0];
        c.way = [1.0, -1.0];
        let (hit, s) = c.move_ball(&rules);
        assert_eq!(hit, Hit::Missed(0x100));
        assert_eq!(c.ball, [12.0, 22.0]);
        assert_eq!(c.way, [-1.0, 1.0]);
        assert_eq!(s, 0);
        c.ball = [700.0, 400.0];
        c.way = [0.0, 0.0];
        c.move_ball(&rules);
        assert_eq!(
            (c.ball, c.ball_size, c.bat, c.steps),
            ([1.0, 22.0], [3.0, 4.0], [5.0, 6.0], 5)
        );
        c.ball = [100.0, 455.0];
        c.ball_size = [6.0, 6.0];
        c.move_ball(&rules);
        assert_eq!(c.ball[1], 444.0);
        let mut r = rules;
        r.start[0] = 0x7F80_0001;
        c.reset(&r);
        assert_eq!(c.ball[0].to_bits(), 0x7FC0_0001);
        assert_eq!(cell_place(2, 3, [5.5, 4.0], [0x100, 0x384]), [267.0, 912.0]);
    }

    #[test]
    fn score() {
        assert_eq!(pattern(9), Ok(0xB4));
        assert!(pattern(10).is_err());
        let c: Vec<_> = cells().collect();
        assert_eq!((c.len(), c[0], c[5], c[19]), (20, (0, 0), (1, 1), (3, 4)));
        assert_eq!(best(5, 4), 5);
        assert_eq!(best(4, 4), 4);
        assert_eq!(best(3, 4), 4);
        assert_eq!(best(0x8000_0000, 1), 1);
        assert_eq!(score_digits(0x1F7), (0xF7, [2, 4, 7]));
    }
}
