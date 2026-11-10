//! The finish routine's camera (converted 2026-10-21 from `drive::finish_camera`
//! 0x45941D): four of the camera's figures, each a line in the eased time and the share.
//! The figures are the camera's yaw, height, distance and pitch: each
//! is a start value plus a rate times the eased time (or the share), the constants being
//! `Finish::camera_*` (`src/drive/finish_store.rs`).
//!
//! Floats: what the x87 keeps on its stack is an `f64`, rounded to `f32` where the
//! original stores: the same bits at the game's double precision.

/// The lines' figures (`flt_64BB4C` .. `flt_64BB7C`), the time (`flt_64BB68`) and the
/// share (`flt_64BB6C`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FinishLines {
    /// `flt_64BB4C`, `flt_64BB50`, `flt_64BB54`, `flt_64BB58`, `flt_64BB5C`.
    pub k4c: f32,
    pub k50: f32,
    pub k54: f32,
    pub k58: f32,
    pub k5c: f32,
    pub time: f32,
    pub share: f32,
    /// `flt_64BB78`, `flt_64BB7C`.
    pub k78: f32,
    pub k7c: f32,
}

/// The four figures: (`dword_606748`, `dword_606730`, `dword_606734`, `dword_606744`).
pub fn finish_camera(l: &FinishLines) -> [f32; 4] {
    let (t, s) = (l.time as f64, l.share as f64);
    [
        (t * l.k4c as f64) as f32,
        (t * l.k5c as f64 + l.k54 as f64 + s * l.k7c as f64) as f32,
        (t * l.k58 as f64 + l.k50 as f64) as f32,
        (s * l.k78 as f64) as f32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines() {
        let l = FinishLines {
            k4c: 2.0,
            k50: 1.0,
            k54: 3.0,
            k58: 4.0,
            k5c: 5.0,
            time: 0.5,
            share: 0.25,
            k78: 8.0,
            k7c: 4.0,
        };
        assert_eq!(finish_camera(&l), [1.0, 6.5, 3.0, 2.0]);
    }
}
