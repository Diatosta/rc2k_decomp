//! The roadside tyre change's variables (`race::tyre_change_gauge`, `race::draw_tyre_change`,
//! `drive::tyre_screen` and its parts), out of RAL.EXE's image (phase 5), typed (phase 7): the
//! timing's constants, the screen's layouts and the work the screen leaves for the next frame.
//!
//! A `#[repr(C)]` struct of atomic cells with the image's layout, first values the image's
//! (`tests`); `moved::MOVED` points the image's addresses at it (`home`).

use crate::session::{Bytes, Dword, Float};

/// The tyre change's constants, layouts and work (IDA 0x5BEAC0 .. 0x5BEC30, the image's layout).
#[repr(C, align(16))]
pub struct TyreChange {
    /// 0x5BEAC0: `flt_5BEAC0` (150.0): added to the press time (ms, before the scaling to hundredths) (`race::tyre_change_gauge::press_time`).
    pub time_base: Float,
    /// 0x5BEAC4: `flt_5BEAC4` (150.0): the press time's logarithm is multiplied by it.
    pub time_scale: Float,
    /// 0x5BEAC8: `flt_5BEAC8` (1 / 65536): a gauge step in turns (the press time's square root takes the gauge's shortfall times it; the wheel's quad turns the device's gauge times it times 2 pi).
    pub gauge_unit: Float,
    /// 0x5BEACC: `flt_5BEACC` (log2 10 = 3.3219): the press time's logarithm is taken in tens and brought back by it.
    pub log_two_ten: Float,
    /// 0x5BEAD0: `dword_5BEAD0` ..: the panel's rectangle on the screen (x, y, w, h), copied from the layout in use (`drive::tyre_screen`).
    pub panel: [Dword; 4],
    /// 0x5BEAE0: `dword_5BEAE0` ..: the wheel's rectangle (x, y, w, h): its place (the panel's middle less the wheel's half, worked out by `drive::tyre_screen`) and its size (0xDE, 0xDE).
    pub wheel: [Dword; 4],
    /// 0x5BEAF0: `unk_5BEAF0`: the turning wheel's rectangle (x, y, w, h; 109.5, 109.5, 11, 109), `drive::tyre_turned_quad`'s `d`.
    pub turning_rect: [Float; 4],
    /// 0x5BEB00: `unk_5BEB00`: the panel's texture rectangle.
    pub panel_texture: [Dword; 4],
    /// 0x5BEB10: `unk_5BEB10`: the wheel's quad's texture rectangle.
    pub wheel_texture: [Dword; 4],
    /// 0x5BEB20: `unk_5BEB20`: the turning part's texture rectangle.
    pub turning_texture: [Dword; 4],
    /// 0x5BEB30: `unk_5BEB30` .. `unk_5BEBB0`: five layouts of 0x20 bytes: the panel's rectangle then the fixed parts' rectangle (`rc2k_core::drive::tyre_screen::layout`).
    pub layouts: [Dword; 40],
    /// 0x5BEBD0: `dword_5BEBD0` ..: the fixed parts' rectangle of the layout in use (`drive::tyre_screen_parts`).
    pub parts: [Dword; 4],
    /// 0x5BEBE0: `g_tyreChangeTexture`: the texture the screen is drawn into (`race::draw_tyre_change`).
    pub texture: Dword,
    /// 0x5BEBE4: `g_tyreChangeDeviceTexture`: its copy on the device (`d3d::copy_to_device_texture`).
    pub device_texture: Dword,
    /// 0x5BEBE8: `g_tyreChangeDrawn`: 1 once the texture's fixed texts have been drawn.
    pub drawn: Dword,
    /// 0x5BEBEC: `g_tyreChangePlayer`: the player (car) whose change it is.
    pub player: Dword,
    /// 0x5BEBF0: `flt_5BEBF0`: the sine of the wheel's turn (`drive::tyre_turned_quad`).
    pub turn_sin: Float,
    /// 0x5BEBF4: `flt_5BEBF4`: its cosine.
    pub turn_cos: Float,
    /// 0x5BEBF8: `flt_5BEBF8`: the wheel rectangle's h times the sine.
    pub half_h_sin: Float,
    /// 0x5BEBFC: `flt_5BEBFC`: minus its h times the cosine.
    pub neg_half_h_cos: Float,
    /// 0x5BEC00: `flt_5BEC00`: its w times the cosine.
    pub half_w_cos: Float,
    /// 0x5BEC04: `flt_5BEC04`: its w times the sine.
    pub half_w_sin: Float,
    /// 0x5BEC08: `g_tyreChangeRecordName`: the record holder's name (30 bytes used).
    pub record_name: Bytes<32>,
    /// 0x5BEC28: `g_tyreChangeRecord` (21000): the record time (ms / 10).
    pub record: Dword,
    /// 0x5BEC2C: `unk_5BEC2C`: the fill's palette (`gfx::fill_rect`).
    pub palette: Bytes<4>,
}

pub static TYRE_CHANGE: TyreChange = TyreChange {
    time_base: Float::from_bits(0x43160000),
    time_scale: Float::from_bits(0x43160000),
    gauge_unit: Float::from_bits(0x37800000),
    log_two_ten: Float::from_bits(0x40549a78),
    panel: [
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x320),
        Dword::new(0x258),
    ],
    wheel: [
        Dword::new(0xa),
        Dword::new(0xa),
        Dword::new(0xde),
        Dword::new(0xde),
    ],
    turning_rect: [
        Float::from_bits(0x42db0000),
        Float::from_bits(0x42db0000),
        Float::from_bits(0x41300000),
        Float::from_bits(0x42da0000),
    ],
    panel_texture: [
        Dword::new(0x74),
        Dword::new(0x63),
        Dword::new(0x8),
        Dword::new(0x8),
    ],
    wheel_texture: [
        Dword::new(0x2),
        Dword::new(0x2),
        Dword::new(0x6f),
        Dword::new(0x6f),
    ],
    turning_texture: [
        Dword::new(0x73),
        Dword::new(0x2),
        Dword::new(0xb),
        Dword::new(0x36),
    ],
    layouts: [
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x320),
        Dword::new(0x258),
        Dword::new(0x114),
        Dword::new(0x64),
        Dword::new(0xd8),
        Dword::new(0x1f4),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x320),
        Dword::new(0x12c),
        Dword::new(0x1dc),
        Dword::new(0x4b),
        Dword::new(0x1a0),
        Dword::new(0xaf),
        Dword::new(0x0),
        Dword::new(0x12c),
        Dword::new(0x320),
        Dword::new(0x12c),
        Dword::new(0x1dc),
        Dword::new(0x177),
        Dword::new(0x1a0),
        Dword::new(0x1db),
        Dword::new(0x0),
        Dword::new(0x0),
        Dword::new(0x190),
        Dword::new(0x258),
        Dword::new(0x4c),
        Dword::new(0x64),
        Dword::new(0x10),
        Dword::new(0x1f4),
        Dword::new(0x190),
        Dword::new(0x0),
        Dword::new(0x190),
        Dword::new(0x258),
        Dword::new(0x1dc),
        Dword::new(0x64),
        Dword::new(0x1a0),
        Dword::new(0x1f4),
    ],
    parts: [const { Dword::new(0x0) }; 4],
    texture: Dword::new(0x0),
    device_texture: Dword::new(0x0),
    drawn: Dword::new(0x0),
    player: Dword::new(0x0),
    turn_sin: Float::from_bits(0x0),
    turn_cos: Float::from_bits(0x0),
    half_h_sin: Float::from_bits(0x0),
    neg_half_h_cos: Float::from_bits(0x0),
    half_w_cos: Float::from_bits(0x0),
    half_w_sin: Float::from_bits(0x0),
    record_name: Bytes::new([0; 32]),
    record: Dword::new(0x5208),
    palette: Bytes::new([0; 4]),
};

impl TyreChange {
    pub const START: usize = 0x5BEAC0;
    pub const LEN: usize = 0x170;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const TYRE_CHANGE) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tyrechange_starts_as_the_games() {
        let size = size_of::<TyreChange>();
        assert!(TyreChange::LEN <= size && size <= (TyreChange::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(TyreChange::START, TyreChange::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(TyreChange::home()).cast_const(),
                TyreChange::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
