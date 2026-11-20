//! The effects' frame timing, constants and kind floats: `MarksFrame`, `MarksFloats` and
//! `MarksKindFloats`.

use crate::memory::session::{PlainDword, PlainFloat, PlainInt};

/// The marks' frame timing and counters (`effects::frame_marks`, `effects::run_frame`,
/// `effects::draw_frame`).
#[derive(rc2k_derive::Cells)]
pub struct MarksFrame {
    /// The time stamp counter's low half at the last frame step.
    pub last_count: PlainDword,
    /// The cycles between the last two frame steps.
    pub frame_cycles: PlainInt,
    /// The share of the frame an empty timed stretch took (`effects::draw_frame`).
    pub empty_share: PlainFloat,
    /// The share the ring's frame took (`effects::run_frame`).
    pub ring_share: PlainFloat,
    /// The share the whole of the marks' frame took (`effects::run_frame`).
    pub whole_share: PlainFloat,
    /// The share `effects::begin_quads` took (`effects::draw_frame`).
    pub quads_share: PlainFloat,
    /// The share the frame step took (`effects::frame_marks`).
    pub step_share: PlainFloat,
    /// The frame clock the debug page shows as "clock5" (a share like its neighbours; nothing
    /// reads or writes it).
    pub clock5: PlainFloat,
    /// The frame clock the debug page shows as "clock6" (as `clock5`).
    pub clock6: PlainFloat,
    /// The share the draw took (`effects::run_frame`).
    pub draw_share: PlainFloat,
    /// A particle's side / size in hand while it is emitted (`effects::emit_rolling_dust`,
    /// `effects::emit_wheel_grit_bursts`; a scratch).
    pub emit_side: PlainDword,
    /// The frame steps counted (cleared at `effects::init_marks`).
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

/// The marks' constants (a pool of integers and floats the effects' code multiplies by).
#[derive(rc2k_derive::Cells)]
pub struct MarksFloats {
    /// 0x10000, the ring's pull limit (`effects::run_ring`).
    pub pull_limit: PlainInt,
    /// -65.536, multiplied with a burst's strength and direction factors to make the burst's
    /// direction vector (`effects::emit_wheel_grit_bursts`).
    pub burst_direction_k: PlainFloat,
    /// -0.125, the burst's turn scale (`effects::emit_wheel_grit_bursts`).
    pub minus_eighth: PlainFloat,
    /// 0.5 (`effects::emit_wheel_grit_bursts`, `effects::emit_car_particles`, `effects::draw_pong_box`).
    pub half: PlainFloat,
    /// 0.05, the ring's frame scale (`effects::run_ring`).
    pub twentieth: PlainFloat,
    /// 0.2 (`effects::emit_rolling_dust`).
    pub fifth: PlainFloat,
    /// 0.3 (`effects::emit_rolling_dust`).
    pub three_tenths: PlainFloat,
    /// 1.0.
    pub one: PlainFloat,
    /// 3.0 (`effects::emit_body_particles`).
    pub three: PlainFloat,
    /// 16.0 (`effects::draw_snow`).
    pub sixteen: PlainFloat,
    /// 32.0 (`effects::emit_wheel_grit_bursts`).
    pub thirty_two: PlainFloat,
    /// 128.0 (`effects::emit_body_contact_particles`).
    pub hundred_twenty_eight: PlainFloat,
    /// 255.0, as bits (the particles' fade scale).
    pub two_fifty_five: PlainFloat,
    /// 256.0, as bits.
    pub two_fifty_six: PlainFloat,
    /// 257.0 (the particles' life scale).
    pub two_fifty_seven: PlainFloat,
    /// 0.25.
    pub quarter: PlainFloat,
    /// 0.125 (`effects::set_up_surface`).
    pub eighth: PlainFloat,
    /// 0.0625 (`effects::make_fade_colours`).
    pub sixteenth: PlainFloat,
    /// 1/512, the surface places' edge value (`effects::set_up_surface`'s `edge`).
    pub surface_edge: PlainFloat,
    /// 1/256, the surface places' pixel value (`effects::set_up_surface`'s `pixel`).
    pub surface_pixel: PlainFloat,
    /// 1/32: the surface places' step (`effects::set_up_surface`), the emitters' size factor
    /// (`effects::emit_body_contact_particles`, `effects::emit_rolling_dust`'s count) and the
    /// least strength.
    pub thirty_second: PlainFloat,
    /// 65.536, the scale `effects::emit_exhaust_smoke` turns the spline point's place with
    /// (probably the model units to metres).
    pub particle_13_place_k: PlainFloat,
    /// 65536.0, the 16-bit fixed-point one: a chance (0 .. 1) times it is compared with a 16-bit
    /// random, and the ring's fractions are scaled by it.
    pub fixed_16_scale: PlainFloat,
    /// 700.0 (`effects::emit_exhaust_smoke`).
    pub seven_hundred: PlainFloat,
    /// 100.0.
    pub hundred: PlainFloat,
    /// 2000.0 (`effects::emit_rolling_dust`).
    pub two_thousand: PlainFloat,
    /// 1/65536 (`effects::emit_wheel_grit_bursts`, `effects::emit_exhaust_smoke`).
    pub over_65536: PlainFloat,
    /// A double (low dword, high dword), 2^-31 (`effects::spread_point`).
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

/// The settings' copies and the kinds' floats (`effects::frame_marks`, `effects::init_marks`).
#[derive(rc2k_derive::Cells)]
pub struct MarksKindFloats {
    /// The settings' `after_loading_copy`, copied each frame step (a reading: which colours the ribbons use).
    pub other: PlainDword,
    /// The settings' `listener_near`, copied each frame step (a reading: the snow fades).
    pub fading: PlainDword,
    /// 0x17 floats by kind (the depth bias).
    pub kind_floats: [PlainFloat; 23],
    /// This kind's float.
    pub kind_float: PlainFloat,
    /// The settings' `other_marks_shown`, copied each frame step (a reading: whether the effects run).
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
