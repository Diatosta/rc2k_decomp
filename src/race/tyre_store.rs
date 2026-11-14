//! The roadside tyre change's variables (`race::tyre_change_gauge`, `race::draw_tyre_change`,
//! `drive::tyre_screen` and its parts): the timing's constants, the screen's layouts and the work
//! the screen leaves for the next frame.

use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// The tyre change's constants, layouts and work.
#[cfg_attr(test, repr(C))]
#[derive(rc2k_derive::Cells)]
pub struct TyreChange {
    /// (150.0): added to the press time (ms, before the scaling to hundredths)
    /// (`race::tyre_change_gauge::press_time`).
    pub time_base: PlainFloat,
    /// (150.0): the press time's logarithm is multiplied by it.
    pub time_scale: PlainFloat,
    /// (1 / 65536): a gauge step in turns (the press time's square root takes the gauge's shortfall
    /// times it; the wheel's quad turns the device's gauge times it times 2 pi).
    pub gauge_unit: PlainFloat,
    /// (log2 10 = 3.3219): the press time's logarithm is taken in tens and brought back by it.
    pub log_two_ten: PlainFloat,
    /// The panel's rectangle on the screen (x, y, w, h), copied from the layout in use
    /// (`drive::tyre_screen`).
    pub panel: [PlainDword; 4],
    /// The wheel's rectangle (x, y, w, h): its place (the panel's middle less the wheel's half,
    /// worked out by `drive::tyre_screen`) and its size (0xDE, 0xDE).
    pub wheel: [PlainDword; 4],
    /// The turning wheel's rectangle (x, y, w, h; 109.5, 109.5, 11, 109),
    /// `drive::tyre_turned_quad`'s `d`.
    pub turning_rect: [PlainFloat; 4],
    /// The panel's texture rectangle.
    pub panel_texture: [PlainDword; 4],
    /// The wheel's quad's texture rectangle.
    pub wheel_texture: [PlainDword; 4],
    /// The turning part's texture rectangle.
    pub turning_texture: [PlainDword; 4],
    /// Five layouts of 0x20 bytes: the panel's rectangle then the fixed parts' rectangle
    /// (`rc2k_core::drive::tyre_screen::layout`).
    pub layouts: [PlainDword; 40],
    /// The fixed parts' rectangle of the layout in use (`drive::tyre_screen_parts`).
    pub parts: [PlainDword; 4],
    /// The texture the screen is drawn into (`race::draw_tyre_change`).
    pub texture: PlainDword,
    /// Its copy on the device (`d3d::copy_to_device_texture`).
    pub device_texture: PlainDword,
    /// 1 once the texture's fixed texts have been drawn.
    pub drawn: PlainDword,
    /// The player (car) whose change it is.
    pub player: PlainDword,
    /// The sine of the wheel's turn (`drive::tyre_turned_quad`).
    pub turn_sin: PlainFloat,
    /// Its cosine.
    pub turn_cos: PlainFloat,
    /// The wheel rectangle's h times the sine.
    pub half_h_sin: PlainFloat,
    /// Minus its h times the cosine.
    pub neg_half_h_cos: PlainFloat,
    /// Its w times the cosine.
    pub half_w_cos: PlainFloat,
    /// Its w times the sine.
    pub half_w_sin: PlainFloat,
    /// The record holder's name (30 bytes used).
    pub record_name: PlainBytes<32>,
    /// The record time (ms / 10); starts at 21000.
    pub record: PlainDword,
    /// The fill's palette (`gfx::fill_rect`).
    pub palette: PlainBytes<4>,
}

impl TyreChange {
    pub const fn new() -> Self {
        TyreChange {
            time_base: PlainFloat::new(150.0),
            time_scale: PlainFloat::new(150.0),
            gauge_unit: PlainFloat::new(1.5258789e-5),
            log_two_ten: PlainFloat::new(std::f32::consts::LOG2_10),
            panel: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x320),
                PlainDword::new(0x258),
            ],
            wheel: [
                PlainDword::new(0xa),
                PlainDword::new(0xa),
                PlainDword::new(0xde),
                PlainDword::new(0xde),
            ],
            turning_rect: [
                PlainFloat::new(109.5),
                PlainFloat::new(109.5),
                PlainFloat::new(11.0),
                PlainFloat::new(109.0),
            ],
            panel_texture: [
                PlainDword::new(0x74),
                PlainDword::new(0x63),
                PlainDword::new(0x8),
                PlainDword::new(0x8),
            ],
            wheel_texture: [
                PlainDword::new(0x2),
                PlainDword::new(0x2),
                PlainDword::new(0x6f),
                PlainDword::new(0x6f),
            ],
            turning_texture: [
                PlainDword::new(0x73),
                PlainDword::new(0x2),
                PlainDword::new(0xb),
                PlainDword::new(0x36),
            ],
            layouts: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x320),
                PlainDword::new(0x258),
                PlainDword::new(0x114),
                PlainDword::new(0x64),
                PlainDword::new(0xd8),
                PlainDword::new(0x1f4),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x320),
                PlainDword::new(0x12c),
                PlainDword::new(0x1dc),
                PlainDword::new(0x4b),
                PlainDword::new(0x1a0),
                PlainDword::new(0xaf),
                PlainDword::new(0x0),
                PlainDword::new(0x12c),
                PlainDword::new(0x320),
                PlainDword::new(0x12c),
                PlainDword::new(0x1dc),
                PlainDword::new(0x177),
                PlainDword::new(0x1a0),
                PlainDword::new(0x1db),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x190),
                PlainDword::new(0x258),
                PlainDword::new(0x4c),
                PlainDword::new(0x64),
                PlainDword::new(0x10),
                PlainDword::new(0x1f4),
                PlainDword::new(0x190),
                PlainDword::new(0x0),
                PlainDword::new(0x190),
                PlainDword::new(0x258),
                PlainDword::new(0x1dc),
                PlainDword::new(0x64),
                PlainDword::new(0x1a0),
                PlainDword::new(0x1f4),
            ],
            parts: [const { PlainDword::new(0x0) }; 4],
            texture: PlainDword::new(0x0),
            device_texture: PlainDword::new(0x0),
            drawn: PlainDword::new(0x0),
            player: PlainDword::new(0x0),
            turn_sin: PlainFloat::new(0.0),
            turn_cos: PlainFloat::new(0.0),
            half_h_sin: PlainFloat::new(0.0),
            neg_half_h_cos: PlainFloat::new(0.0),
            half_w_cos: PlainFloat::new(0.0),
            half_w_sin: PlainFloat::new(0.0),
            record_name: PlainBytes::new([0; 32]),
            record: PlainDword::new(0x5208),
            palette: PlainBytes::new([0; 4]),
        }
    }
}
