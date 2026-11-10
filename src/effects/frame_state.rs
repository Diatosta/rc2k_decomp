//! The marks' frame state, out of the marks' block (phase 7, typed state): the sprites' and
//! profiles' work places (`DrawWork`), the view's heading, step and ribbon work (`ViewVars`),
//! the frame's turned points, view and quads (`DrawFrame`), and the lens flare's tables
//! (`FlareParts`, `FlareScales`).

#[cfg(test)]
use crate::effects::owner::effects;
use crate::session::{Bytes, Dword, Float};

/// The marks' sprites' and profiles' work places (IDA 0x8EB871 .. 0x8EB971, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawWork {
    /// 0x8EB871: `flt_8EB871` ..: the profile tables' running values (`effects::make_profiles`; as bits).
    pub profile_work: [Float; 11],
    /// 0x8EB89D: `dword_8EB89D`: Not named: 24 bytes (a gap, or unreferenced data).
    pub _gap_8eb89d: Bytes<24>,
    /// 0x8EB8B5: `dword_8EB8B5`, `dword_8EB8B9`: half a sprite's size across and along (whole numbers; `effects::scale_corners` leaves a corner's two sides here).
    pub half: [Dword; 2],
    /// 0x8EB8BD: `dword_8EB8BD` .. `dword_8EB8C9`: a sprite's half sides turned (whole numbers at first, floats once worked on).
    pub sprite_half: [Dword; 4],
    /// 0x8EB8CD: `flt_8EB8CD`, `flt_8EB8D1`: the two ends' scales of a ribbon piece (`effects::draw_ribbon`).
    pub ends_scale: [Float; 2],
    /// 0x8EB8D5: `flt_8EB8D5` ..: the place turned into the view's frame (x, y, depth).
    pub turned: [Float; 3],
    /// 0x8EB8E1: `dword_8EB8E1` ..: a ribbon piece's second place in the view's frame (x, y, depth; floats' bits).
    pub second: [Dword; 3],
    /// 0x8EB8ED: Not named: 84 bytes (a gap, or unreferenced data).
    pub _gap_8eb8ed: Bytes<84>,
    /// 0x8EB941: `flt_8EB941`, `flt_8EB945`: a flare part's sides (`effects::draw_flare_part`).
    pub sides: [Float; 2],
    /// 0x8EB949: Not named: 8 bytes (a gap, or unreferenced data).
    pub _gap_8eb949: Bytes<8>,
    /// 0x8EB951: `flt_8EB951` ..: the view's step since the last frame (`effects::view_motion`).
    pub step: [Float; 3],
    /// 0x8EB95D: `dword_8EB95D`: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8eb95d: Bytes<4>,
    /// 0x8EB961: `dword_8EB961`: a depth at or below it (compared as a signed whole number of the float's bits) is not drawn.
    pub near: Dword,
    /// 0x8EB965: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8eb965: Bytes<4>,
    /// 0x8EB969: `flt_8EB969`: 1 / 65536, what a cell's words are multiplied by to make texture places.
    pub uv_k: Float,
    /// 0x8EB96D: `flt_8EB96D`: 1 over the view's farthest depth (`effects::begin_quads`), what depths are multiplied by.
    pub z_k: Float,
}

impl DrawWork {
    pub const fn new() -> Self {
        Self {
            profile_work: [const { Float::from_bits(0x0) }; 11],
            _gap_8eb89d: Bytes::new([0; 24]),
            half: [Dword::new(0x0), Dword::new(0x0)],
            sprite_half: [
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
                Dword::new(0x0),
            ],
            ends_scale: [Float::from_bits(0x0), Float::from_bits(0x0)],
            turned: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ],
            second: [Dword::new(0x0), Dword::new(0x0), Dword::new(0x0)],
            _gap_8eb8ed: Bytes::new([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F,
            ]),
            sides: [Float::from_bits(0x0), Float::from_bits(0x0)],
            _gap_8eb949: Bytes::new([0; 8]),
            step: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ],
            _gap_8eb95d: Bytes::new([0; 4]),
            near: Dword::new(0x47800000),
            _gap_8eb965: Bytes::new([0x00, 0x00, 0x80, 0x3B]),
            uv_k: Float::from_bits(0x37800000),
            z_k: Float::from_bits(0x0),
        }
    }
}

impl DrawWork {
    pub const START: usize = 0x8EB871;
    pub const LEN: usize = 0x100;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().frame.draw_work) as usize
    }
}

/// The marks' view heading, step and ribbon work (IDA 0x8F3975 .. 0x8F39CD, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewVars {
    /// 0x8F3975: `g_marksViewHeading`: the view's heading (`effects::view_heading`).
    pub heading: Float,
    /// 0x8F3979: `dword_8F3979`: the frame's step, of 8 (`effects::draw_frame`).
    pub step: Dword,
    /// 0x8F397D: `flt_8F397D`: the view's third row dotted with its step (`effects::view_motion`).
    pub facing: Float,
    /// 0x8F3981: `dword_8F3981`: 0 at each frame's start (`effects::draw_frame`); `effects::draw_flare_part` adds one for each part it draws.
    pub cleared: Dword,
    /// 0x8F3985: `flt_8F3985`, `flt_8F3989`: a ribbon piece's way turned a quarter (`effects::draw_ribbon`).
    pub ribbon_way: [Float; 2],
    /// 0x8F398D: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8f398d: Bytes<4>,
    /// 0x8F3991: `flt_8F3991`, `flt_8F3995`: the ribbon's screen-space half-width offset (x, y) at its first end: the way turned a quarter times the half width times the first end's perspective scale; the quad's corners 0 and 1 are the first end's point less / plus it (`effects::draw_ribbon`).
    pub first_end_half_width: [Float; 2],
    /// 0x8F3999: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8f3999: Bytes<4>,
    /// 0x8F399D: `flt_8F399D`, `flt_8F39A1`: the same half-width offset at the second end (its own perspective scale); corners 2 and 3 are the second end's point plus / less it.
    pub second_end_half_width: [Float; 2],
    /// 0x8F39A5: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_8f39a5: Bytes<4>,
    /// 0x8F39A9: `flt_8F39A9` ..: three points turned by the view matrix (`effects::draw_frame`; x, y, z each).
    pub turned_points: [[Float; 3]; 3],
}

impl ViewVars {
    pub const fn new() -> Self {
        Self {
            heading: Float::from_bits(0x0),
            step: Dword::new(0x0),
            facing: Float::from_bits(0x0),
            cleared: Dword::new(0x0),
            ribbon_way: [Float::from_bits(0x0), Float::from_bits(0x0)],
            _gap_8f398d: Bytes::new([0; 4]),
            first_end_half_width: [Float::from_bits(0x0), Float::from_bits(0x0)],
            _gap_8f3999: Bytes::new([0; 4]),
            second_end_half_width: [Float::from_bits(0x0), Float::from_bits(0x0)],
            _gap_8f39a5: Bytes::new([0; 4]),
            turned_points: [const {
                [
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                    Float::from_bits(0x0),
                ]
            }; 3],
        }
    }
}

impl ViewVars {
    pub const START: usize = 0x8F3975;
    pub const LEN: usize = 0x58;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().frame.view_vars) as usize
    }
}

/// The marks' frame: profile mark and steps, turned points, view, quads and flare (IDA 0x91FF3E .. 0x91FFE2, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct DrawFrame {
    /// 0x91FF3E: `g_marksProfilesDone`: 0, or where `effects::make_profiles` goes on.
    pub profiles_done: Dword,
    /// 0x91FF42: `flt_91FF42`, `flt_91FF46`: 256.0, the ramps' and the curves' steps (`effects::make_profiles`).
    pub profile_steps: [Float; 2],
    /// 0x91FF4A: `flt_91FF4A` ..: three points the frame turns by the view matrix (x, y, z each; `effects::draw_frame`).
    pub points: [[Float; 3]; 3],
    /// 0x91FF6E: `dword_91FF6E`: `dword_621A80` as of the last frame.
    pub last_source: Dword,
    /// 0x91FF72: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_91ff72: Bytes<12>,
    /// 0x91FF7E: `flt_91FF7E` ..: the view's forward level part (`effects::view_heading`).
    pub level: [Float; 3],
    /// 0x91FF8A: `flt_91FF8A` ..: the direction the heading is taken against (`effects::view_heading`).
    pub other: [Float; 3],
    /// 0x91FF96: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_91ff96: Bytes<4>,
    /// 0x91FF9A: `flt_91FF9A`, `flt_91FF9E`: two time marks kept from `flt_765124` / `flt_765118` (`effects::draw_frame`).
    pub times: [Float; 2],
    /// 0x91FFA2: `dword_91FFA2`: Not named: 4 bytes (a gap, or unreferenced data).
    pub _gap_91ffa2: Bytes<4>,
    /// 0x91FFA6: `dword_91FFA6`: the quads' vertex colour (`effects::begin_quads`).
    pub colour: Dword,
    /// 0x91FFAA: `dword_91FFAA` .. `dword_91FFB6`: the quads' texture places (u a, u c, v b, v d).
    pub places: [Dword; 4],
    /// 0x91FFBA: `dword_91FFBA`: 0, set by `effects::begin_quads`.
    pub cleared: Dword,
    /// 0x91FFBE: `flt_91FFBE` ..: the view's place at the last frame (`effects::view_motion`).
    pub last: [Float; 3],
    /// 0x91FFCA: Not named: 12 bytes (a gap, or unreferenced data).
    pub _gap_91ffca: Bytes<12>,
    /// 0x91FFD6: `flt_91FFD6`: the lens flare's length (`effects::draw_lens_flare`).
    pub flare_length: Float,
    /// 0x91FFDA: `flt_91FFDA`, `flt_91FFDE`: the lens flare's way.
    pub flare_way: [Float; 2],
}

impl DrawFrame {
    pub const fn new() -> Self {
        Self {
            profiles_done: Dword::new(0x0),
            profile_steps: [Float::from_bits(0x43800000), Float::from_bits(0x43800000)],
            points: [
                [
                    Float::from_bits(0xC7BF6800),
                    Float::from_bits(0xC7333333),
                    Float::from_bits(0xC7475C00),
                ],
                [
                    Float::from_bits(0xC7BF6800),
                    Float::from_bits(0xC7333333),
                    Float::from_bits(0x47475C00),
                ],
                [
                    Float::from_bits(0xC7BF6800),
                    Float::from_bits(0xC7333333),
                    Float::from_bits(0x0),
                ],
            ],
            last_source: Dword::new(0x0),
            _gap_91ff72: Bytes::new([
                0x00, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
            ]),
            level: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ],
            other: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x3F800000),
            ],
            _gap_91ff96: Bytes::new([0; 4]),
            times: [Float::from_bits(0x42C80000), Float::from_bits(0x42C80000)],
            _gap_91ffa2: Bytes::new([0; 4]),
            colour: Dword::new(0x43800000),
            places: [
                Dword::new(0x3B000000),
                Dword::new(0x3B000000),
                Dword::new(0x3EEDCCCD),
                Dword::new(0x3EEDCCCD),
            ],
            cleared: Dword::new(0x0),
            last: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ],
            _gap_91ffca: Bytes::new([
                0x00, 0x00, 0x00, 0x00, 0xCD, 0xCC, 0xA0, 0x41, 0x1A, 0x00, 0x48, 0x42,
            ]),
            flare_length: Float::from_bits(0x0),
            flare_way: [Float::from_bits(0x0), Float::from_bits(0x0)],
        }
    }
}

impl DrawFrame {
    pub const START: usize = 0x91FF3E;
    pub const LEN: usize = 0xA4;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().frame.draw_frame) as usize
    }
}

/// The lens flare's unit way and its parts' cells and sizes (IDA 0x92051E .. 0x9205F6, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FlareParts {
    /// 0x92051E: `flt_92051E` .. `flt_920526`: the lens flare's unit way (`effects::draw_lens_flare`).
    pub unit: [Float; 3],
    /// 0x92052A: `dword_92052A`: the 0x11 parts' corner cells.
    pub corner: [Dword; 17],
    /// 0x92056E: `dword_92056E`: 0x11 more (the port does not read them).
    pub unread: [Dword; 17],
    /// 0x9205B2: `dword_9205B2`: the 0x11 parts' sizes.
    pub size: [Dword; 17],
}

impl FlareParts {
    pub const fn new() -> Self {
        Self {
            unit: [
                Float::from_bits(0x0),
                Float::from_bits(0x0),
                Float::from_bits(0x0),
            ],
            corner: [
                Dword::new(0x40),
                Dword::new(0x41),
                Dword::new(0x42),
                Dword::new(0x45),
                Dword::new(0x43),
                Dword::new(0x43),
                Dword::new(0x43),
                Dword::new(0x44),
                Dword::new(0x45),
                Dword::new(0x45),
                Dword::new(0x46),
                Dword::new(0x46),
                Dword::new(0x46),
                Dword::new(0x47),
                Dword::new(0x45),
                Dword::new(0x48),
                Dword::new(0x49),
            ],
            unread: [
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xF0FFF),
                Dword::new(0xFFFFFF),
                Dword::new(0xFFFFFF),
            ],
            size: [
                Dword::new(0x85),
                Dword::new(0xBA),
                Dword::new(0x31),
                Dword::new(0x50),
                Dword::new(0x16),
                Dword::new(0x47),
                Dword::new(0x26),
                Dword::new(0x18),
                Dword::new(0x6),
                Dword::new(0x8),
                Dword::new(0x30),
                Dword::new(0x57),
                Dword::new(0x16),
                Dword::new(0x24),
                Dword::new(0x11),
                Dword::new(0x7E),
                Dword::new(0xFB),
            ],
        }
    }
}

impl FlareParts {
    pub const START: usize = 0x92051E;
    pub const LEN: usize = 0xD8;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().frame.flare_parts) as usize
    }
}

/// The lens flare's constants (IDA 0x9206D2 .. 0x9206E6, the image's layout; phase 7, typed state).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct FlareScales {
    /// 0x9206D2: `flt_9206D2`, `flt_9206D6`: the lens flare's fade constants (`effects::draw_lens_flare`).
    pub fade: [Float; 2],
    /// 0x9206DA: `flt_9206DA`: the parts' size constant.
    pub size_k: Float,
    /// 0x9206DE: `flt_9206DE`, `flt_9206E2`: a flare part's scales (`effects::draw_flare_part`).
    pub scale: [Float; 2],
}

impl FlareScales {
    pub const fn new() -> Self {
        Self {
            fade: [Float::from_bits(0x3F000000), Float::from_bits(0x437F0000)],
            size_k: Float::from_bits(0x3FC00000),
            scale: [Float::from_bits(0x3F000000), Float::from_bits(0x3F000000)],
        }
    }
}

impl FlareScales {
    pub const START: usize = 0x9206D2;
    pub const LEN: usize = 0x14;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const effects().frame.flare_scales) as usize
    }
}

#[cfg(test)]
mod typed_tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn draw_work_starts_as_the_games() {
        // Other tests write the work: read it while none runs.
        let _lock = crate::store::test_lock();
        assert_eq!(size_of::<DrawWork>(), DrawWork::LEN);
        let image = crate::moved::image_bytes(DrawWork::START, DrawWork::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(DrawWork::home()).cast_const(),
                DrawWork::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "DRAW_WORK starts differently from the image"
        );
        assert_eq!(offset_of!(DrawWork, half), 0x8EB8B5 - DrawWork::START);
        assert_eq!(
            offset_of!(DrawWork, sprite_half),
            0x8EB8BD - DrawWork::START
        );
        assert_eq!(offset_of!(DrawWork, ends_scale), 0x8EB8CD - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, turned), 0x8EB8D5 - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, second), 0x8EB8E1 - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, sides), 0x8EB941 - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, step), 0x8EB951 - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, near), 0x8EB961 - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, uv_k), 0x8EB969 - DrawWork::START);
        assert_eq!(offset_of!(DrawWork, z_k), 0x8EB96D - DrawWork::START);
    }

    #[test]
    fn view_vars_starts_as_the_games() {
        assert_eq!(size_of::<ViewVars>(), ViewVars::LEN);
        let image = crate::moved::image_bytes(ViewVars::START, ViewVars::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ViewVars::home()).cast_const(),
                ViewVars::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "VIEW_VARS starts differently from the image"
        );
        assert_eq!(offset_of!(ViewVars, step), 0x8F3979 - ViewVars::START);
        assert_eq!(offset_of!(ViewVars, facing), 0x8F397D - ViewVars::START);
        assert_eq!(offset_of!(ViewVars, cleared), 0x8F3981 - ViewVars::START);
        assert_eq!(offset_of!(ViewVars, ribbon_way), 0x8F3985 - ViewVars::START);
        assert_eq!(
            offset_of!(ViewVars, first_end_half_width),
            0x8F3991 - ViewVars::START
        );
        assert_eq!(
            offset_of!(ViewVars, second_end_half_width),
            0x8F399D - ViewVars::START
        );
        assert_eq!(
            offset_of!(ViewVars, turned_points),
            0x8F39A9 - ViewVars::START
        );
    }

    #[test]
    fn marks_frame_starts_as_the_games() {
        assert_eq!(size_of::<DrawFrame>(), DrawFrame::LEN);
        let image = crate::moved::image_bytes(DrawFrame::START, DrawFrame::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(DrawFrame::home()).cast_const(),
                DrawFrame::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "DRAW_FRAME starts differently from the image"
        );
        assert_eq!(
            offset_of!(DrawFrame, profile_steps),
            0x91FF42 - DrawFrame::START
        );
        assert_eq!(offset_of!(DrawFrame, points), 0x91FF4A - DrawFrame::START);
        assert_eq!(
            offset_of!(DrawFrame, last_source),
            0x91FF6E - DrawFrame::START
        );
        assert_eq!(offset_of!(DrawFrame, level), 0x91FF7E - DrawFrame::START);
        assert_eq!(offset_of!(DrawFrame, other), 0x91FF8A - DrawFrame::START);
        assert_eq!(offset_of!(DrawFrame, times), 0x91FF9A - DrawFrame::START);
        assert_eq!(offset_of!(DrawFrame, colour), 0x91FFA6 - DrawFrame::START);
        assert_eq!(offset_of!(DrawFrame, places), 0x91FFAA - DrawFrame::START);
        assert_eq!(offset_of!(DrawFrame, cleared), 0x91FFBA - DrawFrame::START);
        assert_eq!(offset_of!(DrawFrame, last), 0x91FFBE - DrawFrame::START);
        assert_eq!(
            offset_of!(DrawFrame, flare_length),
            0x91FFD6 - DrawFrame::START
        );
        assert_eq!(
            offset_of!(DrawFrame, flare_way),
            0x91FFDA - DrawFrame::START
        );
    }

    #[test]
    fn flare_parts_starts_as_the_games() {
        assert_eq!(size_of::<FlareParts>(), FlareParts::LEN);
        let image = crate::moved::image_bytes(FlareParts::START, FlareParts::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(FlareParts::home()).cast_const(),
                FlareParts::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "FLARE_PARTS starts differently from the image"
        );
        assert_eq!(offset_of!(FlareParts, corner), 0x92052A - FlareParts::START);
        assert_eq!(offset_of!(FlareParts, unread), 0x92056E - FlareParts::START);
        assert_eq!(offset_of!(FlareParts, size), 0x9205B2 - FlareParts::START);
    }

    #[test]
    fn flare_scales_starts_as_the_games() {
        assert_eq!(size_of::<FlareScales>(), FlareScales::LEN);
        let image = crate::moved::image_bytes(FlareScales::START, FlareScales::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(FlareScales::home()).cast_const(),
                FlareScales::LEN,
            )
        };
        assert!(
            mine == &image[..],
            "FLARE_SCALES starts differently from the image"
        );
        assert_eq!(
            offset_of!(FlareScales, size_k),
            0x9206DA - FlareScales::START
        );
        assert_eq!(
            offset_of!(FlareScales, scale),
            0x9206DE - FlareScales::START
        );
    }
}
