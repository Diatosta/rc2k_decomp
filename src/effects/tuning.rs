//! The effects' frame timing, constants and kind floats: `MarksFrame` (IDA 0x765110 .. 0x765140), `MarksFloats` (0x765140 .. 0x765238) and
//! `MarksKindFloats` (0x765238 .. 0x7652A4).

use crate::session::{PlainDword, PlainFloat, PlainInt};

/// The marks' frame timing and counters (`effects::frame_marks`, `effects::run_frame`, `effects::draw_frame`) (IDA 0x765110 .. 0x765140).
#[derive(rc2k_derive::Cells)]
pub struct MarksFrame {
    /// 0x765110: `dword_765110`: the time stamp counter's low half at the last frame step.
    pub last_count: PlainDword,
    /// 0x765114: `dword_765114`: the cycles between the last two frame steps.
    pub frame_cycles: PlainInt,
    /// 0x765118: `flt_765118`: the share of the frame an empty timed stretch took (`effects::draw_frame`).
    pub empty_share: PlainFloat,
    /// 0x76511C: `flt_76511C`: the share the ring's frame took (`effects::run_frame`).
    pub ring_share: PlainFloat,
    /// 0x765120: `flt_765120`: the share the whole of the marks' frame took (`effects::run_frame`).
    pub whole_share: PlainFloat,
    /// 0x765124: `flt_765124`: the share `effects::begin_quads` took (`effects::draw_frame`).
    pub quads_share: PlainFloat,
    /// 0x765128: `flt_765128`: the share the frame step took (`effects::frame_marks`).
    pub step_share: PlainFloat,
    /// 0x76512C: `flt_76512C`: the frame clock the debug page shows as "clock5" (a share like its neighbours; nothing ported reads or writes it).
    pub clock5: PlainFloat,
    /// 0x765130: `flt_765130`: the frame clock the debug page shows as "clock6" (as `clock5`).
    pub clock6: PlainFloat,
    /// 0x765134: `flt_765134`: the share the draw took (`effects::run_frame`).
    pub draw_share: PlainFloat,
    /// 0x765138: `dword_765138`: a particle's side / size in hand while it is emitted (`effects::emit_rolling_particles`, `effects::emit_bursts`; a scratch).
    pub emit_side: PlainDword,
    /// 0x76513C: `dword_76513C`: the frame steps counted (cleared at `effects::init_marks`).
    pub steps: PlainDword,
}

impl MarksFrame {
    pub const fn new() -> Self {
        Self {
            last_count: PlainDword::new(0x0),
            frame_cycles: PlainInt::new(0x0),
            empty_share: PlainFloat::new(0.0),
            ring_share: PlainFloat::new(0.0),
            whole_share: PlainFloat::new(0.0),
            quads_share: PlainFloat::new(0.0),
            step_share: PlainFloat::new(0.0),
            clock5: PlainFloat::new(0.0),
            clock6: PlainFloat::new(0.0),
            draw_share: PlainFloat::new(0.0),
            emit_side: PlainDword::new(0x0),
            steps: PlainDword::new(0x0),
        }
    }
}

/// The marks' constants (a pool of integers and floats the effects' code multiplies by) (IDA 0x765140 .. 0x765238).
#[derive(rc2k_derive::Cells)]
pub struct MarksFloats {
    /// 0x765140: `dword_765140`: 0x10000, the ring's pull limit (`effects::run_ring`).
    pub pull_limit: PlainInt,
    /// 0x76514C: `flt_76514C`: -65.536, multiplied with a burst's strength and direction factors to make the burst's direction vector (`effects::emit_bursts`).
    pub burst_direction_k: PlainFloat,
    /// 0x765150: `flt_765150`: -0.125, the burst's turn scale (`effects::emit_bursts`).
    pub minus_eighth: PlainFloat,
    /// 0x765158: `flt_765158`: 0.5 (`effects::emit_bursts`, `effects::emit_car_particles`, `effects::draw_pong_box`).
    pub half: PlainFloat,
    /// 0x76515C: `flt_76515C`: 0.05, the ring's frame scale (`effects::run_ring`).
    pub twentieth: PlainFloat,
    /// 0x765164: `flt_765164`: 0.2 (`effects::emit_rolling_particles`).
    pub fifth: PlainFloat,
    /// 0x765168: `flt_765168`: 0.3 (`effects::emit_rolling_particles`).
    pub three_tenths: PlainFloat,
    /// 0x76516C: `flt_76516C`: 1.0.
    pub one: PlainFloat,
    /// 0x765174: `flt_765174`: 3.0 (`effects::emit_body_particles`).
    pub three: PlainFloat,
    /// 0x76518C: `flt_76518C`: 16.0 (`effects::draw_snow`).
    pub sixteen: PlainFloat,
    /// 0x765190: `flt_765190`: 32.0 (`effects::emit_bursts`).
    pub thirty_two: PlainFloat,
    /// 0x765198: `flt_765198`: 128.0 (`effects::emit_body_contact_particles`).
    pub hundred_twenty_eight: PlainFloat,
    /// 0x76519C: `dword_76519C`: 255.0, as bits (the particles' fade scale).
    pub two_fifty_five: PlainFloat,
    /// 0x7651A0: `dword_7651A0`: 256.0, as bits.
    pub two_fifty_six: PlainFloat,
    /// 0x7651A4: `flt_7651A4`: 257.0 (the particles' life scale).
    pub two_fifty_seven: PlainFloat,
    /// 0x7651BC: `flt_7651BC`: 0.25.
    pub quarter: PlainFloat,
    /// 0x7651C0: `flt_7651C0`: 0.125 (`effects::set_up_surface`).
    pub eighth: PlainFloat,
    /// 0x7651C4: `flt_7651C4`: 0.0625 (`effects::make_fade_colours`).
    pub sixteenth: PlainFloat,
    /// 0x7651C8: `flt_7651C8`: 1/512, the surface places' edge value (`effects::set_up_surface`'s `edge`).
    pub surface_edge: PlainFloat,
    /// 0x7651CC: `flt_7651CC`: 1/256, the surface places' pixel value (`effects::set_up_surface`'s `pixel`).
    pub surface_pixel: PlainFloat,
    /// 0x7651D0: `flt_7651D0`: 1/32: the surface places' step (`effects::set_up_surface`), the emitters' size factor (`effects::emit_body_contact_particles`, `effects::emit_rolling_particles`'s count) and the least strength.
    pub thirty_second: PlainFloat,
    /// 0x7651D4: `flt_7651D4`: 65.536, the scale `effects::emit_exhaust_smoke` turns the spline point's place with (probably the model units to metres).
    pub particle_13_place_k: PlainFloat,
    /// 0x7651DC: `flt_7651DC`: 65536.0, the 16-bit fixed-point one: a chance (0 .. 1) times it is compared with a 16-bit random, and the ring's fractions are scaled by it.
    pub fixed_16_scale: PlainFloat,
    /// 0x7651E4: `flt_7651E4`: 700.0 (`effects::emit_exhaust_smoke`).
    pub seven_hundred: PlainFloat,
    /// 0x7651E8: `flt_7651E8`: 100.0.
    pub hundred: PlainFloat,
    /// 0x7651F0: `flt_7651F0`: 2000.0 (`effects::emit_rolling_particles`).
    pub two_thousand: PlainFloat,
    /// 0x765214: `flt_765214`: 1/65536 (`effects::emit_bursts`, `effects::emit_exhaust_smoke`).
    pub over_65536: PlainFloat,
    /// 0x76521C: `dbl_76521C`: a double (low dword, high dword), 2^-31 (`effects::spread_point`).
    pub spread_scale: [PlainDword; 2],
}

impl MarksFloats {
    pub const fn new() -> Self {
        Self {
            pull_limit: PlainInt::new(0x10000),
            burst_direction_k: PlainFloat::new(-65.536),
            minus_eighth: PlainFloat::new(-0.125),
            half: PlainFloat::new(0.5),
            twentieth: PlainFloat::new(0.05),
            fifth: PlainFloat::new(0.2),
            three_tenths: PlainFloat::new(0.3),
            one: PlainFloat::new(1.0),
            three: PlainFloat::new(3.0),
            sixteen: PlainFloat::new(16.0),
            thirty_two: PlainFloat::new(32.0),
            hundred_twenty_eight: PlainFloat::new(128.0),
            two_fifty_five: PlainFloat::new(255.0),
            two_fifty_six: PlainFloat::new(256.0),
            two_fifty_seven: PlainFloat::new(257.0),
            quarter: PlainFloat::new(0.25),
            eighth: PlainFloat::new(0.125),
            sixteenth: PlainFloat::new(0.0625),
            surface_edge: PlainFloat::new(0.001953125),
            surface_pixel: PlainFloat::new(0.00390625),
            thirty_second: PlainFloat::new(0.03125),
            particle_13_place_k: PlainFloat::new(65.536),
            fixed_16_scale: PlainFloat::new(65536.0),
            seven_hundred: PlainFloat::new(700.0),
            hundred: PlainFloat::new(100.0),
            two_thousand: PlainFloat::new(2000.0),
            over_65536: PlainFloat::new(0.000015259),
            spread_scale: [PlainDword::new(0), PlainDword::new(0x3E00_0000)],
        }
    }
}

/// The settings' copies and the kinds' floats (`effects::frame_marks`, `effects::init_marks`) (IDA 0x765238 .. 0x7652A4).
#[derive(rc2k_derive::Cells)]
pub struct MarksKindFloats {
    /// 0x765238: `dword_765238`: the settings' +0x4C, copied each frame step (a reading: which colours the ribbons use).
    pub other: PlainDword,
    /// 0x76523C: `dword_76523C`: the settings' +0x50, copied each frame step (a reading: the snow fades).
    pub fading: PlainDword,
    /// 0x765240: `g_marksKindFloatTable`: 0x17 floats by kind (the depth bias).
    pub kind_floats: [PlainFloat; 23],
    /// 0x76529C: `g_marksKindFloat`: this kind's float.
    pub kind_float: PlainFloat,
    /// 0x7652A0: `dword_7652A0`: the settings' +0x10, copied each frame step (a reading: whether the effects run).
    pub running: PlainDword,
}

impl MarksKindFloats {
    pub const fn new() -> Self {
        Self {
            other: PlainDword::new(0x1),
            fading: PlainDword::new(0x1),
            kind_floats: [
                PlainFloat::new(16240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(17240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(17240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(25240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(15240.0),
                PlainFloat::new(13240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(16240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
                PlainFloat::new(18240.0),
            ],
            kind_float: PlainFloat::new(0.0),
            running: PlainDword::new(0x0),
        }
    }
}
