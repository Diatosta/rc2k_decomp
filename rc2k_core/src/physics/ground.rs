//! Small rules of the car on the ground (converted 2026-10-21 from
//! `physics::note_surface_mode` 0x43F180, `start_skid_mark` 0x43F272,
//! `flip_cell_index` 0x43DB13 and `plane_height` 0x4425B6).

/// `physics::note_surface_mode`: `dword_71A6C8` by the mode `dword_71A65C`: 1 for 2 or 3,
/// else 0. What the mode is is not known.
pub fn surface_flag(mode: u32) -> u32 {
    u32::from(mode == 2 || mode == 3)
}

/// A skid mark's start (`dword_71A660` .. `dword_71A674`; by the look of it, a reading).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SkidStart {
    /// `dword_71A660`: the pending value less one.
    pub from: u32,
    /// `dword_71A664`: pending (cleared once taken).
    pub pending: u32,
    /// `dword_71A668`: the step count when it started.
    pub step: u32,
    /// `dword_71A66C`: 1 once started.
    pub on: u32,
    /// `dword_71A670`: 10 once started.
    pub w670: u32,
    /// `dword_71A674`: 0 once started.
    pub w674: u32,
}

impl SkidStart {
    /// `physics::start_skid_mark`: when either of the two qwords `wheels`
    /// (`qword_71A5D8`, `qword_71A5E0`, as four dwords) is not 0: a pending value starts
    /// the mark at the step count `step` (asked only then), and pending is cleared. The
    /// original's eax: `None` (kept) when both are 0, else the step count, or 0 when
    /// nothing was pending.
    pub fn start(&mut self, wheels: [u32; 4], step: impl FnOnce() -> u32) -> Option<u32> {
        if wheels.iter().all(|&w| w == 0) {
            return None;
        }
        let mut eax = self.pending;
        if eax != 0 {
            self.from = eax - 1;
            self.on = 1;
            self.w674 = 0;
            self.w670 = 10;
            eax = step();
            self.step = eax;
        }
        self.pending = 0;
        Some(eax)
    }
}

/// `physics::flip_cell_index`: a 2 x 2 cell (`x`, `y`, each 0 or 1) turned one of eight
/// ways (`way` & 7: 1 flips x, 2 flips y, 3 both; 4 .. 7 swap the two first, then 4
/// flips y, 5 both, 6 none, 7 x); the cell's index 2 y + x. (way & 7, x, index) as the
/// original leaves them in eax, ebx and edx.
pub fn flip_cell_index(way: u32, x: u32, y: u32) -> (u32, u32, u32) {
    let way = way & 7;
    let (mut b, mut d) = (x, y);
    if way >= 4 {
        std::mem::swap(&mut b, &mut d);
    }
    match way {
        1 | 7 => b ^= 1,
        2 | 4 => d ^= 1,
        3 | 5 => {
            b ^= 1;
            d ^= 1;
        }
        _ => {}
    }
    (way, b, d.wrapping_add(d).wrapping_add(b))
}

/// `physics::plane_height`: the height of a course face's plane under a point, from its
/// corners' heights `a`, `b`, `c` (whole numbers) and the point's shares `f`, `g`:
/// (a - c) f + ((b - c) g + c), `fistp`ed (the sums in the original's order; what the x87
/// keeps on its stack an `f64`, the same bits at the game's double precision).
pub fn plane_height(a: i32, b: i32, c: i32, f: f32, g: f32) -> i32 {
    let (a, b, c) = (a as f64, b as f64, c as f64);
    super::x87::fistp64((a - c) * f as f64 + ((b - c) * g as f64 + c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane() {
        assert_eq!(plane_height(10, 20, 0, 0.5, 0.25), 10);
        assert_eq!(
            plane_height(i32::MAX, i32::MAX, i32::MIN, 2.0, 2.0),
            i32::MIN
        );
        assert_eq!(plane_height(3, 0, 0, 0.5, 0.0), 2);
    }

    #[test]
    fn surface() {
        assert_eq!(
            (0..5).map(surface_flag).collect::<Vec<_>>(),
            [0, 0, 1, 1, 0]
        );
    }

    #[test]
    fn skid() {
        let mut s = SkidStart {
            pending: 5,
            ..Default::default()
        };
        assert_eq!(s.start([0; 4], || unreachable!()), None);
        assert_eq!(s.start([0, 0, 0, 1], || 77), Some(77));
        assert_eq!(
            s,
            SkidStart {
                from: 4,
                pending: 0,
                step: 77,
                on: 1,
                w670: 10,
                w674: 0
            }
        );
        assert_eq!(s.start([1, 0, 0, 0], || unreachable!()), Some(0));
    }

    #[test]
    fn cells() {
        assert_eq!(flip_cell_index(0, 1, 0), (0, 1, 1));
        assert_eq!(flip_cell_index(9, 1, 0), (1, 0, 0));
        assert_eq!(flip_cell_index(4, 1, 0), (4, 0, 0));
        assert_eq!(flip_cell_index(6, 1, 0), (6, 0, 2));
        assert_eq!(flip_cell_index(7, 1, 1), (7, 0, 2));
        assert_eq!(flip_cell_index(3, 0, 0), (3, 1, 3));
    }
}
