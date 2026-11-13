//! The windscreen drops' and windscreen flakes' variables:
//! `WindscreenDropState` (IDA 0x814710 .. 0x814770) and `WindscreenDropWork` (0x814772 .. 0x81518A; on an
//! address 2 more than a multiple of 4 in the image).

use crate::session::{PlainDword, PlainFloat};

/// The windscreen drops' and windscreen flakes' frame variables (`effects::windscreen_drops_*`, `effects::squares_*`, `effects::draw_plane`) (IDA 0x814710 .. 0x814770).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDropState {
    /// 0x814710: `dword_814710`: the live windscreen drop items counted in this frame (`effects::windscreen_drops_frame`).
    pub live: PlainDword,
    /// 0x814714: `dword_814714`: the clock the items' steps are counted from (`effects::windscreen_drops_step`, `effects::windscreen_flakes_step`).
    pub clock: PlainDword,
    /// 0x814718: `flt_814718`: the place's way dotted with the third axis, negative when it faces the third axis away (`effects::windscreen_drops_top`, `effects::windscreen_flakes_frame`, `effects::draw_plane`).
    pub facing: PlainFloat,
    /// 0x81471C: `dword_81471C`, `dword_814720`: the cell (x, z) of the item being looked at (`effects::expire_windscreen_drops`, `effects::expire_windscreen_flakes`).
    pub cell: [PlainDword; 2],
    /// 0x814724: `dword_814724`: the frames counted (`effects::windscreen_drops_spawn`).
    pub frames: PlainDword,
    /// 0x814728: `flt_814728`: the windscreen drops' rate (`effects::windscreen_drops_top`, `effects::windscreen_drops_spawn`).
    pub rate: PlainFloat,
    /// 0x81472C: `flt_81472C`: the spawn rate worked out each frame (`effects::windscreen_drops_spawn`).
    pub spawn_rate: PlainFloat,
    /// 0x814734: `flt_814734`: the sprite's depth (`effects::windscreen_drops_frame`).
    pub depth: PlainFloat,
}

impl WindscreenDropState {
    pub const fn new() -> Self {
        WindscreenDropState {
            live: PlainDword::new(0x0),
            clock: PlainDword::new(0x0),
            facing: PlainFloat::new(0.0),
            cell: [PlainDword::new(0x0), PlainDword::new(0x0)],
            frames: PlainDword::new(0x0),
            rate: PlainFloat::new(0.0),
            spawn_rate: PlainFloat::new(1.125),
            depth: PlainFloat::new(0.0),
        }
    }
}

/// The windscreen drops' work (`effects::windscreen_drops_*`, `effects::shapes`, `effects::spawn_windscreen_drop`): the drift, the curve, the radii, the axes, the sprite's corners and the windscreen drop points (IDA 0x814772 .. 0x81518A).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDropWork {
    /// 0x814772: `flt_814772`, `flt_814776`: the drift at the item's cell (`effects::windscreen_drops_frame`).
    pub drift: [PlainFloat; 2],
    /// 0x8147C2: `g_marksWindscreenDropCurve`: a quarter-circle curve of 0x100 floats (`effects::shapes`).
    pub curve: [PlainFloat; 256],
    /// 0x814BC2: `flt_814BC2` ..: the circles' radii (eight kinds used).
    pub radii: [PlainFloat; 16],
    /// 0x814C02: `flt_814C02`: 1 over `flt_6241C8` (`effects::windscreen_drops_setup`).
    pub inverse: PlainFloat,
    /// 0x814C06: `flt_814C06`, `flt_814C0A`: the axes' lengths' shares (`effects::windscreen_drops_top`).
    pub shares: [PlainFloat; 2],
    /// 0x814C0E: `flt_814C0E` ..: the view's place (the settings' +0x2720 ..).
    pub place: [PlainFloat; 3],
    /// 0x814C1A: `flt_814C1A` ..: the three axes (rows of three).
    pub axes: [PlainFloat; 9],
    /// 0x814C6E: `unk_814C6E`, `unk_814C7A`, `unk_814C86`, `unk_814C92`: the two axes scaled (`effects::windscreen_drops_setup`).
    pub setup_axes: [PlainFloat; 12],
    /// 0x814C9E: `flt_814C9E`, `flt_814CA2`: the sprite's centre on the screen (`effects::windscreen_drops_frame`).
    pub screen: [PlainFloat; 2],
    /// 0x814CAA: `flt_814CAA`, `flt_814CAE`, `flt_814CB2`: the item's place turned, its depth last.
    pub cell: [PlainFloat; 3],
    /// 0x814CB6: `flt_814CB6` ..: the six corners (x, y, z).
    pub corners: [PlainFloat; 18],
    /// 0x814CFE: `flt_814CFE` ..: the 0x30 points scaled and turned onto the axes (`effects::windscreen_drops_top`).
    pub scaled: [PlainFloat; 144],
    /// 0x814F3E: `g_marksWindscreenDropPoints`: 0x30 points of 12 bytes (eight circles of six).
    pub points: [PlainFloat; 144],
    /// 0x81517E: `dword_81517E`: an offset into the items (`effects::spawn_windscreen_drop`).
    pub cursor: PlainDword,
    /// 0x815182: `dword_815182`, `dword_815186`: the item's place as whole numbers (x, z).
    pub whole: [PlainDword; 2],
}

impl WindscreenDropWork {
    pub const fn new() -> Self {
        WindscreenDropWork {
            drift: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            curve: [const { PlainFloat::new(0.0) }; 256],
            radii: [
                PlainFloat::new(7.0),
                PlainFloat::new(6.0),
                PlainFloat::new(5.0),
                PlainFloat::new(4.0),
                PlainFloat::new(3.0),
                PlainFloat::new(2.0),
                PlainFloat::new(4.0),
                PlainFloat::new(5.0),
                PlainFloat::new(2.0),
                PlainFloat::new(1.8),
                PlainFloat::new(1.6),
                PlainFloat::new(1.4),
                PlainFloat::new(1.2),
                PlainFloat::new(1.0),
                PlainFloat::new(1.2),
                PlainFloat::new(1.4),
            ],
            inverse: PlainFloat::new(0.0),
            shares: [PlainFloat::new(57.6), PlainFloat::new(57.6)],
            place: [
                PlainFloat::new(-58982.4),
                PlainFloat::new(-29491.2),
                PlainFloat::new(65536.0),
            ],
            axes: [
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            setup_axes: [
                PlainFloat::new(1664.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(-1188.0),
                PlainFloat::new(1188.0),
                PlainFloat::new(1664.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(-1188.0),
                PlainFloat::new(1188.0),
            ],
            screen: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            cell: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            corners: [const { PlainFloat::new(0.0) }; 18],
            scaled: [const { PlainFloat::new(0.0) }; 144],
            points: [const { PlainFloat::new(0.0) }; 144],
            cursor: PlainDword::new(0x0),
            whole: [PlainDword::new(0x0), PlainDword::new(0x0)],
        }
    }
}
