//! The ground's search: which course face lies under a point (whole-number x, z). The last
//! face found is tried first, then the nine grid cells about the point, each cell's course
//! record and its faces in turn. A face has the point when each of its edges has it on the
//! inner side; a quad is split by a diagonal into two halves. A face is 0x40 bytes: a kind
//! byte, a flags word at +2 and corners at +4 .. +0x10 (a triangle when the corners at +0xC
//! and +0x10 are equal). The contacts walk the same cells and records, every face of each.

use crate::Invalid;

/// No face.
pub const NO_FACE: u32 = u32::MAX;

/// How a face's test ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceTest {
    /// Out at its kind (0x10 or more).
    Kind,
    /// Out at its flags: the flags in the mask.
    Flags(u32),
    /// Out at an edge: its ends are the same place, or the point is on its other side.
    Edge,
    /// Inside: the half (0, or 1 for a quad's second) and its three corners (face offsets).
    Inside { half: u32, corners: [u32; 3] },
}

/// One edge between two corners (face offsets): `side` answers `None` when its ends are the
/// same place, else whether the point lies on the outer side; true when it has the point.
fn inner(side: &mut impl FnMut(u32, u32) -> Option<bool>, from: u32, to: u32) -> bool {
    side(from, to) == Some(false)
}

/// The test of one face: its kind, then its flags (read only for a kind below 0x10) against
/// the mask, then its edges (`corner(offset)` reads a corner word); the first edge failing
/// ends it. A triangle's edges are 4-8, 8-0xC, 0xC-4; a quad's are 8-0xC first, then 0xC-4
/// and 4-8 (the first half), else 0x10-0xC and 8-0x10 (the second).
pub fn face_under_point(
    kind: u8,
    flags: impl FnOnce() -> u16,
    mask: u32,
    corner: impl Fn(u32) -> u32,
    mut side: impl FnMut(u32, u32) -> Option<bool>,
) -> FaceTest {
    if kind >= 0x10 {
        return FaceTest::Kind;
    }
    let f = u32::from(flags()) & mask;
    if f != 0 {
        return FaceTest::Flags(f);
    }
    let first = FaceTest::Inside {
        half: 0,
        corners: [4, 8, 0xC],
    };
    if corner(0xC) == corner(0x10) {
        let all = inner(&mut side, 4, 8) && inner(&mut side, 8, 0xC) && inner(&mut side, 0xC, 4);
        return if all { first } else { FaceTest::Edge };
    }
    match side(8, 0xC) {
        None => FaceTest::Edge,
        Some(false) => {
            if inner(&mut side, 0xC, 4) && inner(&mut side, 4, 8) {
                first
            } else {
                FaceTest::Edge
            }
        }
        Some(true) => {
            if inner(&mut side, 0x10, 0xC) && inner(&mut side, 8, 0x10) {
                FaceTest::Inside {
                    half: 1,
                    corners: [0x10, 0xC, 8],
                }
            } else {
                FaceTest::Edge
            }
        }
    }
}

/// A face number the data may hold.
pub fn check_face(face: u32, face_count: u32) -> Result<(), Invalid> {
    if face >= face_count {
        return Err(Invalid("a face at or past g_courseFaceCount"));
    }
    Ok(())
}

/// A course record below the count.
pub fn check_record(record: u32, record_count: u32) -> Result<(), Invalid> {
    if record >= record_count {
        return Err(Invalid("a course record at or past g_courseRecordCount"));
    }
    Ok(())
}

/// A record's faces `first` ..= `last`: when they would be tried, the last is below the face
/// count.
pub fn check_record_faces(first: u32, last: u32, face_count: u32) -> Result<(), Invalid> {
    if first <= last && first != NO_FACE && last >= face_count {
        return Err(Invalid("a record's faces past g_courseFaceCount"));
    }
    Ok(())
}

/// The faces `first` ..= `last` (none when the first is past the last, unsigned, or -1) tried
/// in order until one has the point; returns whether found, and the face found, one past the
/// last, or the first when none was tried.
pub fn search_faces<E>(
    first: u32,
    last: u32,
    mut try_face: impl FnMut(u32) -> Result<bool, E>,
) -> Result<(bool, u32), E> {
    if first > last || first == NO_FACE {
        return Ok((false, first));
    }
    let mut face = first;
    loop {
        if try_face(face)? {
            return Ok((true, face));
        }
        face = face.wrapping_add(1);
        if face > last {
            return Ok((false, face));
        }
    }
}

/// The cell of (x, z): the key (z >> 21) << 11 + (x >> 21) and the z part left over.
pub fn cell_key(x: u32, z: u32) -> (u32, u32) {
    let z_part = (z >> 21) << 11;
    ((x >> 21).wrapping_add(z_part), z_part)
}

/// The last face found, when it is one to try (not -1, below the count).
pub fn last_face(last: u32, face_count: u32) -> Option<u32> {
    (last != NO_FACE && last < face_count).then_some(last)
}

/// The nine cells tried after the last face, as (x, z) offsets.
pub const CELLS: [(u32, u32); 9] = [
    (0, 0),
    (0xFFE0_0000, 0xFFE0_0000),
    (0, 0xFFE0_0000),
    (0x20_0000, 0xFFE0_0000),
    (0x20_0000, 0),
    (0x20_0000, 0x20_0000),
    (0, 0x20_0000),
    (0xFFE0_0000, 0x20_0000),
    (0xFFE0_0000, 0),
];

/// The last face (`try_last`); else `forget` and `search_cell` on the point (read before each)
/// plus each of `CELLS` until one finds it. Whether found.
pub fn ground_search<E>(
    try_last: impl FnOnce() -> Result<bool, E>,
    forget: impl FnOnce(),
    point: impl Fn() -> (u32, u32),
    mut search_cell: impl FnMut(u32, u32) -> Result<bool, E>,
) -> Result<bool, E> {
    if try_last()? {
        return Ok(true);
    }
    forget();
    for (dx, dz) in CELLS {
        let (x, z) = point();
        if search_cell(x.wrapping_add(dx), z.wrapping_add(dz))? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `cell` on the point (read before each) plus each of `CELLS`, all nine.
pub fn cells_around<E>(
    point: impl Fn() -> (u32, u32),
    mut cell: impl FnMut(u32, u32) -> Result<(), E>,
) -> Result<(), E> {
    for (dx, dz) in CELLS {
        let (x, z) = point();
        cell(x.wrapping_add(dx), z.wrapping_add(dz))?;
    }
    Ok(())
}

/// Refused: faces that would be walked running to -1 or past the face count.
pub fn check_contact_faces(first: u32, last: u32, face_count: u32) -> Result<(), Invalid> {
    if first <= last && first != NO_FACE && (last == NO_FACE || last >= face_count) {
        return Err(Invalid(
            "a record whose faces run to -1 or past g_courseFaceCount",
        ));
    }
    Ok(())
}

/// `contact` on each face `first` ..= `last` (none when the first is past the last, unsigned,
/// or -1); returns one past the last, or the first when none was walked.
pub fn each_face<E: From<Invalid>>(
    first: u32,
    last: u32,
    face_count: u32,
    mut contact: impl FnMut(u32) -> Result<(), E>,
) -> Result<u32, E> {
    check_contact_faces(first, last, face_count)?;
    if first > last || first == NO_FACE {
        return Ok(first);
    }
    for face in first..=last {
        contact(face)?;
    }
    Ok(last + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces() {
        let corners = |c: [u32; 4]| move |o: u32| c[(o / 4 - 1) as usize];
        let all_in = |_: u32, _: u32| Some(false);
        assert_eq!(
            face_under_point(0x10, || 0, 0, corners([0; 4]), all_in),
            FaceTest::Kind
        );
        assert_eq!(
            face_under_point(1, || 0x0140, 0x100, corners([0; 4]), all_in),
            FaceTest::Flags(0x100)
        );
        let tri = corners([1, 2, 3, 3]);
        assert_eq!(
            face_under_point(1, || 0, 0x100, tri, all_in),
            FaceTest::Inside {
                half: 0,
                corners: [4, 8, 0xC]
            }
        );
        let mut tried = Vec::new();
        let out = face_under_point(
            1,
            || 0,
            0,
            tri,
            |a, b| {
                tried.push((a, b));
                Some(a == 0xC)
            },
        );
        assert_eq!(out, FaceTest::Edge);
        assert_eq!(tried, [(4, 8), (8, 0xC), (0xC, 4)]);
        // A quad: the diagonal's outer side takes the second half.
        let quad = corners([1, 2, 3, 4]);
        let mut tried = Vec::new();
        let out = face_under_point(
            1,
            || 0,
            0,
            quad,
            |a, b| {
                tried.push((a, b));
                Some(a == 8 && b == 0xC)
            },
        );
        assert_eq!(
            out,
            FaceTest::Inside {
                half: 1,
                corners: [0x10, 0xC, 8]
            }
        );
        assert_eq!(tried, [(8, 0xC), (0x10, 0xC), (8, 0x10)]);
        assert_eq!(
            face_under_point(1, || 0, 0, quad, |_, _| None),
            FaceTest::Edge
        );
    }

    #[test]
    fn searches() {
        assert_eq!(search_faces::<()>(5, 3, |_| Ok(true)), Ok((false, 5)));
        assert_eq!(
            search_faces::<()>(NO_FACE, NO_FACE, |_| Ok(true)),
            Ok((false, NO_FACE))
        );
        assert_eq!(search_faces::<()>(3, 6, |f| Ok(f == 5)), Ok((true, 5)));
        assert_eq!(search_faces::<()>(3, 6, |_| Ok(false)), Ok((false, 7)));
        assert!(check_record_faces(3, 9, 9).is_err());
        assert!(check_record_faces(9, 3, 9).is_ok());
        assert!(check_record_faces(NO_FACE, NO_FACE, 9).is_ok());
        assert_eq!(cell_key(0x0060_0000, 0x0040_0000), (0x1003, 0x1000));
        assert_eq!(last_face(NO_FACE, 10), None);
        assert_eq!(last_face(10, 10), None);
        assert_eq!(last_face(9, 10), Some(9));
        let mut cells = Vec::new();
        let found = ground_search::<()>(
            || Ok(false),
            || {},
            || (0x0100_0000, 0x0100_0000),
            |x, z| {
                cells.push((x, z));
                Ok(cells.len() == 3)
            },
        );
        assert_eq!(found, Ok(true));
        assert_eq!(cells[1], (0x00E0_0000, 0x00E0_0000));
        assert_eq!(cells[2], (0x0100_0000, 0x00E0_0000));
        let mut n = 0;
        cells_around::<()>(
            || (0, 0),
            |_, _| {
                n += 1;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(n, 9);
        let mut walked = Vec::new();
        let result = each_face::<Invalid>(3, 5, 9, |f| {
            walked.push(f);
            Ok(())
        });
        assert_eq!((result, walked), (Ok(6), vec![3, 4, 5]));
        assert_eq!(each_face::<Invalid>(5, 3, 9, |_| Ok(())), Ok(5));
        assert!(each_face::<Invalid>(3, NO_FACE, 9, |_| Ok(())).is_err());
        assert!(each_face::<Invalid>(3, 9, 9, |_| Ok(())).is_err());
    }
}
