//! The finish routine's variables: the "no place" switch, the stage numbers of the finish, the
//! finish lengths and limits, the clock at the finish's start, the time, the eased time and the
//! share. Plain cells: a drawing store (`session::DrawShared`), written on the game's thread and
//! read by the drawing parts (the mirror's "no place" test) through their `scene::workers::DrawReads`.

use crate::memory::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Finish {
    /// Set while the race has no place or panel to draw.
    pub no_place: PlainDword,
    /// The stage number for each finish stage index.
    pub finish_stages: [PlainDword; 7],
    /// How far the lead car is lifted at the finish's start (x 65536 added to the car's height).
    pub car_lift: PlainFloat,
    /// The time the finish lasts, by `finished` 0 or 1.
    pub finish_lengths: [PlainFloat; 2],
    /// The first limit of the eased time.
    pub limit_a: PlainFloat,
    /// The second limit.
    pub limit_b: PlainFloat,
    /// The finish camera's yaw per unit of eased time.
    pub camera_yaw_rate: PlainFloat,
    /// The camera's distance offset (z) when the eased time is 0.
    pub camera_distance_start: PlainFloat,
    /// The camera's height offset (y) when the eased time is 0.
    pub camera_height_start: PlainFloat,
    /// The camera's distance offset (z) added per unit of eased time.
    pub camera_distance_rate: PlainFloat,
    /// The camera's height offset (y) added per unit of eased time.
    pub camera_height_rate: PlainFloat,
    /// The scene clock at the finish's start.
    pub start_clock: PlainDword,
    /// The time since the start.
    pub time: PlainFloat,
    pub eased: PlainFloat,
    /// The share of the last stretch, 0 to 1.
    pub share: PlainFloat,
    /// The share's first limit.
    pub limit_c: PlainFloat,
    /// The share's second limit.
    pub limit_d: PlainFloat,
    /// The camera's pitch added per unit of the share.
    pub camera_pitch_per_share: PlainFloat,
    /// The camera's height offset added per unit of the share.
    pub camera_height_per_share: PlainFloat,
    /// Two loading-buffer slots (0x3E8000 bytes, and its second half) that nothing reads or
    /// writes here.
    pub unreached_loader_buffers: [PlainDword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl Finish {
    /// The stage number for finish stage `n`. The original does not check `n` against the seven
    /// stages: past them it reads the fields after the table, in order.
    pub(super) fn finish_stage(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(stage) = self.finish_stages.get(n) {
            return Ok(stage.get());
        }
        crate::memory::session::read_runs(
            &[
                &self.finish_stages,
                &self.car_lift,
                &self.finish_lengths,
                &self.limit_a,
                &self.limit_b,
                &self.camera_yaw_rate,
                &self.camera_distance_start,
                &self.camera_height_start,
                &self.camera_distance_rate,
                &self.camera_height_rate,
                &self.start_clock,
                &self.time,
                &self.eased,
                &self.share,
                &self.limit_c,
                &self.limit_d,
                &self.camera_pitch_per_share,
                &self.camera_height_per_share,
                &self.unreached_loader_buffers,
            ],
            n,
            "the finish stages",
        )
    }

    pub const fn new() -> Self {
        use crate::memory::cells::ZeroCells;
        use crate::memory::session::{PlainDword, PlainFloat};
        Self {
            finish_stages: {
                let mut v: [PlainDword; 7] = ZeroCells::ZERO;
                v[0] = PlainDword::new(6);
                v[1] = PlainDword::new(5);
                v[2] = PlainDword::new(4);
                v[3] = PlainDword::new(3);
                v[4] = PlainDword::new(1);
                v[5] = PlainDword::new(2);
                v
            },
            car_lift: PlainFloat::new(0.6),
            finish_lengths: {
                let mut v: [PlainFloat; 2] = ZeroCells::ZERO;
                v[0] = PlainFloat::new(52.0);
                v[1] = PlainFloat::new(99.0);
                v
            },
            limit_a: PlainFloat::new(16.0),
            limit_b: PlainFloat::new(24.0),
            camera_yaw_rate: PlainFloat::new(0.125),
            camera_distance_start: PlainFloat::new(-27.0),
            camera_height_start: PlainFloat::new(6.0),
            camera_distance_rate: PlainFloat::new(1.0),
            camera_height_rate: PlainFloat::new(-0.25),
            limit_c: PlainFloat::new(32.0),
            limit_d: PlainFloat::new(40.0),
            camera_pitch_per_share: PlainFloat::new(-0.125),
            camera_height_per_share: PlainFloat::new(8.0),
            ..ZeroCells::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every index to the end of the fields gives the entry of the fields after the table, and
    /// the first past them is refused.
    #[test]
    fn finish_stage_runs_on_through_the_fields_after() {
        let f = Finish::new();
        let runs: [&dyn crate::memory::session::Run; 19] = [
            &f.finish_stages,
            &f.car_lift,
            &f.finish_lengths,
            &f.limit_a,
            &f.limit_b,
            &f.camera_yaw_rate,
            &f.camera_distance_start,
            &f.camera_height_start,
            &f.camera_distance_rate,
            &f.camera_height_rate,
            &f.start_clock,
            &f.time,
            &f.eased,
            &f.share,
            &f.limit_c,
            &f.limit_d,
            &f.camera_pitch_per_share,
            &f.camera_height_per_share,
            &f.unreached_loader_buffers,
        ];
        for n in 0..27 {
            crate::memory::session::write_runs(&runs, n, 0x1000 + n as u32, "t").unwrap();
        }
        for n in 0..40 {
            let read = f.finish_stage(n);
            assert_eq!(read.is_ok(), n < 27, "index {n}");
            if n < 27 {
                assert_eq!(read.ok(), Some(0x1000 + n as u32), "index {n}");
            }
        }
    }
}
