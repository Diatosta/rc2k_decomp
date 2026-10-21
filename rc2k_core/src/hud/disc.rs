//! The HUD's disc (converted 2026-10-21 from `hud::disc_vertex` 0x4B6D10): a ring of 0x11
//! entries (the centre and sixteen round it) made vertices. As in `math::vector`, what the
//! x87 keeps on its stack is an `f64`, rounded to `f32` where the original stores it: the
//! same bits at the game's double precision.

/// The ring's entries.
pub const ENTRIES: u32 = 0x11;

/// One ring entry: its place (`flt_75C188`, `flt_75C1CC`) and its texture place
/// (`flt_75C210`, `flt_75C254`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingEntry {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
}

/// The disc's size and places: its radius (`dword_75C050`), the texture's scale
/// (`dword_75C04C`), middle (`flt_75C01C`), shift (`dword_75C054`, `dword_75C058`) and
/// size (`flt_75C020`), and the centre on the screen (`dword_75C044`, `dword_75C048`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disc {
    pub radius: f32,
    pub texture_scale: f32,
    pub texture_middle: f32,
    pub texture_shift: [f32; 2],
    pub texture_size: f32,
    pub centre: [f32; 2],
}

/// A vertex's place and texture place, as stored (+0, +4 and +0x18, +0x1C).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscVertex {
    pub place: [f32; 2],
    pub texture: [f32; 2],
}

/// `hud::disc_vertex`'s floats: the texture place (u, -v) times the radius times the
/// texture's scale, plus the middle and the shift, over the size; the place (x, -y) times
/// the radius plus the centre.
pub fn vertex(e: RingEntry, d: &Disc) -> DiscVertex {
    let (r, k) = (d.radius as f64, d.texture_scale as f64);
    let texture = |t: f32, n: usize| {
        ((t as f64 * r * k + d.texture_middle as f64 + d.texture_shift[n] as f64)
            / d.texture_size as f64) as f32
    };
    let place = |p: f32, n: usize| (p as f64 * r + d.centre[n] as f64) as f32;
    DiscVertex {
        texture: [texture(e.u, 0), texture(-e.v, 1)],
        place: [place(e.x, 0), place(-e.y, 1)],
    }
}

/// White under the quad record's alpha (its low byte).
pub fn white_under(alpha: u32) -> u32 {
    (alpha << 24) | 0xFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertices() {
        let d = Disc {
            radius: 40.0,
            texture_scale: 0.5,
            texture_middle: 128.0,
            texture_shift: [3.0, -2.0],
            texture_size: 256.0,
            centre: [320.0, 240.0],
        };
        let e = RingEntry {
            x: 1.0,
            y: 0.5,
            u: 1.0,
            v: 0.5,
        };
        let v = vertex(e, &d);
        assert_eq!(v.place, [360.0, 220.0]);
        assert_eq!(v.texture, [(20.0 + 131.0) / 256.0, (-10.0 + 126.0) / 256.0]);
        assert_eq!(white_under(0xC1), 0xC1FF_FFFF);
    }
}
