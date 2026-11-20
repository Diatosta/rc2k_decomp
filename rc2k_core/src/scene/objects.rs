//! The scene's objects as they are loaded and listed: the memory they load into, an object's
//! middle and extent, the textures its file's faces use, its files' names, its faces.

use crate::Invalid;

/// The rooms of the objects' points and faces, in bytes: smaller in a race.
pub fn object_rooms(in_race: bool) -> (u32, u32) {
    if in_race {
        (0xC_0000, 0x24_0000)
    } else {
        (0x18_0000, 0x40_0000)
    }
}

/// Where the lowest of each axis starts, and the highest (`HIGHEST_START`).
pub const LOWEST_START: u32 = 0x0064_0000;
pub const HIGHEST_START: u32 = 0xFF9C_0000;

/// An object's measures: the middle, half the extent, the lowest, the highest; three axes each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Measures {
    pub middle: [u32; 3],
    pub half: [u32; 3],
    pub lowest: [u32; 3],
    pub highest: [u32; 3],
}

/// The lowest and highest of each axis over the points, half the extent, and the middle
/// (that plus the lowest); the middle is taken off every point. None for no points.
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

/// The textures a list holds at most.
pub const MOST_TEXTURES: usize = 0x100;
/// The slots with a record of textures (0x40 bytes each).
pub const RECORD_SLOTS: u32 = 0x40;
pub const RECORD_LEN: usize = 0x40;

/// The file's faces (after its 0x2C-byte header and its points, 12 bytes each; a face is
/// 0x10 bytes and as many again a vertex) each name a texture (the low word of +4), listed
/// once in the order met. `dword(offset)` reads the file. Refuses a file of no faces, faces
/// past the room and more textures than the list holds.
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

/// Where a slot's record of `count` textures starts in the records (a long list runs on
/// into the next slots' records), or None for a slot without one. Refuses a list that would
/// leave the records.
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

/// The folders of an object's files by its kind: under 0x190 `files\a\`, under 0x258
/// `files\b\`, under 0x320 `files\g\frnt\`, else `files\s\extr\`.
pub const FOLDER_ENDS: [u32; 3] = [0x190, 0x258, 0x320];

/// The folder of a kind (0 ..= 3).
pub fn folder(kind: u32) -> usize {
    FOLDER_ENDS
        .iter()
        .position(|&end| kind < end)
        .unwrap_or(FOLDER_ENDS.len())
}

/// The four characters written over a name's `????`: thousands, hundreds, tens, units (above
/// 9999 the thousands byte runs past '9').
pub fn number_digits(kind: u32) -> [u8; 4] {
    [
        (kind / 1000) as u8,
        (kind % 1000 / 100) as u8,
        (kind % 100 / 10) as u8,
        (kind % 10) as u8,
    ]
    .map(|digit| digit.wrapping_add(b'0'))
}

/// The faces' table and the listed faces' room (`LIST_ROOM`).
pub const FACES: u32 = 0xC000;
pub const LIST_ROOM: u32 = 0x1000;

/// The object's `count` faces from `first` on (`face(n)` the table's entry), every one
/// checked before anything is listed. Refuses no faces, more than the list holds, a number
/// past the table, a face that is not there.
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
mod tests;
