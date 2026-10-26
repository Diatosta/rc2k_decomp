//! Where the view is drawn on the screen (converted 2026-10-21 from
//! `scene::choose_view_window` 0x44FE1A): the whole screen, or in the framed layout
//! (`dword_621514`) 800 x 600 at 0, 0 (with `dword_621480`, or mode `word_636526` 2) or
//! 720 x 540 at 0x50, 0; and the window's middle, clip rectangle, scale and depths
//! (converted 2026-10-21 from `scene::set_view_window` 0x44E47B; what the x87 keeps on
//! its stack an `f64`, the same at the game's double precision).

use crate::math::geometry::fistp64;

/// The settings the choice reads (the screen's size only when not framed).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewSettings {
    pub framed: bool,
    pub wide: bool,
    pub mode: u16,
}

/// The window chosen: its size, its offset, and the scale (`dword_624080`, 0x100).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewWindow {
    pub size: [u32; 2],
    pub offset: [u32; 2],
    pub scale: u32,
}

/// The scale every choice sets.
pub const SCALE: u32 = 0x100;
/// The mode `scene::full_view_window` sets around the choice.
pub const FULL_MODE: u16 = 2;

/// `choose_view_window`'s window; `screen` (width, height) is asked only when not framed.
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

/// What `set_view_window` reads: the reach (`dword_60A87C`), the window's size
/// (`dword_62408C`) and offsets (`dword_623FB0`), whether in a race (`g_inRace` 1), the
/// screen's width, the scale's three (`dword_606294` a float, `dword_624080`,
/// `flt_623DCC`), the nearer setting (`dword_60AC80`) and its factor (`flt_5C5CFC`), the
/// fade's two (`flt_624228`, `flt_624220`), 65536 (`flt_5C5D98`), the clip's factor
/// (`flt_62422C`), one (`flt_5C5D38`).
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

/// What it works out (in the order of the header of `scene::set_view_window`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowSettings {
    /// `dword_63DC54` (and `dword_63DC58`): the reach times 0x20.
    pub depth: u32,
    pub middle: [u32; 2],
    /// Top, bottom, left, right.
    pub edges: [u32; 4],
    pub scale: f32,
    pub divisor: f32,
    /// The depth times 0x10000 (64 bits: eax, edx).
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

/// The nearer setting's share: 100 less it, held to 0x5A .. 0x64 by the differences'
/// signs (`cmp; jns`, `cmp; js`, as the original).
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

/// `set_view_window`'s work.
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
