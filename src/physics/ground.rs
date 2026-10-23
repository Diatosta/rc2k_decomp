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
use crate::screens::harness::Left;
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
