//! The finish routine's camera: four of the camera's figures, each a line in the eased time and the
//! share. The figures are the camera's yaw, height, distance and pitch: each is a start value plus
//! a rate times the eased time (or the share), the constants being `Finish::camera_*`
//! (`src/drive/finish_store.rs`).
//!
//! Floats: sums are `f64` and rounded to `f32` where stored, the same bits as the game's x87 at
//! double precision.

/// The lines' figures, the time and the share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FinishLines {
    /// `Finish.camera_yaw_rate`, `Finish.camera_distance_start`, `Finish.camera_height_start`,
    /// `Finish.camera_distance_rate`, `Finish.camera_height_rate`.
    pub k4c: f32,
    pub k50: f32,
    pub k54: f32,
    pub k58: f32,
    pub k5c: f32,
    pub time: f32,
    pub share: f32,
    /// `Finish.camera_pitch_per_share`, `Finish.camera_height_per_share`.
    pub k78: f32,
    pub k7c: f32,
}

/// The four figures: yaw, height, distance and pitch.
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
