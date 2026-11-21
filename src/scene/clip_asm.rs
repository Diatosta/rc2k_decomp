//! The x87 arithmetic of the clippers, in plain Rust over `X` (`math/x87_float.rs`): where the
//! segment between two points crosses a screen x or y edge or a depth edge, and the new point's
//! record fields and entry values there. The operations keep their order. The tests compare
//! against recorded vectors (`data/vectors`).

use crate::math::x87_float::{X, int, ld, stf, to_int};

/// What a clipper interpolates of one polygon vertex: its point record's fields and its
/// entry's texture floats and five bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CutPoint {
    /// The place on the screen.
    pub screen_x: f32,
    pub screen_y: f32,
    /// The place in the view; the third is the depth.
    pub view: [f32; 3],
    /// The record's first three whole numbers (whole numbers in the record, read as numbers).
    pub whole: [i32; 3],
    /// The entry's four texture coordinates.
    pub texture: [f32; 4],
    /// The entry's light, fog and amount bytes as whole numbers, in the entry's order.
    pub bytes: [i32; 5],
}

impl CutPoint {
    /// A point from the clip's work variables: twelve dwords (screen x and y, the place in
    /// the view, the three whole numbers, the texture coordinates) and five (the bytes, in
    /// the order the clip's cut stores them).
    pub fn from_out_words(out_1: [u32; 12], out_2: [u32; 5]) -> CutPoint {
        let float = |k: usize| f32::from_bits(out_1[k]);
        CutPoint {
            screen_x: float(0),
            screen_y: float(1),
            view: [float(2), float(3), float(4)],
            whole: [out_1[5] as i32, out_1[6] as i32, out_1[7] as i32],
            texture: [float(8), float(9), float(10), float(11)],
            bytes: BYTE_SLOTS.map(|slot| out_2[slot] as i32),
        }
    }

    /// The work variables' words, as `from_out_words` takes them.
    pub fn out_words(&self) -> ([u32; 12], [u32; 5]) {
        let mut out_1 = [0u32; 12];
        out_1[0] = self.screen_x.to_bits();
        out_1[1] = self.screen_y.to_bits();
        for k in 0..3 {
            out_1[2 + k] = self.view[k].to_bits();
            out_1[5 + k] = self.whole[k] as u32;
        }
        for k in 0..4 {
            out_1[8 + k] = self.texture[k].to_bits();
        }
        let mut out_2 = [0u32; 5];
        for (byte, slot) in self.bytes.into_iter().zip(BYTE_SLOTS) {
            out_2[slot] = byte as u32;
        }
        (out_1, out_2)
    }
}

/// Where the clip's work variables hold each of the five bytes.
const BYTE_SLOTS: [usize; 5] = [2, 3, 4, 0, 1];

/// One cut: the segment's two end points, the plane's depth, and the new point. On the way
/// in `made` holds the edge's value (its screen x for `cut_at_x`, its screen y for
/// `cut_at_y`) and, besides what the cut writes, the leftovers of the cut before; on the way
/// out the fields the cut makes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cut {
    pub first: CutPoint,
    pub second: CutPoint,
    pub made: CutPoint,
    /// The depth edge's whole number (`cut_at_depth`).
    pub plane: i32,
    /// The cut's fraction along the segment: as a whole number after a screen cut, as float
    /// bits after a depth cut.
    pub scratch: u32,
}

/// The screen axis a cut is made on.
#[derive(Clone, Copy)]
enum Axis {
    X,
    Y,
}

impl CutPoint {
    /// The coordinate on the cut axis, and on the other axis.
    fn on(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.screen_x,
            Axis::Y => self.screen_y,
        }
    }

    fn off(&self, axis: Axis) -> f32 {
        match axis {
            Axis::X => self.screen_y,
            Axis::Y => self.screen_x,
        }
    }
}

impl Cut {
    /// Cuts at a screen x edge.
    pub fn cut_at_x(&mut self) {
        self.cut_at_screen_edge(Axis::X);
    }

    /// Cuts at a screen y edge.
    pub fn cut_at_y(&mut self) {
        self.cut_at_screen_edge(Axis::Y);
    }

    /// Cuts the segment between the two points at the edge of one axis: the fraction along
    /// it (`scratch` takes it rounded), every interpolated record field and entry byte, and
    /// the other axis' edge value.
    fn cut_at_screen_edge(&mut self, axis: Axis) {
        let (first, second) = (self.first, self.second);
        let depth_first = ld(first.view[2]);
        let depth_second = ld(second.view[2]);
        let edge = ld(self.made.on(axis));
        let along_first = ld(first.on(axis));
        let along_second = ld(second.on(axis));
        let edge_term = edge * (depth_first - depth_second);
        let first_term = -(depth_first * along_first);
        let sum = first_term + edge_term;
        let second_term = depth_second * along_second;
        let denominator = second_term + sum;
        let numerator = depth_second * (along_second - edge);
        let fraction = numerator / denominator;
        self.interpolate_from_second_to_first(fraction);
        let span = along_second - along_first;
        let from_edge = along_first - edge;
        let other_first = ld(first.off(axis));
        let other_span = ld(second.off(axis)) - other_first;
        let other_edge = (other_span * -from_edge) / span;
        let other_edge = stf(other_edge + other_first);
        match axis {
            Axis::X => self.made.screen_y = other_edge,
            Axis::Y => self.made.screen_x = other_edge,
        }
    }

    /// The part every screen cut shares: the cut point's record fields (floats and whole
    /// numbers), the entry's floats and the five entry bytes, each
    /// `second + (first - second) * fraction`, then the fraction itself as a whole number in
    /// `scratch`.
    fn interpolate_from_second_to_first(&mut self, fraction: X) {
        let (first, second) = (self.first, self.second);
        let made = &mut self.made;
        for k in 0..3 {
            made.view[k] = stf(blend(ld(second.view[k]), ld(first.view[k]), fraction));
        }
        for k in 0..3 {
            made.whole[k] = to_int(blend(int(second.whole[k]), int(first.whole[k]), fraction));
        }
        for k in 0..4 {
            made.texture[k] = stf(blend(ld(second.texture[k]), ld(first.texture[k]), fraction));
        }
        for k in 0..5 {
            made.bytes[k] = to_int(blend(int(second.bytes[k]), int(first.bytes[k]), fraction));
        }
        self.scratch = to_int(fraction) as u32;
    }

    /// Cuts at a depth edge: the new point's view x and y, texture and bytes, its depth the
    /// plane's, and the fraction (as float bits) in `scratch`.
    pub fn cut_at_depth(&mut self) {
        let (first, second) = (self.first, self.second);
        let plane = int(self.plane);
        let fraction = (plane - ld(first.view[2])) / (ld(second.view[2]) - ld(first.view[2]));
        let made = &mut self.made;
        for k in 0..2 {
            made.view[k] = stf(blend(ld(first.view[k]), ld(second.view[k]), fraction));
        }
        made.view[2] = stf(plane);
        for k in 0..4 {
            made.texture[k] = stf(blend(ld(first.texture[k]), ld(second.texture[k]), fraction));
        }
        for k in 0..5 {
            made.bytes[k] = to_int(blend(int(first.bytes[k]), int(second.bytes[k]), fraction));
        }
        self.scratch = stf(fraction).to_bits();
    }
}

/// `from + (to - from) * fraction`.
fn blend(from: X, to: X, fraction: X) -> X {
    from + (to - from) * fraction
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Differential, Kind, Rng, differential};

    /// The recorded blocks' layout, in dwords: a point's record (0), its entry (0x40 bytes
    /// on) and its five entry bytes (0xD0 bytes on) for the first point, the same at 0x20,
    /// 0x88 and 0xE4 for the second; then the work variables at 0xF8 (twelve), 0x128 (five),
    /// the scratch at 0x13C and the plane at 0x140.
    const POINTS: [(usize, usize, usize); 2] = [(0x00, 0x48, 0xD0), (0x20, 0x90, 0xE4)];
    const OUT_1: usize = 0xF8;
    const OUT_2: usize = 0x128;
    const SCRATCH: usize = 0x13C;
    const PLANE: usize = 0x140;

    fn point_of(w: &[u32; 0x51], (record, texture, bytes): (usize, usize, usize)) -> CutPoint {
        let (record, texture, bytes) = (record / 4, texture / 4, bytes / 4);
        let float = |k: usize| f32::from_bits(w[k]);
        CutPoint {
            screen_x: float(record + 3),
            screen_y: float(record + 7),
            view: [float(record + 4), float(record + 5), float(record + 6)],
            whole: [w[record] as i32, w[record + 1] as i32, w[record + 2] as i32],
            texture: [
                float(texture),
                float(texture + 1),
                float(texture + 2),
                float(texture + 3),
            ],
            bytes: std::array::from_fn(|k| w[bytes + k] as i32),
        }
    }

    /// Writes into `w` what a cut changes: the new point and the scratch.
    fn write_made(w: &mut [u32; 0x51], cut: &Cut) {
        let (out_1, out_2) = cut.made.out_words();
        w[OUT_1 / 4..OUT_1 / 4 + 12].copy_from_slice(&out_1);
        w[OUT_2 / 4..OUT_2 / 4 + 5].copy_from_slice(&out_2);
        w[SCRATCH / 4] = cut.scratch;
    }

    fn cut_of(w: &[u32; 0x51]) -> Cut {
        let out_1: [u32; 12] = w[OUT_1 / 4..OUT_1 / 4 + 12].try_into().unwrap();
        let out_2: [u32; 5] = w[OUT_2 / 4..OUT_2 / 4 + 5].try_into().unwrap();
        Cut {
            first: point_of(w, POINTS[0]),
            second: point_of(w, POINTS[1]),
            made: CutPoint::from_out_words(out_1, out_2),
            plane: w[PLANE / 4] as i32,
            scratch: w[SCRATCH / 4],
        }
    }

    /// Runs one cut over the block as the vectors recorded it.
    fn run(cells: &[std::cell::Cell<u32>], cut: impl FnOnce(&mut Cut)) {
        let mut w: [u32; 0x51] = std::array::from_fn(|k| cells[k].get());
        let mut cut_state = cut_of(&w);
        cut(&mut cut_state);
        write_made(&mut w, &cut_state);
        for (cell, word) in cells.iter().zip(w) {
            cell.set(word);
        }
    }
    /// A block: the records' first three, the entries' five bytes and the plane are whole
    /// numbers, floats elsewhere.
    fn fill(rng: &mut Rng, m: &mut [u32; 0x51]) {
        let moderate = rng.below(2) == 0;
        for d in m.iter_mut() {
            *d = if moderate { rng.moderate() } else { rng.fl() };
        }
        for k in [0, 1, 2, 8, 9, 10, 80].into_iter().chain(52..62) {
            m[k] = if rng.below(2) == 0 {
                rng.small() as u32
            } else {
                rng.int()
            };
        }
    }

    #[test]
    fn cut_at_x_matches_the_assembly() {
        let test = Differential {
            name: "scene.clip_asm.cut_at_x",
            cases: 4000,
            seed: 30,
            kinds: [Kind::Float; 0x51],
            fill: &fill,
            plain: &|c| run(c, Cut::cut_at_x),
        };
        differential(test);
    }

    #[test]
    fn cut_at_y_matches_the_assembly() {
        let test = Differential {
            name: "scene.clip_asm.cut_at_y",
            cases: 4000,
            seed: 31,
            kinds: [Kind::Float; 0x51],
            fill: &fill,
            plain: &|c| run(c, Cut::cut_at_y),
        };
        differential(test);
    }

    #[test]
    fn cut_at_depth_matches_the_assembly() {
        let test = Differential {
            name: "scene.clip_asm.cut_at_depth",
            cases: 4000,
            seed: 32,
            kinds: [Kind::Float; 0x51],
            fill: &fill,
            plain: &|c| run(c, Cut::cut_at_depth),
        };
        differential(test);
    }
}
