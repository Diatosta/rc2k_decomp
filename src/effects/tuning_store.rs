//! The effects' frame timing, constants and kind floats: `MarksFrame`, `MarksFloats` and
//! `MarksKindFloats`.

use core::cell::Cell;

/// The marks' frame timing and counters (`effects::frame_marks`, `effects::run_frame`,
/// `effects::draw_frame`).
pub struct MarksFrame {
    /// The time stamp counter's low half at the last frame step.
    pub last_count: Cell<u32>,
    /// The cycles between the last two frame steps.
    pub frame_cycles: Cell<i32>,
    /// The share of the frame an empty timed stretch took (`effects::draw_frame`).
    pub empty_share: Cell<f32>,
    /// The share the ring's frame took (`effects::run_frame`).
    pub ring_share: Cell<f32>,
    /// The share the whole of the marks' frame took (`effects::run_frame`).
    pub whole_share: Cell<f32>,
    /// The share `effects::begin_quads` took (`effects::draw_frame`).
    pub quads_share: Cell<f32>,
    /// The share the frame step took (`effects::frame_marks`).
    pub step_share: Cell<f32>,
    /// The frame clock the debug page shows as "clock5" (a share like its neighbours; nothing
    /// reads or writes it).
    pub clock5: Cell<f32>,
    /// The frame clock the debug page shows as "clock6" (as `clock5`).
    pub clock6: Cell<f32>,
    /// The share the draw took (`effects::run_frame`).
    pub draw_share: Cell<f32>,
    /// A particle's side / size in hand while it is emitted (`effects::emit_rolling_dust`,
    /// `effects::emit_wheel_grit_bursts`; a scratch).
    pub emit_side: Cell<u32>,
    /// The frame steps counted (cleared at `effects::init_marks`).
    pub steps: Cell<u32>,
}

impl MarksFrame {
    pub const fn new() -> Self {
        Self {
            last_count: Cell::new(0x0),
            frame_cycles: Cell::new(0x0),
            empty_share: Cell::new(0.0),
            ring_share: Cell::new(0.0),
            whole_share: Cell::new(0.0),
            quads_share: Cell::new(0.0),
            step_share: Cell::new(0.0),
            clock5: Cell::new(0.0),
            clock6: Cell::new(0.0),
            draw_share: Cell::new(0.0),
            emit_side: Cell::new(0x0),
            steps: Cell::new(0x0),
        }
    }
}

/// The marks' constants (a pool of integers and floats the effects' code multiplies by).
pub struct MarksFloats {
    /// 0x10000, the ring's pull limit (`effects::run_ring`).
    pub pull_limit: i32,
    /// -65.536, multiplied with a burst's strength and direction factors to make the burst's
    /// direction vector (`effects::emit_wheel_grit_bursts`).
    pub burst_direction_k: f32,
    /// -0.125, the burst's turn scale (`effects::emit_wheel_grit_bursts`).
    pub minus_eighth: f32,
    /// 0.5 (`effects::emit_wheel_grit_bursts`, `effects::emit_car_particles`, `effects::draw_pong_box`).
    pub half: f32,
    /// 0.05, the ring's frame scale (`effects::run_ring`).
    pub twentieth: f32,
    /// 0.2 (`effects::emit_rolling_dust`).
    pub fifth: f32,
    /// 0.3 (`effects::emit_rolling_dust`).
    pub three_tenths: f32,
    /// 1.0.
    pub one: f32,
    /// 3.0 (`effects::emit_body_particles`).
    pub three: f32,
    /// 16.0 (`effects::draw_snow`).
    pub sixteen: f32,
    /// 32.0 (`effects::emit_wheel_grit_bursts`).
    pub thirty_two: f32,
    /// 128.0 (`effects::emit_body_contact_particles`).
    pub hundred_twenty_eight: f32,
    /// 255.0, as bits (the particles' fade scale).
    pub two_fifty_five: f32,
    /// 256.0, as bits.
    pub two_fifty_six: f32,
    /// 257.0 (the particles' life scale).
    pub two_fifty_seven: f32,
    /// 0.25.
    pub quarter: f32,
    /// 0.125 (`effects::set_up_surface`).
    pub eighth: f32,
    /// 0.0625 (`effects::make_fade_colours`).
    pub sixteenth: f32,
    /// 1/512, the surface places' edge value (`effects::set_up_surface`'s `edge`).
    pub surface_edge: f32,
    /// 1/256, the surface places' pixel value (`effects::set_up_surface`'s `pixel`).
    pub surface_pixel: f32,
    /// 1/32: the surface places' step (`effects::set_up_surface`), the emitters' size factor
    /// (`effects::emit_body_contact_particles`, `effects::emit_rolling_dust`'s count) and the
    /// least strength.
    pub thirty_second: f32,
    /// 65.536, the scale `effects::emit_exhaust_smoke` turns the spline point's place with
    /// (probably the model units to metres).
    pub particle_13_place_k: f32,
    /// 65536.0, the 16-bit fixed-point one: a chance (0 .. 1) times it is compared with a 16-bit
    /// random, and the ring's fractions are scaled by it.
    pub fixed_16_scale: f32,
    /// 700.0 (`effects::emit_exhaust_smoke`).
    pub seven_hundred: f32,
    /// 100.0.
    pub hundred: f32,
    /// 2000.0 (`effects::emit_rolling_dust`).
    pub two_thousand: f32,
    /// 1/65536 (`effects::emit_wheel_grit_bursts`, `effects::emit_exhaust_smoke`).
    pub over_65536: f32,
    /// A double (low dword, high dword), 2^-31 (`effects::spread_point`).
    pub spread_scale: [u32; 2],
}

impl MarksFloats {
    pub const fn new() -> Self {
        Self {
            pull_limit: 0x10000,
            burst_direction_k: -65.536,
            minus_eighth: -0.125,
            half: 0.5,
            twentieth: 0.05,
            fifth: 0.2,
            three_tenths: 0.3,
            one: 1.0,
            three: 3.0,
            sixteen: 16.0,
            thirty_two: 32.0,
            hundred_twenty_eight: 128.0,
            two_fifty_five: 255.0,
            two_fifty_six: 256.0,
            two_fifty_seven: 257.0,
            quarter: 0.25,
            eighth: 0.125,
            sixteenth: 0.0625,
            surface_edge: 0.001953125,
            surface_pixel: 0.00390625,
            thirty_second: 0.03125,
            particle_13_place_k: 65.536,
            fixed_16_scale: 65536.0,
            seven_hundred: 700.0,
            hundred: 100.0,
            two_thousand: 2000.0,
            over_65536: 0.000015259,
            spread_scale: [0, 0x3E00_0000],
        }
    }
}

/// The settings' copies and the kinds' floats (`effects::frame_marks`, `effects::init_marks`).
pub struct MarksKindFloats {
    /// The settings' `after_loading_copy`, copied each frame step (a reading: which colours the ribbons use).
    pub other: u32,
    /// The settings' `listener_near`, copied each frame step (a reading: the snow fades).
    pub fading: u32,
    /// 0x17 floats by kind (the depth bias).
    pub kind_floats: [f32; 23],
    /// This kind's float.
    pub kind_float: f32,
    /// The settings' `other_marks_shown`, copied each frame step (a reading: whether the effects run).
    pub running: u32,
}

impl MarksKindFloats {
    pub const fn new() -> Self {
        Self {
            other: 0x1,
            fading: 0x1,
            kind_floats: [
                16240.0, 18240.0, 18240.0, 18240.0, 18240.0, 18240.0, 17240.0, 18240.0, 17240.0,
                18240.0, 25240.0, 18240.0, 15240.0, 13240.0, 18240.0, 18240.0, 16240.0, 18240.0,
                18240.0, 18240.0, 18240.0, 18240.0, 18240.0,
            ],
            kind_float: 0.0,
            running: 0x0,
        }
    }
}
