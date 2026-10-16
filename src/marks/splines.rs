//! `init_marks`' step 1 (IDA, 2026-10-16): the splines. `sub_4B96CD` reads
//! `files\m\spline.dat` once and, for each of the settings' objects, `sub_4B9A32` fills the
//! next of 8 places in four tables of `MarkSpline` records (0xE90 bytes): the points of the
//! object's faces of one kind, centred, turned, graded by height and ordered. These are the
//! leaves; every function takes the record in ebp.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::math::x87;

pub const SPLINE_LEN: u32 = 0xE90;
/// The most points a record holds.
pub const MOST_POINTS: u32 = 0x80;
const COUNT: u32 = 0x50;
const IDS: u32 = 0x454;
const FIRSTS: u32 = 0x54;
const SECONDS: u32 = 0x254;
const COORDS: u32 = 0x654;
const CENTRE: u32 = 0xC54;
const GRADES: u32 = 0xC90;
/// `sub_4BA316`'s two heights (its reading of +0x18 and +0x1C is overwritten: section 16).
const LOW: i32 = 0x3830;
const HIGH: i32 = 0x6720;
const OUTLINE_FROM_RVA: usize = rva(0x769182); // flt_769182 .. flt_76919E
/// Where `reset_outline` puts the eight floats: four (x, y) pairs 0x18 apart.
const OUTLINE_TO: [usize; 8] = [
    0x768688, 0x76868C, 0x7686A0, 0x7686A4, 0x7686B8, 0x7686BC, 0x7686D0, 0x7686D4,
];
pub const OUTLINE_COUNT_RVA: usize = rva(0x768700); // dword_768700
pub const OUTLINE_MARK_RVA: usize = rva(0x7687D0); // dword_7687D0
pub const FOUND_RVA: usize = rva(0x7695B2); // g_marksSplineFaces: faces of the kind found
/// `turn_spline_points`' matrices: the three turns, then X x Y, then all three.
pub const TURN_X_RVA: usize = rva(0x76A28E); // g_marksTurnX
pub const TURN_Y_RVA: usize = rva(0x76A2BE); // g_marksTurnY
pub const TURN_Z_RVA: usize = rva(0x76A2EE); // g_marksTurnZ
pub const TURN_XY_RVA: usize = rva(0x76A22E); // g_marksTurnXY
pub const TURN_RVA: usize = rva(0x76A25E); // g_marksTurn
/// `order_spline_points`' work: the ids, then the points.
pub const ORDERED_IDS_RVA: usize = rva(0x76A31E); // g_marksOrderedIds
pub const ORDERED_COORDS_RVA: usize = rva(0x76A51E); // g_marksOrderedCoords

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The record's point count, refused past `MOST_POINTS` (the loops would run past it).
unsafe fn count(spline: u32) -> Result<u32, Refused> {
    let n = unsafe { get(spline + COUNT) };
    if n as i32 > MOST_POINTS as i32 {
        return Err(BadInput("a spline of more than 0x80 points").into());
    }
    Ok((n as i32).max(0) as u32)
}

/// `marks::centre_spline_points` (0x4B9F95): each point (+0x654, 12 bytes, the last first)
/// less the centre (+0xC54). Leaves eax, ebx, ecx the first point, edi 0 (as they were with
/// none). Refuses more than 0x80 points.
pub unsafe fn centre_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        for k in (0..count(s)?).rev() {
            g.edi = k * 12;
            let p = s + COORDS + g.edi;
            g.eax = get(p).wrapping_sub(get(s + CENTRE));
            g.ebx = get(p + 4).wrapping_sub(get(s + CENTRE + 4));
            g.ecx = get(p + 8).wrapping_sub(get(s + CENTRE + 8));
            put(p, g.eax);
            put(p + 4, g.ebx);
            put(p + 8, g.ecx);
        }
    }
    Ok(g)
}

/// `marks::grade_spline_points` (0x4BA316): each point's grade (+0xC90, the last first) by
/// its height (+0x658): 0 below 0x3830, 1 below 0x6720, else 2. Leaves eax the first
/// point's height, esi and edi 0 (eax 0x6720 with none). Refuses more than 0x80 points.
pub unsafe fn grade_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        let n = count(s)?;
        g.eax = HIGH as u32;
        for k in (0..n).rev() {
            g.edi = k;
            g.esi = k * 12;
            g.eax = get(s + COORDS + 4 + g.esi);
            let h = g.eax as i32;
            let grade = if h < LOW {
                0
            } else if h < HIGH {
                1
            } else {
                2
            };
            put(s + GRADES + 4 * k, grade);
        }
    }
    Ok(g)
}

/// `marks::find_spline_ends` (0x4BA4F1): of the +0xC points from +0 the last with the
/// highest third value, into +0x30; of the +0x10 from +4 the last with the lowest, into
/// +0x34. The value is read at +0x65C + 0x30 a point, not 12 (section 16): read as the
/// original does. Leaves eax the second count, ecx the second found, edx its value, edi
/// the last point read times 12. Refuses a negative start or count, or points past 0x80.
pub unsafe fn find_spline_ends(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        for (start, n) in [(0u32, 0xCu32), (4, 0x10)] {
            let (a, c) = (get(s + start) as i32, get(s + n) as i32);
            if a < 0 || c < 0 || a + c > MOST_POINTS as i32 {
                return Err(BadInput("a spline's grade past its points").into());
            }
        }
        for (start, n, to, first, highest) in [
            (0u32, 0xCu32, 0x30u32, 0x8000_0001u32, true),
            (4, 0x10, 0x34, 0x7FFF_FFFF, false),
        ] {
            g.ecx = 0;
            g.edx = first;
            let from = get(s + start);
            let c = get(s + n);
            for k in 0..c {
                g.edi = (k + from) * 12;
                let v = get(s + 4 * g.edi + 0x65C);
                let take = if highest {
                    g.edx as i32 <= v as i32
                } else {
                    g.edx as i32 >= v as i32
                };
                if take {
                    g.ecx = k;
                    g.edx = v;
                }
            }
            g.eax = c;
            g.ecx = g.ecx.wrapping_add(from);
            put(s + to, g.ecx);
        }
    }
    Ok(g)
}

/// `marks::add_face_points` (0x4B9E29): face edi's points (+0 the count, from +0x10, 16
/// bytes each: the point and two values) added to the record's (+0x50 the count, ids
/// +0x454, values +0x54 and +0x254) unless there already, up to 0x80. Leaves eax the
/// count, ecx where the last search stopped, edx the last point read. Refuses a record of
/// more than 0x80 points.
pub unsafe fn add_face_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        if get(s + COUNT) > MOST_POINTS {
            return Err(BadInput("a spline of more than 0x80 points").into());
        }
        let mut at = g.edi + 0x10;
        g.eax = get(s + COUNT);
        g.ecx = get(g.edi);
        for _ in 0..(g.ecx as i32).max(0) {
            if g.eax == MOST_POINTS {
                continue;
            }
            g.edx = get(at);
            g.ecx = 0;
            while g.ecx != g.eax && get(s + IDS + 4 * g.ecx) != g.edx {
                g.ecx += 1;
            }
            if g.ecx == g.eax {
                put(s + IDS + 4 * g.eax, g.edx);
                put(s + FIRSTS + 4 * g.eax, get(at + 4));
                put(s + SECONDS + 4 * g.eax, get(at + 8));
                g.eax += 1;
            }
            at += 0x10;
        }
        put(s + COUNT, g.eax);
    }
    Ok(g)
}

/// `marks::find_farthest_points` (0x4BA578): looks for the two farthest apart of eax points
/// at esi, all inside a `pusha` .. `popa`, so nothing is kept (section 16): it does
/// nothing seen.
pub fn find_farthest_points(entry: &Regs) -> Regs {
    *entry
}

/// `marks::reset_outline` (0x4B7E64): the eight floats from `flt_769182` to four (x, y)
/// pairs from `flt_768688`, `dword_768700` 5, `dword_7687D0` 0. Every register kept.
pub unsafe fn reset_outline() {
    unsafe {
        for (n, to) in OUTLINE_TO.into_iter().enumerate() {
            game_write(rva(to), x87::loaded(game_read(OUTLINE_FROM_RVA + 4 * n)));
        }
        game_write(OUTLINE_COUNT_RVA, 5u32);
        game_write(OUTLINE_MARK_RVA, 0u32);
    }
}

/// What `reset_outline` writes.
pub fn outline_regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let mut r: Vec<_> = OUTLINE_TO.into_iter().map(|a| at(rva(a))).collect();
    r.extend([at(OUTLINE_COUNT_RVA), at(OUTLINE_MARK_RVA)]);
    r
}

/// `marks::gather_spline_points` (0x4B9D5F): record ebp emptied, then the points of object
/// eax's faces of kind ebx (+7, low five bits) added (`add_face_points`; the faces counted
/// in `g_marksSplineFaces`); the ids made pointers into `g_objectPoints` and the points
/// copied to +0x654 (both the last first); then `find_farthest_points`. Leaves eax the
/// count, esi +0x654. Refuses an object past 0xFF, a negative face count or faces past
/// `g_faces`.
pub unsafe fn gather_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    use crate::scene::objects::{
        FACE_COUNTS_RVA, FACES, FACES_RVA, FIRST_FACE_RVA, FIRST_POINT_RVA, POINTS_RVA,
    };
    let mut g = *entry;
    unsafe {
        let object = g.eax as usize;
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = game_read(FIRST_FACE_RVA + 4 * object);
        let faces: u32 = game_read(FACE_COUNTS_RVA + 4 * object);
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        let s = g.ebp;
        put(s + COUNT, 0);
        g.esi = first;
        g.edx = game_read(FIRST_POINT_RVA + 4 * object);
        game_write(FOUND_RVA, 0u32);
        for _ in 0..faces {
            g.edi = game_read(FACES_RVA + 4 * g.esi as usize);
            g.eax = (*((g.edi + 7) as *const u8) & 0x1F) as u32;
            if g.eax == g.ebx {
                game_write(FOUND_RVA, game_read::<u32>(FOUND_RVA).wrapping_add(1));
                g = add_face_points(&g)?;
            }
            g.esi += 1;
        }
        g.eax = faces;
        let n = get(s + COUNT);
        let points: u32 = game_read(POINTS_RVA);
        for k in (0..n).rev() {
            g.ebx = k;
            g.ecx = get(s + IDS + 4 * k).wrapping_mul(12).wrapping_add(points);
            put(s + IDS + 4 * k, g.ecx);
        }
        for k in (0..n).rev() {
            g.esi = get(s + IDS + 4 * k);
            g.edi = k * 12;
            for c in 0..3 {
                put(s + COORDS + g.edi + 4 * c, get(g.esi + 4 * c));
            }
        }
        g.esi = s + COORDS;
        g.eax = n;
        g = find_farthest_points(&g);
    }
    Ok(g)
}

/// An angle as `fld dword [x]; fchs` leaves it.
fn negated(f: u32) -> [u8; 10] {
    let mut a = crate::scene::matrix3::extended(f);
    a[9] ^= 0x80;
    a
}

/// `marks::turn_spline_points` (0x4B9FE6): the turns about X, Y, Z by minus record ebp's
/// +0x20, +0x24, +0x28 into `g_marksTurnX` .. `Z`, X x Y into `g_marksTurnXY`, that x Z
/// into `g_marksTurn`, and the record's points turned by it in place. Leaves ecx -1, edx
/// 0, esi and edi past the points, ebp `g_marksTurn`. Refuses more than 0x80 points or an
/// angle `fsincos` cannot take, before writing anything.
pub unsafe fn turn_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    use crate::scene::matrix3::{
        Axis, multiply_matrices3, transform_int_points, turn_matrix3, turnable,
    };
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        count(s)?;
        let angles = [0x20u32, 0x24, 0x28].map(|o| negated(get(s + o)));
        for a in &angles {
            turnable(a)?;
        }
        let at = |r: usize| game_addr(r) as u32;
        for ((axis, to), angle) in [
            (Axis::X, TURN_X_RVA),
            (Axis::Y, TURN_Y_RVA),
            (Axis::Z, TURN_Z_RVA),
        ]
        .into_iter()
        .zip(angles)
        {
            turn_matrix3(axis, at(to), angle)?;
        }
        g.esi = at(TURN_Z_RVA);
        for (a, b, out) in [
            (TURN_X_RVA, TURN_Y_RVA, TURN_XY_RVA),
            (TURN_XY_RVA, TURN_Z_RVA, TURN_RVA),
        ] {
            g = multiply_matrices3(&Regs {
                esi: at(a),
                edi: at(b),
                ebp: at(out),
                ..g
            });
        }
        g.esi = s + COORDS;
        g.edi = s + COORDS;
        g.ecx = get(s + COUNT);
        g.ebp = at(TURN_RVA);
        g = transform_int_points(&g)?;
    }
    Ok(g)
}

/// `marks::order_spline_points` (0x4BA388): record ebp's ids and points put in grade order
/// (each grade's the last first) through `g_marksOrderedIds` and `g_marksOrderedCoords`;
/// +0 .. +8 each grade's start, +0xC .. +0x14 its count; then the count's ids and points
/// moved back (`memory::move_bytes`, as many as the count, whatever their grade). Leaves
/// eax +0x654, ebx `g_marksOrderedCoords`, ecx the count times 12, esi the last taken
/// times 12, edi 0. Refuses more than 0x80 points or a negative count.
pub unsafe fn order_spline_points(entry: &Regs) -> Result<Regs, Refused> {
    use crate::memory::move_bytes::move_bytes;
    let mut g = *entry;
    unsafe {
        let s = g.ebp;
        let n = count(s)?;
        if (get(s + COUNT) as i32) < 0 {
            return Err(BadInput("a spline of a negative count: the moves would run away").into());
        }
        let (ids, coords) = (
            game_addr(ORDERED_IDS_RVA) as u32,
            game_addr(ORDERED_COORDS_RVA) as u32,
        );
        g.eax = 0;
        g.ebx = 0;
        put(s, 0);
        for grade in 0..3u32 {
            for k in (0..n).rev() {
                g.edi = k;
                if get(s + GRADES + 4 * k) != grade {
                    continue;
                }
                g.esi = k * 12;
                put(ids + 4 * g.eax, get(s + IDS + 4 * k));
                for c in 0..3 {
                    put(coords + g.ebx + 4 * c, get(s + COORDS + g.esi + 4 * c));
                }
                g.eax += 1;
                g.ebx += 12;
            }
            g.ecx = g.eax.wrapping_sub(get(s + 4 * grade));
            put(s + 0xC + 4 * grade, g.ecx);
            if grade < 2 {
                put(s + 4 + 4 * grade, g.eax);
            }
        }
        for (to, from, size) in [(s + IDS, ids, 4), (s + COORDS, coords, 12)] {
            g.eax = to;
            g.ebx = from;
            g.ecx = get(s + COUNT).wrapping_mul(size);
            move_bytes(&Regs {
                eax: g.ecx,
                esi: g.ebx,
                edi: g.eax,
                ..g
            });
        }
    }
    Ok(g)
}

/// What `turn_spline_points` and `order_spline_points` write besides the record.
pub fn work_regions() -> Vec<crate::service::harness::Region> {
    let a = |r: usize| game_addr(r) as u32;
    vec![
        (a(TURN_XY_RVA), 0xF0),
        (a(ORDERED_IDS_RVA), 4 * MOST_POINTS as usize),
        (a(ORDERED_COORDS_RVA), 12 * MOST_POINTS as usize),
        crate::service::harness::at(FOUND_RVA),
    ]
}

// ---- for the self-tests ----

/// A record of the test's (with room past it for `find_spline_ends`' reads), filled by lot.
pub struct TestSpline {
    pub bytes: Vec<u8>,
}

/// How far past a record `find_spline_ends` can read.
const READ_REACH: u32 = 0x65C + 0x30 * MOST_POINTS;

impl TestSpline {
    pub fn new() -> Self {
        Self {
            bytes: vec![0; READ_REACH as usize],
        }
    }

    pub fn addr(&self) -> u32 {
        self.bytes.as_ptr() as u32
    }

    pub fn region(&self) -> crate::service::harness::Region {
        (self.addr(), SPLINE_LEN as usize)
    }

    /// Everything by lot from `seed`, `points` points (heights about the grades' bounds),
    /// the grades' starts and counts a split of the points.
    pub fn fill(&self, seed: u32, points: u32) {
        let mut s = seed;
        let mut next = move || {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            s >> 4
        };
        let a = self.addr();
        unsafe {
            for n in 0..READ_REACH / 4 {
                put(a + 4 * n, next());
            }
            put(a + COUNT, points);
            for k in 0..MOST_POINTS {
                let p = a + COORDS + 12 * k;
                for c in 0..3 {
                    put(p + 4 * c, (next() % 0x2_0000).wrapping_sub(0x1_0000));
                }
                put(p + 4, next() % 0x9000);
                put(a + IDS + 4 * k, next() % 0x60);
            }
            let first = next() % (points + 1);
            let second = next() % (points - first + 1);
            for k in 0..MOST_POINTS {
                put(a + GRADES + 4 * k, next() % 4);
            }
            for o in [0x20u32, 0x24, 0x28] {
                put(
                    a + o,
                    (((next() % 0x8000) as f32 - 16384.0) / 2048.0).to_bits(),
                );
            }
            put(a, 0);
            put(a + 0xC, first);
            put(a + 4, first);
            put(a + 0x10, second);
        }
    }
}

impl Default for TestSpline {
    fn default() -> Self {
        Self::new()
    }
}
