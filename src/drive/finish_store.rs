//! The finish routine's variables (IDA 0x64BB18 .. 0x64BB88; phase 7, typed): the
//! "no place" switch, the stage numbers of the finish, the finish lengths and limits, the
//! clock at the finish's start, the time, the eased time and the share. `#[repr(C)]` with the
//! image's layout, over the race block's bytes (`FINISH`). Plain cells: a drawing store
//! (`session::DrawShared`), written on the game's thread and read by the drawing parts (the
//! mirror's "no place" test) through their `workers::DrawReads`. Names are readings unless IDA's.

use crate::session::{PlainDword, PlainFloat};

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Finish {
    /// +0x0 (0x64BB18): `dword_64BB18`: set while the race has no place or panel to draw (`race::car_starts`, `race::draw_mirror_view`, `race::draw_race_scene`, `race::loadlevel`, `race::stage_loading`, `physics::passed_split_point`, `drive::finish_race`).
    pub no_place: PlainDword,
    /// +0x4 (0x64BB1C): `dword_64BB1C`: the stage number for each finish stage index (`drive::finish_race`).
    pub finish_stages: [PlainDword; 7],
    /// +0x20 (0x64BB38): `flt_64BB38`: how far the lead car is lifted at the finish's start, in
    /// units of 1/65536 of the world's unit (x 65536 added to the car's height, `drive::finish_run`). Sure.
    pub car_lift: PlainFloat,
    /// +0x24 (0x64BB3C): `flt_64BB3C`: the time the finish lasts, by `finished` 0 or 1 (`drive::finish_run`).
    pub finish_lengths: [PlainFloat; 2],
    /// +0x2c (0x64BB44): `flt_64BB44`: the first limit of the eased time (`drive::finish_timing`).
    pub limit_a: PlainFloat,
    /// +0x30 (0x64BB48): `flt_64BB48`: the second limit.
    pub limit_b: PlainFloat,
    /// +0x34 (0x64BB4C): `flt_64BB4C`: the finish camera's yaw turned per unit of eased time
    /// (`drive::finish_camera`: `angles[1]` = eased x this). Sure by the formula.
    pub camera_yaw_rate: PlainFloat,
    /// +0x38 (0x64BB50): `flt_64BB50`: the camera's distance offset (z) when the eased time is 0.
    pub camera_distance_start: PlainFloat,
    /// +0x3c (0x64BB54): `flt_64BB54`: the camera's height offset (y) when the eased time is 0.
    pub camera_height_start: PlainFloat,
    /// +0x40 (0x64BB58): `flt_64BB58`: the camera's distance offset (z) added per unit of eased time.
    pub camera_distance_rate: PlainFloat,
    /// +0x44 (0x64BB5C): `flt_64BB5C`: the camera's height offset (y) added per unit of eased time.
    pub camera_height_rate: PlainFloat,
    /// +0x48 (0x64BB60): `dword_64BB60`: the scene clock at the finish's start (`drive::finish_run`, `drive::finish_timing`).
    pub start_clock: PlainDword,
    /// +0x4c (0x64BB64): `flt_64BB64`: the time since the start (`drive::finish_timing`, `drive::finish_run`).
    pub time: PlainFloat,
    /// +0x50 (0x64BB68): `flt_64BB68`: the eased time.
    pub eased: PlainFloat,
    /// +0x54 (0x64BB6C): `flt_64BB6C`: the share of the last stretch, 0 to 1.
    pub share: PlainFloat,
    /// +0x58 (0x64BB70): `flt_64BB70`: the share's first limit.
    pub limit_c: PlainFloat,
    /// +0x5c (0x64BB74): `flt_64BB74`: the share's second limit.
    pub limit_d: PlainFloat,
    /// +0x60 (0x64BB78): `flt_64BB78`: the camera's pitch (`angles[0]`) added per unit of the share
    /// (`drive::finish_camera`).
    pub camera_pitch_per_share: PlainFloat,
    /// +0x64 (0x64BB7C): `flt_64BB7C`: the camera's height offset added per unit of the share.
    pub camera_height_per_share: PlainFloat,
    /// +0x68 (0x64BB80): `dword_64BB80`, `dword_64BB84`: the addresses of a loading buffer (0x3E8000
    /// bytes, or 1 MB / 0xF4240 in other places) and of its second half (+0x1F4000), set only by the
    /// loaders at 0x459764, 0x4599D4 and 0x459C1B (`dword_64BB80` with `memory::allocate`); the port
    /// reaches none of them and reads neither.
    pub unreached_loader_buffers: [PlainDword; 2],
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

/// The piece, with the image's first values.
impl Finish {
    /// The stage number for finish stage `n`. The original does not check `n` against the seven
    /// stages: past them it reads the fields after the table, in order.
    pub fn finish_stage(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(stage) = self.finish_stages.get(n) {
            return Ok(stage.get());
        }
        crate::session::read_runs(
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
        use crate::cells::ZeroCells;
        use crate::session::{PlainDword, PlainFloat};
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

    /// Every index to the end of the record gives what the byte-offset read gave, and the first
    /// past it is refused as before.
    #[test]
    fn finish_stage_runs_on_as_the_old_read() {
        let f = Finish::new();
        for n in 0..27 {
            crate::session::write_on_in(&f, &f.finish_stages, n, 0x1000 + n as u32, "t").unwrap();
        }
        for n in 0..40 {
            let old = crate::session::read_on_in(&f, &f.finish_stages, n, "t");
            assert_eq!(old.is_ok(), n < 27, "index {n}");
            assert_eq!(f.finish_stage(n).ok(), old.ok(), "index {n}");
        }
    }
}
