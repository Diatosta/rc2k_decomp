//! The course's faces as the frame draws them: a face record is 0x40 bytes (+0 the kind
//! and flags, +1 the object, +2 a word of switches, +4 .. +0x10 four scene points, the
//! fourth the third's again for a triangle, +0x11 .. +0x13 and +0x30 .. +0x38 its lights,
//! +0x20 / +0x21 the second triangle's kind and object). Converted 2026-10-21 from
//! `race::reset_face_lights` 0x45ED70, `face_corners` 0x452444, `face_switches` 0x45322C,
//! `face_texture_coordinates` 0x453267 / `face_texture_coordinates_2` 0x45349F,
//! `face_corners_to_new_points` 0x452722, `face_hide_mask` 0x456056,
//! `gather_record_faces` 0x454C99, `gather_lamps` 0x454E08, `lay_out_course` 0x421BE3,
//! `store_norms` 0x421F02 and the whole-number part of `store_face_normal` 0x421F72.

use crate::Invalid;

/// A face record's size.
pub const FACE_BYTES: usize = 0x40;
/// A corner that is not there (a triangle's fourth).
pub const NO_CORNER: u32 = u32::MAX;
/// The kind byte of the faces whose lights `reset_lights` clears (and of the lit ones).
pub const LIT_KIND: u8 = 0xFF;

/// The faces the course memory's count may give (`race::reset_face_lights`,
/// `race::store_norms`).
pub const MOST_FACES: u32 = 0x10_0000;

/// What `race::reset_face_lights` and `race::store_norms` refuse: no faces (the
/// original's loop tests at its end and counts round) or 0x100000 or more.
pub fn check_face_count(count: u32) -> Result<(), Invalid> {
    if count == 0 || count >= MOST_FACES {
        return Err(Invalid(
            "no faces, or 0x100000 or more: the original counts round",
        ));
    }
    Ok(())
}

/// Where `race::lay_out_course` puts the course's parts (`dword_5CCCB8` ..): from the
/// course memory + 0x100 (after its header), the 0x20-byte records, the faces, the
/// points, what follows and its offset from the memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CourseLayout {
    pub records: u32,
    pub faces: u32,
    pub points: u32,
    pub after: u32,
    pub after_offset: u32,
}

impl CourseLayout {
    /// The header's size before the records.
    pub const HEADER: u32 = 0x100;

    /// `race::lay_out_course`: from the memory's address and the three counts (each
    /// product wrapping).
    pub fn of(memory: u32, records: u32, faces: u32, points: u32) -> Self {
        let first = memory.wrapping_add(Self::HEADER);
        let face_at = first.wrapping_add(records.wrapping_mul(0x20));
        let point_at = face_at.wrapping_add(faces.wrapping_mul(FACE_BYTES as u32));
        let after = point_at.wrapping_add(points.wrapping_mul(12));
        Self {
            records: first,
            faces: face_at,
            points: point_at,
            after,
            after_offset: after.wrapping_sub(memory),
        }
    }

    /// The five dwords as the game keeps them.
    pub fn dwords(&self) -> [u32; 5] {
        [
            self.records,
            self.faces,
            self.points,
            self.after,
            self.after_offset,
        ]
    }
}

/// The faces whose normals `race::store_face_normal` stores: kinds below 0x10.
pub fn takes_normals(kind: u8) -> bool {
    kind < 0x10
}

/// `race::store_face_normal`: whether two of the face's points (+4, +8, +0xC, and +0x10
/// when `quad`, asked only when the first three have none alike) are the same in their
/// low twelve bits.
pub fn repeated_points(points: [u32; 4], quad: impl FnOnce() -> bool) -> bool {
    let same = |x: usize, y: usize| (points[x] ^ points[y]) & 0xFFF == 0;
    same(0, 1) || same(0, 2) || same(1, 2) || (quad() && (same(0, 3) || same(1, 3) || same(2, 3)))
}

/// The switches word's bit for a face with repeated points.
pub const REPEATED: u16 = 0x200;

/// `race::store_face_normal`: the switches word (+2) with `REPEATED` set or cleared.
pub fn with_repeated(word: u16, repeated: bool) -> u16 {
    if repeated {
        word | REPEATED
    } else {
        word & !REPEATED
    }
}

/// `race::store_face_normal`: the two triangles' points whose normals are made (+4, +8,
/// +0xC and +0x10, +0xC, +8), and where each normal goes (+0x18, +0x30, +0x38 and +0x1C,
/// +0x34, +0x3C).
pub const NORMAL_TRIANGLES: [([usize; 3], [u32; 3]); 2] = [
    ([0, 1, 2], [0x18, 0x30, 0x38]),
    ([3, 2, 1], [0x1C, 0x34, 0x3C]),
];

/// `race::store_face_normal`: whether a triangle's normal is made: all three points below
/// the scene's point count (else the work vector is stored as it was).
pub fn normal_made(points: [u32; 3], count: u32) -> bool {
    points.iter().all(|&p| p < count)
}

/// `race::reset_face_lights`: a face of kind 0xFF gets its lights (+0x30, +0x34, +0x38
/// and the bytes +0x11 .. +0x13) 0; the others are left.
pub fn reset_lights(face: &mut [u8; FACE_BYTES]) {
    if face[0] == LIT_KIND {
        face[0x30..0x3C].fill(0);
        face[0x11..0x14].fill(0);
    }
}

/// `race::face_corners`: the face's four points as corners, the fourth `NO_CORNER` when
/// it is the third (a triangle).
pub fn corners(points: [u32; 4]) -> [u32; 4] {
    let mut c = points;
    if c[3] == c[2] {
        c[3] = NO_CORNER;
    }
    c
}

/// `race::face_switches`: 1 for each of the face's two textures that is animated (not
/// 0), else 0.
pub fn switches(textures: [u32; 2]) -> [u32; 2] {
    textures.map(|t| u32::from(t != 0))
}

/// A triangle's texture as `texture_coordinates` reads it (its 0x40-byte record).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureInfo {
    /// +0x20: the size word.
    pub size: u32,
    /// +0x2D, signed: 0 plain, above 0 mapped from the scene's x and z.
    pub mapping: i8,
}

/// What `texture_coordinates` works out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coordinates {
    /// The size word with bit 31 set.
    pub size: u32,
    /// The mapping, sign-extended.
    pub mapping: u32,
    /// 1 when the mapping is 0.
    pub plain: u32,
    /// The four corners' u, v.
    pub uv: [u32; 8],
}

/// `race::face_texture_coordinates` / `_2`: the four corners' u, v from the texture's
/// corner pairs (`pairs`, u16 each, << 8, doubled for sizes 0x190 .. 0x31F); with a
/// mapping above 0 the corners' scene x and z (`corner_xz(k)`, asked only then) shifted by
/// (mapping & 3) + 2 instead, less the first corner's top byte.
pub fn texture_coordinates(
    info: TextureInfo,
    pairs: [(u16, u16); 4],
    mut corner_xz: impl FnMut(usize) -> (u32, u32),
) -> Coordinates {
    let mut uv = [0u32; 8];
    for (k, (u, v)) in pairs.into_iter().enumerate() {
        uv[2 * k] = (u as u32) << 8;
        uv[2 * k + 1] = (v as u32) << 8;
    }
    if (0x190..0x320).contains(&info.size) {
        for v in &mut uv {
            *v <<= 1;
        }
    }
    if info.mapping > 0 {
        let shift = (info.mapping as u32 & 3) + 2;
        for k in 0..4 {
            let (x, z) = corner_xz(k);
            uv[2 * k] = x << shift;
            uv[2 * k + 1] = z << shift;
        }
        let (du, dv) = (uv[0] & 0xFF00_0000, uv[1] & 0xFF00_0000);
        for k in 0..4 {
            uv[2 * k] = uv[2 * k].wrapping_sub(du);
            uv[2 * k + 1] = uv[2 * k + 1].wrapping_sub(dv);
        }
    }
    Coordinates {
        size: info.size | 0x8000_0000,
        mapping: info.mapping as i32 as u32,
        plain: u32::from(info.mapping == 0),
        uv,
    }
}

/// The scene point records' mask (`race::face_corners_to_new_points`: 0x1000 records).
pub const POINT_RECORD_MASK: u32 = 0xFFF;

/// `race::face_corners_to_new_points`: the four new points after the last (`count` + 0 ..
/// 3; all four records are marked unmade), and for each corner that is there (the fourth
/// only when not `NO_CORNER`) the point it is copied from and the new one it then names.
pub fn new_points(count: u32, corners: [u32; 4]) -> ([u32; 4], Vec<(u32, u32)>) {
    let marked = [0, 1, 2, 3].map(|k| count.wrapping_add(k) & POINT_RECORD_MASK);
    let copies = (0..4u32)
        .take_while(|&k| !(k == 3 && corners[3] == NO_CORNER))
        .map(|k| (corners[k as usize], count.wrapping_add(k)))
        .collect();
    (marked, copies)
}

/// `race::face_hide_mask`: the mask from the face's switches word (+2): bits 1 and 3
/// flipped when any is set, or'd with bits 4 and 5 likewise.
pub fn hide_mask(word: u16) -> u32 {
    let word = word as u32;
    let flip = |bits: u32| {
        if word & bits != 0 {
            (word & bits) ^ bits
        } else {
            0
        }
    };
    flip(0xA) | flip(0x30)
}

/// Which list a gathered face goes to (`race::gather_record_faces`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gathered {
    /// Not gathered (a kind other than 0xFF or below 0x10).
    No,
    /// A lit face (kind 0xFF): its object +1, the second tables.
    Lit,
    /// A plain face (kind below 0x10): its object +1, or +0x21 when that is 0.
    Plain,
}

/// `race::gather_record_faces`: how a face of `kind` is gathered (before its hidden test).
pub fn gathered(kind: u8) -> Gathered {
    if kind == LIT_KIND {
        Gathered::Lit
    } else if kind < 0x10 {
        Gathered::Plain
    } else {
        Gathered::No
    }
}

/// `race::gather_record_faces`: a gathered face's object: +1, for a plain face +0x21
/// (`second`, asked only then) when +1 is 0.
pub fn face_object(how: Gathered, first: u8, second: impl FnOnce() -> u8) -> u32 {
    if how == Gathered::Plain && first == 0 {
        second() as u32
    } else {
        first as u32
    }
}

/// The lamps' object (`race::gather_lamps`).
pub const LAMP_OBJECT: u32 = 0x3B;
/// The most lamps (`g_lamps`).
pub const MOST_LAMPS: u32 = 16;
/// A lamp's third dword.
pub const LAMP_COLOUR: u32 = 0x00FF_FFFF;

/// `race::gather_lamps`: whether a lit face of `object` becomes a lamp with `lamps` made.
pub fn takes_lamp(object: u32, lamps: u32) -> bool {
    object == LAMP_OBJECT && lamps != MOST_LAMPS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lights_corners_switches() {
        let mut f = [0xAAu8; FACE_BYTES];
        let kept = f;
        reset_lights(&mut f);
        assert_eq!(f, kept);
        f[0] = 0xFF;
        reset_lights(&mut f);
        assert_eq!(f[0x10..0x15], [0xAA, 0, 0, 0, 0xAA]);
        assert_eq!(
            f[0x2F..0x3D],
            [0xAA, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xAA]
        );
        assert_eq!(corners([1, 2, 3, 3]), [1, 2, 3, NO_CORNER]);
        assert_eq!(corners([1, 2, 3, 4]), [1, 2, 3, 4]);
        assert_eq!(switches([0, 7]), [0, 1]);
    }

    #[test]
    fn coordinates() {
        let pairs = [(1, 2), (3, 4), (0xFFFF, 0), (5, 6)];
        let c = texture_coordinates(
            TextureInfo {
                size: 0x200,
                mapping: 0,
            },
            pairs,
            |_| unreachable!(),
        );
        assert_eq!((c.size, c.mapping, c.plain), (0x8000_0200, 0, 1));
        assert_eq!(
            c.uv,
            [0x200, 0x400, 0x600, 0x800, 0x1FF_FE00, 0, 0xA00, 0xC00]
        );
        let c = texture_coordinates(
            TextureInfo {
                size: 0x100,
                mapping: -1,
            },
            pairs,
            |_| unreachable!(),
        );
        assert_eq!((c.mapping, c.plain, c.uv[0]), (u32::MAX, 0, 0x100));
        let c = texture_coordinates(
            TextureInfo {
                size: 0x100,
                mapping: 5,
            },
            pairs,
            |k| (0x0100_0000 * k as u32 + 0x0080_0001, 0x10 + k as u32),
        );
        // Shift 3: the first corner's top bytes (0x04, 0) taken off.
        assert_eq!(c.uv[0], 0x0000_0008);
        assert_eq!(c.uv[2], 0x0800_0008);
        assert_eq!((c.uv[1], c.uv[7]), (0x80, 0x98));
    }

    #[test]
    fn new_points_and_masks() {
        let (marked, copies) = new_points(0xFFE, [7, 8, 9, NO_CORNER]);
        assert_eq!(marked, [0xFFE, 0xFFF, 0, 1]);
        assert_eq!(copies, [(7, 0xFFE), (8, 0xFFF), (9, 0x1000)]);
        assert_eq!(new_points(0, [1, 2, 3, 4]).1.len(), 4);
        assert_eq!(hide_mask(0), 0);
        assert_eq!(hide_mask(0x2), 0x8);
        assert_eq!(hide_mask(0xA | 0x10), 0x20);
        assert_eq!(hide_mask(0x8 | 0x30), 0x2);
    }

    #[test]
    fn layout_and_normals() {
        assert!(check_face_count(0).is_err() && check_face_count(MOST_FACES).is_err());
        assert!(check_face_count(1).is_ok());
        let l = CourseLayout::of(0x1000, 2, 3, 4);
        assert_eq!(l.dwords(), [0x1100, 0x1140, 0x1200, 0x1230, 0x230]);
        assert!(takes_normals(0xF) && !takes_normals(0x10));
        assert!(repeated_points([0x1005, 0x2005, 9, 7], || unreachable!()));
        assert!(!repeated_points([1, 2, 3, 0x1003], || false));
        assert!(repeated_points([1, 2, 3, 0x1003], || true));
        assert!(!repeated_points([1, 2, 3, 4], || true));
        assert_eq!(with_repeated(0x8001, true), 0x8201);
        assert_eq!(with_repeated(0xFFFF, false), 0xFDFF);
        assert!(normal_made([0, 1, 2], 3) && !normal_made([0, 3, 2], 3));
    }

    #[test]
    fn gathering_and_lamps() {
        assert_eq!(gathered(0xFF), Gathered::Lit);
        assert_eq!(gathered(0xF), Gathered::Plain);
        assert_eq!(gathered(0x10), Gathered::No);
        assert_eq!(face_object(Gathered::Lit, 0, || unreachable!()), 0);
        assert_eq!(face_object(Gathered::Plain, 0, || 9), 9);
        assert_eq!(face_object(Gathered::Plain, 4, || unreachable!()), 4);
        assert!(takes_lamp(0x3B, 15) && !takes_lamp(0x3B, 16) && !takes_lamp(0x3A, 0));
    }
}
