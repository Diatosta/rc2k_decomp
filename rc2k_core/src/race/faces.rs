//! The course's faces as the frame draws them: a face record is 0x40 bytes (+0 the kind
//! and flags, +1 the object, +2 a word of switches, +4 .. +0x10 four scene points, the
//! fourth the third's again for a triangle, +0x11 .. +0x13 and +0x30 .. +0x38 its lights,
//! +0x20 / +0x21 the second triangle's kind and object). Converted 2026-10-21 from
//! `race::reset_face_lights` 0x45ED70, `face_corners` 0x452444, `face_switches` 0x45322C,
//! `face_texture_coordinates` 0x453267 / `face_texture_coordinates_2` 0x45349F,
//! `face_corners_to_new_points` 0x452722, `face_hide_mask` 0x456056,
//! `gather_record_faces` 0x454C99, `gather_lamps` 0x454E08, `lay_out_course` 0x421BE3,
//! `store_norms` 0x421F02 and the whole-number part of `store_face_normal` 0x421F72;
//! the faces' drawing decisions, depths and materials converted 2026-10-21 from
//! `race::face_depth_test_off` 0x452358, `face_depth_test_on` 0x45237A, `face_lights`
//! 0x453715, `face_hidden` 0x454D92, `face_depth` 0x4552D8, `add_depth_entry` 0x454D2A
//! and `face_materials` 0x45265C.

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

/// `race::face_depth_test_off`: whether the depth test goes off for a face: no texture
/// animation (`dword_63DA2C` 0) and polygon mode 4.
pub fn depth_test_off(animation: u32, mode: u32) -> bool {
    animation == 0 && mode == 4
}

/// `race::face_depth_test_on`: whether the depth test goes on after a face: when it was
/// not turned off (`dword_63D9E0` 0; the test looks the wrong way round, kept).
pub fn depth_test_on(turned_off: u32) -> bool {
    turned_off == 0
}

/// `race::face_lights`: whether the face's corners take the lamps' light: lamps wanted
/// (`g_lampsWanted`) and not a plain mode-4 face.
pub fn corners_lit(wanted: u32, animation: u32, mode: u32) -> bool {
    wanted != 0 && !depth_test_off(animation, mode)
}

/// `race::face_hidden`: the face's switch word's bits the view hides (the shown view's
/// `dword_606478` entry 0 and `dword_621480` set; the word asked only then): Some(bits)
/// when it is hidden by them, None to go on to the depth test (the bits, 0, in eax only
/// when asked).
pub fn hidden_by_switches(
    view_shows: u32,
    hiding: u32,
    bits: impl FnOnce() -> u32,
) -> (Option<u32>, bool) {
    if view_shows == 0 && hiding != 0 {
        let b = bits();
        (Some(b), b != 0)
    } else {
        (None, false)
    }
}

/// `race::face_hidden`: hidden by depth: the depth (whole) below |x| (the sign of the
/// difference).
pub fn hidden_by_depth(depth: u32, x: u32) -> bool {
    (depth.wrapping_sub(x) as i32) < 0
}

/// How `race::face_depth` keys a face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DepthBy {
    /// Kind 0x10 or more: its first point's depth.
    First,
    /// Its object's record +0xE bit 1: the deepest of its four points.
    Deepest,
    /// Else the four points' sum over four.
    Average,
}

/// `race::face_depth`: how a face is keyed (the object's flags asked only below 0x10).
pub fn depth_by(kind: u8, object_flags: impl FnOnce() -> u8) -> DepthBy {
    if kind >= 0x10 {
        DepthBy::First
    } else if object_flags() & 2 != 0 {
        DepthBy::Deepest
    } else {
        DepthBy::Average
    }
}

/// The deeper of two whole depths (signed; the later of equals).
pub fn deeper(key: u32, depth: u32) -> u32 {
    if (depth.wrapping_sub(key) as i32) >= 0 {
        depth
    } else {
        key
    }
}

/// The average's key: the four depths' rounded sum over four (arithmetic).
pub fn average_key(sum: i32) -> u32 {
    (sum >> 2) as u32
}

/// A face's depth key (`dword_63DC40`): the depth + 0x2000000, logical >> 10.
pub fn depth_key(depth: u32) -> u32 {
    depth.wrapping_add(0x200_0000) >> 10
}

/// The depth entries' room (`dword_9D3730`).
pub const MOST_DEPTH_ENTRIES: u32 = 0x1000;

/// `race::add_depth_entry`: refused past the room (the original writes past the table).
pub fn depth_entry_room(count: u32) -> Result<(), Invalid> {
    if count >= MOST_DEPTH_ENTRIES {
        return Err(Invalid("a depth entry past the 0x1000"));
    }
    Ok(())
}

/// `race::face_materials`: a triangle's kind's material bits: the low three and bit 3.
pub fn material_bits(kind: u8) -> (u32, u32) {
    ((kind & 7) as u32, (kind & 8) as u32)
}

/// `race::face_materials`: the animation frame's place in `byte_609384` (8 frames an
/// object) from the rounded clock product (& 7).
pub fn frame_place(object: u32, frame: u32) -> (u32, u32) {
    let f = frame & 7;
    (f, f + object * 8)
}

/// A course face (`CourseFace`, 0x40 bytes), its fields by name. The same bytes mean
/// different things for the different kinds (a checkpoint face's +0xC is its next link,
/// the fourth point's dword +0x10 shares its high bytes with the lights +0x11 .. +0x13),
/// so the record stays its bytes and each reading is a method.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Face(pub [u8; FACE_BYTES]);

impl Default for Face {
    fn default() -> Self {
        Self([0; FACE_BYTES])
    }
}

impl Face {
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.0[at], self.0[at + 1]])
    }

    fn dword(&self, at: usize) -> u32 {
        u32::from_le_bytes(self.0[at..at + 4].try_into().unwrap_or_default())
    }

    /// +0: the kind and flags (`LIT_KIND` for the lit faces and the cameras).
    pub fn kind(&self) -> u8 {
        self.0[0]
    }

    /// +1: the object.
    pub fn object(&self) -> u8 {
        self.0[1]
    }

    /// +2: the switches word (`REPEATED`, `EXTRA_LIGHTS`, 0x1000 for a wheel's surface 5).
    pub fn switches(&self) -> u16 {
        self.word(2)
    }

    pub fn set_switches(&mut self, word: u16) {
        self.0[2..4].copy_from_slice(&word.to_le_bytes());
    }

    /// +4 + 4 `n`: point `n` (0 .. 3) of the scene points.
    pub fn point(&self, n: usize) -> u32 {
        self.dword(4 + 4 * (n & 3))
    }

    /// Triangle `t`'s (0, 1) normal as stored (`NORMAL_TRIANGLES`): x, y, z, floats' bits.
    pub fn normal(&self, t: usize) -> [u32; 3] {
        NORMAL_TRIANGLES[t & 1].1.map(|at| self.dword(at as usize))
    }

    /// A checkpoint face's +0xC: the next checkpoint face in its chain.
    pub fn next_link(&self) -> u32 {
        self.dword(0xC)
    }

    /// A sound spot face's +8 (point 1's low byte): its sound.
    pub fn sound(&self) -> u8 {
        self.0[8]
    }

    /// A checkpoint face's +0xA and +0xB (point 1's high bytes): the computer driver's
    /// bends.
    pub fn bends(&self) -> (u8, u8) {
        (self.0[0xA], self.0[0xB])
    }

    /// `physics::touch_face`'s marks: +9 0 when `clear_9`, then +0x11 `mark`, +0x13 0,
    /// +0x12 `angle`.
    pub fn touch(&mut self, mark: u8, angle: u8, clear_9: bool) {
        if clear_9 {
            self.0[9] = 0;
        }
        self.0[0x11] = mark;
        self.0[0x13] = 0;
        self.0[0x12] = angle;
    }

    /// +0x11: the mark a touch leaves (0 untouched).
    pub fn mark(&self) -> u8 {
        self.0[0x11]
    }

    pub fn set_mark(&mut self, mark: u8) {
        self.0[0x11] = mark;
    }

    /// +0x21: the second triangle's object.
    pub fn second_object(&self) -> u8 {
        self.0[0x21]
    }

    /// A checkpoint face's +0x26: its check (0 and 1 are none).
    pub fn check(&self) -> u8 {
        self.0[0x26]
    }

    /// The extra lights' first lamp indices (+0x28 + `k`) and second (+0x2C + `k`).
    pub fn lamps(&self, k: usize) -> (u8, u8) {
        (self.0[0x28 + (k & 3)], self.0[0x2C + (k & 3)])
    }

    /// A checkpoint face's map place (+0x2C, +0x2E); a map anchor when not both 0.
    pub fn map_place(&self) -> (u16, u16) {
        (self.word(0x2C), self.word(0x2E))
    }
}

/// A course record (0x20 bytes): its fields by name.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Record(pub [u8; RECORD_BYTES]);

/// A course record's size.
pub const RECORD_BYTES: usize = 0x20;

impl Default for Record {
    fn default() -> Self {
        Self([0; RECORD_BYTES])
    }
}

impl Record {
    /// +0: the first dword (its low four bits and bits 11 .. 14 the kind the records are
    /// grouped by).
    pub fn head(&self) -> u32 {
        u32::from_le_bytes(self.0[0..4].try_into().unwrap_or_default())
    }

    /// +4: a scene point record's point number.
    pub fn number(&self) -> u32 {
        u32::from_le_bytes(self.0[4..8].try_into().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_and_record_fields() {
        let mut b = [0u8; FACE_BYTES];
        for (n, x) in b.iter_mut().enumerate() {
            *x = n as u8;
        }
        let mut f = Face(b);
        assert_eq!((f.kind(), f.object(), f.switches()), (0, 1, 0x0302));
        assert_eq!(f.point(0), 0x0706_0504);
        assert_eq!(f.point(3), 0x1312_1110);
        assert_eq!(f.next_link(), f.point(2));
        assert_eq!((f.mark(), f.second_object(), f.check()), (0x11, 0x21, 0x26));
        assert_eq!(f.lamps(1), (0x29, 0x2D));
        assert_eq!(f.map_place(), (0x2D2C, 0x2F2E));
        f.set_switches(0xBEEF);
        assert_eq!((f.0[2], f.0[3], f.0[4]), (0xEF, 0xBE, 4));
        let r = Record(b[..RECORD_BYTES].try_into().unwrap());
        assert_eq!((r.head(), r.number()), (0x0302_0100, 0x0706_0504));
    }

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

    #[test]
    fn drawing_decisions() {
        assert!(depth_test_off(0, 4) && !depth_test_off(1, 4) && !depth_test_off(0, 3));
        assert!(depth_test_on(0) && !depth_test_on(1));
        assert!(corners_lit(1, 0, 3) && !corners_lit(1, 0, 4) && !corners_lit(0, 1, 3));
        assert_eq!(hidden_by_switches(0, 1, || 4), (Some(4), true));
        assert_eq!(hidden_by_switches(0, 1, || 0), (Some(0), false));
        assert_eq!(hidden_by_switches(1, 1, || unreachable!()), (None, false));
        assert!(hidden_by_depth(3, 4) && !hidden_by_depth(4, 4));
        assert!(!hidden_by_depth(0x8000_0000, 1));
        assert_eq!(depth_by(0x10, || unreachable!()), DepthBy::First);
        assert_eq!(depth_by(3, || 2), DepthBy::Deepest);
        assert_eq!(depth_by(3, || 1), DepthBy::Average);
        assert_eq!(deeper(5, 7), 7);
        assert_eq!(deeper(5, u32::MAX), 5);
        assert_eq!(average_key(-8), (-2i32) as u32);
        assert_eq!(depth_key(0x400), 0x8001);
        assert_eq!(depth_key(0xFE00_0000), 0);
        assert!(depth_entry_room(0xFFF).is_ok() && depth_entry_room(0x1000).is_err());
        assert_eq!(material_bits(0x1F), (7, 8));
        assert_eq!(frame_place(3, 0x1D), (5, 29));
    }
}
