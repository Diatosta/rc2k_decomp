//! The ground's search: which course face lies under the point (`dword_639824`,
//! `dword_63982C`, whole-number x and z), and the shares and height there (IDA,
//! 2026-10-17). `sub_441E2F` tries the last face found (`physics::try_last_face`), then
//! the nine grid cells around the point (`physics::search_cell`: the cell's course record,
//! `physics::find_course_record`, then its faces, `physics::search_record_faces`), each
//! face by `physics::face_under_point` (its edges by `physics::edge_side`; when inside,
//! `physics::face_shares` and `physics::plane_height`).
//!
//! The cores here work on a register model (`Regs`, changed as the original's
//! instructions change them), so that every hook leaves what the original leaves. The
//! variables are reached through `Ground`: the game's, or a self-test's copies of the
//! game's ranges (`TestGround`), against copies of the originals whose references to
//! those ranges are patched and whose calls go to each other (the ground variables are
//! the game's every frame).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva};
use crate::physics::edge_side::{EdgePlaces, edge_side_at};
use crate::physics::find_course_record::{RecordPlaces, find_course_record_at};
use crate::physics::plane_height::{Plane, plane_height_at};
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::Region;
use rc2k_core::physics::ground_search::{self, FaceTest};

/// The game's ranges the search uses (IDA addresses, end exclusive): the course tables
/// and pointers (`g_kindFirstRecords` .. `g_courseFaceCount`), the edge test's product
/// and the face mask, the SSE switch, the ground's work variables; for
/// `physics::probe_ground` also the car's ground points (0x6D1E14 ..), the work copy
/// `g_car` and the scratch `dword_62400C`.
pub const RANGES: [(u32, u32); 7] = [
    (0x5CC3E4, 0x5CCD20),
    (0x620460, 0x620474),
    (0x621A58, 0x621A5C),
    (0x6396FC, 0x63989C),
    (0x6D1E14, 0x6D3B88),
    (0x719FD8, 0x71AFB8),
    (0x62400C, 0x624010),
];

pub const FACE_LEN: u32 = 0x40;
pub const RECORD_LEN: u32 = 0x20;
const POINTS: u32 = 0x5CCCC0; // g_scenePoints (x, y, z whole numbers, 12 bytes each)
const FACES: u32 = 0x5CCCBC; // g_courseFaces
const FACE_COUNT: u32 = 0x5CCD1C; // g_courseFaceCount
const RECORDS: u32 = 0x5CCCB8; // g_courseRecords
const RECORD_COUNT: u32 = 0x5CCD18; // g_courseRecordCount
const LAST_FACE: u32 = 0x5CCC78; // dword_5CCC78: the face found last time

/// Where the ground variables are: the game's, or a test's copies of `RANGES`.
#[derive(Clone, Copy)]
pub struct Ground {
    test: Option<[u32; 7]>,
}

impl Ground {
    pub fn game() -> Self {
        Self { test: None }
    }

    /// The address of the variable IDA shows at `ida`.
    pub fn a(&self, ida: u32) -> u32 {
        if let Some(bases) = self.test {
            for (k, &(lo, hi)) in RANGES.iter().enumerate() {
                if (lo..hi).contains(&ida) {
                    return bases[k] + (ida - lo);
                }
            }
        }
        game_addr(rva(ida as usize)) as u32
    }

    pub unsafe fn get(&self, ida: u32) -> u32 {
        unsafe { (self.a(ida) as *const u32).read_unaligned() }
    }

    pub unsafe fn put(&self, ida: u32, v: u32) {
        unsafe { (self.a(ida) as *mut u32).write_unaligned(v) }
    }

    pub fn edge(&self) -> EdgePlaces {
        EdgePlaces {
            x: self.a(0x639824),
            z: self.a(0x63982C),
            product: self.a(0x620460),
        }
    }

    pub fn plane(&self) -> Plane {
        Plane {
            a: self.a(0x639718),
            b: self.a(0x639724),
            c: self.a(0x63970C),
            f: self.a(0x639810),
            g: self.a(0x639814),
            out: self.a(0x639828),
        }
    }

    pub unsafe fn records(&self) -> RecordPlaces {
        RecordPlaces {
            firsts: self.a(0x5CC3E4),
            lasts: self.a(0x5CC7E4),
            records: self.a(RECORDS),
            count: unsafe { self.get(RECORD_COUNT) },
        }
    }

    /// What the search may write: the face variables, the product, the work variables.
    pub fn regions(&self) -> Vec<Region> {
        vec![
            (self.a(0x5CCC60), 0x1C),
            (self.a(0x620460), 8),
            (self.a(0x6396FC), (0x63989C - 0x6396FC) as usize),
        ]
    }

    unsafe fn point(&self, index: u32) -> u32 {
        unsafe { self.get(POINTS) }.wrapping_add(index.wrapping_mul(12))
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// Every register as the model left it, and the carry.
pub fn left_of(g: &Regs, carry: bool) -> Left {
    use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI};
    Left::carry(carry)
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx)
        .with(EDX, g.edx)
        .with(ESI, g.esi)
        .with(EDI, g.edi)
        .with(EBP, g.ebp)
}

/// `physics::face_shares` (0x442323): the three corners named by `dword_639890` ..
/// `98` copied (0x639708 ..), the point (0x6396FC, 0x639704), the differences, and the
/// shares by the x87 (`math::x87_run::face_shares`; the `rcpss` way with the SSE switch).
/// Leaves eax the last difference (0x63974C), ebx, ecx the third corner's y, z.
pub unsafe fn face_shares_at(g: &Ground, r: &mut Regs) {
    unsafe {
        let mut corner = [[0u32; 3]; 3];
        for (k, c) in corner.iter_mut().enumerate() {
            let p = g.point(g.get(0x639890 + 4 * k as u32));
            *c = [at(p), at(p + 4), at(p + 8)];
            for (n, &v) in c.iter().enumerate() {
                g.put(0x639708 + 12 * k as u32 + 4 * n as u32, v);
            }
        }
        let (x, z) = (g.get(0x639824), g.get(0x63982C));
        g.put(0x6396FC, x);
        g.put(0x639704, z);
        let d = [
            (0x639738, x.wrapping_sub(corner[0][0])),
            (0x63973C, corner[1][0].wrapping_sub(corner[0][0])),
            (0x639740, corner[2][0].wrapping_sub(corner[0][0])),
            (0x639750, z.wrapping_sub(corner[0][2])),
            (0x639754, corner[1][2].wrapping_sub(corner[0][2])),
            (0x639758, corner[2][2].wrapping_sub(corner[0][2])),
            (0x639748, corner[1][1].wrapping_sub(corner[0][1])),
            (0x63974C, corner[2][1].wrapping_sub(corner[0][1])),
        ];
        for &(v, value) in &d {
            g.put(v, value);
        }
        r.eax = d[7].1;
        r.ebx = corner[2][1];
        r.ecx = corner[2][2];
        let sse = g.get(0x621A58) != 0;
        let i = [
            d[0].1,
            d[1].1,
            d[2].1,
            d[3].1,
            d[4].1,
            d[5].1,
            g.get(0x5C5D98),
        ];
        let mut o = [0u32; 8];
        crate::math::x87_run::face_shares(&i, &mut o, sse);
        for (v, k) in [
            (0x639764, 0),
            (0x63975C, 1),
            (0x639760, 2),
            (0x639810, 3),
            (0x639800, 4),
            (0x639814, 5),
            (0x639804, 6),
        ] {
            g.put(v, o[k]);
        }
        if sse {
            g.put(0x639768, o[7]);
        }
    }
}

/// `face_under_point`'s refusal, worked out without writing.
pub unsafe fn face_refusal(g: &Ground, face: u32) -> Option<&'static str> {
    ground_search::check_face(face, unsafe { g.get(FACE_COUNT) })
        .err()
        .map(|e| e.0)
}

/// `physics::face_under_point` (0x442006): whether the point is inside face ebx (see
/// `physics/face_under_point.rs`). Returns the carry; `r` as the original leaves it.
// Converted: `rc2k_core::physics::ground_search::face_under_point`.
pub unsafe fn face_under_point_at(g: &Ground, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        if let Some(why) = face_refusal(g, r.ebx) {
            return Err(BadInput(why).into());
        }
        g.put(0x5CCC70, r.ebx);
        let face = (r.ebx << 6).wrapping_add(g.get(FACES));
        r.esi = face;
        let kind = (face as *const u8).read();
        let entry_eax = r.eax;
        // One edge, the registers as the original's edge code leaves them.
        let side = |from: u32, to: u32| -> Option<bool> {
            r.ecx = face;
            r.ebx = at(face + from);
            r.ebp = g.point(r.ebx);
            r.ebx = at(face + to);
            r.edi = g.point(r.ebx);
            r.esi = face;
            let s = edge_side_at(&g.edge(), r.ebp, r.edi);
            r.eax = s.eax;
            (s.zero == Some(true)).then_some(s.carry)
        };
        let test = ground_search::face_under_point(
            kind,
            || ((face + 2) as *const u16).read_unaligned(),
            g.get(0x620470),
            |o| at(face + o),
            side,
        );
        let (half, corners) = match test {
            FaceTest::Kind => {
                r.eax = (entry_eax & !0xFF) | u32::from(kind);
                return Ok(false);
            }
            FaceTest::Flags(f) => {
                r.eax = f;
                return Ok(false);
            }
            FaceTest::Edge => return Ok(false),
            FaceTest::Inside { half, corners } => (half, corners),
        };
        let number = g.get(0x5CCC70);
        r.ebx = number;
        g.put(0x5CCC74, number);
        let face = (number << 6).wrapping_add(g.get(FACES));
        r.esi = face;
        g.put(0x5CCC6C, half);
        for (k, &o) in corners.iter().enumerate() {
            r.eax = at(face + o);
            g.put(0x639890 + 4 * k as u32, r.eax);
            g.put(0x5CCC60 + 4 * k as u32, r.eax);
        }
        face_shares_at(g, r);
        g.put(0x639820, 1);
        plane_height_at(&g.plane());
        r.ebx = number;
        Ok(true)
    }
}

/// `search_record_faces`' refusal, worked out without writing.
pub unsafe fn record_refusal(g: &Ground, record: u32) -> Option<&'static str> {
    unsafe {
        if let Err(e) = ground_search::check_record(record, g.get(RECORD_COUNT)) {
            return Some(e.0);
        }
        let rec = (record << 5).wrapping_add(g.get(RECORDS));
        ground_search::check_record_faces(at(rec + 8), at(rec + 0xC), g.get(FACE_COUNT))
            .err()
            .map(|e| e.0)
    }
}

/// `physics::search_record_faces` (0x441FD9): faces +8 ..= +0xC of course record ebx,
/// each by `face_under_point` (ebx and esi kept around it); the carry when one has the
/// point.
// Converted: `rc2k_core::physics::ground_search::search_faces`.
pub unsafe fn search_record_faces_at(g: &Ground, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        if let Some(why) = record_refusal(g, r.ebx) {
            return Err(BadInput(why).into());
        }
        let rec = (r.ebx << 5).wrapping_add(g.get(RECORDS));
        r.esi = rec;
        let (found, ebx) = ground_search::search_faces(at(rec + 8), at(rec + 0xC), |face| {
            r.ebx = face;
            let found = face_under_point_at(g, r)?;
            r.esi = rec;
            r.ebx = face;
            Ok::<_, Refused>(found)
        })?;
        r.ebx = ebx;
        Ok(found)
    }
}

/// `physics::search_cell` (0x441FBA): the cell of (eax, ecx) (each >> 21; key z << 11 +
/// x), its record by `find_course_record`, then its faces. The carry when found.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub unsafe fn search_cell_at(g: &Ground, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        (r.eax, r.ecx) = ground_search::cell_key(r.eax, r.ecx);
        let left = find_course_record_at(&g.records(), r)?;
        let mut after = *r;
        left.apply(&mut after);
        if after.carry() {
            *r = after;
            return Ok(false);
        }
        *r = after;
        if let Some(why) = record_refusal(g, r.ebx) {
            return Err(BadInput(why).into());
        }
        search_record_faces_at(g, r)
    }
}

/// `physics::try_last_face` (0x441E0C): ebx = `dword_5CCC78`; when it is a face (not -1,
/// below the count) `face_under_point` on it inside `pusha`/`popa`. The carry when found.
// Converted: `rc2k_core::physics::ground_search::last_face`.
pub unsafe fn try_last_face_at(g: &Ground, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        r.ebx = g.get(LAST_FACE);
        if ground_search::last_face(r.ebx, g.get(FACE_COUNT)).is_none() {
            return Ok(false);
        }
        let mut inner = *r;
        face_under_point_at(g, &mut inner)
    }
}

pub use rc2k_core::physics::ground_search::CELLS;
const FOUND: u32 = 0x639830; // g_groundFound

/// `physics::ground_search` (0x441E2F), inside `pusha` / `popa`: `try_last_face`; else
/// `dword_5CCC74` = -1 and `search_cell` on the point plus each of `CELLS` until one
/// finds it. `g_groundFound` 1 when found, else 0. Registers kept; the carry the found.
// Converted: `rc2k_core::physics::ground_search::ground_search`.
pub unsafe fn ground_search_at(g: &Ground, r: &mut Regs) -> Result<bool, Refused> {
    unsafe {
        if let Some(why) = ground_refusal(g) {
            return Err(BadInput(why).into());
        }
        let inner = std::cell::RefCell::new(*r);
        let found = ground_search::ground_search(
            || try_last_face_at(g, &mut inner.borrow_mut()),
            || g.put(0x5CCC74, ground_search::NO_FACE),
            || (g.get(0x639824), g.get(0x63982C)),
            |x, z| {
                let mut r = inner.borrow_mut();
                r.eax = x;
                r.ecx = z;
                search_cell_at(g, &mut r)
            },
        )?;
        g.put(FOUND, u32::from(found));
        Ok(found)
    }
}

/// `ground_search`'s refusals, worked out without writing: those of the cells' records it
/// would come to (all nine are looked at; a refusal past the face found is refused too).
pub unsafe fn ground_refusal(g: &Ground) -> Option<&'static str> {
    unsafe {
        let records = g.records();
        for (dx, dz) in CELLS {
            let x = g.get(0x639824).wrapping_add(dx);
            let z = g.get(0x63982C).wrapping_add(dz);
            let mut r = Regs {
                eax: ground_search::cell_key(x, z).0,
                ..Regs::default()
            };
            match find_course_record_at(&records, &r) {
                Err(_) => return Some("a cell's kind runs past g_courseRecordCount"),
                Ok(left) => {
                    left.apply(&mut r);
                    if !r.carry()
                        && let Some(why) = record_refusal(g, r.ebx)
                    {
                        return Some(why);
                    }
                }
            }
        }
        None
    }
}

/// A self-test's copies of `RANGES`, with its own faces, points and records, and copies
/// of the originals pointed at them (`TestGround::copy`).
pub struct TestGround {
    blocks: Vec<Vec<u8>>,
    bases: [u32; 7],
    pub faces: Vec<u8>,
    pub points: Vec<u32>,
    pub records: Vec<u8>,
}

/// One original as a copy: (IDA address, length, entry bytes, call offsets, references
/// to `RANGES`).
pub type CopySpec = (u32, usize, &'static [u8], &'static [usize], usize);

pub const EDGE_SIDE: CopySpec = (0x4425EF, 0x5C, &[0x53, 0x52, 0x8B, 0x45, 0x00], &[], 6);
pub const PLANE_HEIGHT: CopySpec = (
    0x4425B6,
    0x39,
    &[0xDB, 0x05, 0x18, 0x97, 0x63, 0x00],
    &[],
    8,
);
pub const FIND_RECORD: CopySpec = (
    0x4220B6,
    0x41,
    &[0x8B, 0xD0, 0x8B, 0xC2, 0x83, 0xE0, 0x0F],
    &[],
    3,
);
pub const FACE_SHARES: CopySpec = (0x442323, 0x27E, &[0xA1, 0x90, 0x98, 0x63, 0x00], &[], 90);
pub const FACE_UNDER_POINT: CopySpec = (
    0x442006,
    0x31D,
    &[0x89, 0x1D, 0x70, 0xCC, 0x5C, 0x00],
    &[101, 164, 227, 295, 354, 417, 477, 532, 774, 789],
    50,
);
pub const SEARCH_RECORD_FACES: CopySpec =
    (0x441FD9, 0x2D, &[0x8B, 0xF3, 0xC1, 0xE6, 0x05], &[26], 1);
pub const SEARCH_CELL: CopySpec = (
    0x441FBA,
    0x1F,
    &[0xC1, 0xE9, 0x15, 0xC1, 0xE8, 0x15],
    &[11, 18],
    0,
);
pub const GROUND_SEARCH: CopySpec = (
    0x441E2F,
    0x134,
    &[0x60, 0xE8, 0xD7, 0xFF, 0xFF, 0xFF],
    &[0x1, 0x27, 0x48, 0x67, 0x88, 0xA6, 0xC3, 0xDE, 0xFB, 0x115],
    21,
);
pub const PROBE_GROUND: CopySpec = (
    0x45D305,
    0x3D6,
    &[0x83, 0x3D, 0x38, 0xA2, 0x71, 0x00, 0x01],
    &[0x83, 0x138],
    115,
);
pub const PROBE_AND_KEEP: CopySpec = (0x45DC12, 0x2C, &[0xE8, 0xEE, 0xF6, 0xFF, 0xFF], &[0], 4);
pub const FIND_GROUND: CopySpec = (0x441E29, 6, &[0xE8, 0x01, 0x00, 0x00, 0x00], &[0], 0);
pub const TRY_LAST_FACE: CopySpec = (
    0x441E0C,
    0x1D,
    &[0x8B, 0x1D, 0x78, 0xCC, 0x5C, 0x00],
    &[20],
    2,
);

/// The copies of a whole search, each calling the others' copies.
pub struct TestCopies {
    pub edge_side: RedirectedCopy,
    pub plane_height: RedirectedCopy,
    pub find_record: RedirectedCopy,
    pub face_shares: RedirectedCopy,
    pub face_under_point: RedirectedCopy,
    pub search_record_faces: RedirectedCopy,
    pub search_cell: RedirectedCopy,
    pub try_last_face: RedirectedCopy,
    pub ground_search: RedirectedCopy,
    pub find_ground: RedirectedCopy,
}

impl TestGround {
    /// `faces` faces, `points` points, `records` records, all 0; the ranges' copies
    /// filled with 0xAA but for the pointers and counts, which point at these.
    pub fn new(faces: u32, points: u32, records: u32) -> Self {
        let mut blocks: Vec<Vec<u8>> = RANGES
            .iter()
            .map(|&(lo, hi)| vec![0xAA; (hi - lo) as usize])
            .collect();
        let bases: [u32; 7] = std::array::from_fn(|k| blocks[k].as_mut_ptr() as u32);
        let mut t = Self {
            blocks,
            bases,
            faces: vec![0; (faces * FACE_LEN) as usize],
            points: vec![0; 3 * points as usize],
            records: vec![0; (records * RECORD_LEN) as usize],
        };
        let g = t.ground();
        let (f, p, r) = (
            t.faces.as_mut_ptr() as u32,
            t.points.as_mut_ptr() as u32,
            t.records.as_mut_ptr() as u32,
        );
        unsafe {
            g.put(FACES, f);
            g.put(POINTS, p);
            g.put(RECORDS, r);
            g.put(FACE_COUNT, faces);
            g.put(RECORD_COUNT, records);
            for k in 0..0x200u32 {
                g.put(0x5CC3E4 + 4 * k, if k < 0x100 { u32::MAX } else { 0 });
            }
        }
        t
    }

    pub fn ground(&self) -> Ground {
        Ground {
            test: Some(self.bases),
        }
    }

    /// Everything the search may read or write, for comparing.
    pub fn regions(&self) -> Vec<Region> {
        let mut r: Vec<Region> = self
            .blocks
            .iter()
            .map(|b| (b.as_ptr() as u32, b.len()))
            .collect();
        r.push((self.faces.as_ptr() as u32, self.faces.len()));
        r.push((self.points.as_ptr() as u32, 4 * self.points.len()));
        r.push((self.records.as_ptr() as u32, self.records.len()));
        r
    }

    /// A copy of one original, its references into `RANGES` made the test's (their
    /// number checked), its calls sent by `redirect` (RVAs as IDA shows them).
    pub fn copy(
        &self,
        spec: CopySpec,
        redirect: &dyn Fn(usize) -> Option<usize>,
    ) -> Result<RedirectedCopy, String> {
        let (address, len, entry, calls, references) = spec;
        let original = game_addr(rva(address as usize));
        let copy = unsafe {
            RedirectedCopy::new(original, len, entry, calls, &|r| {
                redirect(r + crate::framework::IMAGE_BASE)
            })
        }
        .map_err(str::to_string)?;
        let g = self.ground();
        let mut patched = 0;
        for offset in 0..len - 3 {
            let at = (copy.addr() + offset) as *const u32;
            let v = unsafe { at.read_unaligned() };
            if RANGES.iter().any(|&(lo, hi)| (lo..hi).contains(&v)) {
                unsafe { copy.patch_u32(offset, g.a(v)) };
                patched += 1;
            }
        }
        if patched != references {
            return Err(format!(
                "{address:#x}: {patched} references to the ground's ranges, not {references}"
            ));
        }
        Ok(copy)
    }

    /// The whole search's copies.
    pub fn copies(&self) -> Result<TestCopies, String> {
        let edge_side = self.copy(EDGE_SIDE, &|_| None)?;
        let plane_height = self.copy(PLANE_HEIGHT, &|_| None)?;
        let find_record = self.copy(FIND_RECORD, &|_| None)?;
        let face_shares = self.copy(FACE_SHARES, &|_| None)?;
        let (e, p, s) = (edge_side.addr(), plane_height.addr(), face_shares.addr());
        let face_under_point = self.copy(FACE_UNDER_POINT, &|t| match t {
            0x4425EF => Some(e),
            0x4425B6 => Some(p),
            0x442323 => Some(s),
            _ => None,
        })?;
        let f = face_under_point.addr();
        let search_record_faces =
            self.copy(SEARCH_RECORD_FACES, &|t| (t == 0x442006).then_some(f))?;
        let (fr, sr) = (find_record.addr(), search_record_faces.addr());
        let search_cell = self.copy(SEARCH_CELL, &|t| match t {
            0x4220B6 => Some(fr),
            0x441FD9 => Some(sr),
            _ => None,
        })?;
        let try_last_face = self.copy(TRY_LAST_FACE, &|t| (t == 0x442006).then_some(f))?;
        let (l, c) = (try_last_face.addr(), search_cell.addr());
        let ground_search = self.copy(GROUND_SEARCH, &|t| match t {
            0x441E0C => Some(l),
            0x441FBA => Some(c),
            _ => None,
        })?;
        let gs = ground_search.addr();
        let find_ground = self.copy(FIND_GROUND, &|t| (t == 0x441E2F).then_some(gs))?;
        Ok(TestCopies {
            edge_side,
            plane_height,
            find_record,
            face_shares,
            face_under_point,
            search_record_faces,
            search_cell,
            try_last_face,
            ground_search,
            find_ground,
        })
    }

    /// A 5 x 5 grid of points 0x100000 apart (x, z), heights by `seed`; faces 0 .. 15
    /// over its sixteen squares, by `seed` quads or triangles, in either turn, some of
    /// kind 0x10 or with flags; then the rest of the faces by lot.
    pub fn lay_out(&mut self, seed: u32) {
        let mut s = seed.wrapping_mul(0x9E37_79B9) | 1;
        let mut next = move || {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            s >> 8
        };
        for j in 0..5u32 {
            for i in 0..5u32 {
                let k = 3 * (5 * j + i) as usize;
                if k + 2 < self.points.len() {
                    self.points[k] = i * 0x10_0000;
                    self.points[k + 1] = (next() & 0x3_FFFF).wrapping_sub(0x2_0000);
                    self.points[k + 2] = j * 0x10_0000;
                }
            }
        }
        let faces = self.faces.len() as u32 / FACE_LEN;
        let points = (self.points.len() / 3) as u32;
        for n in 0..faces {
            let f = &mut self.faces[(n * FACE_LEN) as usize..((n + 1) * FACE_LEN) as usize];
            f.fill(0);
            let r = next();
            let corners: [u32; 4] = if n < 16 && points >= 25 {
                let (i, j) = (n % 4, n / 4);
                let c = [
                    5 * j + i,
                    5 * j + i + 1,
                    5 * (j + 1) + i + 1,
                    5 * (j + 1) + i,
                ];
                let c = if r & 1 != 0 {
                    [c[3], c[2], c[1], c[0]]
                } else {
                    c
                };
                if r & 2 != 0 {
                    [c[0], c[1], c[2], c[2]]
                } else {
                    c
                }
            } else {
                std::array::from_fn(|_| next() % points.max(1))
            };
            f[0] = if r % 13 == 0 {
                0x10
            } else {
                (r >> 4) as u8 & 0xF
            };
            let flags: u16 = if r % 11 == 0 { 0x0100 } else { 0 };
            f[2..4].copy_from_slice(&flags.to_le_bytes());
            for (k, &c) in corners.iter().enumerate() {
                f[4 + 4 * k..8 + 4 * k].copy_from_slice(&c.to_le_bytes());
            }
        }
    }

    /// The test's point, and the rest of the inputs the search reads.
    pub fn set_point(&self, x: u32, z: u32, mask: u32, sse: bool) {
        let g = self.ground();
        unsafe {
            g.put(0x639824, x);
            g.put(0x63982C, z);
            g.put(0x620470, mask);
            g.put(0x621A58, u32::from(sse));
        }
    }

    /// Fills the work variables with 0xAA again (between the two runs of a case).
    pub fn clear_work(&self) {
        let g = self.ground();
        unsafe {
            std::ptr::write_bytes(g.a(0x5CCC60) as *mut u8, 0xAA, 0x18);
            std::ptr::write_bytes(g.a(0x620460) as *mut u8, 0xAA, 8);
            let keep = [g.get(0x639824), g.get(0x63982C)];
            std::ptr::write_bytes(
                g.a(0x6396FC) as *mut u8,
                0xAA,
                (0x63989C - 0x6396FC) as usize,
            );
            g.put(0x639824, keep[0]);
            g.put(0x63982C, keep[1]);
        }
    }
}

/// The points a test asks about: inside squares, on edges and corners, outside.
pub fn test_points() -> Vec<(u32, u32)> {
    let mut v = Vec::new();
    for (x, z) in [
        (0x08_0000u32, 0x08_0000u32),
        (0x18_0000, 0x0C_0000),
        (0x10_0000, 0x10_0000),
        (0x30_0000, 0x08_0000),
        (0x2F_FFFF, 0x30_0001),
        (0x00_0000, 0x00_0000),
        (0x05_0000, 0x0A_0000),
        (0x3A_0000, 0x35_0000),
        (0x50_0000, 0x50_0000),
        (0xFFF0_0000, 0x08_0000),
        (0x22_2222, 0x1E_1E1E),
        (0x0C_0000, 0x04_0000),
    ] {
        v.push((x, z));
    }
    v
}

/// A ground core as the tests run it: the registers in, the carry out.
pub type Core<'a> = &'a dyn Fn(&Ground, &mut Regs) -> Result<bool, Refused>;

/// One self-test case: `prepare`, the copy at `copy` from `before`, `prepare` again, the
/// core; the test's memory and the registers (and, with `carry`, the carry) compared.
/// Returns whether the core found the point (for counting the cases that reach it).
#[allow(clippy::too_many_arguments)] // one case's inputs, as `check_state_with`'s
pub fn check_case(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: usize,
    test: &TestGround,
    before: Regs,
    prepare: &dyn Fn(),
    core: Core,
    carry: bool,
) -> bool {
    let regions = test.regions();
    prepare();
    let mut ro = before;
    unsafe { crate::framework::call_game(&mut ro, copy) };
    let theirs = crate::service::harness::capture(&regions);
    prepare();
    let mut g = before;
    let mine = core(&test.ground(), &mut g);
    let state = crate::service::harness::capture(&regions);
    let regs_ok = match &mine {
        Ok(c) => {
            let mut left = left_of(&g, *c);
            if !carry {
                left.carry = None;
            }
            left.matches(&before, &ro)
        }
        Err(_) => false,
    };
    let same = theirs == state;
    t.check(regs_ok && same, || {
        let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
        format!(
            "{what}: regs_ok={regs_ok} (original eax {:#x} ebx {:#x} ecx {:#x} esi {:#x} edi {:#x} ebp {:#x} carry {}; rust {:?}) memory same {same} (first difference at byte {first:?})",
            ro.eax,
            ro.ebx,
            ro.ecx,
            ro.esi,
            ro.edi,
            ro.ebp,
            ro.carry(),
            mine.as_ref().map(|_| (g.eax, g.ebx, g.ecx, g.esi, g.edi, g.ebp)),
        )
    });
    matches!(mine, Ok(true))
}
