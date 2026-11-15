//! The windscreen drops' and windscreen flakes' variables: `WindscreenDropState` and
//! `WindscreenDropWork`.

use crate::memory::session::{PlainDword, PlainFloat};

/// The windscreen drops' and windscreen flakes' frame variables (`effects::windscreen_drops_*`,
/// `effects::windscreen_flakes_*`, `effects::draw_plane`).
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDropState {
    /// The live windscreen drop items counted in this frame (`effects::windscreen_drops_frame`).
    pub live: PlainDword,
    /// The clock the items' steps are counted from (`effects::windscreen_drops_step`,
    /// `effects::windscreen_flakes_step`).
    pub clock: PlainDword,
    /// The place's way dotted with the third axis, negative when it faces the third axis away
    /// (`effects::windscreen_drops_top`, `effects::windscreen_flakes_frame`,
    /// `effects::draw_plane`).
    pub facing: PlainFloat,
    /// The cell (x, z) of the item being looked at (`effects::expire_windscreen_drops`,
    /// `effects::expire_windscreen_flakes`).
    pub cell: [PlainDword; 2],
    /// The frames counted (`effects::windscreen_drops_spawn`).
    pub frames: PlainDword,
    /// The windscreen drops' rate (`effects::windscreen_drops_top`,
    /// `effects::windscreen_drops_spawn`).
    pub rate: PlainFloat,
    /// The spawn rate worked out each frame (`effects::windscreen_drops_spawn`).
    pub spawn_rate: PlainFloat,
    /// The sprite's depth (`effects::windscreen_drops_frame`).
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

/// The windscreen drops' work (`effects::windscreen_drops_*`, `effects::shapes`,
/// `effects::spawn_windscreen_drop`): the drift, the curve, the radii, the axes, the sprite's
/// corners and the windscreen drop points.
#[derive(rc2k_derive::Cells)]
pub struct WindscreenDropWork {
    /// The drift at the item's cell (`effects::windscreen_drops_frame`).
    pub drift: [PlainFloat; 2],
    /// A quarter-circle curve of 0x100 floats (`effects::shapes`).
    pub curve: [PlainFloat; 256],
    /// The circles' radii (eight kinds used).
    pub radii: [PlainFloat; 16],
    /// 1 over the view's farthest depth (`effects::windscreen_drops_setup`).
    pub inverse: PlainFloat,
    /// The axes' lengths' shares (`effects::windscreen_drops_top`).
    pub shares: [PlainFloat; 2],
    /// The view's place (the settings' +0x2720 ..).
    pub place: [PlainFloat; 3],
    /// The three axes (rows of three).
    pub axes: [PlainFloat; 9],
    /// The two axes scaled (`effects::windscreen_drops_setup`).
    pub setup_axes: [PlainFloat; 12],
    /// The sprite's centre on the screen (`effects::windscreen_drops_frame`).
    pub screen: [PlainFloat; 2],
    /// The item's place turned, its depth last.
    pub cell: [PlainFloat; 3],
    /// The six corners (x, y, z).
    pub corners: [PlainFloat; 18],
    /// The 0x30 points scaled and turned onto the axes (`effects::windscreen_drops_top`).
    pub scaled: [PlainFloat; 144],
    /// 0x30 points of 12 bytes (eight circles of six).
    pub points: [PlainFloat; 144],
    /// An offset into the items (`effects::spawn_windscreen_drop`).
    pub cursor: PlainDword,
    /// The item's place as whole numbers (x, z).
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
