//! `make_masks`' shape set (`g_marksShapeSet`, a `MarkShapes` of 0xAAC bytes; IDA,
//! 2026-10-17): two lists of points (the masks' points lists A and B), turned by two
//! matrices into `g_marksTurnedA` / `B` and, offset, into the points their targets name;
//! and the two farthest apart of each list on one face.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::masks::{POINTS_A, POINTS_B, PointList};
use crate::math::x87;
use crate::scene::matrix3::{
    Axis, extended, multiply_matrices3, transform_int_points, turn_matrix3, turnable,
};
use crate::service::harness::{Region, at};

pub const SET_RVA: usize = rva(0x885A6C); // g_marksShapeSet
pub const TURNED_RVA: [usize; 2] = [rva(0x8853F4), rva(0x8856F4)]; // g_marksTurnedA, _B
pub const FARTHEST_RVA: [usize; 2] = [rva(0x884FB4), rva(0x884FBC)]; // g_marksFarthestA, _B
const TURN_X_RVA: usize = rva(0x884E60); // g_marksShapeTurnX
const TURN_RVA: [usize; 2] = [rva(0x884F50), rva(0x884F80)]; // g_marksShapeTurnA, _B
const TILT_RVA: usize = rva(0x811BC3); // g_marksShapeTilt
const SCRATCH_RVA: usize = rva(0x764F40); // dword_764F40
pub const MOST_POINTS: u32 = 0x40;

/// `MarkShapes`' fields for list n (0 A, 1 B): count, targets, points, offset, matrix.
const COUNT: [u32; 2] = [0x38, 0x3C];
const TARGETS: [u32; 2] = [0x40, 0x140];
const POINTS: [u32; 2] = [0x440, 0x740];
const OFFSET: [u32; 2] = [0xA40, 0xA4C];
const MATRIX: [u32; 2] = [0xA58, 0xA88];

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// `marks::find_farthest_pairs` (0x4BDD1D): for points lists A and B, of the pairs of
/// their points on one face (`g_marksPointsAFaces`), the farthest apart by
/// `g_marksTurnedA` / `B` (the length rounded; a later pair as far wins) into
/// `g_marksFarthestA` / `B` (the inner then the outer number). Leaves the registers as
/// the last pair looked at leaves them (edx the farthest of B). Refuses a list of more
/// than 0x40.
pub unsafe fn find_farthest_pairs(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let lists: [&PointList; 2] = [&POINTS_A, &POINTS_B];
        for list in lists {
            if game_read::<u32>(list.count) as i32 > MOST_POINTS as i32 {
                return Err(BadInput("a points list of more than 0x40").into());
            }
        }
        for (list, n) in lists.into_iter().zip(0..) {
            let faces = game_addr(list.faces) as u32;
            let points = game_addr(TURNED_RVA[n]) as u32;
            let count = (game_read::<u32>(list.count) as i32).max(0) as u32;
            g.edx = 0;
            for j in (0..count).rev() {
                for i in (0..count).rev() {
                    g.esi = i;
                    g.edi = j;
                    if i == j {
                        continue;
                    }
                    g.eax = get(faces + 4 * i);
                    if g.eax != get(faces + 4 * j) {
                        continue;
                    }
                    g.esi = i * 12;
                    g.edi = j * 12;
                    let d = |o: u32| get(points + g.esi + o).wrapping_sub(get(points + g.edi + o));
                    [g.eax, g.ebx, g.ecx] = [d(0), d(4), d(8)];
                    g.ebp = x87::int_length3(g.eax as i32, g.ebx as i32, g.ecx as i32) as u32;
                    if g.edx as i32 > g.ebp as i32 {
                        continue;
                    }
                    g.edx = g.ebp;
                    g.eax = i;
                    g.ebx = j;
                    game_write(FARTHEST_RVA[n], [i, j]);
                }
            }
        }
    }
    Ok(g)
}

/// What `find_farthest_pairs` writes.
pub fn farthest_regions() -> Vec<Region> {
    FARTHEST_RVA
        .iter()
        .map(|&r| (game_addr(r) as u32, 8))
        .collect()
}

/// What `turn_shape_points` refuses for turns ebx, ecx.
pub unsafe fn turn_shape_checks(ebx: u32, ecx: u32) -> Result<(), Refused> {
    unsafe {
        let set = game_addr(SET_RVA) as u32;
        for c in COUNT {
            let n = get(set + c);
            if n as i32 > MOST_POINTS as i32 || n == 0x8000_0000 {
                return Err(BadInput("a shape list of more than 0x40").into());
            }
        }
        for a in [extended(game_read(TILT_RVA)), extended(ebx), extended(ecx)] {
            turnable(&a)?;
        }
    }
    Ok(())
}

/// `marks::turn_shape_points` (0x4BD5ED): the turn about X by `g_marksShapeTilt` and about
/// Z by ebx and by ecx (floats; `dword_764F40` left holding ecx); X times each into the
/// shape set's two matrices; its two lists of points turned by them into `g_marksTurnedA` /
/// `B`, and each turned point plus the list's offset (`fiadd`) into the point its target
/// names (the last first). Leaves eax the set, the rest as the last list leaves them.
/// Refuses a list of more than 0x40 or of 0x80000000, or an angle `fsincos` cannot take,
/// before writing.
pub unsafe fn turn_shape_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        turn_shape_checks(g.ebx, g.ecx)?;
        let set = game_addr(SET_RVA) as u32;
        let angles = [
            extended(game_read(TILT_RVA)),
            extended(g.ebx),
            extended(g.ecx),
        ];
        g.eax = set;
        let a = |r: usize| game_addr(r) as u32;
        turn_matrix3(Axis::X, a(TURN_X_RVA), angles[0])?;
        game_write(SCRATCH_RVA, g.ebx);
        turn_matrix3(Axis::Z, a(TURN_RVA[0]), angles[1])?;
        game_write(SCRATCH_RVA, g.ecx);
        turn_matrix3(Axis::Z, a(TURN_RVA[1]), angles[2])?;
        for n in 0..2 {
            g = multiply_matrices3(&Regs {
                esi: a(TURN_X_RVA),
                edi: a(TURN_RVA[n]),
                ebp: set + MATRIX[n],
                ..g
            });
        }
        for n in 0..2 {
            g = transform_int_points(&Regs {
                ebp: set + MATRIX[n],
                esi: set + POINTS[n],
                edi: a(TURNED_RVA[n]),
                ecx: get(set + COUNT[n]),
                ..g
            })?;
        }
        for n in 0..2 {
            let turned = a(TURNED_RVA[n]);
            for k in (0..(get(set + COUNT[n]) as i32).max(0) as u32).rev() {
                g.edi = get(set + TARGETS[n] + 4 * k);
                g.esi = turned + 12 * k;
                for c in 0..3 {
                    let v = x87::int_sum(
                        get(g.esi + 4 * c) as i32,
                        get(set + OFFSET[n] + 4 * c) as i32,
                    );
                    put(g.edi + 4 * c, v as u32);
                }
            }
        }
    }
    Ok(g)
}

/// What `turn_shape_points` writes besides the targets.
pub fn turn_regions() -> Vec<Region> {
    let a = |r: usize| game_addr(r) as u32;
    vec![
        (a(TURN_X_RVA), 0x24),
        (a(TURN_RVA[0]), 0x54),
        at(SCRATCH_RVA),
        (a(SET_RVA) + MATRIX[0], 0x54),
        (a(TURNED_RVA[0]), 0x600),
    ]
}

/// The targets `turn_shape_points` writes (12 bytes each).
pub unsafe fn target_regions() -> Vec<Region> {
    let set = game_addr(SET_RVA) as u32;
    let mut r = Vec::new();
    for n in 0..2 {
        let count = unsafe { (get(set + COUNT[n]) as i32).clamp(0, MOST_POINTS as i32) as u32 };
        for k in 0..count {
            r.push((unsafe { get(set + TARGETS[n] + 4 * k) }, 12));
        }
    }
    r
}

/// The set's fields the tests fill, by list: count, targets, points, offset.
pub fn set_inputs() -> Vec<Region> {
    let set = game_addr(SET_RVA) as u32;
    vec![(set, 0xAAC)]
}

/// Fills list `n` of the set with `count` points by lot whose targets are in `targets`
/// (12 bytes each).
pub unsafe fn fill_set(seed: u32, counts: [u32; 2], targets: u32) {
    let set = game_addr(SET_RVA) as u32;
    let mut s = seed;
    let mut next = move || {
        s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
        s >> 8
    };
    unsafe {
        for n in 0..2 {
            put(set + COUNT[n], counts[n]);
            for k in 0..MOST_POINTS {
                put(
                    set + TARGETS[n] + 4 * k,
                    targets + 12 * (MOST_POINTS * n as u32 + k),
                );
                for c in 0..3 {
                    put(
                        set + POINTS[n] + 12 * k + 4 * c,
                        (next() % 0x4_0000).wrapping_sub(0x2_0000),
                    );
                }
            }
            for c in 0..3 {
                put(
                    set + OFFSET[n] + 4 * c,
                    (next() % 0x10_0000).wrapping_sub(0x8_0000),
                );
            }
        }
    }
}

/// The two shapes' steps and ends for `turn_all_shapes`: the step (each shape's +0), a, b,
/// and the switch that picks the second result.
const ALL_STEPS: [(usize, usize, usize, usize); 2] = [
    (rva(0x82FC56), rva(0x89C974), rva(0x884FC4), rva(0x844604)),
    (rva(0x82FC7A), rva(0x89C97C), rva(0x884FC8), rva(0x844608)),
];

/// `marks::turn_all_shapes` (0x4BD122): for each of the settings' objects (the last
/// first) the two shapes' eased steps (`smooth_steps` from `dword_89C974` / `_7C` to
/// `dword_884FC4` / `_C8` by each shape's +0), the second result of each unless its switch
/// (`dword_844604` / `_08`) is 0, and `turn_shape_points` with them. The object's offset
/// (times 0xAB8) is worked out and not used: the one set is turned every time (section
/// 16). Leaves esi that offset (the settings with none), the rest as the last turn.
/// Refuses what `turn_shape_points` would, before writing.
pub unsafe fn turn_all_shapes(entry: &Regs) -> Result<Regs, Refused> {
    use crate::marks::turns::{smooth_pair, smooth_steps};
    let mut g = *entry;
    unsafe {
        let settings: u32 = game_read(crate::marks::lines::SETTINGS_RVA);
        g.esi = settings;
        let count = get(settings + 0x26B8) as i32;
        let pick = |n: usize, g: &Regs| -> u32 {
            if game_read::<u32>(ALL_STEPS[n].3) == 0 {
                g.ecx
            } else {
                g.ebx
            }
        };
        if count > 0 {
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in ALL_STEPS.iter().enumerate() {
                let (ecx, ebx) = smooth_pair(game_read(step), game_read(a), game_read(b));
                turns[n] = pick(n, &Regs { ecx, ebx, ..g });
            }
            turn_shape_checks(turns[0], turns[1])?;
        }
        for k in (0..count.max(0) as u32).rev() {
            g.esi = k * 0xAB8;
            let mut turns = [0u32; 2];
            for (n, &(step, a, b, _)) in ALL_STEPS.iter().enumerate() {
                g.eax = game_read(step);
                g.ebx = game_read(a);
                g.ecx = game_read(b);
                g = smooth_steps(&g);
                turns[n] = pick(n, &g);
            }
            g.ebx = turns[0];
            g.ecx = turns[1];
            g.eax = k;
            let esi = g.esi;
            g = turn_shape_points(&g)?;
            g.esi = esi;
        }
    }
    Ok(g)
}

/// The working block a shape set is made in (`g_marksFarthestA` .. 0x885A6C, a
/// `MarkShapes` image) and the sets it is copied to.
pub const BLOCK_RVA: usize = rva(0x884FB4);
pub const SET_LEN: u32 = 0xAB8;
pub const SETS: u32 = 0x22;
pub const SET_COUNT_RVA: usize = rva(0x884FB0); // g_marksShapeSets: how many made
const KINDS_RVA: [usize; 2] = [rva(0x89C724), rva(0x89C728)]; // the two face kinds
const STEPS_IN_RVA: usize = rva(0x764F60); // dword_764F60 .. flt_764F78: the set's inputs
const BLOCK_STEPS_RVA: usize = rva(0x884FC4); // dword_884FC4 .. dword_884FE0
const STEP_SCALES_RVA: usize = rva(0x89C984); // flt_89C984 .. flt_89C990
const FIRST_POINT_RVA: usize = rva(0x884E5C); // dword_884E5C
const LOW_POINT: [u32; 2] = [0xA40, 0xA4C];

/// What `make_shape_set` refuses for object `object`: a set count past 0x22 (or below 0),
/// an object past 0xFF or its faces past `g_faces`, a tilt `fsincos` cannot take.
unsafe fn shape_set_checks(object: u32) -> Result<(), Refused> {
    use crate::scene::objects::{FACE_COUNTS_RVA, FACES, FIRST_FACE_RVA};
    unsafe {
        let made = game_read::<u32>(SET_COUNT_RVA);
        if made > SETS {
            return Err(BadInput("more than 0x22 shape sets").into());
        }
        if made == SETS {
            return Ok(());
        }
        if object >= 0x100 {
            return Err(BadInput("an object past 0xFF").into());
        }
        let first: u32 = game_read(FIRST_FACE_RVA + 4 * object as usize);
        let faces: u32 = game_read(FACE_COUNTS_RVA + 4 * object as usize);
        if (faces as i32) < 0 || first.checked_add(faces).is_none_or(|end| end > FACES) {
            return Err(BadInput("an object's faces past g_faces").into());
        }
        let steps: [u32; 2] = game_read(STEPS_IN_RVA);
        for t in steps {
            let mut a = extended(t);
            a[9] ^= 0x80;
            turnable(&a)?;
        }
    }
    Ok(())
}

/// `marks::make_shape_set` (0x4BD1A1): unless 0x22 are made, in the working block: the
/// inputs (0x764F60 ..) copied, four scaled; points lists A and B emptied and filled from
/// object ebp's faces of the two kinds (+7: `dword_89C724`, `_28`; `collect_points_a` /
/// `_b`); their ids made pointers into `g_objectPoints`, the points copied to
/// `g_marksTurnedA` / `B`, each list's lowest point (y, the first found lowest) taken from
/// all of it (kept at +0xA40 / +0xA4C), all 0x40 turned about X by minus the tilts
/// (`turn_matrix3_x`, `transform_int_points`); `find_farthest_pairs`; the block copied to
/// set n (`memory::move_bytes`), n counted. Leaves edi the set, the rest as
/// `find_farthest_pairs` leaves them. Refuses a set count past 0x22, an object past 0xFF
/// or faces past `g_faces`, a tilt `fsincos` cannot take, before writing.
pub unsafe fn make_shape_set(entry: &Regs) -> Result<Regs, Refused> {
    use crate::marks::masks::collect_points;
    use crate::scene::objects::{
        FACE_COUNTS_RVA, FACES_RVA, FIRST_FACE_RVA, FIRST_POINT_RVA as OBJECT_FIRST_POINT_RVA,
        POINTS_RVA,
    };
    let mut g = *entry;
    unsafe {
        shape_set_checks(g.ebp)?;
        let made: u32 = game_read(SET_COUNT_RVA);
        if made == SETS {
            return Ok(g);
        }
        let block = game_addr(BLOCK_RVA) as u32;
        let inputs: [u32; 6] = [
            game_read(STEPS_IN_RVA),
            game_read(STEPS_IN_RVA + 4),
            game_read(STEPS_IN_RVA + 0xC),
            game_read(STEPS_IN_RVA + 0x10),
            game_read(STEPS_IN_RVA + 0x14),
            game_read(STEPS_IN_RVA + 0x18),
        ];
        let scales: [u32; 4] = game_read(STEP_SCALES_RVA);
        // 0x884FC4 .. : a, b (0x764F6C, _74), c, d (_70, _78) scaled, the tilts, two zeros.
        let steps = [
            x87::mul(inputs[2], scales[0]),
            x87::mul(inputs[4], scales[1]),
            x87::mul(inputs[3], scales[2]),
            x87::mul(inputs[5], scales[3]),
            0,
            0,
            inputs[0],
            inputs[1],
        ];
        game_write(BLOCK_STEPS_RVA, steps);
        for list in [&POINTS_A, &POINTS_B] {
            game_write(list.count, 0u32);
        }
        let object = g.ebp as usize;
        g.esi = game_read(FIRST_FACE_RVA + 4 * object);
        game_write(
            FIRST_POINT_RVA,
            game_read::<u32>(OBJECT_FIRST_POINT_RVA + 4 * object),
        );
        let faces: u32 = game_read(FACE_COUNTS_RVA + 4 * object);
        let kinds: [u32; 2] = KINDS_RVA.map(|r| game_read(r));
        for _ in 0..faces {
            g.edi = game_read(FACES_RVA + 4 * g.esi as usize);
            let kind = *((g.edi + 7) as *const u8);
            if kind == kinds[0] as u8 {
                g = collect_points(&g, &POINTS_A)?;
            } else if kind == kinds[1] as u8 {
                g = collect_points(&g, &POINTS_B)?;
            }
            g.esi += 1;
        }
        let lists = [&POINTS_A, &POINTS_B];
        let points: u32 = game_read(POINTS_RVA);
        for list in lists {
            let ids = game_addr(list.ids) as u32;
            for k in (0..game_read::<u32>(list.count)).rev() {
                g.ebx = k;
                g.ecx = get(ids + 4 * k).wrapping_mul(12).wrapping_add(points);
                put(ids + 4 * k, g.ecx);
            }
        }
        for (list, n) in lists.into_iter().zip(0..) {
            let ids = game_addr(list.ids) as u32;
            let turned = game_addr(TURNED_RVA[n]) as u32;
            for k in (0..game_read::<u32>(list.count)).rev() {
                g.esi = get(ids + 4 * k);
                g.edi = k * 12;
                for c in 0..3 {
                    put(turned + g.edi + 4 * c, get(g.esi + 4 * c));
                }
            }
        }
        for (list, n) in lists.into_iter().zip(0..) {
            let turned = game_addr(TURNED_RVA[n]) as u32;
            g.eax = 0x7FFF_FFFF;
            g.ebx = 0;
            for k in (0..game_read::<u32>(list.count)).rev() {
                g.esi = k * 12;
                if (get(turned + g.esi + 4) as i32) < g.eax as i32 {
                    g.eax = get(turned + g.esi + 4);
                    g.ebx = k;
                    for c in 0..3 {
                        put(block + LOW_POINT[n] + 4 * c, get(turned + g.esi + 4 * c));
                    }
                }
            }
        }
        for (list, n) in lists.into_iter().zip(0..) {
            let turned = game_addr(TURNED_RVA[n]) as u32;
            for k in (0..game_read::<u32>(list.count)).rev() {
                g.esi = k * 12;
                let p = turned + g.esi;
                let d = |c: u32| get(p + 4 * c).wrapping_sub(get(block + LOW_POINT[n] + 4 * c));
                [g.eax, g.ebx, g.ecx] = [d(0), d(1), d(2)];
                put(p, g.eax);
                put(p + 4, g.ebx);
                put(p + 8, g.ecx);
            }
        }
        let turn_x = game_addr(TURN_X_RVA) as u32;
        for n in 0..2 {
            let mut angle = extended(steps[6 + n]);
            angle[9] ^= 0x80;
            turn_matrix3(Axis::X, turn_x, angle)?;
            let turned = game_addr(TURNED_RVA[n]) as u32;
            g = transform_int_points(&Regs {
                ebp: turn_x,
                esi: turned,
                edi: turned,
                ecx: MOST_POINTS,
                ..g
            })?;
        }
        g = find_farthest_pairs(&g)?;
        g.edi = game_addr(SET_RVA) as u32 + made * SET_LEN;
        std::ptr::copy(block as *const u8, g.edi as *mut u8, SET_LEN as usize);
        game_write(SET_COUNT_RVA, made + 1);
    }
    Ok(g)
}

/// What `make_shape_set` writes besides the set: 0x884E5C .. 0x885A6C.
pub fn shape_set_regions() -> Vec<Region> {
    let a = game_addr(FIRST_POINT_RVA) as u32;
    vec![(a, (game_addr(SET_RVA) as u32 - a) as usize)]
}

/// The kind's two face kinds and the set's ends (`dword_89C72C`, `_7B4`, `_8C4`, `_83C` by
/// `g_marksKind`; `flt_89C96C`, `_70`).
const BY_KIND_RVA: [usize; 4] = [rva(0x89C72C), rva(0x89C7B4), rva(0x89C8C4), rva(0x89C83C)];
const SET_ENDS_RVA: [usize; 2] = [rva(0x89C96C), rva(0x89C970)];

/// `marks::make_shape_sets` (0x4BD019): no sets made; the two face kinds of `g_marksKind`
/// into `dword_89C724`, `_28`; for each of the settings' objects (the last first) the set's
/// inputs (the tilt twice into 0x764F60, _64; the kind's two values and two floats into
/// 0x764F6C .. _78) and `make_shape_set`. Leaves esi the settings, the rest as the last set
/// (eax the kind with none). Refuses a kind of 0x22 or more, more than 19 objects, what
/// `make_shape_set` refuses for any of them, before writing.
pub unsafe fn make_shape_sets(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let kind: u32 = game_read(crate::marks::kind_values::KIND_RVA);
        if kind >= SETS {
            return Err(BadInput("a marks' kind of 0x22 or more").into());
        }
        let settings: u32 = game_read(crate::marks::lines::SETTINGS_RVA);
        let count = get(settings + 0x26B8) as i32;
        if count > 19 {
            return Err(BadInput("more than 19 shape objects").into());
        }
        let objects: Vec<u32> = (0..count.max(0) as u32)
            .rev()
            .map(|k| get(settings + 0x26BC + 4 * k))
            .collect();
        let by_kind = BY_KIND_RVA.map(|r| game_read::<u32>(r + 4 * kind as usize));
        let tilt: u32 = game_read(TILT_RVA);
        let tilts = [tilt, tilt];
        let ends = [
            by_kind[2],
            x87::loaded(game_read(SET_ENDS_RVA[0])),
            by_kind[3],
            x87::loaded(game_read(SET_ENDS_RVA[1])),
        ];
        // Checked as the sets will see them: none made, these tilts.
        let saved: ([u32; 2], u32) = (game_read(STEPS_IN_RVA), game_read(SET_COUNT_RVA));
        game_write(STEPS_IN_RVA, tilts);
        game_write(SET_COUNT_RVA, 0u32);
        let checked = objects.iter().try_for_each(|&o| shape_set_checks(o));
        game_write(STEPS_IN_RVA, saved.0);
        game_write(SET_COUNT_RVA, saved.1);
        checked?;
        game_write(SET_COUNT_RVA, 0u32);
        game_write(KINDS_RVA[0], by_kind[0]);
        game_write(KINDS_RVA[1], by_kind[1]);
        g.eax = kind;
        g.esi = settings;
        for o in objects {
            g.ebp = o;
            game_write(STEPS_IN_RVA, tilts);
            game_write(STEPS_IN_RVA + 0xC, ends);
            g = make_shape_set(&g)?;
            g.esi = settings;
        }
    }
    Ok(g)
}

/// What `make_shape_sets` writes besides the sets.
pub fn shape_sets_regions() -> Vec<Region> {
    let mut r = shape_set_regions();
    r.extend([
        at(KINDS_RVA[0]),
        at(KINDS_RVA[1]),
        (game_addr(STEPS_IN_RVA) as u32, 0x1C),
    ]);
    r
}

// ---- for the self-tests ----

/// Everything a shape-set test writes, for capture: the block, the first three sets, the
/// count, the kinds, the inputs, `g_marksKind`.
pub fn shape_test_regions() -> Vec<Region> {
    let mut r = shape_sets_regions();
    r.extend([
        (game_addr(SET_RVA) as u32, 3 * SET_LEN as usize),
        at(SET_COUNT_RVA),
        at(crate::marks::kind_values::KIND_RVA),
    ]);
    r
}

/// The test scene's faces given the two face kinds (and another) in turn, the kinds set.
pub unsafe fn shape_test_faces(scene: &crate::marks::lines::TestScene, seed: u32) {
    unsafe {
        let kinds: [u32; 2] = [0x1E, 0x1D];
        game_write(KINDS_RVA[0], kinds[0]);
        game_write(KINDS_RVA[1], kinds[1]);
        for n in 0..2 * crate::marks::lines::TEST_FACES {
            let k = [kinds[0], kinds[1], 3][((n + seed) % 3) as usize];
            *((scene.face(n) + 7) as *mut u8) = k as u8;
        }
    }
}

/// Tilts and ends by lot into 0x764F60 ...
pub unsafe fn shape_test_inputs(seed: u32) {
    let mut v = [0u32; 7];
    crate::scene::matrix3::test_floats(seed + 80, &mut v);
    unsafe { game_write(STEPS_IN_RVA, v) };
}
