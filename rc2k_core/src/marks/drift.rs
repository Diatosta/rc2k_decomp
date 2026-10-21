//! The rain's two drifting offsets (converted 2026-10-21 from `marks::drift_offsets`
//! 0x4C1BA9): whole-number offsets moved each frame by a pace that eases toward targets
//! drawn anew after a wait. As in `math::geometry`, what the x87 keeps on its stack is an
//! `f64`, rounded to `f32` where the original stores it; the offsets' rounding follows the
//! x87's control word and comes in as a closure.

/// The drift's variables: the clock at the last targets (`dword_8BC409`), the wait
/// (`dword_8BC40D`), the targets and paces (`flt_8BC411`, `flt_8BC419`; `flt_8BC41D`,
/// `flt_8BC425`), a second random's halves times 0x100 (`dword_8BC429`, `dword_8BC431`:
/// only written) and the offsets (`dword_8BDD7D`, `dword_8BDD85`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Drift {
    pub since: u32,
    pub wait: u32,
    pub target: [f32; 2],
    pub pace: [f32; 2],
    pub halves: [u32; 2],
    pub offset: [i32; 2],
}

/// What `drift_offsets` leaves in eax, and after new targets in ebx and ecx.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DriftRegs {
    pub eax: u32,
    pub ebx: Option<u32>,
    pub ecx: Option<u32>,
}

impl Drift {
    /// One frame at `clock`: when the wait has passed since the last targets (unsigned),
    /// new targets from the marks' random (`random()`: the next seed, the marks'
    /// `next_seed`) (its low 12 bits and the next 12, less 0x7FF,
    /// times `scale()`, asked only then), a new wait ((its top byte + 0x40) times 5), the clock kept, and a
    /// second random's halves times 0x100. Then each pace eased toward its target (`fld t;
    /// fsub p; fmul k; fadd p`) and added to its offset (`add(offset, pace)`: `fild; fadd;
    /// fistp`).
    pub fn step(
        &mut self,
        clock: u32,
        mut random: impl FnMut() -> u32,
        scale: impl FnOnce() -> [f32; 2],
        ease: [f32; 2],
        add: impl Fn(i32, f32) -> i32,
    ) -> DriftRegs {
        let mut r = DriftRegs {
            eax: clock.wrapping_sub(self.since),
            ..DriftRegs::default()
        };
        if r.eax >= self.wait {
            let s = random();
            let parts = [s & 0xFFF, (s >> 12) & 0xFFF].map(|p| p.wrapping_sub(0x7FF));
            let scale = scale();
            for n in 0..2 {
                self.target[n] = (parts[n] as i32 as f64 * scale[n] as f64) as f32;
            }
            let wait = ((s >> 24) + 0x40) * 5;
            self.wait = wait;
            self.since = clock;
            let s = random();
            self.halves = [(s & 0xFFFF) << 8, (s >> 16) << 8];
            r = DriftRegs {
                eax: self.halves[0],
                ebx: Some(self.halves[1]),
                ecx: Some(wait),
            };
        }
        for (n, k) in ease.into_iter().enumerate() {
            let (t, p) = (self.target[n] as f64, self.pace[n] as f64);
            self.pace[n] = ((t - p) * k as f64 + p) as f32;
            self.offset[n] = add(self.offset[n], self.pace[n]);
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marks::noise::next_seed;

    #[test]
    fn drifts() {
        let add = |o: i32, p: f32| (o as f64 + p as f64).round() as i32;
        let mut d = Drift {
            since: 900,
            wait: 101,
            target: [2.0, -1.0],
            pace: [0.5, -0.25],
            offset: [10, -10],
            ..Drift::default()
        };
        let mut seed = 0x1234_5678u32;
        // the wait not passed: the paces eased only
        let r = d.step(1000, || unreachable!(), || unreachable!(), [0.5, 0.5], add);
        assert_eq!(
            (r, seed),
            (
                DriftRegs {
                    eax: 100,
                    ebx: None,
                    ecx: None
                },
                0x1234_5678
            )
        );
        assert_eq!((d.pace, d.offset), ([1.25, -0.625], [11, -11]));
        // passed: new targets, wait and halves
        let random = || {
            seed = next_seed(seed);
            seed
        };
        let r = d.step(1001, random, || [0.5, 2.0], [1.0, 1.0], add);
        let s1 = next_seed(0x1234_5678);
        let s2 = next_seed(s1);
        assert_eq!(seed, s2);
        assert_eq!(d.target[0], ((s1 & 0xFFF) as i32 - 0x7FF) as f32 * 0.5);
        assert_eq!(
            d.target[1],
            (((s1 >> 12) & 0xFFF) as i32 - 0x7FF) as f32 * 2.0
        );
        assert_eq!((d.wait, d.since), (((s1 >> 24) + 0x40) * 5, 1001));
        assert_eq!(
            r,
            DriftRegs {
                eax: (s2 & 0xFFFF) << 8,
                ebx: Some((s2 >> 16) << 8),
                ecx: Some(d.wait)
            }
        );
        assert_eq!(d.pace, d.target);
    }
}
