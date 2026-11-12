//! The marks' frame state: the sprites' and
//! profiles' work places (`DrawWork`), the view's heading, step and ribbon work (`ViewVars`),
//! the frame's turned points, view and quads (`DrawFrame`), and the lens flare's tables
//! (`FlareParts`, `FlareScales`).

use crate::session::{PlainDword, PlainFloat};

/// The marks' sprites' and profiles' work places (IDA 0x8EB871 .. 0x8EB971).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawWork {
    /// 0x8EB871: `flt_8EB871` ..: the profile tables' running values (`effects::make_profiles`; as bits).
    pub profile_work: [PlainFloat; 11],
    /// 0x8EB8B5: `dword_8EB8B5`, `dword_8EB8B9`: half a sprite's size across and along (whole numbers; `effects::scale_corners` leaves a corner's two sides here).
    pub half: [PlainFloat; 2],
    /// 0x8EB8BD: `dword_8EB8BD` .. `dword_8EB8C9`: a sprite's half sides turned (whole numbers at first, floats once worked on).
    pub sprite_half: [PlainDword; 4],
    /// 0x8EB8CD: `flt_8EB8CD`, `flt_8EB8D1`: the two ends' scales of a ribbon piece (`effects::draw_ribbon`).
    pub ends_scale: [PlainFloat; 2],
    /// 0x8EB8D5: `flt_8EB8D5` ..: the place turned into the view's frame (x, y, depth).
    pub turned: [PlainFloat; 3],
    /// 0x8EB8E1: `dword_8EB8E1` ..: a ribbon piece's second place in the view's frame (x, y, depth).
    pub second: [PlainFloat; 3],
    /// 0x8EB941: `flt_8EB941`, `flt_8EB945`: a flare part's sides (`effects::draw_flare_part`).
    pub sides: [PlainFloat; 2],
    /// 0x8EB951: `flt_8EB951` ..: the view's step since the last frame (`effects::view_motion`).
    pub step: [PlainFloat; 3],
    /// 0x8EB961: `dword_8EB961`: a depth at or below it (compared as a signed whole number of the float's bits) is not drawn.
    pub near: PlainFloat,
    /// 0x8EB969: `flt_8EB969`: 1 / 65536, what a cell's words are multiplied by to make texture places.
    pub uv_k: PlainFloat,
    /// 0x8EB96D: `flt_8EB96D`: 1 over the view's farthest depth (`effects::begin_quads`), what depths are multiplied by.
    pub z_k: PlainFloat,
}

impl DrawWork {
    pub const fn new() -> Self {
        Self {
            profile_work: [const { PlainFloat::new(0.0) }; 11],
            half: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            sprite_half: [
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
                PlainDword::new(0x0),
            ],
            ends_scale: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            turned: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            second: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            sides: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            step: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            near: PlainFloat::new(65536.0),
            uv_k: PlainFloat::new(0.000015258789),
            z_k: PlainFloat::new(0.0),
        }
    }
}

impl DrawWork {
    /// What a sprite's square is projected by: the view's scale and centre (the front end's
    /// view window) and the depth scale.
    pub fn view_k(&self) -> [f32; 4] {
        let window = &crate::scene::polygon_work::front_end().window;
        [
            window.view_scale.get(),
            window.view_centre[0].get(),
            window.view_centre[1].get(),
            self.z_k.get(),
        ]
    }
}

/// The marks' view heading, step and ribbon work (IDA 0x8F3975 .. 0x8F39CD).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewVars {
    /// 0x8F3975: `g_marksViewHeading`: the view's heading (`effects::view_heading`).
    pub heading: PlainFloat,
    /// 0x8F3979: `dword_8F3979`: the frame's step, of 8 (`effects::draw_frame`).
    pub step: PlainDword,
    /// 0x8F397D: `flt_8F397D`: the view's third row dotted with its step (`effects::view_motion`).
    pub facing: PlainFloat,
    /// 0x8F3981: `dword_8F3981`: 0 at each frame's start (`effects::draw_frame`); `effects::draw_flare_part` adds one for each part it draws.
    pub cleared: PlainDword,
    /// 0x8F3985: `flt_8F3985`, `flt_8F3989`: a ribbon piece's way turned a quarter (`effects::draw_ribbon`).
    pub ribbon_way: [PlainFloat; 2],
    /// 0x8F3991: `flt_8F3991`, `flt_8F3995`: the ribbon's screen-space half-width offset (x, y) at its first end: the way turned a quarter times the half width times the first end's perspective scale; the quad's corners 0 and 1 are the first end's point less / plus it (`effects::draw_ribbon`).
    pub first_end_half_width: [PlainFloat; 2],
    /// 0x8F399D: `flt_8F399D`, `flt_8F39A1`: the same half-width offset at the second end (its own perspective scale); corners 2 and 3 are the second end's point plus / less it.
    pub second_end_half_width: [PlainFloat; 2],
    /// 0x8F39A9: `flt_8F39A9` ..: three points turned by the view matrix (`effects::draw_frame`; x, y, z each).
    pub turned_points: [[PlainFloat; 3]; 3],
}

impl ViewVars {
    pub const fn new() -> Self {
        Self {
            heading: PlainFloat::new(0.0),
            step: PlainDword::new(0x0),
            facing: PlainFloat::new(0.0),
            cleared: PlainDword::new(0x0),
            ribbon_way: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            first_end_half_width: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            second_end_half_width: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            turned_points: [const {
                [
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                    PlainFloat::new(0.0),
                ]
            }; 3],
        }
    }
}

/// The marks' frame: profile mark and steps, turned points, view, quads and flare (IDA 0x91FF3E .. 0x91FFE2).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawFrame {
    /// 0x91FF3E: `g_marksProfilesDone`: 0, or where `effects::make_profiles` goes on.
    pub profiles_done: PlainDword,
    /// 0x91FF42: `flt_91FF42`, `flt_91FF46`: 256.0, the ramps' and the curves' steps (`effects::make_profiles`).
    pub profile_steps: [PlainFloat; 2],
    /// 0x91FF4A: `flt_91FF4A` ..: three points the frame turns by the view matrix (x, y, z each; `effects::draw_frame`).
    pub points: [[PlainFloat; 3]; 3],
    /// 0x91FF6E: `dword_91FF6E`: `dword_621A80` as of the last frame.
    pub last_source: PlainDword,
    /// 0x91FF7E: `flt_91FF7E` ..: the view's forward level part (`effects::view_heading`).
    pub level: [PlainFloat; 3],
    /// 0x91FF8A: `flt_91FF8A` ..: the direction the heading is taken against (`effects::view_heading`).
    pub other: [PlainFloat; 3],
    /// 0x91FF9A: `flt_91FF9A`, `flt_91FF9E`: two time marks kept from `flt_765124` / `flt_765118` (`effects::draw_frame`).
    pub times: [PlainFloat; 2],
    /// 0x91FFA2: `dword_91FFA2`: the draw amount the debug page compares with `cleared` (the original sets it to 0 at each frame's start, `sub_4C4084`, and counts nothing into it; `effects::draw_frame` does not touch it).
    pub drawn: PlainDword,
    /// 0x91FFA6: `dword_91FFA6`: the quads' vertices' rhw, 256 (`effects::begin_quads`).
    pub rhw: PlainFloat,
    /// 0x91FFAA: `dword_91FFAA` .. `dword_91FFB6`: the quads' texture places (u a, u c, v b, v d).
    pub places: [PlainFloat; 4],
    /// 0x91FFBA: `dword_91FFBA`: 0, set by `effects::begin_quads`.
    pub cleared: PlainDword,
    /// 0x91FFBE: `flt_91FFBE` ..: the view's place at the last frame (`effects::view_motion`).
    pub last: [PlainFloat; 3],
    /// 0x91FFD6: `flt_91FFD6`: the lens flare's length (`effects::draw_lens_flare`).
    pub flare_length: PlainFloat,
    /// 0x91FFDA: `flt_91FFDA`, `flt_91FFDE`: the lens flare's way.
    pub flare_way: [PlainFloat; 2],
}

impl DrawFrame {
    pub const fn new() -> Self {
        Self {
            profiles_done: PlainDword::new(0x0),
            profile_steps: [PlainFloat::new(256.0), PlainFloat::new(256.0)],
            points: [
                [
                    PlainFloat::new(-98000.0),
                    PlainFloat::new(-45875.2),
                    PlainFloat::new(-51036.0),
                ],
                [
                    PlainFloat::new(-98000.0),
                    PlainFloat::new(-45875.2),
                    PlainFloat::new(51036.0),
                ],
                [
                    PlainFloat::new(-98000.0),
                    PlainFloat::new(-45875.2),
                    PlainFloat::new(0.0),
                ],
            ],
            last_source: PlainDword::new(0x0),
            level: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            other: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(1.0),
            ],
            times: [PlainFloat::new(100.0), PlainFloat::new(100.0)],
            drawn: PlainDword::new(0x0),
            rhw: PlainFloat::new(256.0),
            places: [
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.001953125),
                PlainFloat::new(0.46445313),
                PlainFloat::new(0.46445313),
            ],
            cleared: PlainDword::new(0x0),
            last: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            flare_length: PlainFloat::new(0.0),
            flare_way: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
        }
    }
}

/// The lens flare's unit way and its parts' cells and sizes (IDA 0x92051E .. 0x9205F6).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FlareParts {
    /// 0x92051E: `flt_92051E` .. `flt_920526`: the lens flare's unit way (`effects::draw_lens_flare`).
    pub unit: [PlainFloat; 3],
    /// 0x92052A: `dword_92052A`: the 0x11 parts' corner cells.
    pub corner: [PlainDword; 17],
    /// 0x92056E: `dword_92056E`: 0x11 more (the port does not read them).
    pub unread: [PlainDword; 17],
    /// 0x9205B2: `dword_9205B2`: the 0x11 parts' sizes.
    pub size: [PlainDword; 17],
}

impl FlareParts {
    pub const fn new() -> Self {
        Self {
            unit: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            corner: [
                PlainDword::new(0x40),
                PlainDword::new(0x41),
                PlainDword::new(0x42),
                PlainDword::new(0x45),
                PlainDword::new(0x43),
                PlainDword::new(0x43),
                PlainDword::new(0x43),
                PlainDword::new(0x44),
                PlainDword::new(0x45),
                PlainDword::new(0x45),
                PlainDword::new(0x46),
                PlainDword::new(0x46),
                PlainDword::new(0x46),
                PlainDword::new(0x47),
                PlainDword::new(0x45),
                PlainDword::new(0x48),
                PlainDword::new(0x49),
            ],
            unread: [
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xF0FFF),
                PlainDword::new(0xFFFFFF),
                PlainDword::new(0xFFFFFF),
            ],
            size: [
                PlainDword::new(0x85),
                PlainDword::new(0xBA),
                PlainDword::new(0x31),
                PlainDword::new(0x50),
                PlainDword::new(0x16),
                PlainDword::new(0x47),
                PlainDword::new(0x26),
                PlainDword::new(0x18),
                PlainDword::new(0x6),
                PlainDword::new(0x8),
                PlainDword::new(0x30),
                PlainDword::new(0x57),
                PlainDword::new(0x16),
                PlainDword::new(0x24),
                PlainDword::new(0x11),
                PlainDword::new(0x7E),
                PlainDword::new(0xFB),
            ],
        }
    }
}

/// The lens flare's constants (IDA 0x9206D2 .. 0x9206E6).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FlareScales {
    /// 0x9206D2: `flt_9206D2`, `flt_9206D6`: the lens flare's fade constants (`effects::draw_lens_flare`).
    pub fade: [PlainFloat; 2],
    /// 0x9206DA: `flt_9206DA`: the parts' size constant.
    pub size_k: PlainFloat,
    /// 0x9206DE: `flt_9206DE`, `flt_9206E2`: a flare part's scales (`effects::draw_flare_part`).
    pub scale: [PlainFloat; 2],
}

impl FlareScales {
    pub const fn new() -> Self {
        Self {
            fade: [PlainFloat::new(0.5), PlainFloat::new(255.0)],
            size_k: PlainFloat::new(1.5),
            scale: [PlainFloat::new(0.5), PlainFloat::new(0.5)],
        }
    }
}
