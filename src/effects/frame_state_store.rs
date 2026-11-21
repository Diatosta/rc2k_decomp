//! The marks' frame state: the sprites' and profiles' work places (`DrawWork`), the view's heading,
//! step and ribbon work (`ViewVars`), the frame's turned points, view and quads (`DrawFrame`), and
//! the lens flare's tables (`FlareParts`, `FlareScales`).

/// The marks' sprites' and profiles' work places.
pub struct DrawWork {
    /// The profile tables' running values (`effects::make_profiles`; as bits).
    pub profile_work: [f32; 11],
    /// Half a sprite's size across and along (whole numbers; `effects::scale_corners` leaves a
    /// corner's two sides here).
    pub half: [f32; 2],
    /// A sprite's half sides turned (whole numbers at first, floats once worked on).
    pub sprite_half: [u32; 4],
    /// The two ends' scales of a ribbon piece (`effects::draw_ribbon`).
    pub ends_scale: [f32; 2],
    /// The place turned into the view's frame (x, y, depth).
    pub turned: [f32; 3],
    /// A ribbon piece's second place in the view's frame (x, y, depth).
    pub second: [f32; 3],
    /// A flare part's sides (`effects::draw_flare_part`).
    pub sides: [f32; 2],
    /// The view's step since the last frame (`effects::view_motion`).
    pub step: [f32; 3],
    /// A depth at or below it (compared as a signed whole number of the float's bits) is not drawn.
    pub near: f32,
    /// 1 / 65536, what a cell's words are multiplied by to make texture places.
    pub uv_k: f32,
    /// 1 over the view's farthest depth (`effects::begin_quads`), what depths are multiplied by.
    pub z_k: f32,
}

impl DrawWork {
    pub const fn new() -> Self {
        Self {
            profile_work: [0.0; 11],
            half: [0.0, 0.0],
            sprite_half: [0x0, 0x0, 0x0, 0x0],
            ends_scale: [0.0, 0.0],
            turned: [0.0, 0.0, 0.0],
            second: [0.0, 0.0, 0.0],
            sides: [0.0, 0.0],
            step: [0.0, 0.0, 0.0],
            near: 65536.0,
            uv_k: 0.000015258789,
            z_k: 0.0,
        }
    }
}

impl DrawWork {
    /// What a sprite's square is projected by: the view's scale and centre (the front end's
    /// view window) and the depth scale.
    pub(super) fn view_k(&self, front: &crate::scene::polygon_work_store::PolygonWork) -> [f32; 4] {
        let window = &front.window;
        [
            window.borrow().view_scale,
            window.borrow().view_centre[0],
            window.borrow().view_centre[1],
            self.z_k,
        ]
    }
}

/// The marks' view heading, step and ribbon work.
pub struct ViewVars {
    /// The view's heading (`effects::view_heading`).
    pub heading: f32,
    /// The frame's step, of 8 (`effects::draw_frame`).
    pub step: u32,
    /// The view's third row dotted with its step (`effects::view_motion`).
    pub facing: f32,
    /// 0 at each frame's start (`effects::draw_frame`); `effects::draw_flare_part` adds one for
    /// each part it draws.
    pub cleared: u32,
    /// A ribbon piece's way turned a quarter (`effects::draw_ribbon`).
    pub ribbon_way: [f32; 2],
    /// The ribbon's screen-space half-width offset (x, y) at its first end: the way turned a
    /// quarter times the half width times the first end's perspective scale; the quad's corners 0
    /// and 1 are the first end's point less / plus it (`effects::draw_ribbon`).
    pub first_end_half_width: [f32; 2],
    /// The same half-width offset at the second end (its own perspective scale); corners 2 and 3
    /// are the second end's point plus / less it.
    pub second_end_half_width: [f32; 2],
    /// Three points turned by the view matrix (`effects::draw_frame`; x, y, z each).
    pub turned_points: [[f32; 3]; 3],
}

impl ViewVars {
    pub const fn new() -> Self {
        Self {
            heading: 0.0,
            step: 0x0,
            facing: 0.0,
            cleared: 0x0,
            ribbon_way: [0.0, 0.0],
            first_end_half_width: [0.0, 0.0],
            second_end_half_width: [0.0, 0.0],
            turned_points: [[0.0; 3]; 3],
        }
    }
}

/// The marks' frame: profile mark and steps, turned points, view, quads and flare.
pub struct DrawFrame {
    /// Set once `effects::make_profiles` has made its tables.
    pub profiles_done: bool,
    /// 256.0, the ramps' and the curves' steps (`effects::make_profiles`).
    pub profile_steps: [f32; 2],
    /// Three points the frame turns by the view matrix (x, y, z each; `effects::draw_frame`).
    pub points: [[f32; 3]; 3],
    /// The marks' source switch as of the last frame.
    pub last_source: u32,
    /// The view's forward level part (`effects::view_heading`).
    pub level: [f32; 3],
    /// The direction the heading is taken against (`effects::view_heading`).
    pub other: [f32; 3],
    /// Two time marks kept from the race's view timing (`effects::draw_frame`).
    pub times: [f32; 2],
    /// The draw amount the debug page compares with `cleared` (`effects::draw_frame` does not touch
    /// it).
    pub drawn: u32,
    /// The quads' vertices' rhw, 256 (`effects::begin_quads`).
    pub rhw: f32,
    /// The quads' texture places (u a, u c, v b, v d).
    pub places: [f32; 4],
    /// 0, set by `effects::begin_quads`.
    pub cleared: u32,
    /// The view's place at the last frame (`effects::view_motion`).
    pub last: [f32; 3],
    /// The lens flare's length (`effects::draw_lens_flare`).
    pub flare_length: f32,
    /// The lens flare's way.
    pub flare_way: [f32; 2],
}

impl DrawFrame {
    pub const fn new() -> Self {
        Self {
            profiles_done: false,
            profile_steps: [256.0, 256.0],
            points: [
                [-98000.0, -45875.2, -51036.0],
                [-98000.0, -45875.2, 51036.0],
                [-98000.0, -45875.2, 0.0],
            ],
            last_source: 0x0,
            level: [0.0, 0.0, 0.0],
            other: [0.0, 0.0, 1.0],
            times: [100.0, 100.0],
            drawn: 0x0,
            rhw: 256.0,
            places: [0.001953125, 0.001953125, 0.46445313, 0.46445313],
            cleared: 0x0,
            last: [0.0, 0.0, 0.0],
            flare_length: 0.0,
            flare_way: [0.0, 0.0],
        }
    }
}

/// The lens flare's unit way and its parts' cells and sizes.
pub struct FlareParts {
    /// The lens flare's unit way (`effects::draw_lens_flare`).
    pub unit: [f32; 3],
    /// The 0x11 parts' corner cells.
    pub corner: [u32; 17],
    /// 0x11 more (nothing reads them).
    pub unread: [u32; 17],
    /// The 0x11 parts' sizes.
    pub size: [u32; 17],
}

impl FlareParts {
    pub const fn new() -> Self {
        Self {
            unit: [0.0, 0.0, 0.0],
            corner: [
                0x40, 0x41, 0x42, 0x45, 0x43, 0x43, 0x43, 0x44, 0x45, 0x45, 0x46, 0x46, 0x46, 0x47,
                0x45, 0x48, 0x49,
            ],
            unread: [
                0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF,
                0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xF0FFF, 0xFFFFFF,
                0xFFFFFF,
            ],
            size: [
                0x85, 0xBA, 0x31, 0x50, 0x16, 0x47, 0x26, 0x18, 0x6, 0x8, 0x30, 0x57, 0x16, 0x24,
                0x11, 0x7E, 0xFB,
            ],
        }
    }
}

/// The lens flare's constants.
pub struct FlareScales {
    /// The lens flare's fade constants (`effects::draw_lens_flare`).
    pub fade: [f32; 2],
    /// The parts' size constant.
    pub size_k: f32,
    /// A flare part's scales (`effects::draw_flare_part`).
    pub scale: [f32; 2],
}

impl FlareScales {
    pub const fn new() -> Self {
        Self {
            fade: [0.5, 255.0],
            size_k: 1.5,
            scale: [0.5, 0.5],
        }
    }
}
