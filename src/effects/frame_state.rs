//! The marks' frame state: the sprites' and
//! profiles' work places (`DrawWork`), the view's heading, step and ribbon work (`ViewVars`),
//! the frame's turned points, view and quads (`DrawFrame`), and the lens flare's tables
//! (`FlareParts`, `FlareScales`).

#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{PlainBytes, PlainDword, PlainFloat};

/// The marks' sprites' and profiles' work places (IDA 0x8EB871 .. 0x8EB971).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawWork {
    /// 0x8EB871: `flt_8EB871` ..: the profile tables' running values (`effects::make_profiles`; as bits).
    pub profile_work: [PlainFloat; 11],
    /// 0x8EB89D: `dword_8EB89D`: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8eb89d: PlainBytes<24>,
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
    /// 0x8EB8ED: Not named: 84 bytes (a gap, or unreferenced data).
    pub _gap_8eb8ed: PlainBytes<84>,
    /// 0x8EB941: `flt_8EB941`, `flt_8EB945`: a flare part's sides (`effects::draw_flare_part`).
    pub sides: [PlainFloat; 2],
    /// 0x8EB949: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_8eb949: PlainBytes<8>,
    /// 0x8EB951: `flt_8EB951` ..: the view's step since the last frame (`effects::view_motion`).
    pub step: [PlainFloat; 3],
    /// 0x8EB95D: `dword_8EB95D`: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8eb95d: PlainBytes<4>,
    /// 0x8EB961: `dword_8EB961`: a depth at or below it (compared as a signed whole number of the float's bits) is not drawn.
    pub near: PlainFloat,
    /// 0x8EB965: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8eb965: PlainBytes<4>,
    /// 0x8EB969: `flt_8EB969`: 1 / 65536, what a cell's words are multiplied by to make texture places.
    pub uv_k: PlainFloat,
    /// 0x8EB96D: `flt_8EB96D`: 1 over the view's farthest depth (`effects::begin_quads`), what depths are multiplied by.
    pub z_k: PlainFloat,
}

impl DrawWork {
    pub const fn new() -> Self {
        Self {
            profile_work: [const { PlainFloat::new(0.0) }; 11],
            _gap_8eb89d: PlainBytes::new([0; 24]),
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
            _gap_8eb8ed: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
            ]),
            sides: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_8eb949: PlainBytes::new([0; 8]),
            step: [
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
                PlainFloat::new(0.0),
            ],
            _gap_8eb95d: PlainBytes::new([0; 4]),
            near: PlainFloat::new(65536.0),
            _gap_8eb965: PlainBytes::new([0x00, 0x00, 0x80, 0x3B]),
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

impl DrawWork {
    pub const START: usize = 0x8EB871;
    pub const LEN: usize = 0x100;
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
    /// 0x8F398D: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8f398d: PlainBytes<4>,
    /// 0x8F3991: `flt_8F3991`, `flt_8F3995`: the ribbon's screen-space half-width offset (x, y) at its first end: the way turned a quarter times the half width times the first end's perspective scale; the quad's corners 0 and 1 are the first end's point less / plus it (`effects::draw_ribbon`).
    pub first_end_half_width: [PlainFloat; 2],
    /// 0x8F3999: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8f3999: PlainBytes<4>,
    /// 0x8F399D: `flt_8F399D`, `flt_8F39A1`: the same half-width offset at the second end (its own perspective scale); corners 2 and 3 are the second end's point plus / less it.
    pub second_end_half_width: [PlainFloat; 2],
    /// 0x8F39A5: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8f39a5: PlainBytes<4>,
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
            _gap_8f398d: PlainBytes::new([0; 4]),
            first_end_half_width: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_8f3999: PlainBytes::new([0; 4]),
            second_end_half_width: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
            _gap_8f39a5: PlainBytes::new([0; 4]),
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

impl ViewVars {
    pub const START: usize = 0x8F3975;
    pub const LEN: usize = 0x58;
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
    /// 0x91FF72: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_91ff72: PlainBytes<12>,
    /// 0x91FF7E: `flt_91FF7E` ..: the view's forward level part (`effects::view_heading`).
    pub level: [PlainFloat; 3],
    /// 0x91FF8A: `flt_91FF8A` ..: the direction the heading is taken against (`effects::view_heading`).
    pub other: [PlainFloat; 3],
    /// 0x91FF96: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_91ff96: PlainBytes<4>,
    /// 0x91FF9A: `flt_91FF9A`, `flt_91FF9E`: two time marks kept from `flt_765124` / `flt_765118` (`effects::draw_frame`).
    pub times: [PlainFloat; 2],
    /// 0x91FFA2: `dword_91FFA2`: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_91ffa2: PlainBytes<4>,
    /// 0x91FFA6: `dword_91FFA6`: the quads' vertices' rhw, 256 (`effects::begin_quads`).
    pub rhw: PlainFloat,
    /// 0x91FFAA: `dword_91FFAA` .. `dword_91FFB6`: the quads' texture places (u a, u c, v b, v d).
    pub places: [PlainFloat; 4],
    /// 0x91FFBA: `dword_91FFBA`: 0, set by `effects::begin_quads`.
    pub cleared: PlainDword,
    /// 0x91FFBE: `flt_91FFBE` ..: the view's place at the last frame (`effects::view_motion`).
    pub last: [PlainFloat; 3],
    /// 0x91FFCA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_91ffca: PlainBytes<12>,
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
            _gap_91ff72: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            ]),
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
            _gap_91ff96: PlainBytes::new([0; 4]),
            times: [PlainFloat::new(100.0), PlainFloat::new(100.0)],
            _gap_91ffa2: PlainBytes::new([0; 4]),
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
            _gap_91ffca: PlainBytes::new([
                0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0xA0, 0x41, 0x1A, 0x00, 0x48, 0x42,
            ]),
            flare_length: PlainFloat::new(0.0),
            flare_way: [PlainFloat::new(0.0), PlainFloat::new(0.0)],
        }
    }
}

impl DrawFrame {
    pub const START: usize = 0x91FF3E;
    pub const LEN: usize = 0xA4;
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

impl FlareParts {
    pub const START: usize = 0x92051E;
    pub const LEN: usize = 0xD8;
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

impl FlareScales {
    pub const START: usize = 0x9206D2;
    pub const LEN: usize = 0x14;
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn draw_work_starts_as_the_games() {
        // Other tests write the work: read it while none runs.
        let _lock = crate::test_support::test_lock();
        assert_eq!(size_of::<DrawWork>(), DrawWork::LEN);
        let image =
            crate::test_support::first_values::bytes("effects.owner.EFFECTS.frame.draw_work/256");
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const effects().frame.draw_work).cast::<u8>(),
                DrawWork::LEN,
            )
        };
        assert!(mine == image, "DRAW_WORK starts differently from the image");
        assert_eq!(offset_of!(DrawWork, half), 0x44);
        assert_eq!(offset_of!(DrawWork, sprite_half), 0x4c);
        assert_eq!(offset_of!(DrawWork, ends_scale), 0x5c);
        assert_eq!(offset_of!(DrawWork, turned), 0x64);
        assert_eq!(offset_of!(DrawWork, second), 0x70);
        assert_eq!(offset_of!(DrawWork, sides), 0xd0);
        assert_eq!(offset_of!(DrawWork, step), 0xe0);
        assert_eq!(offset_of!(DrawWork, near), 0xf0);
        assert_eq!(offset_of!(DrawWork, uv_k), 0xf8);
        assert_eq!(offset_of!(DrawWork, z_k), 0xfc);
    }

    #[test]
    fn view_vars_starts_as_the_games() {
        assert_eq!(size_of::<ViewVars>(), ViewVars::LEN);
        let image =
            crate::test_support::first_values::bytes("effects.owner.EFFECTS.frame.view_vars/88");
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const effects().frame.view_vars).cast::<u8>(),
                ViewVars::LEN,
            )
        };
        assert!(mine == image, "VIEW_VARS starts differently from the image");
        assert_eq!(offset_of!(ViewVars, step), 0x4);
        assert_eq!(offset_of!(ViewVars, facing), 0x8);
        assert_eq!(offset_of!(ViewVars, cleared), 0xc);
        assert_eq!(offset_of!(ViewVars, ribbon_way), 0x10);
        assert_eq!(offset_of!(ViewVars, first_end_half_width), 0x1c);
        assert_eq!(offset_of!(ViewVars, second_end_half_width), 0x28);
        assert_eq!(offset_of!(ViewVars, turned_points), 0x34);
    }

    #[test]
    fn marks_frame_starts_as_the_games() {
        assert_eq!(size_of::<DrawFrame>(), DrawFrame::LEN);
        let image =
            crate::test_support::first_values::bytes("effects.owner.EFFECTS.frame.draw_frame/164");
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const effects().frame.draw_frame).cast::<u8>(),
                DrawFrame::LEN,
            )
        };
        assert!(
            mine == image,
            "DRAW_FRAME starts differently from the image"
        );
        assert_eq!(offset_of!(DrawFrame, profile_steps), 0x4);
        assert_eq!(offset_of!(DrawFrame, points), 0xc);
        assert_eq!(offset_of!(DrawFrame, last_source), 0x30);
        assert_eq!(offset_of!(DrawFrame, level), 0x40);
        assert_eq!(offset_of!(DrawFrame, other), 0x4c);
        assert_eq!(offset_of!(DrawFrame, times), 0x5c);
        assert_eq!(offset_of!(DrawFrame, rhw), 0x68);
        assert_eq!(offset_of!(DrawFrame, places), 0x6c);
        assert_eq!(offset_of!(DrawFrame, cleared), 0x7c);
        assert_eq!(offset_of!(DrawFrame, last), 0x80);
        assert_eq!(offset_of!(DrawFrame, flare_length), 0x98);
        assert_eq!(offset_of!(DrawFrame, flare_way), 0x9c);
    }

    #[test]
    fn flare_parts_starts_as_the_games() {
        assert_eq!(size_of::<FlareParts>(), FlareParts::LEN);
        let image =
            crate::test_support::first_values::bytes("effects.owner.EFFECTS.frame.flare_parts/216");
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const effects().frame.flare_parts).cast::<u8>(),
                FlareParts::LEN,
            )
        };
        assert!(
            mine == image,
            "FLARE_PARTS starts differently from the image"
        );
        assert_eq!(offset_of!(FlareParts, corner), 0xc);
        assert_eq!(offset_of!(FlareParts, unread), 0x50);
        assert_eq!(offset_of!(FlareParts, size), 0x94);
    }

    #[test]
    fn flare_scales_starts_as_the_games() {
        assert_eq!(size_of::<FlareScales>(), FlareScales::LEN);
        let image =
            crate::test_support::first_values::bytes("effects.owner.EFFECTS.frame.flare_scales/20");
        let mine = unsafe {
            std::slice::from_raw_parts(
                (&raw const effects().frame.flare_scales).cast::<u8>(),
                FlareScales::LEN,
            )
        };
        assert!(
            mine == image,
            "FLARE_SCALES starts differently from the image"
        );
        assert_eq!(offset_of!(FlareScales, size_k), 0x8);
        assert_eq!(offset_of!(FlareScales, scale), 0xc);
    }
}
