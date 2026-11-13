//! The stage's objects, pictures, light records and courses as loading sets them up: object
//! slot kinds, texture kind pictures, courses on the disc, lights' data and crash matrix angles.

use crate::Invalid;

// ---- the object slots' kinds ----

/// The eight fixed kinds.
pub const FIXED_KINDS: [u32; 8] = [0xCF, 0x79, 0xCC, 0xCD, 0xC8, 0xC9, 0xCA, 0xCB];
/// The slots set from their defaults: 1 ..= 0x3F.
pub const DEFAULT_SLOTS: std::ops::Range<u32> = 1..0x40;
/// The cars' slots start here, `CAR_SLOT_STEP` a car.
pub const FIRST_CAR_SLOT: u32 = 0x50;
/// Slots a car takes.
pub const CAR_SLOT_STEP: u32 = 0x14;
/// The car blocks there are.
pub const CAR_BLOCKS: u32 = 8;

/// A car's three kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarKind {
    /// The scene's known kind.
    Known,
    /// The kind from table 11.
    Table11,
    /// The kind from table 10.
    Table10,
}

/// A car's kinds in order, each with its slot past the car's first.
pub const CAR_KINDS: [(CarKind, u32); 3] = [
    (CarKind::Known, 0),
    (CarKind::Table11, 11),
    (CarKind::Table10, 10),
];

/// Car n's first slot.
pub fn car_slot(n: u32) -> u32 {
    FIRST_CAR_SLOT + CAR_SLOT_STEP * n
}

/// Refuses more cars than car blocks.
pub fn check_object_cars(cars: u32) -> Result<(), Invalid> {
    if cars > CAR_BLOCKS {
        Err(Invalid("more cars than the eight blocks"))
    } else {
        Ok(())
    }
}

// ---- the texture kinds' pictures ----

/// The texture kinds' pictures: 0x258 dwords.
pub const PICTURE_KINDS: usize = 0x258;
/// The stage prefs' picture lists copied in: (first kind, dwords).
pub const PREF_PIECES: [(usize, usize); 3] = [(0, 0x34), (0x40, 0x30), (0x80, 8)];
/// The kinds whose pictures are fixed: (kind, picture).
pub const FIXED_PICTURES: [(usize, u32); 23] = [
    (0xC3, 0x7A8),
    (0xC8, 0x7CF),
    (0xD3, 0x708),
    (0xD4, 0x71A),
    (0xCA, 0x384),
    (0xC4, 0x6FE),
    (0xC5, 0x6FF),
    (0xC9, 0x700),
    (0xD2, 0x722),
    (0xDC, 0x70C),
    (0xDD, 0x70E),
    (0xDE, 0x710),
    (0xDF, 0x712),
    (0xE0, 0x714),
    (0xE1, 0x716),
    (0x21C, 0x71E),
    (0x21D, 0x720),
    (0xC0, 0x708),
    (0xC1, 0x70A),
    (0xC2, 0x706),
    (0x257, 0x71C),
    (0xC6, 0x73A),
    (0xC7, 0x73C),
];
/// The cars' picture kinds start here: eight kinds a car.
pub const CAR_PICTURE_KINDS: usize = 0x100;
pub const PICTURES_A_CAR: usize = 8;

/// A car's eight pictures: its kind's picture and the seven after it.
pub fn car_pictures(first: u32) -> [u32; PICTURES_A_CAR] {
    std::array::from_fn(|k| first.wrapping_add(k as u32))
}

/// Refuses more cars than car blocks.
pub fn check_picture_cars(cars: u32) -> Result<(), Invalid> {
    if cars > CAR_BLOCKS {
        Err(Invalid("more than 8 cars"))
    } else {
        Ok(())
    }
}

// ---- the courses there ----

/// The courses looked for.
pub const COURSES: u32 = 100;
/// The bytes read from a course's file to find it there.
pub const PROBE_BYTES: u32 = 4;

/// Whether a course is there: its files tried in order (`srclvls\levelNN\cdat00c`, then
/// `files\levels\levelNN\cdat00c.sq`); the first read that gives all four bytes wins.
pub fn course_present<E>(
    files: u32,
    mut read: impl FnMut(u32) -> Result<u32, E>,
) -> Result<bool, E> {
    for file in 0..files {
        if read(file)? == PROBE_BYTES {
            return Ok(true);
        }
    }
    Ok(false)
}

// ---- the lights' data ----

/// The lights' records: 0x22 of 0x134 bytes, each taking 0x90 bytes of data at +0xA4.
pub const LIGHT_RECORDS: usize = 0x22;
pub const LIGHT_RECORD: usize = 0x134;
pub const LIGHT_DATA: usize = 0x90;
pub const LIGHT_DATA_AT: usize = 0xA4;

/// Lays the data's records into the lights' records.
pub fn lay_lights(data: &[u8], records: &mut [u8]) {
    for n in 0..LIGHT_RECORDS {
        let to = n * LIGHT_RECORD + LIGHT_DATA_AT;
        records[to..to + LIGHT_DATA].copy_from_slice(&data[n * LIGHT_DATA..(n + 1) * LIGHT_DATA]);
    }
}

// ---- the crash matrix ----

/// The crash matrix's turns (x, y, z) from the three angles (y, x, z; whole part in the top
/// half).
pub fn crash_turns(angles: [u32; 3]) -> [u32; 3] {
    [angles[1] >> 16, angles[0] >> 16, angles[2] >> 16]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_and_pictures() {
        assert_eq!(car_slot(0), 0x50);
        assert_eq!(car_slot(7), 0x50 + 7 * 0x14);
        assert!(check_object_cars(8).is_ok() && check_object_cars(9).is_err());
        assert!(check_picture_cars(0).is_ok() && check_picture_cars(9).is_err());
        assert_eq!(
            car_pictures(0x700),
            [0x700, 0x701, 0x702, 0x703, 0x704, 0x705, 0x706, 0x707]
        );
        assert_eq!(car_pictures(u32::MAX)[1], 0);
        assert!(FIXED_PICTURES.iter().all(|&(k, _)| k < PICTURE_KINDS));
        const { assert!(CAR_PICTURE_KINDS + 8 * PICTURES_A_CAR <= PICTURE_KINDS) };
        assert_eq!(PREF_PIECES.iter().map(|p| p.1).sum::<usize>(), 0x6C);
    }

    #[test]
    fn courses_lights_turns() {
        let mut asked = Vec::new();
        let found = course_present(2, |f| -> Result<u32, ()> {
            asked.push(f);
            Ok(if f == 0 { 4 } else { 0 })
        });
        assert_eq!((found, asked.len()), (Ok(true), 1));
        let mut asked = Vec::new();
        let found = course_present(2, |f| -> Result<u32, ()> {
            asked.push(f);
            Ok(if f == 1 { 4 } else { 3 })
        });
        assert_eq!((found, asked), (Ok(true), vec![0, 1]));
        assert_eq!(course_present(2, |_| Ok::<u32, ()>(0)), Ok(false));
        assert_eq!(course_present(2, |_| Err::<u32, u8>(5)), Err(5));
        let data: Vec<u8> = (0..LIGHT_RECORDS * LIGHT_DATA).map(|n| n as u8).collect();
        let mut records = vec![0xAAu8; LIGHT_RECORDS * LIGHT_RECORD];
        lay_lights(&data, &mut records);
        assert_eq!(records[0xA3], 0xAA);
        assert_eq!(records[0xA4], 0);
        assert_eq!(records[LIGHT_RECORD + 0xA4], LIGHT_DATA as u8);
        assert_eq!(records[LIGHT_RECORD + 0xA4 + LIGHT_DATA], 0xAA);
        assert_eq!(crash_turns([0x10000, 0x20000, 0x3FFFF]), [2, 1, 3]);
    }
}
