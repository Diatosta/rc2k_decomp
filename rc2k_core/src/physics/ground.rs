//! Small rules of the car on the ground (converted 2026-10-21 from
//! `physics::note_surface_mode` 0x43F180, `start_skid_mark` 0x43F272,
//! `flip_cell_index` 0x43DB13 and `plane_height` 0x4425B6; the whole-number steps of
//! `surface_noise` 0x43BC68, whose share / 256 stays x87). The probe points' whole-number
//! steps (converted 2026-10-21 from `physics::probe_ground` 0x45D305; the turns and the
//! bump's scaling stay x87): the wheels' lift by the car's bits, and the ground's bump
//! from the noise bytes (64 x 64) mixed by the place's low 14 bits.

/// `physics::note_surface_mode`: `dword_71A6C8` by the mode `dword_71A65C`: 1 for 2 or 3,
/// else 0. What the mode is is not known.
pub fn surface_flag(mode: u32) -> u32 {
    u32::from(mode == 2 || mode == 3)
}

/// A skid mark's start (`dword_71A660` .. `dword_71A674`; a skid mark's start: its place, a count of 10 and the step it began).
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

/// `surface_noise`'s level from a noise sample: lo + (hi - lo) x sample >> 16 (`imul`,
/// `shrd`: the low 32 bits of the 64-bit product >> 16), held to 0 ..= 0xFF (by the
/// sign). (the level, the range hi - lo: ebx, the product's high dword: edx.)
pub fn noise_level(sample: u32, lo: u32, hi: u32) -> (u32, u32, u32) {
    let range = hi.wrapping_sub(lo);
    let product = (sample as i32 as i64).wrapping_mul(range as i32 as i64);
    let mut v = ((product as u64) >> 16) as u32;
    v = v.wrapping_add(lo);
    if (v as i32) < 0 {
        v = 0;
    }
    if (v.wrapping_sub(0xFF) as i32) >= 0 {
        v = 0xFF;
    }
    (v, range, (product >> 32) as u32)
}

/// The roughness kept: the second level, at least `floor` (unsigned).
pub fn rough_floor(level: u32, floor: u32) -> u32 {
    level.max(floor)
}

/// `physics::probe_ground`: whether wheel `b` (0 .. 3) is lifted by `flt_613E8C`: bit
/// b + 5 of the car's bits (`dword_71A270`) clear, or else bit b + 1 clear. (lifted, the
/// last bit tested: the original's ecx.)
pub fn wheel_lifted(bits: u32, b: u32) -> (bool, u32) {
    if bits >> (b + 5) & 1 == 0 {
        (true, b + 5)
    } else {
        (bits >> (b + 1) & 1 == 0, b + 1)
    }
}

/// The noise bytes' cell of a place (whole-number x, z): the row (`dword_6D3060`), the
/// row's offset (`dword_6D3064`, the row << 6), the four bytes' indices in the original's
/// order (`dword_6D3044`, `48`, `50`, `4C`: the corner, the one right, below right, below)
/// and the last row offset (the original's esi).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoiseCell {
    pub row: u32,
    pub row_offset: u32,
    pub corners: [u32; 4],
    pub below: u32,
}

pub fn noise_cell(x: u32, z: u32) -> NoiseCell {
    let row = (z >> 14) & 0x3F;
    let row_offset = row << 6;
    let mut ecx = ((x >> 14) & 0x3F).wrapping_add(row_offset) & 0x3F;
    let top = row_offset & 0xFC0;
    let first = top + ecx;
    ecx = (ecx + 1) & 0x3F;
    let right = top + ecx;
    let below = (top + 0x40) & 0xFC0;
    let below_right = below + ecx;
    ecx = ecx.wrapping_sub(1) & 0x3F;
    NoiseCell {
        row,
        row_offset,
        corners: [first, right, below_right, below + ecx],
        below,
    }
}

/// The noise's mix: a + the low 32 bits of ((b - a) f) >> 14 (`imul`, `shrd`), and the
/// product's high dword (edx).
pub fn noise_lerp(a: u32, b: u32, f: u32) -> (u32, u32) {
    let p = i64::from(b.wrapping_sub(a) as i32) * i64::from(f as i32);
    (((p >> 14) as u32).wrapping_add(a), (p >> 32) as u32)
}

/// The bump's noise from the four bytes (in `NoiseCell::corners`' order) by the place's
/// low 14 bits: (the bottom mix `6D3058`, the top `6D3054`, the whole `6D305C`, its
/// product's high dword).
pub fn bump_noise(n: [u32; 4], x: u32, z: u32) -> (u32, u32, u32, u32) {
    let (fx, fz) = (x & 0x3FFF, z & 0x3FFF);
    let (bottom, _) = noise_lerp(n[3], n[2], fx);
    let (top, _) = noise_lerp(n[0], n[1], fx);
    let (whole, high) = noise_lerp(top, bottom, fz);
    (bottom, top, whole, high)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probes() {
        assert_eq!(wheel_lifted(0, 1), (true, 6));
        assert_eq!(wheel_lifted(1 << 6, 1), (true, 2));
        assert_eq!(wheel_lifted(1 << 6 | 1 << 2, 1), (false, 2));
        let c = noise_cell(0x3F << 14, 0x3F << 14);
        assert_eq!(c.row, 0x3F);
        assert_eq!(c.corners, [0xFFF, 0xFC0, 0x000, 0x03F]);
        assert_eq!(noise_lerp(0, 0x100, 0x2000), (0x80, 0));
        assert_eq!(bump_noise([0, 0x100, 0x100, 0], 0x2000, 0).2, 0x80);
    }

    #[test]
    fn noise() {
        assert_eq!(noise_level(0x8000, 0x10, 0x30), (0x20, 0x20, 0));
        assert_eq!(noise_level(0x10000, 0, 0x200), (0xFF, 0x200, 0));
        assert_eq!(noise_level(0xFFFF_0000, 0, 0x10), (0, 0x10, u32::MAX));
        assert_eq!(rough_floor(3, 5), 5);
        assert_eq!(rough_floor(7, 5), 7);
    }

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
