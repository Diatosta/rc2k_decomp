//! The roadside tyre change's variables (`race::tyre_change_gauge`, `race::draw_tyre_change`,
//! `drive::tyre_screen` and its parts): the timing's constants, the screen's layouts and the work
//! the screen leaves for the next frame.

/// The tyre change's constants, layouts and work.
#[cfg_attr(test, repr(C))]
pub struct TyreChange {
    /// (150.0): added to the press time (ms, before the scaling to hundredths)
    /// (`race::tyre_change_gauge::press_time`).
    pub time_base: f32,
    /// (150.0): the press time's logarithm is multiplied by it.
    pub time_scale: f32,
    /// (1 / 65536): a gauge step in turns (the press time's square root takes the gauge's shortfall
    /// times it; the wheel's quad turns the device's gauge times it times 2 pi).
    pub gauge_unit: f32,
    /// (log2 10 = 3.3219): the press time's logarithm is taken in tens and brought back by it.
    pub log_two_ten: f32,
    /// The panel's rectangle on the screen (x, y, w, h), copied from the layout in use
    /// (`drive::tyre_screen`).
    pub panel: [u32; 4],
    /// The wheel's rectangle (x, y, w, h): its place (the panel's middle less the wheel's half,
    /// worked out by `drive::tyre_screen`) and its size (0xDE, 0xDE).
    pub wheel: [u32; 4],
    /// The turning wheel's rectangle (x, y, w, h; 109.5, 109.5, 11, 109),
    /// `drive::tyre_turned_quad`'s `d`.
    pub turning_rect: [f32; 4],
    /// The panel's texture rectangle.
    pub panel_texture: [u32; 4],
    /// The wheel's quad's texture rectangle.
    pub wheel_texture: [u32; 4],
    /// The turning part's texture rectangle.
    pub turning_texture: [u32; 4],
    /// Five layouts of 0x20 bytes: the panel's rectangle then the fixed parts' rectangle
    /// (`rc2k_core::drive::tyre_screen::layout`).
    pub layouts: [u32; 40],
    /// The fixed parts' rectangle of the layout in use (`drive::tyre_screen_parts`).
    pub parts: [u32; 4],
    /// The texture the screen is drawn into (`race::draw_tyre_change`).
    pub texture: u32,
    /// Its copy on the device (`d3d::copy_to_device_texture`).
    pub device_texture: u32,
    /// 1 once the texture's fixed texts have been drawn.
    pub drawn: u32,
    /// The player (car) whose change it is.
    pub player: u32,
    /// The sine of the wheel's turn (`drive::tyre_turned_quad`).
    pub turn_sin: f32,
    /// Its cosine.
    pub turn_cos: f32,
    /// The wheel rectangle's h times the sine.
    pub half_h_sin: f32,
    /// Minus its h times the cosine.
    pub neg_half_h_cos: f32,
    /// Its w times the cosine.
    pub half_w_cos: f32,
    /// Its w times the sine.
    pub half_w_sin: f32,
    /// The record holder's name (30 bytes used).
    pub record_name: [u8; 32],
    /// The record time (ms / 10); starts at 21000.
    pub record: u32,
    /// The texture the title is drawn into (`race::draw_tyre_change`), and its copy on the device.
    pub title_texture: u32,
    pub title_device_texture: u32,
    /// The title's width in pixels as drawn (0 before it is).
    pub title_width: u32,
    /// The prompt's width in pixels as drawn on the title texture, for each half of the screen
    /// (0 when that half shows the time instead).
    pub prompt_width: [u32; 2],
}

impl TyreChange {
    pub const fn new() -> Self {
        TyreChange {
            time_base: 150.0,
            time_scale: 150.0,
            gauge_unit: 1.5258789e-5,
            log_two_ten: std::f32::consts::LOG2_10,
            panel: [0x0, 0x0, 0x320, 0x258],
            wheel: [0xa, 0xa, 0xde, 0xde],
            turning_rect: [109.5, 109.5, 11.0, 109.0],
            panel_texture: [0x74, 0x63, 0x8, 0x8],
            wheel_texture: [0x2, 0x2, 0x6f, 0x6f],
            turning_texture: [0x73, 0x2, 0xb, 0x36],
            layouts: [
                0x0, 0x0, 0x320, 0x258, 0x114, 0x64, 0xd8, 0x1f4, 0x0, 0x0, 0x320, 0x12c, 0x1dc,
                0x4b, 0x1a0, 0xaf, 0x0, 0x12c, 0x320, 0x12c, 0x1dc, 0x177, 0x1a0, 0x1db, 0x0, 0x0,
                0x190, 0x258, 0x4c, 0x64, 0x10, 0x1f4, 0x190, 0x0, 0x190, 0x258, 0x1dc, 0x64,
                0x1a0, 0x1f4,
            ],
            parts: [0; 4],
            texture: 0x0,
            device_texture: 0x0,
            drawn: 0x0,
            player: 0x0,
            turn_sin: 0.0,
            turn_cos: 0.0,
            half_h_sin: 0.0,
            neg_half_h_cos: 0.0,
            half_w_cos: 0.0,
            half_w_sin: 0.0,
            record_name: [0; 32],
            record: 0x5208,
            title_texture: 0x0,
            title_device_texture: 0x0,
            title_width: 0x0,
            prompt_width: [0x0, 0x0],
        }
    }
}
