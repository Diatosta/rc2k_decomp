//! The view's surroundings: the draw distance's settings, the noise map sampled at the
//! car, the fog table, and the course grid's cells round the view (converted 2026-10-21
//! from `race::draw_distance_settings` 0x43932A, `sample_noise` 0x44FD38, `fog_table`
//! 0x44FC33, `eax_environment` 0x45C7EE, `find_sound_spots` 0x45C605,
//! `cell_sound_spots` 0x45C6B8, `gather_visible_faces` 0x454B71 and the rules of
//! `face_sound_spot` 0x45C6ED; the fog's colour and the mirror's window converted
//! 2026-10-21 from `race::set_up_fog` 0x44FBD9 and `race::mirror_view_window` 0x455F17;
//! the sound spots' sharing converted 2026-10-21 from `race::share_sound_spots` 0x45C7B9).

/// The draw distance's highest level.
pub const MOST_LEVEL: i32 = 4;

/// `race::draw_distance_settings`: the level (`dword_60AC84`) held to 0 ..= 4 by sign.
pub fn draw_distance_level(level: u32) -> u32 {
    (level as i32).clamp(0, MOST_LEVEL) as u32
}

/// `race::draw_distance_settings`: the grid's reach from the level's table: at least 1 by
/// the sign of reach - 1 (0, and above 0x80000000, become 1).
pub fn grid_reach(reach: u32) -> u32 {
    if (reach.wrapping_sub(1) as i32) < 0 {
        1
    } else {
        reach
    }
}

/// The noise map's side (64 x 64 bytes).
pub const NOISE_SIDE: u32 = 64;

/// What `sample_noise` works out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoiseSample {
    /// The four corners round the place (`dword_637620` ..): (x, z), (x + 1, z),
    /// (x, z + 1), (x + 1, z + 1).
    pub corners: [u32; 4],
    /// Blended along x at z and at z + 1 (`dword_637630`, `dword_637634`).
    pub near: u32,
    pub far: u32,
    /// Those blended along z, << 8.
    pub value: u32,
    /// The two fractions (bits 8 .. 23 of the place).
    pub fx: u32,
    pub fz: u32,
    /// The last product's high half.
    pub high: u32,
}

/// A blend step: `imul` of (b - a) by the fraction, `shrd` by 16, plus a; and the
/// product's high half.
fn blend(a: u32, b: u32, f: u32) -> (u32, u32) {
    let p = (b.wrapping_sub(a) as i32 as i64) * (f as i32 as i64);
    (((p >> 16) as u32).wrapping_add(a), (p >> 32) as u32)
}

/// `race::sample_noise`: the 64 x 64 map (`map(index)`, each corner asked once, in order)
/// at the place (`px`, `pz`: the top byte the cell, the next two the fraction).
pub fn sample_noise(map: impl Fn(u32) -> u8, px: u32, pz: u32) -> NoiseSample {
    let (cx, cz) = (px >> 24, pz >> 24);
    let at = |x: u32, z: u32| map((x & 0x3F) + ((z & 0x3F) << 6)) as u32;
    let corners = [
        at(cx, cz),
        at(cx + 1, cz),
        at(cx, cz + 1),
        at(cx + 1, cz + 1),
    ];
    let fx = (px >> 8) & 0xFFFF;
    let fz = (pz >> 8) & 0xFFFF;
    let (near, _) = blend(corners[0], corners[1], fx);
    let (far, _) = blend(corners[2], corners[3], fx);
    let (v, high) = blend(near, far, fz);
    NoiseSample {
        corners,
        near,
        far,
        value: v << 8,
        fx,
        fz,
        high,
    }
}

/// The fog table's longest (bytes).
pub const MOST_FOG: u32 = 0x1000;

/// What `fog_table` reads besides the noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FogSettings {
    /// The noise's spread: `dword_609308` and `dword_609320`.
    pub low: u32,
    pub high: u32,
    /// `dword_6415DC` and `dword_6415D8`, multiplied (>> 4).
    pub scale: [u32; 2],
    /// `dword_609310` and `dword_609360`: the near and far parts of that.
    pub near: u32,
    pub far: u32,
    /// `dword_609358`: the table's first level.
    pub start: u32,
    /// `dword_60930C`: the level past the table.
    pub end: u32,
    /// The table's length (`dword_63DC58` x 8).
    pub length: u32,
}

/// What `fog_table` makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fog {
    /// The step (`dword_62DAF0`): low << 8 plus the noise's share of the spread.
    pub step: u32,
    /// The noise times the spread, >> 8.
    pub share: u32,
    /// `dword_6365F0` / `dword_6365F4`.
    pub near: u32,
    pub far: u32,
    /// `g_fogTable`.
    pub table: Vec<u8>,
    /// The steps' sum at the end.
    pub sum: u32,
    /// `dword_636610`: the next level with the end level, plus near, each held.
    pub next: u32,
}

/// A value held to 0 ..= 0xFF by sign.
fn clamp_byte(v: u32) -> u32 {
    (v as i32).clamp(0, 0xFF) as u32
}

/// `race::fog_table`: the table from the noise at the car (`sample_noise`'s value).
pub fn fog_table(noise: u32, s: &FogSettings) -> Fog {
    let span = s.high.wrapping_sub(s.low);
    let share = ((noise as i32 as i64 * span as i32 as i64) >> 8) as u32;
    let step = (s.low << 8).wrapping_add(share);
    let base = s.scale[0].wrapping_mul(s.scale[1]) >> 4;
    let near = base.wrapping_mul(s.near) >> 8;
    let far = base.wrapping_mul(s.far) >> 8;
    let mut sum = 0u32;
    let table = (0..s.length)
        .map(|_| {
            sum = sum.wrapping_add(step);
            clamp_byte(
                ((sum as i32 >> 16) as u32)
                    .wrapping_add(s.start)
                    .wrapping_add(near),
            ) as u8
        })
        .collect();
    let end = clamp_byte(
        ((sum as i32 >> 16) as u32)
            .wrapping_add(s.start)
            .wrapping_add(s.end),
    );
    Fog {
        step,
        share,
        near,
        far,
        table,
        sum,
        next: clamp_byte(end.wrapping_add(near)),
    }
}

/// The course grid's row (`g_courseGrid`: 32 cells a row, 0x20 dwords apart).
pub const GRID_ROW: u32 = 0x20;

/// The `side` x `side` cells from (`from`, `from`) on the grid's diagonal, row by row
/// (`race::eax_environment`: 3 from reach - 1; `race::find_sound_spots`: 5 from reach -
/// 2).
pub fn cells_from(from: u32, side: u32) -> Vec<u32> {
    let first = from.wrapping_mul(GRID_ROW + 1);
    (0..side)
        .flat_map(|row| (0..side).map(move |c| first.wrapping_add(row * GRID_ROW + c)))
        .collect()
}

/// A grid cell with no record (from this up).
pub const NO_RECORD: u32 = 0x0FFF_FFFF;

/// A course record's faces (+8 the first, +0xC the last): None for -1 or a first past
/// the last.
pub fn record_faces(first: u32, last: u32) -> Option<(u32, u32)> {
    (first != u32::MAX && first <= last).then_some((first, last))
}

/// `race::gather_visible_faces`: the records whose faces are gathered: each grid cell's
/// (in order, those with one), then the last record of the course (`count` - 1) once
/// more, unless there are none (its "already done" test is lost: section 16).
pub fn visible_records(grid: impl IntoIterator<Item = u32>, count: u32) -> Vec<u32> {
    let mut records: Vec<u32> = grid.into_iter().filter(|&n| n < NO_RECORD).collect();
    let last = count.wrapping_sub(1);
    if last != u32::MAX {
        records.push(last);
    }
    records
}

/// The sound spots' object (`race::face_sound_spot`; the face's kind 0xFF).
pub const SPOT_OBJECT: u8 = 0x3F;
/// The most sound spots (`g_soundSpots`).
pub const MOST_SPOTS: u32 = 0x24;
/// The highest sound a spot has (the face's +8, from 1).
pub const MOST_SPOT_SOUND: u32 = 0x15;

/// `race::face_sound_spot`: whether a face of `kind` and `object` is a sound object.
pub fn sound_object(kind: u8, object: u8) -> bool {
    kind == 0xFF && object == SPOT_OBJECT
}

/// `race::face_sound_spot`: whether a spot of `sound` is made with `count` made.
pub fn takes_spot(count: u32, sound: u32) -> bool {
    count != MOST_SPOTS && (1..=MOST_SPOT_SOUND).contains(&sound)
}

/// `race::face_sound_spot`: the spot's place, its point's plus the face's offset
/// (+0x30 ..), each wrapping.
pub fn spot_place(point: [u32; 3], offset: [u32; 3]) -> [u32; 3] {
    [0, 1, 2].map(|k| point[k].wrapping_add(offset[k]))
}

/// `race::set_up_fog`'s fog colour from the stage's red, green and blue (each shifted into
/// its byte and added, as dwords).
pub fn fog_colour(red: u32, green: u32, blue: u32) -> u32 {
    (red << 16).wrapping_add(green << 8).wrapping_add(blue)
}

/// The fog colour as the device takes it (opaque).
pub fn device_fog_colour(colour: u32) -> u32 {
    colour | 0xFF00_0000
}

/// The mirror's window on the screen (each even).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The mirror's place's entry (8 bytes each in its tables): the first (`dword_6238A4`)
/// plus three a view, six on when the second set is asked for (`dword_6D1502` 0).
pub fn mirror_entry(first: u32, view: u32, second_set: bool) -> u32 {
    let n = first.wrapping_add(view.wrapping_mul(3));
    if second_set { n.wrapping_add(6) } else { n }
}

/// The mirror's width and height on an 800 x 600 screen.
pub const MIRROR_SIZE: (u32, u32) = (0xB3, 0x40);

/// `n` x `size` / `of` (unsigned, 64 bits over 32 as `mul` / `div`), even; None where the
/// original's `div` would fault.
fn scaled(n: u32, size: u32, of: u32) -> Option<u32> {
    let q = n as u64 * size as u64 / of as u64;
    (q <= u32::MAX as u64).then_some(q as u32 & !1)
}

/// `race::mirror_view_window`'s window: its corner (`place`, on 800 x 600) and size scaled
/// to the screen. None where the scaling would fault.
pub fn mirror_window(place: (u32, u32), screen: (u32, u32)) -> Option<MirrorRect> {
    Some(MirrorRect {
        x: scaled(place.0, screen.0, 800)?,
        y: scaled(place.1, screen.1, 600)?,
        width: scaled(MIRROR_SIZE.0, screen.0, 800)?,
        height: scaled(MIRROR_SIZE.1, screen.1, 600)?,
    })
}

/// The sound spots a frame makes and the sound side's table holds.
pub const MOST_SHARED_SPOTS: u32 = 36;
/// A sound spot's bytes.
pub const SPOT_BYTES: u32 = 0x14;

/// `race::share_sound_spots`: the bytes of the frame's spots handed to the sound side;
/// refused past 36 (past both tables).
pub fn shared_spot_bytes(count: u32) -> Result<u32, crate::Invalid> {
    if count > MOST_SHARED_SPOTS {
        return Err(crate::Invalid("more than 36 sound spots"));
    }
    Ok(count * SPOT_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_and_mirror() {
        assert_eq!(fog_colour(0x12, 0x34, 0x56), 0x12_3456);
        assert_eq!(fog_colour(0x100, 0, 0), 0x0100_0000);
        assert_eq!(device_fog_colour(0x12_3456), 0xFF12_3456);
        assert_eq!(mirror_entry(4, 1, false), 7);
        assert_eq!(mirror_entry(4, 1, true), 13);
        assert_eq!(
            mirror_window((0x139, 0x10), (1024, 768)),
            Some(MirrorRect {
                x: 0x190,
                y: 0x14,
                width: 0xE4,
                height: 0x50
            })
        );
        assert_eq!(mirror_window((u32::MAX, 0), (u32::MAX, 600)), None);
    }

    #[test]
    fn draw_distance() {
        assert_eq!(draw_distance_level(u32::MAX), 0);
        assert_eq!(draw_distance_level(9), 4);
        assert_eq!(draw_distance_level(2), 2);
        assert_eq!(grid_reach(0), 1);
        assert_eq!(grid_reach(0x8000_0000), 0x8000_0000);
        assert_eq!(grid_reach(0x8000_0001), 1);
        assert_eq!(grid_reach(7), 7);
    }

    #[test]
    fn noise() {
        let map = |n: u32| (n & 0xFF) as u8;
        // Cell (63, 0): its right neighbour wraps to 0.
        let s = sample_noise(map, 0x3F80_0000, 0x0000_8000);
        assert_eq!(s.corners, [63, 0, 63 + 64, 64]);
        assert_eq!((s.fx, s.fz), (0x8000, 0x80));
        // -63 x 0x8000 >> 16 is -32 (the shift floors).
        assert_eq!(s.near, 31);
        assert_eq!(s.far, 95);
        assert_eq!(s.value, 31 << 8);
        assert_eq!(s.high, 0);
        let s = sample_noise(|_| 9, 0, 0);
        assert_eq!((s.value, s.near), (9 << 8, 9));
    }

    #[test]
    fn fog() {
        let s = FogSettings {
            low: 2,
            high: 6,
            scale: [0x20, 0x10],
            near: 0x100,
            far: 0x80,
            start: 0xF0,
            end: 0x20,
            length: 8,
        };
        let f = fog_table(0x80, &s);
        assert_eq!((f.share, f.step), (2, 0x202));
        assert_eq!((f.near, f.far), (0x20, 0x10));
        assert_eq!(f.table, [0xFF; 8]);
        assert_eq!(f.sum, 0x1010);
        assert_eq!(f.next, 0xFF);
        let f = fog_table(0, &FogSettings { start: 0, ..s });
        assert_eq!(f.table[0], 0x20);
        let f = fog_table(
            0,
            &FogSettings {
                start: (-0x40i32) as u32,
                ..s
            },
        );
        assert_eq!((f.table[0], f.next), (0, 0x20));
    }

    #[test]
    fn cells_and_faces() {
        assert_eq!(
            cells_from(2, 3),
            [0x42, 0x43, 0x44, 0x62, 0x63, 0x64, 0x82, 0x83, 0x84]
        );
        let five = cells_from(0, 5);
        assert_eq!((five.len(), five[5], five[24]), (25, 0x20, 0x84));
        assert_eq!(record_faces(u32::MAX, 3), None);
        assert_eq!(record_faces(4, 3), None);
        assert_eq!(record_faces(3, 3), Some((3, 3)));
        assert_eq!(visible_records([4, NO_RECORD, 2, u32::MAX], 9), [4, 2, 8]);
        assert_eq!(visible_records([1], 0), [1]);
        assert!(sound_object(0xFF, 0x3F) && !sound_object(0xFE, 0x3F));
        assert!(takes_spot(0x23, 0x15) && !takes_spot(0x24, 1) && !takes_spot(0, 0));
        assert!(!takes_spot(0, 0x16));
        assert_eq!(spot_place([1, u32::MAX, 3], [1, 1, 1]), [2, 0, 4]);
    }

    #[test]
    fn shared_spots() {
        assert_eq!(shared_spot_bytes(36), Ok(36 * 0x14));
        assert_eq!(shared_spot_bytes(0), Ok(0));
        assert!(shared_spot_bytes(37).is_err());
    }
}
