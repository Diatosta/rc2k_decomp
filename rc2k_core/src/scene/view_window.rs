//! Where the view is drawn on the screen: the whole screen, or in the framed layout 800 x 600
//! at 0, 0 (with `FrameSwitches.scene_set_up`, or mode `LightTables.view_mode` 2) or 720 x 540
//! at 0x50, 0; and the window's middle, clip rectangle, scale and depths (`f64` intermediates).

use crate::math::geometry::fistp64;

/// The width the projection's scale is made from: the screen's, but no more than 4:3 of its
/// height, so a wider screen shows more to the sides at the same field of view up and down
/// (the usual "Hor+"), whatever its shape.
pub fn design_width(width: u32, height: u32) -> u32 {
    width.min((u64::from(height) * 4 / 3) as u32)
}

/// How much wider than 4:3 the screen's view is: the screen's width over `design_width`, 1.0 for
/// a 4:3 screen and any narrower one. The tests that cut the course to the sides of the view
/// (the far cells, a face's sight) were made for a 4:3 view and widen by this.
pub fn side_widening(width: u32, height: u32) -> f32 {
    let design = design_width(width, height);
    if design == 0 || design >= width {
        1.0
    } else {
        width as f32 / design as f32
    }
}

/// The settings the choice reads (the screen's size only when not framed).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewSettings {
    pub framed: bool,
    pub wide: bool,
    pub mode: u16,
}

/// The window chosen: its size, its offset, and the scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewWindow {
    pub size: [u32; 2],
    pub offset: [u32; 2],
    pub scale: u32,
}

/// The scale every choice sets (0x100).
pub const SCALE: u32 = 0x100;
/// The mode the full-screen choice sets around the choice.
pub const FULL_MODE: u16 = 2;

/// The window chosen; `screen` (width, height) is asked only when not framed.
pub fn choose(settings: ViewSettings, screen: impl FnOnce() -> [u32; 2]) -> ViewWindow {
    let (size, x) = if !settings.framed {
        (screen(), 0)
    } else if settings.wide || settings.mode == FULL_MODE {
        ([800, 600], 0)
    } else {
        ([720, 540], 0x50)
    };
    ViewWindow {
        size,
        offset: [x, 0],
        scale: SCALE,
    }
}

/// What `set_view_window` reads: the reach, the window's size and offsets, whether in a race,
/// the screen's width, the scale's three, the nearer setting and its factor, the fade's two,
/// 65536, the clip's factor, one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowInputs {
    pub reach: u32,
    pub size: [u32; 2],
    pub offsets: [u32; 2],
    pub in_race: bool,
    pub screen_width: u32,
    pub scale: (f32, u32, f32),
    pub nearer: u32,
    pub nearer_factor: f32,
    pub fade: [f32; 2],
    pub turn: f32,
    pub clip_factor: f32,
    pub one: f32,
}

/// What `window_settings` works out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowSettings {
    /// The reach times 0x20.
    pub depth: u32,
    pub middle: [u32; 2],
    /// Top, bottom, left, right.
    pub edges: [u32; 4],
    pub scale: f32,
    pub divisor: f32,
    /// The depth times 0x10000 (64 bits).
    pub product: u64,
    pub farthest: f32,
    pub reciprocal: f32,
    pub fade_depth: i32,
    pub clip_farthest: i32,
    pub offset_floats: [f32; 2],
    pub edge_floats: [f32; 4],
    pub middle_floats: [f32; 2],
    pub less_one: [f32; 2],
}

/// The nearer setting's share: 100 less it, held to 0x5A ..= 0x64 by the differences' signs.
pub fn nearer_share(nearer: u32) -> u32 {
    let mut v = 100u32.wrapping_sub(nearer);
    if (v.wrapping_sub(0x5A) as i32) < 0 {
        v = 0x5A;
    }
    if (v.wrapping_sub(0x64) as i32) >= 0 {
        v = 0x64;
    }
    v
}

/// The settings worked out from the inputs.
pub fn window_settings(i: &WindowInputs) -> WindowSettings {
    let int = |v: u32| v as i32 as f64;
    let depth = i.reach.wrapping_mul(0x20);
    let [width, height] = i.size;
    let middle = [width >> 1, height >> 1];
    let o = i.offsets;
    let edges = if i.in_race {
        [0, height, 0, width]
    } else {
        [
            o[1].wrapping_neg(),
            height.wrapping_sub(o[1]),
            o[0].wrapping_neg(),
            width.wrapping_sub(o[0]),
        ]
    };
    let (a, b, c) = i.scale;
    let scale = (int(i.screen_width) * a as f64 * int(b) * c as f64) as f32;
    let divisor = (int(nearer_share(i.nearer)) * i.nearer_factor as f64) as f32;
    let product = depth as u64 * 0x1_0000;
    let farthest = (int(product as u32) / divisor as f64) as f32;
    let f = farthest as f64;
    let float = |v: u32| int(v) as f32;
    WindowSettings {
        depth,
        middle,
        edges,
        scale,
        divisor,
        product,
        farthest,
        reciprocal: (1.0 / f) as f32,
        fade_depth: fistp64(f * i.fade[0] as f64 / i.fade[1] as f64 / i.turn as f64),
        clip_farthest: fistp64(f * i.clip_factor as f64),
        offset_floats: o.map(float),
        edge_floats: edges.map(float),
        middle_floats: middle.map(float),
        less_one: i.size.map(|v| (int(v) - i.one as f64) as f32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_screens_keep_the_vertical_view() {
        assert_eq!(design_width(800, 600), 800);
        assert_eq!(design_width(1280, 1024), 1280);
        assert_eq!(design_width(1920, 1080), 1440);
        assert_eq!(design_width(3840, 1080), 1440);
    }

    #[test]
    fn widening_is_one_up_to_four_by_three() {
        assert_eq!(side_widening(800, 600), 1.0);
        assert_eq!(side_widening(1280, 1024), 1.0);
        assert_eq!(side_widening(0, 0), 1.0);
        assert_eq!(side_widening(1920, 1080), 1920.0 / 1440.0);
        assert_eq!(side_widening(3840, 1080), 3840.0 / 1440.0);
    }

    #[test]
    fn windows() {
        let s = |framed, wide, mode| ViewSettings { framed, wide, mode };
        assert_eq!(
            choose(s(false, true, 2), || [640, 480]),
            ViewWindow {
                size: [640, 480],
                offset: [0, 0],
                scale: 0x100
            }
        );
        assert_eq!(
            choose(s(true, false, 0), || unreachable!()).offset,
            [0x50, 0]
        );
        assert_eq!(
            choose(s(true, false, 2), || unreachable!()).size,
            [800, 600]
        );
        assert_eq!(choose(s(true, true, 1), || unreachable!()).size, [800, 600]);
        assert_eq!(
            choose(s(true, false, 1), || unreachable!()).size,
            [720, 540]
        );
    }

    #[test]
    fn settings() {
        assert_eq!(nearer_share(0), 0x64);
        assert_eq!(nearer_share(5), 0x5F);
        assert_eq!(nearer_share(11), 0x5A);
        assert_eq!(nearer_share(0x8000_0000), 0x5A);
        assert_eq!(nearer_share(0x8000_0064), 0x64);
        let i = WindowInputs {
            reach: 0x100,
            size: [0x320, 0x258],
            offsets: [0x10, 0x20],
            in_race: false,
            screen_width: 0x320,
            scale: (1.0, 0x100, 0.002),
            nearer: 0,
            nearer_factor: 0.5,
            fade: [1.0, 0.5],
            turn: 65536.0,
            clip_factor: 0.9,
            one: 1.0,
        };
        let s = window_settings(&i);
        assert_eq!((s.depth, s.middle), (0x2000, [0x190, 0x12C]));
        assert_eq!(s.edges, [0xFFFF_FFE0, 0x238, 0xFFFF_FFF0, 0x310]);
        assert_eq!((s.divisor, s.product), (50.0, 0x2000_0000));
        assert_eq!(s.farthest, 0x2000_0000 as f32 / 50.0);
        assert_eq!(s.fade_depth, 328);
        assert_eq!(s.less_one, [799.0, 599.0]);
        assert_eq!(s.edge_floats[0], -32.0);
        let s = window_settings(&WindowInputs { in_race: true, ..i });
        assert_eq!(s.edges, [0, 0x258, 0, 0x320]);
    }
}
