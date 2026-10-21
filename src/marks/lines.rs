//! `marks::init_marks`' step 14, its measuring (IDA, 2026-10-16): the lengths of the
//! lines between two objects' faces (each face's points against the other object's same
//! face's), summed into the eight settings blocks (at `g_marksSettings` + 0x29C, 0x4C8
//! apart) by the face's kind, and then made shares of what they were.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::objects::{
    FACE_COUNTS_RVA, FACES, FACES_RVA, FIRST_FACE_RVA, FIRST_POINT_RVA, POINT_COUNTS_RVA,
    POINTS_RVA,
};
use rc2k_core::marks::lines as core;

pub const SETTINGS_RVA: usize = rva(0x767A28); // g_marksSettings
pub const BLOCKS_AT: u32 = core::BLOCKS_AT;
pub const BLOCK_LEN: u32 = core::BLOCK_LEN;
pub const MOST_BLOCKS: u32 = core::MOST_BLOCKS;
/// The last line's x, y, z differences (`dword_93C870` .. `dword_93C878`).
pub const DIFFERENCE_RVA: usize = rva(0x93C870);
pub const CURRENT_RVA: usize = rva(0x93C8A8); // dword_93C8A8: the block being measured
pub const EVERY_RVA: usize = rva(0x93C8B4); // dword_93C8B4: measure every block
pub const SHARES_RVA: usize = rva(0x93C8B8); // dword_93C8B8: make the shares
pub const ANY_RVA: usize = rva(0x93C8BC); // dword_93C8BC: a marked block measured
const HUNDRED_RVA: usize = rva(0x7651E8); // flt_7651E8, 100.0
pub const TAKEN_RVA: usize = rva(0x93C6CC); // g_marksTaken: the last block taken
const BLOCK_START_RVA: usize = rva(0x93C738); // g_marksBlockStart: eight floats

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) };
}

unsafe fn byte(at: u32) -> u8 {
    unsafe { *(at as *const u8) }
}

/// The lines of two faces (`from`, `to`): for each of `from`'s points (+0x10, 0x10 apart)
/// the length to `to`'s, added to the whole number at `sum`. Leaves eax, ebx, ecx the last
/// differences; `then` runs after each difference (the measuring's `mov ebx`).
unsafe fn lines(g: &mut Regs, from: u32, to: u32, sum: u32, then: &dyn Fn(&mut Regs)) {
    unsafe { lines_with(g, from, to, sum, then, x87_length) }
}

/// The x87's `length_plus_int`.
fn x87_length(d: [i32; 3], sum: i32) -> i32 {
    x87::length_plus_int(d[0], d[1], d[2], sum)
}

/// `lines` with the length worked out by `length`.
unsafe fn lines_with(
    g: &mut Regs,
    from: u32,
    to: u32,
    sum: u32,
    then: &dyn Fn(&mut Regs),
    length: fn([i32; 3], i32) -> i32,
) {
    unsafe {
        let count = get(from);
        let points: u32 = game_read(POINTS_RVA);
        let mut n = 0u32;
        while n != count {
            let (a, b) = (get(from + 0x10 + 0x10 * n), get(to + 0x10 + 0x10 * n));
            let p = points.wrapping_add(a.wrapping_mul(12));
            let q = points.wrapping_add(b.wrapping_mul(12));
            let d = core::difference([0, 4, 8].map(|o| get(p + o)), [0, 4, 8].map(|o| get(q + o)));
            [g.eax, g.ebx, g.ecx] = d;
            game_write(DIFFERENCE_RVA, d);
            then(g);
            put(sum, length(d.map(|v| v as i32), get(sum) as i32) as u32);
            n += 1;
        }
        g.eax = count;
    }
}

/// `marks::measure_slot_lines` (0x4D1167): faces esi and edi, block ebp: for each of the
/// block's 8 slots whose kind (the low byte of +0xD0 ..) is the face's +7, its lines into
/// the slot's sum (+0xF0 ..). Leaves eax 8, ebx and ecx the last differences. The x87's,
/// for `measure_lines` (unconverted).
pub unsafe fn measure_slot_lines(entry: &Regs) -> Regs {
    unsafe { slot_lines_with(entry, x87_length) }
}

/// The converted `measure_slot_lines` (plain floats): the hook's.
// Converted: `rc2k_core::marks::lines::{slot_takes, difference, length_plus}`
pub unsafe fn measure_slot_lines_plain(entry: &Regs) -> Regs {
    unsafe { slot_lines_with(entry, core::length_plus) }
}

unsafe fn slot_lines_with(entry: &Regs, length: fn([i32; 3], i32) -> i32) -> Regs {
    let mut g = *entry;
    unsafe {
        let (from, to, block) = (g.esi, g.edi, g.ebp);
        for k in 0..core::SLOTS {
            if core::slot_takes(get(block + 0xD0 + 4 * k), byte(from + 7)) {
                lines_with(&mut g, from, to, block + 0xF0 + 4 * k, &|_| {}, length);
            }
        }
        g.eax = core::SLOTS;
    }
    g
}

/// `marks::measure_kind_lines` (0x4D1225): faces esi and edi, block ebp: for each of the
/// block's 0x11 kinds (the low byte of +0x130 ..) that is the face's +6, with the face's
/// +0xB 1: its point count added to +0x240 .., its lines into +0x1B8 ... Leaves eax 0x11,
/// ebx and ecx the last differences. The x87's, for `measure_lines` (unconverted).
pub unsafe fn measure_kind_lines(entry: &Regs) -> Regs {
    unsafe { kind_lines_with(entry, x87_length) }
}

/// The converted `measure_kind_lines` (plain floats): the hook's.
// Converted: `rc2k_core::marks::lines::{kind_takes, difference, length_plus}`
pub unsafe fn measure_kind_lines_plain(entry: &Regs) -> Regs {
    unsafe { kind_lines_with(entry, core::length_plus) }
}

unsafe fn kind_lines_with(entry: &Regs, length: fn([i32; 3], i32) -> i32) -> Regs {
    let mut g = *entry;
    unsafe {
        let (from, to, block) = (g.esi, g.edi, g.ebp);
        for k in 0..core::KINDS {
            if core::kind_takes(get(block + 0x130 + 4 * k), byte(from + 6), byte(from + 0xB)) {
                let count = block + 0x240 + 4 * k;
                put(count, get(count).wrapping_add(get(from)));
                lines_with(&mut g, from, to, block + 0x1B8 + 4 * k, &|_| {}, length);
            }
        }
        g.eax = core::KINDS;
    }
    g
}

/// The game's tables for `core::measured`: the settings' blocks and the objects' faces.
struct GameBlocks(u32);

impl core::Blocks for GameBlocks {
    fn marked(&self, k: u32) -> bool {
        unsafe { get(core::block_at(self.0, k) + 0xC) != 0 }
    }
    fn objects(&self, k: u32) -> [u32; 2] {
        let block = core::block_at(self.0, k);
        unsafe { [get(block), get(block + 4)] }
    }
    fn face_count(&self, object: u32) -> u32 {
        unsafe { game_read(FACE_COUNTS_RVA + 4 * object as usize) }
    }
    fn first_face(&self, object: u32) -> u32 {
        unsafe { game_read(FIRST_FACE_RVA + 4 * object as usize) }
    }
}

/// The blocks `measure_lines` would measure (`core::measured`): (block, its first faces,
/// its face count).
unsafe fn measured(settings: u32, every: bool) -> Result<Vec<(u32, u32, u32, u32)>, Refused> {
    let count = unsafe { get(settings + 0x5C) };
    let blocks = core::measured(count, every, &GameBlocks(settings), FACES)?;
    Ok(blocks
        .into_iter()
        .map(|m| {
            (
                core::block_at(settings, m.block),
                m.first[0],
                m.first[1],
                m.faces,
            )
        })
        .collect())
}

/// `marks::measure_lines` (0x4D0EB4): `dword_93C8BC` 0; for the blocks (+0x5C of the
/// settings, the last first), every one with `dword_93C8B4` 1 or else those with +0xC set
/// (then `dword_93C8BC` 1): the block in `dword_93C8A8`, its sums cleared (+0x1C .. +0x30,
/// +0xF0 .., +0x1B8 .., +0x240 ..); for each face of its first object (+0) with the other's
/// (+4) same: a face of kind 0x1C (+6) with a line kind (+0xB) 1 .. 6 has its lines summed
/// into +0x18 + 4 kind; then `measure_slot_lines`, `measure_kind_lines`. With
/// `dword_93C8B8` 1 each sum then becomes 100 less its share of 100 of what it was
/// (+0x34 .., +0x110 .., +0x1FC ..), with a `finit` after the first six (as the original).
/// Refuses more than 8 blocks, an object past 0xFF, faces past `g_faces`.
// Converted: `rc2k_core::marks::lines::{measured, block_at}` (the lengths and shares
// stay x87: its `finit` between)
pub unsafe fn measure_lines(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        let every = game_read::<u32>(EVERY_RVA) == 1;
        let blocks = measured(settings, every)?;
        game_write(ANY_RVA, 0u32);
        g.ebp = settings;
        let count = get(settings + 0x5C) as i32;
        let mut todo = blocks.into_iter().peekable();
        for k in (0..count.max(0) as u32).rev() {
            g.ebp = block_at(settings, k);
            let Some(&(block, first_a, first_b, faces)) = todo.peek() else {
                continue;
            };
            if block != g.ebp {
                continue;
            }
            todo.next();
            if !every {
                game_write(ANY_RVA, 1u32);
            }
            game_write(CURRENT_RVA, block);
            for o in (0x1C..=0x30).step_by(4) {
                put(block + o, 0);
            }
            for n in 0..8 {
                put(block + 0xF0 + 4 * n, 0);
            }
            for n in 0..0x11 {
                put(block + 0x1B8 + 4 * n, 0);
                put(block + 0x240 + 4 * n, 0);
            }
            g.eax = faces;
            g.esi = first_a;
            g.edi = first_b;
            for _ in 0..(faces as i32).max(0) {
                let (a, b) = (g.esi, g.edi);
                let from: u32 = game_read(FACES_RVA + 4 * a as usize);
                let to: u32 = game_read(FACES_RVA + 4 * b as usize);
                if byte(from + 6) == 0x1C {
                    g.eax = get(from);
                    g.edx = byte(from + 0xB) as u32;
                    if (1..=6).contains(&(g.edx as i32)) {
                        let sum = block + 0x18 + 4 * g.edx;
                        lines(&mut g, from, to, sum, &|r: &mut Regs| r.ebx = block);
                    }
                }
                g.esi = from;
                g.edi = to;
                g = measure_slot_lines(&g);
                g = measure_kind_lines(&g);
                g.esi = a + 1;
                g.edi = b + 1;
            }
            if game_read::<u32>(SHARES_RVA) == 1 {
                let hundred: u32 = game_read(HUNDRED_RVA);
                let share = |at: u32, of: u32| {
                    put(
                        at,
                        x87::share_left_to_int(get(at) as i32, get(of) as i32, hundred) as u32,
                    )
                };
                for o in (0x1C..=0x30).step_by(4) {
                    share(block + o, block + o + 0x18);
                }
                x87::finit();
                for n in (0..8).rev() {
                    share(block + 0xF0 + 4 * n, block + 0x110 + 4 * n);
                }
                for n in (0..0x11).rev() {
                    share(block + 0x1B8 + 4 * n, block + 0x1FC + 4 * n);
                }
                g.eax = 0;
            }
        }
    }
    Ok(g)
}

fn block_at(settings: u32, k: u32) -> u32 {
    core::block_at(settings, k)
}

/// `marks::measure_starting_sums` (0x4D0DB5): `measure_lines` with every block and no
/// shares; then for each block (the last first) its sums moved to where the shares are made
/// from (+0x1C .. +0x30 to +0x34 .., +0xF0 .. to +0x110 .., +0x1B8 .. to +0x1FC ..) and
/// cleared; then `g_marksMeasureEvery` 0 and `g_marksMakeShares` 1. Leaves eax the first
/// block's last moved, ebx 0, ebp that block (the settings with none). Refuses as
/// `measure_lines` with every block, before writing anything.
// Converted: `rc2k_core::marks::lines::{measured, block_at}`
pub unsafe fn measure_starting_sums(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let settings: u32 = game_read(SETTINGS_RVA);
        measured(settings, true)?;
        game_write(EVERY_RVA, 1u32);
        game_write(SHARES_RVA, 0u32);
        let mut g = measure_lines(entry)?;
        let settings: u32 = game_read(SETTINGS_RVA);
        g.ebp = settings;
        let count = get(settings + 0x5C) as i32;
        for k in (0..count.max(0) as u32).rev() {
            let block = block_at(settings, k);
            g.ebp = block;
            for o in (0x1C..=0x30).step_by(4) {
                g.eax = get(block + o);
                put(block + o + 0x18, g.eax);
            }
            for o in (0x1C..=0x30).step_by(4) {
                put(block + o, 0);
            }
            for (from, to, n) in [(0xF0, 0x110, 8u32), (0x1B8, 0x1FC, 0x11)] {
                for k in (0..n).rev() {
                    g.ebx = k;
                    g.eax = get(block + from + 4 * k);
                    put(block + to + 4 * k, g.eax);
                    put(block + from + 4 * k, 0);
                }
            }
        }
        game_write(EVERY_RVA, 0u32);
        game_write(SHARES_RVA, 1u32);
        Ok(g)
    }
}

/// `marks::copy_block_points` (0x4D0A82): the points of the block's (ebp) object (+0):
/// `g_objectPointCount2` of 12 bytes from `g_objectFirstPoint` in `g_objectPoints`, to the
/// block's +0x8C. Leaves eax the last dword copied (the count with none), esi and edi past
/// them. Refuses an object past 0xFF.
// Converted: `rc2k_core::marks::lines::check_object`
pub unsafe fn copy_block_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let object = get(g.ebp);
        core::check_object(object)?;
        g.edi = get(g.ebp + 0x8C);
        g.eax = game_read(POINT_COUNTS_RVA + 4 * object as usize);
        let count = g.eax as i32;
        let first: u32 = game_read(FIRST_POINT_RVA + 4 * object as usize);
        g.esi = first.wrapping_mul(12).wrapping_add(game_read(POINTS_RVA));
        for _ in 0..count.max(0) {
            for k in 0..3 {
                g.eax = get(g.esi + 4 * k);
                put(g.edi + 4 * k, g.eax);
            }
            g.esi = g.esi.wrapping_add(12);
            g.edi = g.edi.wrapping_add(12);
        }
    }
    Ok(g)
}

/// Where `copy_block_points` writes for the block ebp.
pub fn point_regions(block: u32) -> Vec<crate::service::harness::Region> {
    unsafe {
        let object = get(block);
        if core::check_object(object).is_err() {
            return Vec::new();
        }
        let count: u32 = game_read(POINT_COUNTS_RVA + 4 * object as usize);
        vec![(get(block + 0x8C), 12 * (count as i32).max(0) as usize)]
    }
}

/// `marks::set_up_blocks` (0x4D05BD), step 14: `measure_starting_sums` (place 1 of the
/// part); then for each block (the last first) +0x90 .. +0xCC -1, a block of 12 bytes a
/// point of its object taken on its +0x4C (place 0, the registers kept, as inside the
/// original's `pusha`) into `g_marksTaken` and +0x8C, `copy_block_points` (place 2), and
/// +0x90 .. +0xAC the eight floats of `g_marksBlockStart`. Leaves the registers as the first
/// block's copy (ebx its +0x4C, ebp it). Refuses as `measure_starting_sums`, before
/// anything is taken.
// Converted: `rc2k_core::marks::lines::{measured, block_at}`
pub unsafe fn set_up_blocks(
    entry: &Regs,
    part: crate::race::level_init::Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        measured(game_read(SETTINGS_RVA), true)?;
        part(1, &mut g)?;
        let settings: u32 = game_read(SETTINGS_RVA);
        g.ebp = settings;
        let count = get(settings + 0x5C) as i32;
        for k in (0..count.max(0) as u32).rev() {
            let block = block_at(settings, k);
            g.ebp = block;
            g.eax = u32::MAX;
            for o in (0x90..=0xCC).step_by(4) {
                put(block + o, g.eax);
            }
            g.esi = get(block);
            g.eax = game_read::<u32>(POINT_COUNTS_RVA + 4 * g.esi as usize).wrapping_mul(12);
            g.ebx = block + 0x4C;
            let mut taken = g;
            part(0, &mut taken)?;
            game_write(TAKEN_RVA, taken.eax);
            put(block + 0x8C, taken.eax);
            part(2, &mut g)?;
            for n in 0..8 {
                let v = x87::loaded(game_read(BLOCK_START_RVA + 4 * n));
                put(block + 0x90 + 4 * n as u32, v);
            }
        }
    }
    Ok(g)
}

/// In the game: place 0 `memory::allocate`, 1 `measure_starting_sums`, 2
/// `copy_block_points`.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => return crate::race::level_init::game_part(0, r),
            1 => *r = measure_starting_sums(r)?,
            _ => *r = copy_block_points(r)?,
        }
    }
    Ok(())
}

/// A stand-in handler for `set_up_blocks`' test: noted; place 0 gives `GIVES` with the
/// carry clear, 1 and 2 run the Rust versions.
pub fn give(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    let done = unsafe {
        match place {
            0 => {
                r.set_carry(false);
                return;
            }
            1 => measure_starting_sums(r),
            _ => copy_block_points(r),
        }
    };
    if let Ok(left) = done {
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        };
    }
}

/// The same as a part, for the Rust side.
pub fn test_give(place: u32, r: &mut Regs) -> Result<(), Refused> {
    give(place, r);
    Ok(())
}

/// What the measuring writes: the eight blocks, the differences, the current block, the
/// mark.
pub fn line_regions() -> Vec<crate::service::harness::Region> {
    use crate::service::harness::at;
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    vec![
        (
            settings.wrapping_add(BLOCKS_AT),
            (BLOCK_LEN * MOST_BLOCKS) as usize,
        ),
        (crate::framework::game_addr(DIFFERENCE_RVA) as u32, 12),
        at(CURRENT_RVA),
        at(ANY_RVA),
    ]
}

// ---- for the self-tests ----

/// Two objects (0xF0, 0xF1) of 0x20 faces and 0x20 points each (`g_faces` 0xBF00 ..,
/// 0xBF40 ..), points
/// and settings of the test's, filled by lot; the game's entries put back when dropped.
pub struct TestScene {
    pub settings: Vec<u8>,
    pub faces: Vec<u8>,
    pub points: Vec<u8>,
    inputs: Vec<crate::service::harness::Region>,
    saved: Vec<u8>,
}

pub const TEST_OBJECTS: [u32; 2] = [0xF0, 0xF1];
pub const TEST_FIRST_FACES: [u32; 2] = [0xBF00, 0xBF40];
pub const TEST_FACES: u32 = 0x20;
const FACE_LEN: u32 = 0x60;
const TEST_POINTS: u32 = 0x40;
/// Each test object's points (the first's from 0, the second's after them).
pub const TEST_OBJECT_POINTS: u32 = 0x20;

impl TestScene {
    pub fn new() -> Self {
        use crate::framework::game_addr;
        use crate::service::harness::{at, capture};
        let mut inputs = vec![
            at(SETTINGS_RVA),
            at(POINTS_RVA),
            at(EVERY_RVA),
            at(SHARES_RVA),
        ];
        for (o, f) in TEST_OBJECTS.into_iter().zip(TEST_FIRST_FACES) {
            inputs.push(at(FACE_COUNTS_RVA + 4 * o as usize));
            inputs.push(at(FIRST_FACE_RVA + 4 * o as usize));
            inputs.push((game_addr(FACES_RVA) as u32 + 4 * f, 4 * TEST_FACES as usize));
            inputs.push(at(POINT_COUNTS_RVA + 4 * o as usize));
            inputs.push(at(FIRST_POINT_RVA + 4 * o as usize));
        }
        let saved = capture(&inputs);
        let s = Self {
            settings: vec![0; (BLOCKS_AT + BLOCK_LEN * MOST_BLOCKS) as usize],
            faces: vec![0; (2 * TEST_FACES * FACE_LEN) as usize],
            points: vec![0; (12 * TEST_POINTS) as usize],
            inputs,
            saved,
        };
        unsafe {
            game_write(SETTINGS_RVA, s.settings.as_ptr() as u32);
            game_write(POINTS_RVA, s.points.as_ptr() as u32);
            for (n, (o, f)) in TEST_OBJECTS.into_iter().zip(TEST_FIRST_FACES).enumerate() {
                game_write(FACE_COUNTS_RVA + 4 * o as usize, TEST_FACES);
                game_write(FIRST_FACE_RVA + 4 * o as usize, f);
                game_write(POINT_COUNTS_RVA + 4 * o as usize, TEST_OBJECT_POINTS);
                game_write(
                    FIRST_POINT_RVA + 4 * o as usize,
                    n as u32 * TEST_OBJECT_POINTS,
                );
                for k in 0..TEST_FACES {
                    game_write(
                        FACES_RVA + 4 * (f + k) as usize,
                        s.face(n as u32 * TEST_FACES + k),
                    );
                }
            }
        }
        s
    }

    pub fn face(&self, n: u32) -> u32 {
        self.faces.as_ptr() as u32 + FACE_LEN * n
    }

    pub fn block(&self, k: u32) -> u32 {
        self.settings.as_ptr() as u32 + BLOCKS_AT + BLOCK_LEN * k
    }

    /// Everything filled by lot from `seed`, `blocks` blocks; no divisor 0.
    pub fn fill(&self, seed: u32, blocks: u32) {
        let mut s = seed;
        let mut next = || {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            s >> 8
        };
        unsafe {
            let base = self.settings.as_ptr() as *mut u8;
            for n in 0..self.settings.len() {
                *base.add(n) = next() as u8;
            }
            put(self.settings.as_ptr() as u32 + 0x5C, blocks);
            for k in 0..MOST_BLOCKS {
                let b = self.block(k);
                put(b, TEST_OBJECTS[0]);
                put(b + 4, TEST_OBJECTS[1]);
                put(b + 0xC, next() % 3 / 2);
                for n in 0..8 {
                    put(b + 0xD0 + 4 * n, next() % 4);
                }
                for n in 0..0x11 {
                    put(b + 0x130 + 4 * n, [0x1C, 0x10, 0x11][next() as usize % 3]);
                }
                for o in (0x34..=0x48)
                    .step_by(4)
                    .chain((0x110..0x130).step_by(4))
                    .chain((0x1FC..0x240).step_by(4))
                {
                    put(b + o, next() % 0x5000 + 1);
                }
            }
            for n in 0..2 * TEST_FACES {
                let f = self.face(n);
                for o in 0..FACE_LEN {
                    *((f + o) as *mut u8) = next() as u8;
                }
                put(f, next() % 5);
                *((f + 6) as *mut u8) = [0x1C, 0x10, 0x11][next() as usize % 3];
                *((f + 7) as *mut u8) = (next() % 4) as u8;
                *((f + 0xB) as *mut u8) = (next() % 8) as u8;
                for p in 0..5 {
                    put(f + 0x10 + 0x10 * p, next() % TEST_POINTS);
                }
            }
            for n in 0..3 * TEST_POINTS {
                put(
                    self.points.as_ptr() as u32 + 4 * n,
                    (next() % 0x2_0000).wrapping_sub(0x1_0000),
                );
            }
        }
    }
}

impl Default for TestScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestScene {
    fn drop(&mut self) {
        crate::service::harness::restore(&self.inputs, &self.saved);
    }
}
