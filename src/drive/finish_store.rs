//! The finish routine's variables: the "no place" switch, the stage numbers of the finish, the
//! finish lengths and limits, the clock at the finish's start, the time, the eased time and the
//! share. Plain fields on the game's thread (`race::owner_replay::Replay::finish`); the drawing
//! parts are handed the "no place" switch by value (`scene::workers`).

#[derive(Clone, Copy)]
pub struct Finish {
    /// Set while the race has no place or panel to draw.
    pub no_place: u32,
    /// The stage number for each finish stage index.
    pub finish_stages: [u32; 7],
    /// How far the lead car is lifted at the finish's start (x 65536 added to the car's height).
    pub car_lift: f32,
    /// The time the finish lasts, by `finished` 0 or 1.
    pub finish_lengths: [f32; 2],
    /// The first limit of the eased time.
    pub limit_a: f32,
    /// The second limit.
    pub limit_b: f32,
    /// The finish camera's yaw per unit of eased time.
    pub camera_yaw_rate: f32,
    /// The camera's distance offset (z) when the eased time is 0.
    pub camera_distance_start: f32,
    /// The camera's height offset (y) when the eased time is 0.
    pub camera_height_start: f32,
    /// The camera's distance offset (z) added per unit of eased time.
    pub camera_distance_rate: f32,
    /// The camera's height offset (y) added per unit of eased time.
    pub camera_height_rate: f32,
    /// The scene clock at the finish's start.
    pub start_clock: u32,
    /// The time since the start.
    pub time: f32,
    pub eased: f32,
    /// The share of the last stretch, 0 to 1.
    pub share: f32,
    /// The share's first limit.
    pub limit_c: f32,
    /// The share's second limit.
    pub limit_d: f32,
    /// The camera's pitch added per unit of the share.
    pub camera_pitch_per_share: f32,
    /// The camera's height offset added per unit of the share.
    pub camera_height_per_share: f32,
    /// Two loading-buffer slots (0x3E8000 bytes, and its second half) that nothing reads or
    /// writes here.
    pub unreached_loader_buffers: [u32; 2],
}

impl Finish {
    /// The fields after the stage table as the dwords the original's table runs on into
    /// (floats as their bits), in order.
    fn dwords(&self) -> [u32; 27] {
        let mut words = [0; 27];
        words[..7].copy_from_slice(&self.finish_stages);
        let floats = [
            self.car_lift,
            self.finish_lengths[0],
            self.finish_lengths[1],
            self.limit_a,
            self.limit_b,
            self.camera_yaw_rate,
            self.camera_distance_start,
            self.camera_height_start,
            self.camera_distance_rate,
            self.camera_height_rate,
        ];
        for (word, float) in words[7..17].iter_mut().zip(floats) {
            *word = float.to_bits();
        }
        words[17] = self.start_clock;
        let floats = [
            self.time,
            self.eased,
            self.share,
            self.limit_c,
            self.limit_d,
            self.camera_pitch_per_share,
            self.camera_height_per_share,
        ];
        for (word, float) in words[18..25].iter_mut().zip(floats) {
            *word = float.to_bits();
        }
        words[25..].copy_from_slice(&self.unreached_loader_buffers);
        words
    }

    /// The stage number for finish stage `n`. The original does not check `n` against the seven
    /// stages: past them it reads the fields after the table, in order.
    pub(super) fn finish_stage(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(stage) = self.finish_stages.get(n) {
            return Ok(*stage);
        }
        self.dwords().get(n).copied().ok_or_else(|| {
            crate::memory::host_hooks::log(&format!(
                "pointers: the finish stages: dword {n}, past the 27 dwords listed"
            ));
            crate::errors::BadInput("a dword past the fields of its record")
        })
    }

    pub const fn new() -> Self {
        Self {
            no_place: 0,
            finish_stages: [6, 5, 4, 3, 1, 2, 0],
            car_lift: 0.6,
            finish_lengths: [52.0, 99.0],
            limit_a: 16.0,
            limit_b: 24.0,
            camera_yaw_rate: 0.125,
            camera_distance_start: -27.0,
            camera_height_start: 6.0,
            camera_distance_rate: 1.0,
            camera_height_rate: -0.25,
            start_clock: 0,
            time: 0.0,
            eased: 0.0,
            share: 0.0,
            limit_c: 32.0,
            limit_d: 40.0,
            camera_pitch_per_share: -0.125,
            camera_height_per_share: 8.0,
            unreached_loader_buffers: [0; 2],
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
        let mut f = Finish::new();
        f.finish_stages = core::array::from_fn(|n| 0x1000 + n as u32);
        f.car_lift = f32::from_bits(0x1007);
        f.finish_lengths = [f32::from_bits(0x1008), f32::from_bits(0x1009)];
        f.limit_a = f32::from_bits(0x100A);
        f.limit_b = f32::from_bits(0x100B);
        f.camera_yaw_rate = f32::from_bits(0x100C);
        f.camera_distance_start = f32::from_bits(0x100D);
        f.camera_height_start = f32::from_bits(0x100E);
        f.camera_distance_rate = f32::from_bits(0x100F);
        f.camera_height_rate = f32::from_bits(0x1010);
        f.start_clock = 0x1011;
        f.time = f32::from_bits(0x1012);
        f.eased = f32::from_bits(0x1013);
        f.share = f32::from_bits(0x1014);
        f.limit_c = f32::from_bits(0x1015);
        f.limit_d = f32::from_bits(0x1016);
        f.camera_pitch_per_share = f32::from_bits(0x1017);
        f.camera_height_per_share = f32::from_bits(0x1018);
        f.unreached_loader_buffers = [0x1019, 0x101A];
        for n in 0..40 {
            let read = f.finish_stage(n);
            assert_eq!(read.is_ok(), n < 27, "index {n}");
            if n < 27 {
                assert_eq!(read.ok(), Some(0x1000 + n as u32), "index {n}");
            }
        }
    }
}
