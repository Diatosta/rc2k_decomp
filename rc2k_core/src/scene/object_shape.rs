//! An object file's shape worked out without touching any store: the points, normals and faces a
//! part's group mask takes, and the masks of a car's parts.
//!
//! The file: +4 the number of points, +8 of faces, +0xC of normals; from +0x2C the points
//! (three whole numbers each); then the faces, each 0x10 bytes (+0 its vertices' number, +4 its
//! texture as a word, +7 its group in the low five bits, +0xC its normal) followed by 0x10 bytes a
//! vertex (+0 its point, +4 and +8 texture coordinates, +0xC its normal); then the normals,
//! twelve bytes each.

use crate::Invalid;

/// Entries in the marks tables: how many points, faces or normals a file may have.
pub const MARKS: usize = 0x1000;

const PAST: Invalid = Invalid("the object's file runs past its room");

/// What a part takes out of the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeRequest {
    /// Bit (a face's group) set: the face is taken.
    pub group_mask: u32,
    /// Every point is taken, not only the taken faces'.
    pub all_points: bool,
    /// The object is turned a quarter (z, y - `ground_drop`, -x), as the car kinds are.
    pub turned: bool,
    pub ground_drop: u32,
}

/// One vertex of a taken face: its point and normal are marks (numbers within the shape).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeVertex {
    pub point: u32,
    pub u: u32,
    pub v: u32,
    pub normal: u32,
}

/// A taken face with its points and normals renumbered within the shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeFace {
    pub vertex_count: u32,
    pub texture_word: u32,
    pub third_word: u32,
    pub normal: u32,
    pub corners: Vec<ShapeVertex>,
}

/// The shape of a file that has points and faces.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectShape {
    /// The file's own counts: points, faces, normals.
    pub file_points: u32,
    pub file_faces: u32,
    pub file_normals: u32,
    /// Per file face: whether it is taken; per file point and normal: the number it gets within
    /// the shape, or `u32::MAX` if it is not taken. All `MARKS` long.
    pub face_taken: Vec<bool>,
    pub point_marks: Vec<u32>,
    pub normal_marks: Vec<u32>,
    /// The taken points and normals, in their new order.
    pub points: Vec<[u32; 3]>,
    pub normals: Vec<[u32; 3]>,
    pub faces: Vec<ShapeFace>,
    /// Normals are numbered from the shape's first normal; if the file has none, every face and
    /// vertex gets normal 0.
    pub has_normals: bool,
    /// How many normal references got 0 because the file has no normals.
    pub missing_normals: u32,
    /// The lowest and highest texture word of the taken faces (`u32::MAX` and 0 if none).
    pub lowest_texture: u32,
    pub highest_texture: u32,
    /// The largest distance of a point from the y axis, and the last point's distance.
    pub radius: u32,
    pub last_reach: Option<u32>,
}

/// What a file parses to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    /// No points (`points` None) or no faces (`points` the file's points): nothing is taken.
    Empty {
        points: Option<u32>,
    },
    Shape(Box<ObjectShape>),
}

/// How much a shape adds to the stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Added {
    pub points: usize,
    pub normals: usize,
    pub faces: usize,
    pub face_bytes: usize,
}

impl ObjectShape {
    /// What the shape adds: points, normals, faces and bytes of faces.
    pub fn added(&self) -> Added {
        Added {
            points: self.points.len(),
            normals: self.normals.len(),
            faces: self.faces.len(),
            face_bytes: self
                .faces
                .iter()
                .map(|f| 0x10 + f.corners.len() * 0x10)
                .sum(),
        }
    }

    /// Works out each point's distance from the y axis with `hypot` (as a whole number); the
    /// largest becomes the radius.
    pub fn measure_reach(&mut self, hypot: impl Fn(i32, i32) -> i32) {
        let mut radius = 0u32;
        for point in &self.points {
            let reach = hypot(point[0] as i32, point[2] as i32) as u32;
            self.last_reach = Some(reach);
            if (reach.wrapping_sub(radius) as i32) >= 0 {
                radius = reach;
            }
        }
        self.radius = radius;
    }

    /// The bytes of face `n` as stored: points numbered from `first_point`, normals from
    /// `first_normal`.
    pub fn face_bytes(&self, n: usize, first_point: u32, first_normal: u32) -> Vec<u8> {
        let face = &self.faces[n];
        let normal = |mark: u32| {
            if self.has_normals {
                mark.wrapping_add(first_normal)
            } else {
                0
            }
        };
        let mut bytes = Vec::with_capacity(0x10 + face.corners.len() * 0x10);
        let mut push = |value: u32| bytes.extend_from_slice(&value.to_le_bytes());
        push(face.vertex_count);
        push(face.texture_word);
        push(face.third_word);
        push(normal(face.normal));
        for corner in &face.corners {
            push(corner.point.wrapping_add(first_point));
            push(corner.u);
            push(corner.v);
            push(normal(corner.normal));
        }
        bytes
    }
}

fn dword(file: &[u8], offset: usize) -> Result<u32, Invalid> {
    let end = offset.checked_add(4).filter(|&end| end <= file.len());
    let end = end.ok_or(PAST)?;
    Ok(u32::from_le_bytes(file[offset..end].try_into().unwrap()))
}

fn numbered(marks: &mut [u32]) {
    let mut next = 0;
    for mark in marks {
        if *mark == 0 {
            *mark = u32::MAX;
        } else {
            *mark = next;
            next += 1;
        }
    }
}

fn mark(marks: &mut [u32], number: u32) -> Result<(), Invalid> {
    let slot = marks
        .get_mut(number as usize)
        .ok_or(Invalid("a point's or normal's number of 0x1000 or more"))?;
    *slot = 1;
    Ok(())
}

/// The shape `request` takes out of `file` (the object file's whole room).
pub fn parse_shape(file: &[u8], request: ShapeRequest) -> Result<Parsed, Invalid> {
    let points = dword(file, 4)? as usize;
    if points == 0 {
        return Ok(Parsed::Empty { points: None });
    }
    let face_count = dword(file, 8)? as usize;
    if face_count == 0 {
        return Ok(Parsed::Empty {
            points: Some(points as u32),
        });
    }
    let normals = dword(file, 0xC)? as usize;
    if points > MARKS || face_count > MARKS || normals > MARKS {
        return Err(Invalid("more points, faces or normals than can be marked"));
    }
    let mut point_marks = vec![0u32; MARKS];
    let mut normal_marks = vec![0u32; MARKS];
    let mut face_taken = vec![false; MARKS];
    // Where each taken face starts in the file.
    let mut taken_at = Vec::new();
    let mut at = 0x2C + points * 12;
    for face_flag in face_taken.iter_mut().take(face_count) {
        let vertices = dword(file, at)? as usize;
        let end = vertices
            .checked_mul(0x10)
            .and_then(|bytes| bytes.checked_add(at + 0x10))
            .filter(|&end| end <= file.len())
            .ok_or(PAST)?;
        let taken = request.group_mask & (1 << ((dword(file, at + 4)? >> 24) & 0x1F)) != 0;
        if taken {
            if vertices == 0 {
                return Err(Invalid("a face taken that has no vertices"));
            }
            mark(&mut normal_marks, dword(file, at + 0xC)?)?;
            for vertex in 0..vertices {
                let v = at + 0x10 + vertex * 0x10;
                mark(&mut normal_marks, dword(file, v + 0xC)?)?;
                // Marked by the whole dword, as it is looked up.
                mark(&mut point_marks, dword(file, v)?)?;
            }
            *face_flag = true;
            taken_at.push((at, vertices));
        }
        at = end;
    }
    let normals_at = at;
    if at + normals * 12 > file.len() {
        return Err(PAST);
    }
    if request.all_points {
        point_marks[..points].fill(1);
    }
    numbered(&mut point_marks);
    numbered(&mut normal_marks);

    let mut shape = ObjectShape {
        file_points: points as u32,
        file_faces: face_count as u32,
        file_normals: normals as u32,
        face_taken,
        point_marks,
        normal_marks,
        points: Vec::new(),
        normals: Vec::new(),
        faces: Vec::with_capacity(taken_at.len()),
        has_normals: normals != 0,
        missing_normals: 0,
        lowest_texture: u32::MAX,
        highest_texture: 0,
        radius: 0,
        last_reach: None,
    };
    for n in 0..points {
        if shape.point_marks[n] == u32::MAX {
            continue;
        }
        let at = 0x2C + n * 12;
        let mut point = [dword(file, at)?, dword(file, at + 4)?, dword(file, at + 8)?];
        if request.turned {
            point = [
                point[2],
                point[1].wrapping_sub(request.ground_drop),
                point[0].wrapping_neg(),
            ];
        }
        shape.points.push(point);
    }
    for (at, vertices) in taken_at {
        let mut normal = |number: u32| -> u32 {
            if shape.has_normals {
                shape.normal_marks[number as usize]
            } else {
                shape.missing_normals = shape.missing_normals.wrapping_add(1);
                0
            }
        };
        let vertex_count = dword(file, at)?;
        let texture_word = dword(file, at + 4)?;
        let third_word = dword(file, at + 8)?;
        let face_normal = normal(dword(file, at + 0xC)?);
        let mut corners = Vec::with_capacity(vertices);
        for vertex in 0..vertices {
            let from = at + 0x10 + vertex * 0x10;
            let point = dword(file, from)?;
            let mark = *shape.point_marks.get(point as usize).ok_or(PAST)?;
            corners.push(ShapeVertex {
                point: mark,
                u: dword(file, from + 4)?,
                v: dword(file, from + 8)?,
                normal: normal(dword(file, from + 0xC)?),
            });
        }
        let texture = texture_word & 0xFFFF;
        if texture <= shape.lowest_texture {
            shape.lowest_texture = texture;
        }
        if texture >= shape.highest_texture {
            shape.highest_texture = texture;
        }
        shape.faces.push(ShapeFace {
            vertex_count,
            texture_word,
            third_word,
            normal: face_normal,
            corners,
        });
    }
    for n in 0..normals {
        if shape.normal_marks[n] == u32::MAX {
            continue;
        }
        let at = normals_at + n * 12;
        shape
            .normals
            .push([dword(file, at)?, dword(file, at + 4)?, dword(file, at + 8)?]);
    }
    Ok(Parsed::Shape(Box::new(shape)))
}

// ---- a car's parts ----

/// A car's parts that have a mask of their own.
pub const PARTS: usize = 11;
/// The cell of each part's mask in the group masks; the rest-of-the-car mask is `REST_CELL`.
pub const PART_CELLS: [usize; PARTS] = [0, 1, 2, 3, 6, 9, 10, 7, 8, 11, 12];
pub const REST_CELL: usize = 4;
/// The part whose mask gets more bits for three of the known kinds.
const WIDER_PART: usize = 5;

/// The masks of a car's parts: one bit for the part's group, and everything but the first four
/// groups for the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarMasks {
    pub parts: [u32; PARTS],
    pub rest: u32,
}

/// The masks from the kind's group numbers (`u32::MAX`: no such group, mask 0); `known` is the
/// kind's place among the known kinds.
pub fn car_masks(groups: [u32; PARTS], known: u32) -> CarMasks {
    let mut parts = [0u32; PARTS];
    for (part, &group) in groups.iter().enumerate() {
        if group == u32::MAX {
            continue;
        }
        parts[part] = 1u32 << (group & 31);
        if part == WIDER_PART {
            parts[part] |= match known {
                4 | 7 => 0x0600_0000,
                0xD => 0x0300_0000,
                _ => 0,
            };
        }
    }
    let rest = parts[1..4]
        .iter()
        .fold(u32::MAX.wrapping_sub(parts[0]), |rest, &mask| {
            rest.wrapping_sub(mask)
        });
    CarMasks { parts, rest }
}

/// One slot of a car's first part: its distance from the car's first slot, its mask, and whether
/// it is measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarSlot {
    pub offset: u32,
    pub mask: u32,
    pub measured: bool,
}

/// The slots part 0 loads, in order: the rest and the first four groups, then eleven measured
/// parts.
pub fn first_part_slots(masks: &CarMasks) -> [CarSlot; 16] {
    let plain = |offset: u32, mask: u32| CarSlot {
        offset,
        mask,
        measured: false,
    };
    let measured = |offset: u32, part: usize| CarSlot {
        offset,
        mask: masks.parts[part],
        measured: true,
    };
    [
        plain(0, masks.rest),
        plain(1, masks.parts[0]),
        plain(2, masks.parts[1]),
        plain(3, masks.parts[2]),
        plain(4, masks.parts[3]),
        measured(5, 0),
        measured(6, 1),
        measured(7, 2),
        measured(8, 3),
        measured(9, 4),
        measured(0x10, 7),
        measured(0x11, 8),
        measured(0xD, 5),
        measured(0xE, 6),
        measured(0xC, 9),
        measured(0xF, 10),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(file: &mut Vec<u8>, words: &[u32]) {
        for w in words {
            file.extend_from_slice(&w.to_le_bytes());
        }
    }

    /// Two faces (groups 1 and 2) over three points, one normal.
    fn sample() -> Vec<u8> {
        let mut f = vec![0u8; 4];
        put(&mut f, &[3, 2, 1]);
        f.resize(0x2C, 0);
        put(&mut f, &[10, 20, 30, 40, 50, 60, 70, 80, 90]);
        // Face 0: group 1, three vertices on points 0, 1, 2.
        put(&mut f, &[3, 5 | (1 << 24), 0, 0]);
        for p in 0..3 {
            put(&mut f, &[p, 1, 2, 0]);
        }
        // Face 1: group 2, one vertex on point 2.
        put(&mut f, &[1, 6 | (2 << 24), 0, 0]);
        put(&mut f, &[2, 3, 4, 0]);
        put(&mut f, &[100, 200, 300]);
        f.resize(0x40000, 0);
        f
    }

    fn request(mask: u32) -> ShapeRequest {
        ShapeRequest {
            group_mask: mask,
            all_points: false,
            turned: false,
            ground_drop: 0,
        }
    }

    #[test]
    fn a_mask_takes_its_faces_and_their_points() {
        let Parsed::Shape(shape) = parse_shape(&sample(), request(1 << 2)).unwrap() else {
            panic!("a shape");
        };
        assert_eq!(shape.faces.len(), 1);
        assert_eq!(shape.points, vec![[70, 80, 90]]);
        assert_eq!(shape.normals, vec![[100, 200, 300]]);
        assert_eq!(shape.point_marks[2], 0);
        assert_eq!(shape.point_marks[0], u32::MAX);
        assert!(!shape.face_taken[0] && shape.face_taken[1]);
        let bytes = shape.face_bytes(0, 10, 7);
        let words: Vec<u32> = bytes
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(words, vec![1, 6 | (2 << 24), 0, 7, 10, 3, 4, 7]);
        assert_eq!(shape.added().face_bytes, 0x20);
    }

    #[test]
    fn all_points_and_the_turn_apply() {
        let mut req = request(1 << 2);
        req.all_points = true;
        req.turned = true;
        req.ground_drop = 5;
        let Parsed::Shape(mut shape) = parse_shape(&sample(), req).unwrap() else {
            panic!("a shape");
        };
        assert_eq!(shape.points[0], [30, 15, 10u32.wrapping_neg()]);
        shape.measure_reach(|a, b| a.abs().max(b.abs()));
        assert_eq!(shape.radius, 90);
        assert_eq!(shape.last_reach, Some(90));
    }

    #[test]
    fn a_file_without_normals_gives_normal_zero() {
        let mut file = sample();
        file[0xC..0x10].copy_from_slice(&0u32.to_le_bytes());
        let Parsed::Shape(shape) = parse_shape(&file, request(1 << 1)).unwrap() else {
            panic!("a shape");
        };
        assert_eq!(shape.missing_normals, 4);
        assert_eq!(shape.face_bytes(0, 0, 99)[12..16], [0; 4]);
    }

    #[test]
    fn empty_files_and_bad_files() {
        let mut file = sample();
        file[8..12].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(
            parse_shape(&file, request(1)).unwrap(),
            Parsed::Empty { points: Some(3) }
        );
        file[4..8].copy_from_slice(&0u32.to_le_bytes());
        assert_eq!(
            parse_shape(&file, request(1)).unwrap(),
            Parsed::Empty { points: None }
        );
        assert!(parse_shape(&[0; 6], request(1)).is_err());
    }

    #[test]
    fn car_masks_have_one_bit_each_and_a_rest() {
        let mut groups = [u32::MAX; PARTS];
        groups[0] = 0;
        groups[1] = 3;
        groups[5] = 1;
        let masks = car_masks(groups, 4);
        assert_eq!(masks.parts[0], 1);
        assert_eq!(masks.parts[1], 8);
        assert_eq!(masks.parts[2], 0);
        assert_eq!(masks.parts[5], 2 | 0x0600_0000);
        assert_eq!(masks.rest, u32::MAX - 9);
        let slots = first_part_slots(&masks);
        assert_eq!(slots[0].mask, masks.rest);
        assert!(slots[5..].iter().all(|s| s.measured));
        assert_eq!(slots[12].mask, masks.parts[5]);
    }
}
