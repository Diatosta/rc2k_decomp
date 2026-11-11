//! The effects' frame timing, constants and kind floats: `MarksFrame` (IDA 0x765110 .. 0x765140), `MarksFloats` (0x765140 .. 0x765238) and
//! `MarksKindFloats` (0x765238 .. 0x7652A4).

#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{Bytes, Dword, Float};

/// The marks' frame timing and counters (`effects::frame_marks`, `effects::run_frame`, `effects::draw_frame`) (IDA 0x765110 .. 0x765140).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksFrame {
    /// 0x765110: `dword_765110`: the time stamp counter's low half at the last frame step.
    pub last_count: Dword,
    /// 0x765114: `dword_765114`: the cycles between the last two frame steps.
    pub frame_cycles: Dword,
    /// 0x765118: `flt_765118`: the share of the frame an empty timed stretch took (`effects::draw_frame`).
    pub empty_share: Float,
    /// 0x76511C: `flt_76511C`: the share the ring's frame took (`effects::run_frame`).
    pub ring_share: Float,
    /// 0x765120: `flt_765120`: the share the whole of the marks' frame took (`effects::run_frame`).
    pub whole_share: Float,
    /// 0x765124: `flt_765124`: the share `effects::begin_quads` took (`effects::draw_frame`).
    pub quads_share: Float,
    /// 0x765128: `flt_765128`: the share the frame step took (`effects::frame_marks`).
    pub step_share: Float,
    /// 0x76512C: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_76512c: Bytes<8>,
    /// 0x765134: `flt_765134`: the share the draw took (`effects::run_frame`).
    pub draw_share: Float,
    /// 0x765138: `dword_765138`: a particle's side / size in hand while it is emitted (`effects::emit_rolling_particles`, `effects::emit_bursts`; a scratch).
    pub emit_side: Dword,
    /// 0x76513C: `dword_76513C`: the frame steps counted (cleared at `effects::init_marks`).
    pub steps: Dword,
}

impl MarksFrame {
    pub const fn new() -> Self {
        Self {
            last_count: Dword::new(0x0),
            frame_cycles: Dword::new(0x0),
            empty_share: Float::new(0.0),
            ring_share: Float::new(0.0),
            whole_share: Float::new(0.0),
            quads_share: Float::new(0.0),
            step_share: Float::new(0.0),
            _gap_76512c: Bytes::new([0; 8]),
            draw_share: Float::new(0.0),
            emit_side: Dword::new(0x0),
            steps: Dword::new(0x0),
        }
    }
}

impl MarksFrame {
    pub const START: usize = 0x765110;
    pub const LEN: usize = 0x30;
}

#[cfg(test)]
mod marks_frame_tests {
    use super::*;

    #[test]
    fn marks_frame_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<MarksFrame>(), MarksFrame::LEN);
        let p = (&raw const effects().setup.frame).cast::<u8>();
        let ours: Vec<u8> = (0..size_of::<MarksFrame>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(MarksFrame::START, MarksFrame::LEN),
            "MARKS_FRAME starts differently from the image"
        );
        assert_eq!(
            offset_of!(MarksFrame, last_count),
            0x765110 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, frame_cycles),
            0x765114 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, empty_share),
            0x765118 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, ring_share),
            0x76511C - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, whole_share),
            0x765120 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, quads_share),
            0x765124 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, step_share),
            0x765128 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, draw_share),
            0x765134 - MarksFrame::START
        );
        assert_eq!(
            offset_of!(MarksFrame, emit_side),
            0x765138 - MarksFrame::START
        );
        assert_eq!(offset_of!(MarksFrame, steps), 0x76513C - MarksFrame::START);
    }
}

/// The marks' constants (a pool of integers and floats the effects' code multiplies by) (IDA 0x765140 .. 0x765238).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksFloats {
    /// 0x765140: `dword_765140`: 0x10000, the ring's pull limit (`effects::run_ring`).
    pub pull_limit: Dword,
    /// 0x765144: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_765144: Bytes<8>,
    /// 0x76514C: `flt_76514C`: -65.536, multiplied with a burst's strength and direction factors to make the burst's direction vector (`effects::emit_bursts`).
    pub burst_direction_k: Float,
    /// 0x765150: `flt_765150`: -0.125, the burst's turn scale (`effects::emit_bursts`).
    pub minus_eighth: Float,
    /// 0x765154: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_765154: Bytes<4>,
    /// 0x765158: `flt_765158`: 0.5 (`effects::emit_bursts`, `effects::emit_car_particles`, `effects::draw_pong_box`).
    pub half: Float,
    /// 0x76515C: `flt_76515C`: 0.05, the ring's frame scale (`effects::run_ring`).
    pub twentieth: Float,
    /// 0x765160: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_765160: Bytes<4>,
    /// 0x765164: `flt_765164`: 0.2 (`effects::emit_rolling_particles`).
    pub fifth: Float,
    /// 0x765168: `flt_765168`: 0.3 (`effects::emit_rolling_particles`).
    pub three_tenths: Float,
    /// 0x76516C: `flt_76516C`: 1.0.
    pub one: Float,
    /// 0x765170: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_765170: Bytes<4>,
    /// 0x765174: `flt_765174`: 3.0 (`effects::emit_body_particles`).
    pub three: Float,
    /// 0x765178: Not named: 20 bytes (a gap, or unreferenced data).
    pub _gap_765178: Bytes<20>,
    /// 0x76518C: `flt_76518C`: 16.0 (`effects::draw_snow`).
    pub sixteen: Float,
    /// 0x765190: `flt_765190`: 32.0 (`effects::emit_bursts`).
    pub thirty_two: Float,
    /// 0x765194: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_765194: Bytes<4>,
    /// 0x765198: `flt_765198`: 128.0 (`effects::emit_body_contact_particles`).
    pub hundred_twenty_eight: Float,
    /// 0x76519C: `dword_76519C`: 255.0, as bits (the particles' fade scale).
    pub two_fifty_five: Float,
    /// 0x7651A0: `dword_7651A0`: 256.0, as bits.
    pub two_fifty_six: Float,
    /// 0x7651A4: `flt_7651A4`: 257.0 (the particles' life scale).
    pub two_fifty_seven: Float,
    /// 0x7651A8: Not named: 20 bytes (a gap, or unreferenced data).
    pub _gap_7651a8: Bytes<20>,
    /// 0x7651BC: `flt_7651BC`: 0.25.
    pub quarter: Float,
    /// 0x7651C0: `flt_7651C0`: 0.125 (`effects::set_up_surface`).
    pub eighth: Float,
    /// 0x7651C4: `flt_7651C4`: 0.0625 (`effects::make_fade_colours`).
    pub sixteenth: Float,
    /// 0x7651C8: `flt_7651C8`: 1/512, the surface places' edge value (`effects::set_up_surface`'s `edge`).
    pub surface_edge: Float,
    /// 0x7651CC: `flt_7651CC`: 1/256, the surface places' pixel value (`effects::set_up_surface`'s `pixel`).
    pub surface_pixel: Float,
    /// 0x7651D0: `flt_7651D0`: 1/32: the surface places' step (`effects::set_up_surface`), the emitters' size factor (`effects::emit_body_contact_particles`, `effects::emit_rolling_particles`'s count) and the least strength.
    pub thirty_second: Float,
    /// 0x7651D4: `flt_7651D4`: 65.536, the scale `effects::emit_exhaust_smoke` turns the spline point's place with (probably the model units to metres).
    pub particle_13_place_k: Float,
    /// 0x7651D8: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7651d8: Bytes<4>,
    /// 0x7651DC: `flt_7651DC`: 65536.0, the 16-bit fixed-point one: a chance (0 .. 1) times it is compared with a 16-bit random, and the ring's fractions are scaled by it.
    pub fixed_16_scale: Float,
    /// 0x7651E0: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7651e0: Bytes<4>,
    /// 0x7651E4: `flt_7651E4`: 700.0 (`effects::emit_exhaust_smoke`).
    pub seven_hundred: Float,
    /// 0x7651E8: `flt_7651E8`: 100.0.
    pub hundred: Float,
    /// 0x7651EC: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_7651ec: Bytes<4>,
    /// 0x7651F0: `flt_7651F0`: 2000.0 (`effects::emit_rolling_particles`).
    pub two_thousand: Float,
    /// 0x7651F4: Not named: 32 bytes (a gap, or unreferenced data).
    pub _gap_7651f4: Bytes<32>,
    /// 0x765214: `flt_765214`: 1/65536 (`effects::emit_bursts`, `effects::emit_exhaust_smoke`).
    pub over_65536: Float,
    /// 0x765218: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_765218: Bytes<4>,
    /// 0x76521C: `dbl_76521C`: a double, 2^-31 (`effects::spread_point`).
    pub spread_scale: Bytes<8>,
    /// 0x765224: Not named: 20 bytes (a gap, or unreferenced data).
    pub _gap_765224: Bytes<20>,
}

impl MarksFloats {
    pub const fn new() -> Self {
        Self {
            pull_limit: Dword::new(0x10000),
            _gap_765144: Bytes::new([0x00, 0x00, 0x80, 0xBF, 0x00, 0x00, 0x80, 0xC7]),
            burst_direction_k: Float::new(-65.536),
            minus_eighth: Float::new(-0.125),
            _gap_765154: Bytes::new([0x00, 0x00, 0x80, 0x3E]),
            half: Float::new(0.5),
            twentieth: Float::new(0.05),
            _gap_765160: Bytes::new([0xCD, 0xCC, 0xCC, 0x3D]),
            fifth: Float::new(0.2),
            three_tenths: Float::new(0.3),
            one: Float::new(1.0),
            _gap_765170: Bytes::new([0x00, 0x00, 0x00, 0x40]),
            three: Float::new(3.0),
            _gap_765178: Bytes::new([
                0x00, 0x00, 0x80, 0x40, 0x00, 0x00, 0xA0, 0x40, 0x00, 0x00, 0xC0, 0x40, 0x00, 0x00,
                0xE0, 0x40, 0x00, 0x00, 0x00, 0x41,
            ]),
            sixteen: Float::new(16.0),
            thirty_two: Float::new(32.0),
            _gap_765194: Bytes::new([0x00, 0x00, 0x80, 0x42]),
            hundred_twenty_eight: Float::new(128.0),
            two_fifty_five: Float::new(255.0),
            two_fifty_six: Float::new(256.0),
            two_fifty_seven: Float::new(257.0),
            _gap_7651a8: Bytes::new([
                0x00, 0x00, 0x00, 0x44, 0x00, 0x00, 0x80, 0x44, 0x00, 0x00, 0x02, 0x42, 0x00, 0x00,
                0x7A, 0x42, 0x00, 0x00, 0xFA, 0x42,
            ]),
            quarter: Float::new(0.25),
            eighth: Float::new(0.125),
            sixteenth: Float::new(0.0625),
            surface_edge: Float::new(0.001953125),
            surface_pixel: Float::new(0.00390625),
            thirty_second: Float::new(0.03125),
            particle_13_place_k: Float::new(65.536),
            _gap_7651d8: Bytes::new([0x0A, 0xD7, 0x23, 0x44]),
            fixed_16_scale: Float::new(65536.0),
            _gap_7651e0: Bytes::new([0x00, 0x00, 0x00, 0x47]),
            seven_hundred: Float::new(700.0),
            hundred: Float::new(100.0),
            _gap_7651ec: Bytes::new([0x00, 0x00, 0xFA, 0x43]),
            two_thousand: Float::new(2000.0),
            _gap_7651f4: Bytes::new([
                0xF1, 0xF0, 0x70, 0x3D, 0x00, 0x00, 0x00, 0x3B, 0x39, 0xFE, 0x7F, 0x3A, 0xF7, 0xFF,
                0xFF, 0x39, 0x0D, 0x00, 0x80, 0x39, 0xD5, 0xFF, 0xFF, 0x38, 0xD5, 0xFF, 0x7F, 0x38,
                0x74, 0x00, 0x00, 0x38,
            ]),
            over_65536: Float::new(0.000015259),
            _gap_765218: Bytes::new([0x00, 0x00, 0x00, 0x4F]),
            spread_scale: Bytes::new([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3E]),
            _gap_765224: Bytes::new([
                0x00, 0x00, 0x80, 0x50, 0x00, 0x00, 0x80, 0x4F, 0x00, 0x00, 0x80, 0x4E, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]),
        }
    }
}

impl MarksFloats {
    pub const START: usize = 0x765140;
    pub const LEN: usize = 0xF8;
}

#[cfg(test)]
mod marks_floats_tests {
    use super::*;

    #[test]
    fn marks_floats_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<MarksFloats>(), MarksFloats::LEN);
        let p = (&raw const effects().setup.floats).cast::<u8>();
        let ours: Vec<u8> = (0..size_of::<MarksFloats>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(MarksFloats::START, MarksFloats::LEN),
            "MARKS_FLOATS starts differently from the image"
        );
        assert_eq!(
            offset_of!(MarksFloats, pull_limit),
            0x765140 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, burst_direction_k),
            0x76514C - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, minus_eighth),
            0x765150 - MarksFloats::START
        );
        assert_eq!(offset_of!(MarksFloats, half), 0x765158 - MarksFloats::START);
        assert_eq!(
            offset_of!(MarksFloats, twentieth),
            0x76515C - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, fifth),
            0x765164 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, three_tenths),
            0x765168 - MarksFloats::START
        );
        assert_eq!(offset_of!(MarksFloats, one), 0x76516C - MarksFloats::START);
        assert_eq!(
            offset_of!(MarksFloats, three),
            0x765174 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, sixteen),
            0x76518C - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, thirty_two),
            0x765190 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, hundred_twenty_eight),
            0x765198 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, two_fifty_five),
            0x76519C - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, two_fifty_six),
            0x7651A0 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, two_fifty_seven),
            0x7651A4 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, quarter),
            0x7651BC - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, eighth),
            0x7651C0 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, sixteenth),
            0x7651C4 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, surface_edge),
            0x7651C8 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, surface_pixel),
            0x7651CC - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, thirty_second),
            0x7651D0 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, particle_13_place_k),
            0x7651D4 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, fixed_16_scale),
            0x7651DC - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, seven_hundred),
            0x7651E4 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, hundred),
            0x7651E8 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, two_thousand),
            0x7651F0 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, over_65536),
            0x765214 - MarksFloats::START
        );
        assert_eq!(
            offset_of!(MarksFloats, spread_scale),
            0x76521C - MarksFloats::START
        );
    }
}

/// The settings' copies and the kinds' floats (`effects::frame_marks`, `effects::init_marks`) (IDA 0x765238 .. 0x7652A4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct MarksKindFloats {
    /// 0x765238: `dword_765238`: the settings' +0x4C, copied each frame step (a reading: which colours the ribbons use).
    pub other: Dword,
    /// 0x76523C: `dword_76523C`: the settings' +0x50, copied each frame step (a reading: the snow fades).
    pub fading: Dword,
    /// 0x765240: `g_marksKindFloatTable`: 0x17 floats by kind (the depth bias).
    pub kind_floats: [Float; 23],
    /// 0x76529C: `g_marksKindFloat`: this kind's float.
    pub kind_float: Float,
    /// 0x7652A0: `dword_7652A0`: the settings' +0x10, copied each frame step (a reading: whether the effects run).
    pub running: Dword,
}

impl MarksKindFloats {
    pub const fn new() -> Self {
        Self {
            other: Dword::new(0x1),
            fading: Dword::new(0x1),
            kind_floats: [
                Float::new(16240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(17240.0),
                Float::new(18240.0),
                Float::new(17240.0),
                Float::new(18240.0),
                Float::new(25240.0),
                Float::new(18240.0),
                Float::new(15240.0),
                Float::new(13240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(16240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
                Float::new(18240.0),
            ],
            kind_float: Float::new(0.0),
            running: Dword::new(0x0),
        }
    }
}

impl MarksKindFloats {
    pub const START: usize = 0x765238;
    pub const LEN: usize = 0x6C;
}

#[cfg(test)]
mod marks_kind_floats_tests {
    use super::*;

    #[test]
    fn marks_kind_floats_starts_as_the_games() {
        use core::mem::offset_of;
        assert_eq!(size_of::<MarksKindFloats>(), MarksKindFloats::LEN);
        let p = (&raw const effects().setup.kind_floats).cast::<u8>();
        let ours: Vec<u8> = (0..size_of::<MarksKindFloats>())
            .map(|i| unsafe { p.add(i).read() })
            .collect();
        assert!(
            ours == crate::moved::image_bytes(MarksKindFloats::START, MarksKindFloats::LEN),
            "MARKS_KIND_FLOATS starts differently from the image"
        );
        assert_eq!(
            offset_of!(MarksKindFloats, other),
            0x765238 - MarksKindFloats::START
        );
        assert_eq!(
            offset_of!(MarksKindFloats, fading),
            0x76523C - MarksKindFloats::START
        );
        assert_eq!(
            offset_of!(MarksKindFloats, kind_floats),
            0x765240 - MarksKindFloats::START
        );
        assert_eq!(
            offset_of!(MarksKindFloats, kind_float),
            0x76529C - MarksKindFloats::START
        );
        assert_eq!(
            offset_of!(MarksKindFloats, running),
            0x7652A0 - MarksKindFloats::START
        );
    }
}
