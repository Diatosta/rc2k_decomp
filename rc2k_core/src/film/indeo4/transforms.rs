//! The Indeo 4 decoder's arithmetic on blocks: the inverse Haar and slant transforms, the
//! block of a lone first coefficient, and motion compensation.
//!
//! A port of FFmpeg's `libavcodec/ivi_dsp.c` (DSP functions for Indeo Video Interactive
//! codecs, Copyright (c) 2009-2011 Maxim Poliakovski, LGPL-2.1-or-later), the same sums in
//! the same order so that every value comes out the same. Sums wrap as 32-bit machine
//! arithmetic does: a damaged stream can feed any coefficient.
//!
//! Every function writes a block into `out`, a band's samples from the block's first one,
//! rows `pitch` apart; the caller has checked that the block lies inside.

use std::num::Wrapping;

type W = Wrapping<i32>;

/// The inverse transforms a band header can select.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Transform {
    Haar8x8,
    RowHaar8,
    ColumnHaar8,
    Pixels8x8,
    Slant8x8,
    RowSlant8,
    ColumnSlant8,
    Haar4x4,
    Slant4x4,
    RowHaar4,
    ColumnHaar4,
    RowSlant4,
    ColumnSlant4,
}

/// What a block with nothing coded but its predicted first coefficient becomes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LoneFirst {
    Haar,
    Slant,
    RowSlant,
    ColumnSlant,
    Pixel,
}

impl Transform {
    /// The transform a band header's number names; None for the numbers of transforms
    /// this decoder (and its reference) does not have: the cosine transforms and raw 4x4.
    pub(super) fn numbered(number: u32) -> Option<Self> {
        Some(match number {
            0 => Self::Haar8x8,
            1 => Self::RowHaar8,
            2 => Self::ColumnHaar8,
            3 => Self::Pixels8x8,
            4 => Self::Slant8x8,
            5 => Self::RowSlant8,
            6 => Self::ColumnSlant8,
            10 => Self::Haar4x4,
            11 => Self::Slant4x4,
            13 => Self::RowHaar4,
            14 => Self::ColumnHaar4,
            15 => Self::RowSlant4,
            16 => Self::ColumnSlant4,
            _ => return None,
        })
    }

    /// The side of the blocks it makes.
    pub(super) fn size(self) -> usize {
        match self {
            Self::Haar8x8
            | Self::RowHaar8
            | Self::ColumnHaar8
            | Self::Pixels8x8
            | Self::Slant8x8
            | Self::RowSlant8
            | Self::ColumnSlant8 => 8,
            _ => 4,
        }
    }

    /// Whether intra blocks' first coefficients are predicted from the block before.
    pub(super) fn predicts_first(self) -> bool {
        matches!(
            self,
            Self::Haar8x8
                | Self::Pixels8x8
                | Self::Slant8x8
                | Self::RowSlant8
                | Self::ColumnSlant8
                | Self::Haar4x4
                | Self::Slant4x4
        )
    }

    fn lone_first(self) -> LoneFirst {
        match self {
            Self::Haar8x8
            | Self::RowHaar8
            | Self::ColumnHaar8
            | Self::Haar4x4
            | Self::RowHaar4
            | Self::ColumnHaar4 => LoneFirst::Haar,
            Self::Pixels8x8 => LoneFirst::Pixel,
            Self::Slant8x8 | Self::Slant4x4 => LoneFirst::Slant,
            Self::RowSlant8 | Self::RowSlant4 => LoneFirst::RowSlant,
            Self::ColumnSlant8 | Self::ColumnSlant4 => LoneFirst::ColumnSlant,
        }
    }

    /// The block of the coefficients `input` (row after row, the transform's size wide).
    /// `columns[i]` says whether column `i` of the coefficients has any that is not 0.
    pub(super) fn inverse(
        self,
        input: &[i32; 64],
        out: &mut [i16],
        pitch: usize,
        columns: &[u8; 8],
    ) {
        match self {
            Self::Haar8x8 => haar_8x8(input, out, pitch, columns),
            Self::RowHaar8 => rows(input, out, pitch, 8, haar8),
            Self::ColumnHaar8 => columns_of(input, out, pitch, 8, columns, haar8),
            Self::Pixels8x8 => {
                for y in 0..8 {
                    for x in 0..8 {
                        out[y * pitch + x] = input[y * 8 + x] as i16;
                    }
                }
            }
            Self::Slant8x8 => slant_2d(input, out, pitch, 8, columns),
            Self::RowSlant8 => rows(input, out, pitch, 8, |v| halved(slant8(v))),
            Self::ColumnSlant8 => {
                columns_of(input, out, pitch, 8, columns, |v| halved(slant8(v)));
            }
            Self::Haar4x4 => haar_4x4(input, out, pitch, columns),
            Self::Slant4x4 => slant_2d(input, out, pitch, 4, columns),
            Self::RowHaar4 => rows(input, out, pitch, 4, haar4),
            Self::ColumnHaar4 => columns_of(input, out, pitch, 4, columns, haar4),
            Self::RowSlant4 => rows(input, out, pitch, 4, |v| halved(slant4(v))),
            Self::ColumnSlant4 => {
                columns_of(input, out, pitch, 4, columns, |v| halved(slant4(v)));
            }
        }
    }

    /// The block (`size` a side) of an intra block with no coefficient coded: only the
    /// first one, `first`, as predicted.
    pub(super) fn lone_first_block(self, first: i32, out: &mut [i16], pitch: usize, size: usize) {
        let slant = (first.wrapping_add(1) >> 1) as i16;
        match self.lone_first() {
            LoneFirst::Haar => fill(out, pitch, size, size, |_, _| (first >> 3) as i16),
            LoneFirst::Slant => fill(out, pitch, size, size, |_, _| slant),
            LoneFirst::RowSlant => {
                fill(
                    out,
                    pitch,
                    size,
                    size,
                    |_, y| if y == 0 { slant } else { 0 },
                );
            }
            LoneFirst::ColumnSlant => {
                fill(
                    out,
                    pitch,
                    size,
                    size,
                    |x, _| if x == 0 { slant } else { 0 },
                );
            }
            // always a whole 8x8 block, whatever the band's block size
            LoneFirst::Pixel => fill(out, pitch, 8, 8, |x, y| {
                if x == 0 && y == 0 { first as i16 } else { 0 }
            }),
        }
    }
}

fn fill(
    out: &mut [i16],
    pitch: usize,
    width: usize,
    height: usize,
    value: impl Fn(usize, usize) -> i16,
) {
    for y in 0..height {
        for (x, sample) in out[y * pitch..][..width].iter_mut().enumerate() {
            *sample = value(x, y);
        }
    }
}

/// `a + b` and `a - b`.
fn butterfly(a: W, b: W) -> (W, W) {
    (a + b, a - b)
}

/// The Haar butterfly: half the sum and half the difference.
fn half_butterfly(a: W, b: W) -> (W, W) {
    ((a + b) >> 1, (a - b) >> 1)
}

/// The slant transform's reflection by 1/2 and 5/4.
fn reflect(a: W, b: W) -> (W, W) {
    let two = Wrapping(2);
    (
        ((a + b * two + two) >> 2) + a,
        ((a * two - b + two) >> 2) - b,
    )
}

/// The slant transform's reflection by 1/2 and 7/8.
fn reflect_fine(a: W, b: W) -> (W, W) {
    let four = Wrapping(4);
    (
        b + ((a * four - b + four) >> 3),
        a + ((-a - b * four + four) >> 3),
    )
}

/// The inverse 8-point slant transform of coefficients 0 .. 7.
fn slant8(c: [W; 8]) -> [W; 8] {
    let (t4, t5) = reflect_fine(c[1], c[3]);
    let (t1, t5) = butterfly(c[0], t5);
    let (t2, t6) = butterfly(c[4], c[5]);
    let (t7, t3) = butterfly(c[7], c[6]);
    let (t4, t8) = butterfly(t4, c[2]);

    let (t1, t2) = butterfly(t1, t2);
    let (t4, t3) = reflect(t4, t3);
    let (t5, t6) = butterfly(t5, t6);
    let (t8, t7) = reflect(t8, t7);
    let (t1, t4) = butterfly(t1, t4);
    let (t2, t3) = butterfly(t2, t3);
    let (t5, t8) = butterfly(t5, t8);
    let (t6, t7) = butterfly(t6, t7);
    [t1, t2, t3, t4, t5, t6, t7, t8]
}

/// The inverse 4-point slant transform of coefficients 0 .. 3 (the rest ignored and 0).
fn slant4(c: [W; 8]) -> [W; 8] {
    let (t1, t2) = butterfly(c[0], c[2]);
    let (t4, t3) = reflect(c[1], c[3]);
    let (t1, t4) = butterfly(t1, t4);
    let (t2, t3) = butterfly(t2, t3);
    let zero = Wrapping(0);
    [t1, t2, t3, t4, zero, zero, zero, zero]
}

/// The inverse 8-point Haar transform of coefficients 0 .. 7.
fn haar8(c: [W; 8]) -> [W; 8] {
    let two = Wrapping(2);
    let (t1, t5) = half_butterfly(c[0] * two, c[1] * two);
    let (t1, t3) = half_butterfly(t1, c[2]);
    let (t5, t7) = half_butterfly(t5, c[3]);
    let (t1, t2) = half_butterfly(t1, c[4]);
    let (t3, t4) = half_butterfly(t3, c[5]);
    let (t5, t6) = half_butterfly(t5, c[6]);
    let (t7, t8) = half_butterfly(t7, c[7]);
    [t1, t2, t3, t4, t5, t6, t7, t8]
}

/// The inverse 4-point Haar transform of coefficients 0 .. 3 (the rest ignored and 0).
fn haar4(c: [W; 8]) -> [W; 8] {
    let (t0, t1) = half_butterfly(c[0], c[1]);
    let (d1, d2) = half_butterfly(t0, c[2]);
    let (d3, d4) = half_butterfly(t1, c[3]);
    let zero = Wrapping(0);
    [d1, d2, d3, d4, zero, zero, zero, zero]
}

/// Each value plus one, halved: the slant transform's last pass rounds so.
fn halved(v: [W; 8]) -> [W; 8] {
    v.map(|x| (x + Wrapping(1)) >> 1)
}

/// `size` values of `from`, `step` apart from `first`, the rest 0.
fn gathered(from: &[i32], first: usize, step: usize, size: usize) -> [W; 8] {
    let mut v = [Wrapping(0); 8];
    for (k, value) in v.iter_mut().take(size).enumerate() {
        *value = Wrapping(from[first + k * step]);
    }
    v
}

/// A transform of each row of coefficients on its own; a row of zeros gives zeros.
fn rows(
    input: &[i32],
    out: &mut [i16],
    pitch: usize,
    size: usize,
    pass: impl Fn([W; 8]) -> [W; 8],
) {
    for y in 0..size {
        let row = gathered(input, y * size, 1, size);
        let made = if row.iter().all(|v| v.0 == 0) {
            [Wrapping(0); 8]
        } else {
            pass(row)
        };
        for (sample, value) in out[y * pitch..][..size].iter_mut().zip(made) {
            *sample = value.0 as i16;
        }
    }
}

/// A transform of each column of coefficients on its own; a column marked empty gives
/// zeros.
fn columns_of(
    input: &[i32],
    out: &mut [i16],
    pitch: usize,
    size: usize,
    columns: &[u8; 8],
    pass: impl Fn([W; 8]) -> [W; 8],
) {
    for x in 0..size {
        let made = if columns[x] != 0 {
            pass(gathered(input, x, size, size))
        } else {
            [Wrapping(0); 8]
        };
        for (y, value) in made.iter().take(size).enumerate() {
            out[y * pitch + x] = value.0 as i16;
        }
    }
}

/// The two-pass slant transform: columns first, then rows with the rounding.
fn slant_2d(input: &[i32; 64], out: &mut [i16], pitch: usize, size: usize, columns: &[u8; 8]) {
    let pass = |v| if size == 8 { slant8(v) } else { slant4(v) };
    let mut between = [0i32; 64];
    for x in 0..size {
        if columns[x] != 0 {
            let made = pass(gathered(input, x, size, size));
            for (y, value) in made.iter().take(size).enumerate() {
                between[y * size + x] = value.0;
            }
        }
    }
    rows(&between, out, pitch, size, |v| halved(pass(v)));
}

/// The two-pass 8x8 Haar transform; the low four coefficients of a column are doubled
/// in its left half's columns before the column pass.
fn haar_8x8(input: &[i32; 64], out: &mut [i16], pitch: usize, columns: &[u8; 8]) {
    let mut between = [0i32; 64];
    for x in 0..8 {
        if columns[x] != 0 {
            let mut column = gathered(input, x, 8, 8);
            if x & 4 == 0 {
                for value in &mut column[..4] {
                    *value *= Wrapping(2);
                }
            }
            for (y, value) in haar8(column).iter().enumerate() {
                between[y * 8 + x] = value.0;
            }
        }
    }
    rows(&between, out, pitch, 8, haar8);
}

/// The two-pass 4x4 Haar transform, the low two coefficients doubled in the left half's
/// columns.
fn haar_4x4(input: &[i32; 64], out: &mut [i16], pitch: usize, columns: &[u8; 8]) {
    let mut between = [0i32; 64];
    for x in 0..4 {
        if columns[x] != 0 {
            let mut column = gathered(input, x, 4, 4);
            if x & 2 == 0 {
                column[0] *= Wrapping(2);
                column[1] *= Wrapping(2);
            }
            for (y, value) in haar4(column).iter().take(4).enumerate() {
                between[y * 4 + x] = value.0;
            }
        }
    }
    rows(&between, out, pitch, 4, haar4);
}

/// How a block is taken from the reference picture: as it is, or between two samples
/// across, down, or both.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Interpolation {
    None,
    Across,
    Down,
    Both,
}

impl Interpolation {
    /// From the half-sample bits of a motion vector.
    pub(super) fn of(half_x: bool, half_y: bool) -> Self {
        match (half_x, half_y) {
            (false, false) => Self::None,
            (true, false) => Self::Across,
            (false, true) => Self::Down,
            (true, true) => Self::Both,
        }
    }

    /// Samples read past the block's last one: a row for the vertical kinds, one more for
    /// the horizontal ones.
    pub(super) fn reach(self, pitch: usize) -> usize {
        match self {
            Self::None => 0,
            Self::Across => 1,
            Self::Down => pitch,
            Self::Both => pitch + 1,
        }
    }
}

/// Motion compensation of a block `size` a side: the reference's block at `reference`'s
/// start put into `out`, or added to what the transform left there (`add`). Both have
/// rows `pitch` apart.
pub(super) fn compensate(
    out: &mut [i16],
    reference: &[i16],
    pitch: usize,
    size: usize,
    kind: Interpolation,
    add: bool,
) {
    for y in 0..size {
        let line = &reference[y * pitch..];
        let target = &mut out[y * pitch..][..size];
        for (x, sample) in target.iter_mut().enumerate() {
            let at = |dx: usize, dy: usize| i32::from(line[dy * pitch + x + dx]);
            let value = match kind {
                Interpolation::None => at(0, 0),
                Interpolation::Across => (at(0, 0) + at(1, 0)) >> 1,
                Interpolation::Down => (at(0, 0) + at(0, 1)) >> 1,
                Interpolation::Both => (at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1)) >> 2,
            } as i16;
            *sample = if add {
                sample.wrapping_add(value)
            } else {
                value
            };
        }
    }
}
