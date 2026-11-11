//! The scene's objects as they are loaded and listed (from
//! `scene::measure_object` 0x4222B8, `list_object_textures` 0x426ACA,
//! `set_object_file_names` 0x43964A, `list_object_faces` 0x45048C and
//! `allocate_object_memory` 0x41F814): the memory objects load into, an object's middle
//! and extent, the textures its file's faces use, its files' names, its faces looked up.

use crate::Invalid;

/// `allocate_object_memory`: the rooms of the objects' points and faces, in bytes, taken
/// as one block: smaller in a race.
pub fn object_rooms(in_race: bool) -> (u32, u32) {
    if in_race {
        (0xC_0000, 0x24_0000)
    } else {
        (0x18_0000, 0x40_0000)
    }
}

/// Where the lowest of each axis starts (`measure_object`), and the highest.
pub const LOWEST_START: u32 = 0x0064_0000;
pub const HIGHEST_START: u32 = 0xFF9C_0000;

/// An object's measures (`dword_634B70` ..: the middle, half the extent, the lowest, the
/// highest; three axes each).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measures {
    pub middle: [u32; 3],
    pub half: [u32; 3],
    pub lowest: [u32; 3],
    pub highest: [u32; 3],
}

/// `measure_object`: the lowest and highest of each axis over the points (compared by the
/// difference's sign, as the original does), half the extent (an arithmetic shift), the
/// middle that plus the lowest; the middle taken off every point. None for no points (the
/// original then writes the middle 0 and the two starts, and leaves the half extents).
pub fn measure(points: &mut [[u32; 3]]) -> Option<Measures> {
    if points.is_empty() {
        return None;
    }
    let (mut lowest, mut highest) = ([LOWEST_START; 3], [HIGHEST_START; 3]);
    for point in points.iter() {
        for axis in 0..3 {
            if (point[axis].wrapping_sub(lowest[axis]) as i32) < 0 {
                lowest[axis] = point[axis];
            }
            if (point[axis].wrapping_sub(highest[axis]) as i32) >= 0 {
                highest[axis] = point[axis];
            }
        }
    }
    let half: [u32; 3] =
        std::array::from_fn(|n| ((highest[n].wrapping_sub(lowest[n]) as i32) >> 1) as u32);
    let middle: [u32; 3] = std::array::from_fn(|n| half[n].wrapping_add(lowest[n]));
    for point in points.iter_mut() {
        for axis in 0..3 {
            point[axis] = point[axis].wrapping_sub(middle[axis]);
        }
    }
    Some(Measures {
        middle,
        half,
        lowest,
        highest,
    })
}

/// The textures a list holds at most (`dword_631B6C`).
pub const MOST_TEXTURES: usize = 0x100;
/// The slots with a record of textures (0x40 bytes each, `unk_5DCEBC`).
pub const RECORD_SLOTS: u32 = 0x40;
pub const RECORD_LEN: usize = 0x40;

/// `list_object_textures`: the file's faces (after its header of 0x2C bytes and its points,
/// 12 bytes each; a face 0x10 bytes and as many again a vertex) each name a texture (the
/// low word of +4), listed once in the order met. `dword(offset)` reads the file, refusing
/// past its room. Refuses a file of no faces (the original goes through 2^32), faces past
/// the room and more textures than the list holds.
pub fn texture_list(dword: impl Fn(usize) -> Result<u32, Invalid>) -> Result<Vec<u32>, Invalid> {
    let past = Invalid("the object's file runs past its room");
    let (points, faces) = (dword(4)?, dword(8)?);
    if faces == 0 {
        return Err(Invalid("an object's file of no faces"));
    }
    let mut at = (points as usize)
        .checked_mul(12)
        .and_then(|bytes| bytes.checked_add(0x2C))
        .ok_or(past)?;
    let mut textures: Vec<u32> = Vec::new();
    for _ in 0..faces {
        let vertices = dword(at)?;
        let texture = dword(at + 4)? & 0xFFFF;
        if !textures.contains(&texture) {
            if textures.len() == MOST_TEXTURES {
                return Err(Invalid("more textures than the list holds"));
            }
            textures.push(texture);
        }
        at = (vertices as usize)
            .checked_mul(0x10)
            .and_then(|bytes| bytes.checked_add(at + 0x10))
            .ok_or(past)?;
    }
    Ok(textures)
}

/// Where a slot's record of `count` textures starts in the records (a slot under 0x40;
/// a long list runs on into the next slots' records, as in the original), or None for a
/// slot without one. Refuses a list that would leave the records.
pub fn texture_record(slot: u32, count: usize) -> Result<Option<usize>, Invalid> {
    if slot >= RECORD_SLOTS {
        return Ok(None);
    }
    let record = slot as usize * RECORD_LEN;
    if record + 4 + count * 4 > RECORD_SLOTS as usize * RECORD_LEN {
        return Err(Invalid("a texture list that leaves the slots' records"));
    }
    Ok(Some(record))
}

/// The four folders of an object's files, by its kind (`set_object_file_names`): under
/// 0x190 `files\a\`, under 0x258 `files\b\`, under 0x320 `files\g\frnt\`, else
/// `files\s\extr\`.
pub const FOLDER_ENDS: [u32; 3] = [0x190, 0x258, 0x320];

/// The folder of a kind (0 .. 3).
pub fn folder(kind: u32) -> usize {
    FOLDER_ENDS
        .iter()
        .position(|&end| kind < end)
        .unwrap_or(FOLDER_ENDS.len())
}

/// The four characters written over a name's `????`: thousands, hundreds, tens, units
/// (the thousands as one byte: above 9999 another character).
pub fn number_digits(kind: u32) -> [u8; 4] {
    [
        (kind / 1000) as u8,
        (kind % 1000 / 100) as u8,
        (kind % 100 / 10) as u8,
        (kind % 10) as u8,
    ]
    .map(|digit| digit.wrapping_add(b'0'))
}

/// The faces' table (`g_faces`) and the listed faces' room.
pub const FACES: u32 = 0xC000;
pub const LIST_ROOM: u32 = 0x1000;

/// `list_object_faces`' look-up: the object's `count` faces from `first` on (`face(n)`
/// the table's entry, read in order), every one before anything is listed. Refuses no
/// faces (the original goes through 2^32), more than the list holds, a number past the
/// table, a face that is not there.
pub fn object_faces(
    first: u32,
    count: u32,
    face: impl Fn(u32) -> u32,
) -> Result<Vec<u32>, Invalid> {
    if count == 0 || count > LIST_ROOM {
        return Err(Invalid(
            "an object of no faces, or of more than the list holds",
        ));
    }
    let mut faces = Vec::with_capacity(count as usize);
    for n in 0..count {
        let number = n.wrapping_add(first);
        if number >= FACES {
            return Err(Invalid("a face's number past the table"));
        }
        let f = face(number);
        if f == 0 {
            return Err(Invalid("a face that is not there"));
        }
        faces.push(f);
    }
    Ok(faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms() {
        assert_eq!(object_rooms(true), (0xC_0000, 0x24_0000));
        assert_eq!(object_rooms(false).0 + object_rooms(false).1, 0x58_0000);
    }

    #[test]
    fn measures() {
        assert_eq!(measure(&mut []), None);
        let mut points = [[10, 20, 30], [30, 0, 0xFFFF_FFF6]];
        let m = measure(&mut points).unwrap();
        assert_eq!(m.lowest, [10, 0, 0xFFFF_FFF6]);
        assert_eq!(m.highest, [30, 20, 30]);
        assert_eq!(m.half, [10, 10, 20]);
        assert_eq!(m.middle, [20, 10, 10]);
        assert_eq!(
            points,
            [[0xFFFF_FFF6, 10, 20], [10, 0xFFFF_FFF6, 0xFFFF_FFEC]]
        );
    }

    #[test]
    fn textures() {
        // Two points, three faces: 1 vertex texture 5, 0 vertices texture 0x10005, 2
        // vertices texture 7.
        let mut file = vec![0u32; 0x2C / 4];
        file[1] = 2;
        file[2] = 3;
        file.extend([0; 6]);
        file.extend([1, 5, 0, 0, 9, 9, 9, 9]);
        file.extend([0, 0x10005, 0, 0]);
        file.extend([2, 7, 0, 0]);
        file.extend([0; 8]);
        let dword = |o: usize| {
            file.get(o / 4)
                .copied()
                .ok_or(Invalid("the object's file runs past its room"))
        };
        assert_eq!(texture_list(dword), Ok(vec![5, 7]));
        file[2] = 0;
        let dword = |o: usize| Ok(file[o / 4]);
        assert!(texture_list(dword).is_err());
        assert_eq!(texture_record(0x40, 300), Ok(None));
        assert_eq!(texture_record(1, 15), Ok(Some(0x40)));
        assert!(texture_record(0x3F, 15).is_ok());
        assert!(texture_record(0x3F, 16).is_err());
    }

    #[test]
    fn names() {
        assert_eq!(
            [0, 0x18F, 0x190, 0x257, 0x258, 0x31F, 0x320, u32::MAX].map(folder),
            [0, 0, 1, 1, 2, 2, 3, 3]
        );
        assert_eq!(number_digits(1234), *b"1234");
        assert_eq!(number_digits(10_000)[0], b'0' + 10);
    }

    #[test]
    fn faces() {
        assert_eq!(object_faces(5, 2, |n| n * 0x10), Ok(vec![0x50, 0x60]));
        assert!(object_faces(5, 0, |n| n).is_err());
        assert!(object_faces(FACES - 1, 2, |n| n).is_err());
        assert!(object_faces(0, 2, |n| n).is_err());
    }
}
