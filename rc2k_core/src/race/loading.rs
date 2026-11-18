//! A stage's loading: the course file read into its parts, and the loading bar drawn while a stage
//! loads (a red rectangle along the bottom, as long as the share loaded, laid out on 800 x 600 and
//! scaled to the screen; and a frame's way through).

use crate::Invalid;
use crate::race::faces::CourseLayout;

/// The most bytes a course file fills: the file's room, read as zeros past what the file holds.
pub const FIRST_PART: usize = 0xF0_0000;

/// The header's dwords (`CourseLayout::HEADER` bytes): the counts, the start place and fields
/// nothing reads.
pub const HEADER_WORDS: usize = CourseLayout::HEADER as usize / 4;

/// The course file taken apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Course {
    /// The header's dwords as the file has them (record, face and scene point counts at 1 ..= 3).
    pub header: [u32; HEADER_WORDS],
    /// The 0x20-byte records.
    pub records: Vec<u8>,
    /// The 0x40-byte faces.
    pub faces: Vec<u8>,
    /// The scene points (12 bytes each) and the room after them, up to `FIRST_PART`: new points
    /// are written there.
    pub points: Vec<u8>,
}

/// `bytes[from..to]`, zeros where `bytes` ends.
fn padded(bytes: &[u8], from: usize, to: usize) -> Vec<u8> {
    let mut part = vec![0; to - from];
    if from < bytes.len() {
        let end = bytes.len().min(to);
        part[..end - from].copy_from_slice(&bytes[from..end]);
    }
    part
}

/// Takes the course file's `file` bytes (at most `FIRST_PART`; zeros are read past them) apart by
/// the counts in its header. The counts' products wrap as 32-bit numbers, so a count that wraps
/// lays the parts out of order or outside the file's room and is refused: the records start at
/// 0x100 and must end where the faces start, the faces where the points start, and the points
/// must start before `FIRST_PART`.
pub fn read_course(file: &[u8]) -> Result<Course, Invalid> {
    if file.len() > FIRST_PART {
        return Err(Invalid("a course file larger than its room"));
    }
    let mut header = [0u32; HEADER_WORDS];
    for (word, bytes) in header.iter_mut().zip(file.chunks(4)) {
        let mut le = [0; 4];
        le[..bytes.len()].copy_from_slice(bytes);
        *word = u32::from_le_bytes(le);
    }
    let layout = CourseLayout::of(0, header[1], header[2], header[3]);
    let (records, faces, points) = (
        layout.records as usize,
        layout.faces as usize,
        layout.points as usize,
    );
    if points >= FIRST_PART {
        return Err(Invalid("scene points past the course file's room"));
    }
    if faces > points {
        return Err(Invalid("course faces that end before they start"));
    }
    if records > faces {
        return Err(Invalid("course records that end before they start"));
    }
    Ok(Course {
        header,
        records: padded(file, records, faces),
        faces: padded(file, faces, points),
        points: padded(file, points, FIRST_PART),
    })
}

/// The bar's colour.
pub const BAR_COLOUR: u32 = 0xFF_0000;

/// The bar on the screen: left and right edges, top and bottom, and the right edge's division
/// remainder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bar {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
    pub rest: u32,
}

/// `v` x `size` / `of`: the quotient and the remainder.
fn scaled(v: u32, size: u32, of: u32) -> (u32, u32) {
    let n = u64::from(v) * u64::from(size);
    ((n / u64::from(of)) as u32, (n % u64::from(of)) as u32)
}

/// The share loaded held to 0 ..= 100 (signed).
pub fn held_share(share: u32) -> u32 {
    (share as i32).clamp(0, 0x64) as u32
}

/// The bar for a `width` x `height` screen and the share loaded: x 0 to width x share / 100,
/// y 0x24D ..= 0x258 of 600.
pub fn loading_bar(width: u32, height: u32, share: u32) -> Bar {
    let left = scaled(0, width, 0x320).0;
    let (right, rest) = scaled(width, held_share(share), 0x64);
    Bar {
        left,
        right: right.wrapping_add(left),
        top: scaled(0x24D, height, 0x258).0,
        bottom: scaled(0x258, height, 0x258).0,
        rest,
    }
}

/// The top left of the network loading list of `lines` lines `line_height` tall on a `width` x
/// `height` screen: inside the loading picture's 4:3 middle (a wider screen has only the picture's
/// wash at its sides), a 40th of the middle in from its left edge (clear of the picture's car,
/// and room for the list's box), the list centred half way down the screen.
pub fn list_origin(width: u32, height: u32, lines: u32, line_height: u32) -> (i32, i32) {
    let middle = crate::scene::view_window::design_width(width, height) as i32;
    let left = (width as i32 - middle) / 2 + middle / 40;
    let top = (height as i32 - (lines * line_height) as i32) / 2;
    (left, top)
}

/// The four corners (x, y) of a rectangle, from the top left; whole numbers made floats.
pub fn quad_corners(x0: i32, x1: i32, y0: i32, y1: i32) -> [(f32, f32); 4] {
    let (x0, x1, y0, y1) = (x0 as f32, x1 as f32, y0 as f32, y1 as f32);
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// The scene state mark while the bar is on.
pub const FRAME_MARK: u32 = 8;

/// A frame is tried with the window active and the surfaces not lost (`lost` not 1).
pub fn frame_tried(active: u32, lost: u32) -> bool {
    active != 0 && lost != 1
}

pub fn restored(lost_before: u32, lost_after: u32) -> (u32, bool, bool) {
    let state = (lost_before ^ 1) | lost_after;
    (state, state < 1, state == 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_lies_inside_the_4_by_3_middle_half_way_down() {
        // 4:3: at the left, a 40th in, centred.
        assert_eq!(list_origin(800, 600, 2, 20), (20, 280));
        // 16:9 (a middle of 1440 from 240) and 32:9 (the same middle from 1200).
        assert_eq!(list_origin(1920, 1080, 2, 20), (240 + 36, 520));
        let (left, top) = list_origin(3840, 1080, 3, 20);
        assert_eq!((left, top), (1200 + 36, 510));
    }

    #[test]
    fn frames() {
        assert!(frame_tried(1, 0) && !frame_tried(0, 0) && !frame_tried(1, 1));
        assert!(frame_tried(1, 2));
        assert_eq!(restored(1, 0), (0, true, false));
        assert_eq!(restored(0, 0), (1, false, true));
        assert_eq!(restored(1, 1), (1, false, true));
        assert_eq!(restored(2, 0), (3, false, false));
    }

    #[test]
    fn bar() {
        let b = loading_bar(800, 600, 50);
        assert_eq!(
            (b.left, b.right, b.top, b.bottom, b.rest),
            (0, 400, 0x24D, 0x258, 0)
        );
        assert_eq!(loading_bar(1024, 768, 33).right, 1024 * 33 / 100);
        assert_eq!(loading_bar(1024, 768, 33).rest, 1024 * 33 % 100);
        assert_eq!(held_share(0xFFFF_FFFF), 0);
        assert_eq!(held_share(500), 100);
        assert_eq!(
            quad_corners(0, 800, 589, 600),
            [(0.0, 589.0), (800.0, 589.0), (800.0, 600.0), (0.0, 600.0)]
        );
        assert_eq!(quad_corners(0x0100_0001, 0, 0, 0)[0].0, 16777216.0);
    }

    /// A file of `records`, `faces` and `points` counts with each part's bytes numbered.
    fn file(records: u32, faces: u32, points: u32) -> Vec<u8> {
        let mut bytes = vec![0; 0x100];
        for (k, count) in [records, faces, points].into_iter().enumerate() {
            bytes[4 + 4 * k..8 + 4 * k].copy_from_slice(&count.to_le_bytes());
        }
        bytes[0..4].copy_from_slice(&0xABCD_u32.to_le_bytes());
        let body = records * 0x20 + faces * 0x40 + points * 12;
        bytes.extend((0..body).map(|n| (n % 251) as u8 + 1));
        bytes
    }

    #[test]
    fn a_course_file_is_taken_apart() {
        let bytes = file(2, 3, 4);
        let course = read_course(&bytes).unwrap();
        assert_eq!(course.header[..4], [0xABCD, 2, 3, 4]);
        assert_eq!(course.records, bytes[0x100..0x140]);
        assert_eq!(course.faces, bytes[0x140..0x200]);
        // The points, then the room after them, zeros.
        assert_eq!(course.points.len(), FIRST_PART - 0x200);
        assert_eq!(course.points[..48], bytes[0x200..0x230]);
        assert!(course.points[48..].iter().all(|&b| b == 0));
    }

    #[test]
    fn a_short_file_reads_zeros_past_its_end() {
        let bytes = file(1, 1, 1);
        let cut = &bytes[..0x100 + 0x20 + 0x10];
        let course = read_course(cut).unwrap();
        assert_eq!(course.records, bytes[0x100..0x120]);
        assert_eq!(course.faces[..0x10], bytes[0x120..0x130]);
        assert!(course.faces[0x10..].iter().all(|&b| b == 0));
        assert_eq!(course.points.len(), FIRST_PART - 0x160);
        assert!(course.points.iter().all(|&b| b == 0));
        // A file cut inside the header reads zero counts.
        let course = read_course(&bytes[..2]).unwrap();
        assert_eq!(course.header[0], 0xABCD);
        assert_eq!(course.header[1], 0);
        assert!(course.records.is_empty() && course.faces.is_empty());
        assert_eq!(course.points.len(), FIRST_PART - 0x100);
        // No file at all: no counts either.
        assert_eq!(read_course(&[]).unwrap().header, [0; HEADER_WORDS]);
    }

    #[test]
    fn counts_that_leave_the_room_are_refused() {
        let with = |records: u32, faces: u32, points: u32| {
            let mut bytes = file(0, 0, 0);
            for (k, count) in [records, faces, points].into_iter().enumerate() {
                bytes[4 + 4 * k..8 + 4 * k].copy_from_slice(&count.to_le_bytes());
            }
            read_course(&bytes)
        };
        // The points must start before the file's room ends.
        assert!(with(0, 0x3_BFFB, 0).is_ok());
        assert!(with(0, 0x3_BFFC, 0).is_err());
        assert!(with(0x7_7FF7, 0, 0).is_ok());
        assert!(with(0x7_7FF8, 0, 0).is_err());
        // Counts whose products wrap the 32-bit offsets are laid out as wrapped.
        assert!(with(0x0800_0000, 0, 0).is_ok());
        assert!(with(0, 0x0400_0000, 0).is_ok());
        assert!(with(0x0800_0001, 0x0400_0000, 0).is_ok());
        // The faces would start before the records end, or after the points start.
        assert!(with(0x07FF_FFFF, 0, 0).is_err());
        assert!(with(2, 0x03FF_FFFF, 0).is_err());
        assert!(with(0xFFFF_FFF0, 0, 0).is_err());
        assert!(with(0x10, 0xFFFF_FFF0, 0).is_err());
        assert!(read_course(&vec![0; FIRST_PART + 1]).is_err());
        assert!(read_course(&vec![0; FIRST_PART]).is_ok());
    }
}
