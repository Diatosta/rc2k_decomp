//! The view's surroundings: draw-distance settings, the noise map sampled at the car, the fog
//! table and colour, the mirror's window, the grid cells round the view and the sound spots.

/// The draw distance's highest level.
pub const MOST_LEVEL: i32 = 4;

/// The draw distance level held to 0 ..= 4 by sign.
pub fn draw_distance_level(level: u32) -> u32 {
    (level as i32).clamp(0, MOST_LEVEL) as u32
}

/// The grid's reach from the level's table: at least 1 by the sign of reach - 1 (0 and above
/// 0x80000000 become 1).
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
    /// The four corners round the place: (x, z), (x + 1, z), (x, z + 1), (x + 1, z + 1).
    pub corners: [u32; 4],
    /// Blended along x at z.
    pub near: u32,
    /// Blended along x at z + 1.
    pub far: u32,
    /// Those blended along z, << 8.
    pub value: u32,
    /// The x fraction (bits 8 ..= 23 of the place).
    pub fx: u32,
    /// The z fraction (bits 8 ..= 23 of the place).
    pub fz: u32,
    /// The last product's high half.
    pub high: u32,
}

/// A blend step: (b - a) x fraction >> 16, plus a; and the product's high half.
fn blend(a: u32, b: u32, f: u32) -> (u32, u32) {
    let p = (b.wrapping_sub(a) as i32 as i64) * (f as i32 as i64);
    (((p >> 16) as u32).wrapping_add(a), (p >> 32) as u32)
}

/// Samples the 64 x 64 map (`map(index)`, each corner asked once, in order) at (`px`, `pz`):
/// the top byte is the cell, the next two the fraction.
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
    /// The noise's spread: its low end.
    pub low: u32,
    /// The noise's spread: its high end.
    pub high: u32,
    /// The sun's size and the view distance, multiplied (>> 4).
    pub scale: [u32; 2],
    /// The near part of that.
    pub near: u32,
    /// The far part of that.
    pub far: u32,
    /// The table's first level.
    pub start: u32,
    /// The level past the table.
    pub end: u32,
    /// The table's length (the view depth x 8).
    pub length: u32,
}

/// What `fog_table` makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fog {
    /// The step: low << 8 plus the noise's share of the spread.
    pub step: u32,
    /// The noise times the spread, >> 8.
    pub share: u32,
    /// The near level.
    pub near: u32,
    /// The far level.
    pub far: u32,
    /// The fog table.
    pub table: Vec<u8>,
    /// The steps' sum at the end.
    pub sum: u32,
    /// The next level with the end level, plus near, each held.
    pub next: u32,
}

/// A value held to 0 ..= 0xFF by sign.
fn clamp_byte(v: u32) -> u32 {
    (v as i32).clamp(0, 0xFF) as u32
}

/// The fog table from the noise at the car (`sample_noise`'s value).
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

/// The course grid's row: 32 cells, 0x20 dwords apart.
pub const GRID_ROW: u32 = 0x20;

/// The `side` x `side` cells from (`from`, `from`) on the grid's diagonal, row by row (3 from
/// reach - 1 for the environment; 5 from reach - 2 for the sound spots).
pub fn cells_from(from: u32, side: u32) -> Vec<u32> {
    let first = from.wrapping_mul(GRID_ROW + 1);
    (0..side)
        .flat_map(|row| (0..side).map(move |c| first.wrapping_add(row * GRID_ROW + c)))
        .collect()
}

/// A grid cell with no record (from this up).
pub const NO_RECORD: u32 = 0x0FFF_FFFF;

/// A course record's faces (first, last): None for -1 or a first past the last.
pub fn record_faces(first: u32, last: u32) -> Option<(u32, u32)> {
    (first != u32::MAX && first <= last).then_some((first, last))
}

/// The records whose faces are gathered: each grid cell's (in order, those with one), then
/// the last record of the course unless there are none or the grid held it.
pub fn visible_records(grid: impl IntoIterator<Item = u32>, count: u32) -> Vec<u32> {
    let mut records: Vec<u32> = grid.into_iter().filter(|&n| n < NO_RECORD).collect();
    let last = count.wrapping_sub(1);
    if last != u32::MAX && !records.contains(&last) {
        records.push(last);
    }
    records
}

/// The sound spots' object (the face's kind is 0xFF).
pub const SPOT_OBJECT: u8 = 0x3F;
/// The most sound spots.
pub const MOST_SPOTS: u32 = 0x24;
/// The highest sound a spot has (from 1).
pub const MOST_SPOT_SOUND: u32 = 0x15;

/// Whether a face of `kind` and `object` is a sound object.
pub fn sound_object(kind: u8, object: u8) -> bool {
    kind == 0xFF && object == SPOT_OBJECT
}

/// Whether a spot of `sound` is made with `count` made.
pub fn takes_spot(count: u32, sound: u32) -> bool {
    count != MOST_SPOTS && (1..=MOST_SPOT_SOUND).contains(&sound)
}

/// The spot's place: its point's plus the face's offset, each wrapping.
pub fn spot_place(point: [u32; 3], offset: [u32; 3]) -> [u32; 3] {
    [0, 1, 2].map(|k| point[k].wrapping_add(offset[k]))
}

/// The fog colour from the stage's red, green and blue (each shifted into its byte, added as
/// dwords).
pub fn fog_colour(red: u32, green: u32, blue: u32) -> u32 {
    (red << 16).wrapping_add(green << 8).wrapping_add(blue)
}

/// The fog colour as the device takes it (opaque).
pub fn device_fog_colour(colour: u32) -> u32 {
    colour | 0xFF00_0000
}

/// A player's part of the screen: its corner and size in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerPart {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// How the screen is shared between the players of a split-screen race.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    /// Two players, one above the other.
    Above,
    /// Two players, side by side.
    Beside,
    /// Three or four players, one in each quarter (with three, the fourth quarter has no player).
    Quarters,
    /// Three or four players, one row each.
    Rows,
    /// Three or four players, one column each.
    Columns,
    /// Three players: the first over the whole width above, the other two side by side below.
    WideTop,
}

impl Split {
    /// The split of `players` players, `side_by_side` choosing between the two layouts of two
    /// (a larger count starts from its first layout, `many`); `None` for a count with no split
    /// (one player has the whole screen).
    pub fn of(players: u32, side_by_side: bool) -> Option<Self> {
        match players {
            2 if side_by_side => Some(Self::Beside),
            2 => Some(Self::Above),
            3 | 4 => Self::many(players, 0),
            _ => None,
        }
    }

    /// The layouts three or four players cycle through, the first one the start.
    pub fn many_layouts(players: u32) -> &'static [Split] {
        match players {
            3 => &[Self::WideTop, Self::Rows, Self::Columns, Self::Quarters],
            4 => &[Self::Quarters, Self::Rows, Self::Columns],
            _ => &[],
        }
    }

    /// The layout number `index` of `players` players (three or four), the index wrapping round
    /// the layouts there are.
    pub fn many(players: u32, index: u32) -> Option<Self> {
        let layouts = Self::many_layouts(players);
        layouts.get(index as usize % layouts.len().max(1)).copied()
    }

    /// The index after `index` among `players` players' layouts.
    pub fn next_many(players: u32, index: u32) -> u32 {
        let count = Self::many_layouts(players).len() as u32;
        if count == 0 { 0 } else { (index + 1) % count }
    }

    /// Player `player`'s part of a `screen` (width, height), or `None` where the split has no
    /// part for that player. Parts are the screen's size divided evenly (rounded down) among the
    /// rows and columns, in reading order; `players` only matters to the rows and columns.
    pub fn part(self, player: u32, screen: (u32, u32)) -> Option<PlayerPart> {
        self.part_of(4, player, screen)
    }

    /// `part` for a race of `players` players: the rows and columns of the layouts that have
    /// one each are as many as the players.
    pub fn part_of(self, players: u32, player: u32, screen: (u32, u32)) -> Option<PlayerPart> {
        let (columns, rows) = match self {
            Self::Above => (1, 2),
            Self::Beside => (2, 1),
            Self::Quarters => (2, 2),
            Self::Rows => (1, players.clamp(1, 4)),
            Self::Columns => (players.clamp(1, 4), 1),
            Self::WideTop => {
                let (half, h) = (screen.0 / 2, screen.1 / 2);
                return match player {
                    0 => Some(PlayerPart {
                        x: 0,
                        y: 0,
                        width: screen.0,
                        height: h,
                    }),
                    1 | 2 => Some(PlayerPart {
                        x: (player - 1) * half,
                        y: h,
                        width: half,
                        height: h,
                    }),
                    _ => None,
                };
            }
        };
        if player >= columns * rows {
            return None;
        }
        let (width, height) = (screen.0 / columns, screen.1 / rows);
        Some(PlayerPart {
            x: player % columns * width,
            y: player / columns * height,
            width,
            height,
        })
    }

    /// Whether `player`'s part is the only one that has the quarter of the map, the fourth
    /// quarter that three players leave free: its corner and size.
    pub fn free_part(self, players: u32, screen: (u32, u32)) -> Option<PlayerPart> {
        if self == Self::Quarters && players == 3 {
            self.part_of(4, 3, screen)
        } else {
            None
        }
    }
}

/// The mirror's window on the screen (each even).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// The mirror's place entry (8 bytes each): the first plus three a view, six on for the
/// second set.
pub fn mirror_entry(first: u32, view: u32, second_set: bool) -> u32 {
    let n = first.wrapping_add(view.wrapping_mul(3));
    if second_set { n.wrapping_add(6) } else { n }
}

/// The mirror's width and height on an 800 x 600 screen.
pub const MIRROR_SIZE: (u32, u32) = (0xB3, 0x40);

/// `n` x `size` / `of` (unsigned, 64 bits over 32), even; None where the division would fault.
fn scaled(n: u32, size: u32, of: u32) -> Option<u32> {
    let q = n as u64 * size as u64 / of as u64;
    (q <= u32::MAX as u64).then_some(q as u32 & !1)
}

/// The mirror's window: its corner (`place`, on 800 x 600) and size scaled to the screen; None
/// where the scaling would fault. The mirror keeps to the HUD's box (`hud_box`, centred on the
/// screen); on a box wider than 4:3 it keeps its size's shape and its edge or middle by `rule`
/// (`hud::anchor`).
pub fn mirror_window(
    place: (u32, u32),
    screen: (u32, u32),
    rule: crate::hud::anchor::Rule,
    hud_box: crate::hud::anchor::BoxRatio,
) -> Option<MirrorRect> {
    use crate::hud::anchor;
    let (left, width) = hud_box.area(screen.0, screen.1);
    if anchor::is_wide(width, screen.1) {
        let middle = place.0 as f32 + MIRROR_SIZE.0 as f32 / 2.0;
        let at = anchor::offset(width, screen.1, anchor::anchor_of(middle, rule));
        let x = (f64::from(left) + at + f64::from(place.0) * anchor::factors(width, screen.1).0)
            .floor();
        if x >= f64::from(u32::MAX) {
            return None;
        }
        return Some(MirrorRect {
            x: x as u32 & !1,
            y: scaled(place.1, screen.1, 600)?,
            width: anchor::across(MIRROR_SIZE.0, width, screen.1)? & !1,
            height: scaled(MIRROR_SIZE.1, screen.1, 600)?,
        });
    }
    Some(MirrorRect {
        x: (left + scaled(place.0, width, 800)?) & !1,
        y: scaled(place.1, screen.1, 600)?,
        width: scaled(MIRROR_SIZE.0, width, 800)?,
        height: scaled(MIRROR_SIZE.1, screen.1, 600)?,
    })
}

/// The sound spots a frame makes and the sound side's table holds.
pub const MOST_SHARED_SPOTS: u32 = 36;
/// A sound spot's bytes.
pub const SPOT_BYTES: u32 = 0x14;

/// The bytes of the frame's spots handed to the sound side; refused past 36.
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
            mirror_window(
                (0x139, 0x10),
                (1024, 768),
                crate::hud::anchor::Rule::Thirds,
                crate::hud::anchor::BoxRatio::Full
            ),
            Some(MirrorRect {
                x: 0x190,
                y: 0x14,
                width: 0xE4,
                height: 0x50
            })
        );
        assert_eq!(
            mirror_window(
                (u32::MAX, 0),
                (u32::MAX, 600),
                crate::hud::anchor::Rule::Thirds,
                crate::hud::anchor::BoxRatio::Full
            ),
            None
        );
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
        assert_eq!(visible_records([8, 2, 8], 9), [8, 2, 8]);
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

    #[test]
    fn players_parts_of_the_screen() {
        let p = |x, y, width, height| PlayerPart {
            x,
            y,
            width,
            height,
        };
        let screen = (641, 481);
        let beside = Split::of(2, true).unwrap();
        assert_eq!(beside.part(0, screen), Some(p(0, 0, 320, 481)));
        assert_eq!(beside.part(1, screen), Some(p(320, 0, 320, 481)));
        assert_eq!(beside.part(2, screen), None);
        let above = Split::of(2, false).unwrap();
        assert_eq!(above.part(0, screen), Some(p(0, 0, 641, 240)));
        assert_eq!(above.part(1, screen), Some(p(0, 240, 641, 240)));
        let quarters = Split::of(4, true).unwrap();
        assert_eq!(Split::of(3, false), Some(Split::WideTop));
        assert_eq!(quarters, Split::Quarters);
        assert_eq!(quarters.part(1, screen), Some(p(320, 0, 320, 240)));
        assert_eq!(quarters.part(2, screen), Some(p(0, 240, 320, 240)));
        assert_eq!(quarters.part(3, screen), Some(p(320, 240, 320, 240)));
        assert_eq!(quarters.part(4, screen), None);
        assert_eq!(Split::of(1, true), None);
        assert_eq!(Split::of(5, true), None);
    }

    #[test]
    fn many_players_layouts() {
        let p = |x, y, width, height| PlayerPart {
            x,
            y,
            width,
            height,
        };
        let screen = (640, 480);
        assert_eq!(Split::many_layouts(3).len(), 4);
        assert_eq!(Split::many_layouts(4).len(), 3);
        assert!(Split::many_layouts(2).is_empty());
        // The index wraps and the cycle comes back to its start.
        let mut at = 0;
        for _ in 0..4 {
            at = Split::next_many(3, at);
        }
        assert_eq!(at, 0);
        assert_eq!(Split::many(4, 4), Split::many(4, 1));
        assert_eq!(Split::many(2, 0), None);
        let wide = Split::WideTop;
        assert_eq!(wide.part_of(3, 0, screen), Some(p(0, 0, 640, 240)));
        assert_eq!(wide.part_of(3, 2, screen), Some(p(320, 240, 320, 240)));
        assert_eq!(wide.part_of(3, 3, screen), None);
        let rows = Split::Rows;
        assert_eq!(rows.part_of(3, 2, screen), Some(p(0, 320, 640, 160)));
        assert_eq!(rows.part_of(3, 3, screen), None);
        assert_eq!(rows.part_of(4, 3, screen), Some(p(0, 360, 640, 120)));
        let columns = Split::Columns;
        assert_eq!(columns.part_of(4, 3, screen), Some(p(480, 0, 160, 480)));
        assert_eq!(columns.part_of(3, 1, screen), Some(p(213, 0, 213, 480)));
        assert_eq!(
            Split::Quarters.free_part(3, screen),
            Some(p(320, 240, 320, 240))
        );
        assert_eq!(Split::Quarters.free_part(4, screen), None);
    }
}
