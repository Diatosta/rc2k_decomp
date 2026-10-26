//! The sound spots of the course around the view (`sub_45C605` -> `sub_45C6B8` ->
//! `sub_45C6ED`, part R3, IDA 2026-10-18): course faces of kind 0xFF with object 0x3F
//! (sound objects) in the 5 x 5 cells of `g_courseGrid` around the view, at most 0x24, into
//! `g_soundSpots` (0x14 bytes: +0 the point, +4 .. +0xC its place turned by the view and
//! scaled by `flt_5C5DA0`, +0x10 the sound, the face's +8, 1 .. 0x15).

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};

const GRID: usize = rva(0x63E17C); // g_courseGrid
const REACH: usize = rva(0x60A87C); // g_gridReach
const SPOTS: usize = rva(0x6D1AF4); // g_soundSpots
const COUNT: usize = rva(0x6D1DC4); // g_soundSpotCount
const RECORDS: usize = rva(0x5CCCB8); // g_courseRecords
const FACES: usize = rva(0x5CCCBC); // g_courseFaces
const POINTS: usize = rva(0x5CCCC0); // g_scenePoints
/// The grid and what lies past it that the original may read (as `race::course_grid`).
const GRID_DWORDS: u32 = 0x1084 / 4;

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The 25 cells' numbers, in the original's order (five across, then on by 0x1C after
/// the fifth: rows 0x20 apart, from reach - 2 on the diagonal).
fn cells() -> Vec<u32> {
    // Converted: `rc2k_core::race::view::cells_from`.
    rc2k_core::race::view::cells_from(unsafe { game_read::<u32>(REACH) }.wrapping_sub(2), 5)
}

/// What the original would read through a null or past the grid: refused before anything.
unsafe fn check() -> Result<(), Refused> {
    unsafe {
        for c in cells() {
            if c >= GRID_DWORDS {
                return Err(BadInput("a sound spot cell past the course grid").into());
            }
            let record: u32 = game_read(GRID + 4 * c as usize);
            if record >= 0x0FFF_FFFF {
                continue;
            }
            if game_read::<u32>(RECORDS) == 0
                || game_read::<u32>(FACES) == 0
                || game_read::<u32>(POINTS) == 0
            {
                return Err(BadInput("no course records, faces or points").into());
            }
        }
        Ok(())
    }
}

/// `fld tbyte; fmul dword; fstp dword`.
fn scaled(t: [u8; 10], k: u32) -> u32 {
    let (v, mut out) = ([k], 0u32);
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{t}]",
            "fmul dword ptr [{v}]",
            "fstp dword ptr [{o}]",
            t = in(reg) t.as_ptr(),
            v = in(reg) v.as_ptr(),
            o = in(reg) &raw mut out,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    out
}

/// `race::face_sound_spot` (0x45C6ED): face ebx as a sound spot when it is kind 0xFF,
/// object 0x3F, its sound (+8) 1 .. 0x15 and there is room (fewer than 0x24): its point and
/// place (point + offset +0x30 .. into `dword_63501C` ..) turned by `g_viewMatrix` and
/// scaled. (Its test for sound 0x1B2 needing lamps can never hold: a byte.) Gives the
/// registers it leaves.
pub unsafe fn face_sound_spot(entry: &Regs) -> Regs {
    use crate::race::skidmarks::int_80;
    unsafe {
        let mut r = *entry;
        let face = game_read::<u32>(FACES).wrapping_add(r.ebx << 6);
        r.esi = face;
        // Converted: `rc2k_core::race::view::{sound_object, takes_spot, spot_place}`
        // (the turn and scale stay the x87's: the callers compare exactly).
        use rc2k_core::race::view::{MOST_SPOTS, sound_object, spot_place, takes_spot};
        let byte = |o: u32| ((face + o) as *const u8).read();
        let kind = byte(0);
        if kind != 0xFF || !sound_object(kind, byte(1)) {
            return r;
        }
        let count: u32 = game_read(COUNT);
        r.eax = count;
        if count == MOST_SPOTS {
            return r;
        }
        let spot = game_addr(SPOTS) as u32 + count * 0x14;
        r.edi = spot;
        let sound = byte(8) as u32;
        r.eax = sound;
        if !takes_spot(count, sound) {
            return r;
        }
        let put = |o: u32, v: u32| ((spot + o) as *mut u32).write_unaligned(v);
        put(0x10, sound);
        let point = get(face + 4);
        put(0, point);
        let p = point.wrapping_mul(12).wrapping_add(game_read(POINTS));
        let place = spot_place(
            [get(p), get(p + 4), get(p + 8)],
            [get(face + 0x30), get(face + 0x34), get(face + 0x38)],
        );
        game_write(rva(0x63501C), place);
        (r.eax, r.ebx, r.ecx) = (place[0], place[1], place[2]);
        r.esi = game_addr(rva(0x5EA2CC)) as u32;
        let t = crate::scene::rotate_point::rotate_point(r.esi, place.map(int_80));
        let k: u32 = game_read(rva(0x5C5DA0));
        for (i, v) in t.into_iter().enumerate() {
            put(4 + 4 * i as u32, scaled(v, k));
        }
        game_write(COUNT, count + 1);
        r
    }
}

/// `race::cell_sound_spots` (0x45C6B8): each face of cell ecx's record (`g_courseGrid[c]`,
/// none from 0x0FFFFFFF; the record's +8 .. +0xC, none for -1 or a first past the last) by
/// `face_sound_spot` (inside `pusha` .. `popa`). Gives the registers it leaves.
pub unsafe fn cell_sound_spots(entry: &Regs) -> Regs {
    unsafe {
        let mut r = *entry;
        // Converted: `rc2k_core::race::view::record_faces`.
        r.ebx = game_read(GRID + 4 * r.ecx as usize);
        if r.ebx >= rc2k_core::race::view::NO_RECORD {
            return r;
        }
        r.esi = game_read::<u32>(RECORDS).wrapping_add(r.ebx << 5);
        r.ebx = get(r.esi + 8);
        if rc2k_core::race::view::record_faces(r.ebx, get(r.esi + 0xC)).is_none() {
            return r;
        }
        loop {
            face_sound_spot(&r);
            r.ebx = r.ebx.wrapping_add(1);
            if r.ebx > get(r.esi + 0xC) {
                break;
            }
        }
        r
    }
}

/// `race::find_sound_spots` (0x45C605): `g_soundSpotCount` 0, then the 5 x 5 cells from
/// (`g_gridReach` - 2) x 0x21 (rows 0x20 apart) by `cell_sound_spots`. Gives the registers
/// it leaves (ecx the last cell). Refuses (before anything) a cell past the grid, or no
/// records, faces or points when a cell has a record.
pub unsafe fn find_sound_spots(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        check()?;
        game_write(COUNT, 0u32);
        let mut r = *entry;
        for c in cells() {
            r.ecx = c;
            r = cell_sound_spots(&r);
        }
        Ok(r)
    }
}

/// A test course for the three hooks: faces (some sound objects, some full rooms' worth),
/// records over them, points; the grid's cells around reach 4 pointed at the records.
pub struct SpotStage {
    pub faces: Vec<u8>,
    pub records: Vec<u32>,
    pub points: Vec<u32>,
    pub kept: Vec<crate::service::harness::Region>,
    pub saved: Vec<u8>,
}

impl SpotStage {
    pub fn new() -> Self {
        use crate::service::harness::{at, capture};
        let mut faces = vec![0u8; 0x40 * 64];
        for n in 0..64usize {
            let f = &mut faces[0x40 * n..0x40 * (n + 1)];
            f[0] = if n % 5 == 4 { 3 } else { 0xFF };
            f[1] = if n % 7 == 6 { 0x20 } else { 0x3F };
            f[4] = (n % 10) as u8;
            f[8] = [0u8, 1, 7, 0x15, 0x16, 0xB2][n % 6];
            f[0x30..0x34].copy_from_slice(&((n as i32 - 20) * 0x1000).to_le_bytes());
            f[0x34..0x38].copy_from_slice(&0x200i32.to_le_bytes());
            f[0x38..0x3C].copy_from_slice(&((n as i32) * 0x3000).to_le_bytes());
        }
        // record: +8 the first face, +0xC the last.
        let mut records = vec![0u32; 8 * 8];
        for (k, (a, b)) in [
            (0u32, 9u32),
            (10, 10),
            (u32::MAX, 3),
            (20, 15),
            (16, 63),
            (30, 40),
            (0, 0),
            (41, 50),
        ]
        .into_iter()
        .enumerate()
        {
            records[8 * k + 2] = a;
            records[8 * k + 3] = b;
        }
        let points: Vec<u32> = (0..30u32)
            .map(|k| k.wrapping_mul(0x1_2345) % 0x8_0000)
            .collect();
        let g = |a: usize, len: usize| (game_addr(a) as u32, len);
        let kept = vec![
            g(SPOTS, 0x24 * 0x14),
            at(COUNT),
            g(rva(0x63501C), 0xC),
            g(GRID, 0x1084),
            at(REACH),
            at(RECORDS),
            at(FACES),
            at(POINTS),
        ];
        let saved = capture(&kept);
        Self {
            faces,
            records,
            points,
            kept,
            saved,
        }
    }

    /// Everything put back, then the test course in place with reach `reach` and `count`
    /// spots already.
    pub fn prepare(&self, reach: u32, count: u32) {
        use crate::service::harness::restore;
        restore(&self.kept, &self.saved);
        unsafe {
            game_write(RECORDS, self.records.as_ptr() as u32);
            game_write(FACES, self.faces.as_ptr() as u32);
            game_write(POINTS, self.points.as_ptr() as u32);
            game_write(REACH, reach);
            game_write(COUNT, count);
            for (k, c) in cells().into_iter().enumerate() {
                let v = if k % 4 == 3 {
                    0x0FFF_FFFF
                } else {
                    (k % 8) as u32
                };
                game_write(GRID + 4 * c as usize, v);
            }
        }
    }

    /// What the hooks write.
    pub fn regions(&self) -> Vec<crate::service::harness::Region> {
        self.kept[..3].to_vec()
    }

    /// What the hooks write, on the game's variables (for shadow mode).
    pub fn live_regions() -> Vec<crate::service::harness::Region> {
        use crate::service::harness::at;
        vec![
            (game_addr(SPOTS) as u32, 0x24 * 0x14),
            at(COUNT),
            (game_addr(rva(0x63501C)) as u32, 0xC),
        ]
    }

    pub fn end(&self) {
        crate::service::harness::restore(&self.kept, &self.saved);
    }
}

impl Default for SpotStage {
    fn default() -> Self {
        Self::new()
    }
}
