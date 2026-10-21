//! `scene::project_point` (0x44D004): a point's place in the view and on the screen,
//! worked out once and kept.
//!
//! In:  eax = the point's number
//! Out: edx = its record; a point of the scene: eax unchanged, ecx = the point's address
//!      (or the number of points + 0x40 if the record was already made); a point of an
//!      object (a negative number): eax = the record's offset; everything else preserved
//!
//! The records are 0x20 bytes each at `dword_5C6FB8`: +0 the point's number, +0xC the
//! x on the screen, +0x10, +0x14, +0x18 the point in the view, +0x1C the y on the
//! screen. A point n of the scene has record n & 0xFFF; a point -n of an object has
//! record 0x1000 + (n & 0xFFF), which is made elsewhere and only found here.
//! A record whose number is not the point's is made anew: the point's three whole
//! numbers (12 bytes each at `dword_5CCCC0`) through the matrix `flt_5EA2CC`
//! (`sub_44D110`: three rows of x, y, z factors and a fourth to add; x87, inlined here
//! with its own instructions), the screen's two set to 0, and, unless the depth is
//! under 4096: x = x' * (`flt_624088` / depth) + `flt_62409C`, y = `flt_6240A0` - y' *
//! (the same). With `dword_60932C` set the depth divided by is `dword_5CCD80` + 1
//! instead (and the real one goes to `dword_62400C`). With `dword_6365C4` at 1 the
//! record's depth is then lowered by 262144.
//! Fatal errors 0x80600902 and 0x80600903 for a number past the object's or the
//! scene's points.
//! One caller, `scene::project_polygon`.
//!
//! Converted 2026-10-21: the hook's Rust version is `project_point_plain`, through
//! `rc2k_core::scene::projection` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code); `record_offset` is the core's for both. Shadow mode and the
//! self-test compare the record's floats and `dword_62400C` within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{
    EAX, ECX, EDX, RawHook, Regs, SelfTest, game_addr, game_ptr, game_read, game_write, hook_asm,
    rva,
};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::{dword, put};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::project_point", rva(0x44D004), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const RECORDS_RVA: usize = rva(0x5C6FB8); // dword_5C6FB8
pub const RECORD_LEN: usize = 0x20;
pub const POINTS_RVA: usize = rva(0x5CCCC0); // dword_5CCCC0
pub const POINT_COUNT_RVA: usize = rva(0x5CCD20); // dword_5CCD20
pub const OBJECT_POINTS_RVA: usize = rva(0x5CD680); // dword_5CD680
pub const MATRIX_RVA: usize = rva(0x5EA2CC); // flt_5EA2CC
pub const FLAT_RVA: usize = rva(0x60932C); // dword_60932C
pub const FLAT_DEPTH_RVA: usize = rva(0x5CCD80); // dword_5CCD80
pub const SCALE_RVA: usize = rva(0x624088); // flt_624088
pub const CENTRE_X_RVA: usize = rva(0x62409C); // flt_62409C
pub const CENTRE_Y_RVA: usize = rva(0x6240A0); // flt_6240A0
pub const LOWERED_RVA: usize = rva(0x6365C4); // dword_6365C4
/// `flt_5C5E18`: a point nearer than this is not put on the screen.
pub const NEAR: f32 = 4096.0;
/// `flt_5C5E1C`.
const LOWERED_BY: f32 = 262144.0;

const TOO_HIGH_OBJECT: FatalError = FatalError {
    code: 0x8060_0902,
    message: "OBJPNTROT : point eax is too high",
};
const TOO_HIGH: FatalError = FatalError {
    code: 0x8060_0903,
    message: "CRSPNTROT : point eax is too high",
};

// ---- Rust version ----

macro_rules! x87 {
    ($($line:literal),+; $($name:ident = $value:expr),+) => {
        unsafe {
            core::arch::asm!(
                $($line),+,
                $($name = in(reg) $value),+,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            )
        }
    };
}

/// `sub_44D110` as `project_point` uses it: the point's three whole numbers through the
/// matrix, into the record's +0x10, +0x14, +0x18.
pub(crate) unsafe fn rotate(point: *const i32, matrix: *const u32, record: *mut u32) {
    x87!(
        "fild dword ptr [{p} + 8]", "fild dword ptr [{p} + 4]", "fild dword ptr [{p}]",
        "fld dword ptr [{m}]", "fmul st(0), st(1)",
        "fld dword ptr [{m} + 0x10]", "fmul st(0), st(3)",
        "fld dword ptr [{m} + 0x20]", "fmul st(0), st(5)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x30]", "fxch st(3)",
        "fld dword ptr [{m} + 4]", "fmul st(0), st(2)",
        "fld dword ptr [{m} + 0x14]", "fmul st(0), st(4)",
        "fld dword ptr [{m} + 0x24]", "fmul st(0), st(3)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x34]", "fxch st(3)",
        "fld dword ptr [{m} + 8]", "fmulp st(3), st(0)",
        "fld dword ptr [{m} + 0x18]", "fmulp st(1), st(0)",
        "fld dword ptr [{m} + 0x28]", "fmulp st(2), st(0)",
        "faddp st(1), st(0)", "faddp st(1), st(0)",
        "fadd dword ptr [{m} + 0x38]", "fxch st(2)",
        "fstp dword ptr [{o} + 0x10]", "fstp dword ptr [{o} + 0x14]",
        "fstp dword ptr [{o} + 0x18]";
        p = point, m = matrix, o = record
    );
}

/// The record's place on the screen from its place in the view. `v`: the scale, the
/// centre's x and y, and the whole number the flat depth is made from.
/// (`.byte 0xDE, 0xF1` is the original's `fdivrp st(1), st`.)
pub(crate) unsafe fn to_screen(record: *mut u32, v: &[u32; 4], flat: bool, checked: *mut u32) {
    if flat {
        x87!(
            "fld dword ptr [{o} + 0x10]", "fld dword ptr [{o} + 0x14]",
            "fld dword ptr [{o} + 0x18]",
            "fstp dword ptr [{w}]", "fild dword ptr [{v} + 12]", "fld1", "faddp st(1), st(0)",
            "fld dword ptr [{v}]", ".byte 0xDE, 0xF1",
            "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
            "fadd dword ptr [{v} + 8]", "fxch st(1)", "fadd dword ptr [{v} + 4]",
            "fstp dword ptr [{o} + 0xC]", "fstp dword ptr [{o} + 0x1C]";
            o = record, v = v.as_ptr(), w = checked
        );
    } else {
        x87!(
            "fld dword ptr [{o} + 0x10]", "fld dword ptr [{o} + 0x14]",
            "fld dword ptr [{o} + 0x18]",
            "fld dword ptr [{v}]", ".byte 0xDE, 0xF1",
            "fmul st(1), st(0)", "fmulp st(2), st(0)", "fchs",
            "fadd dword ptr [{v} + 8]", "fxch st(1)", "fadd dword ptr [{v} + 4]",
            "fstp dword ptr [{o} + 0xC]", "fstp dword ptr [{o} + 0x1C]";
            o = record, v = v.as_ptr()
        );
    }
}

/// The offset of a point's record, or the fatal error for a number too high.
// Converted: `rc2k_core::scene::projection::record_offset`
pub unsafe fn record_offset(point: u32) -> Result<u32, FatalError> {
    use rc2k_core::scene::projection::TooHigh;
    unsafe {
        rc2k_core::scene::projection::record_offset(
            point,
            game_read(OBJECT_POINTS_RVA),
            game_read(POINT_COUNT_RVA),
        )
        .map_err(|e| match e {
            TooHigh::Object => TOO_HIGH_OBJECT,
            TooHigh::Scene => TOO_HIGH,
        })
    }
}

/// What the function leaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projected {
    /// The record's address.
    pub record: u32,
    pub eax: u32,
    /// None: unchanged.
    pub ecx: Option<u32>,
}

/// A scene point's record made anew (`sub_44CF4E`, and the same code in
/// `project_point`): the point through the view's matrix and onto the screen.
/// Returns the point's address.
pub(crate) unsafe fn make_scene_record(point: u32, record: u32) -> Result<u32, BadInput> {
    unsafe {
        let at = record as usize;
        let points: u32 = game_read(POINTS_RVA);
        if points == 0 {
            return Err(BadInput("no points"));
        }
        let from = points.wrapping_add(point.wrapping_mul(12));
        put(at, point);
        rotate(from as *const i32, game_ptr(MATRIX_RVA), record as *mut u32);
        put(at + 0xC, 0);
        put(at + 0x1C, 0);
        if f32::from_bits(dword(at + 0x18)) >= NEAR {
            let v = [
                game_read(SCALE_RVA),
                game_read(CENTRE_X_RVA),
                game_read(CENTRE_Y_RVA),
                game_read(FLAT_DEPTH_RVA),
            ];
            let flat = game_read::<u32>(FLAT_RVA) != 0;
            to_screen(record as *mut u32, &v, flat, game_ptr(CHECKED_RVA));
        }
        if game_read::<u32>(LOWERED_RVA) == 1 {
            put(at + 0x18, x87::sub(dword(at + 0x18), LOWERED_BY.to_bits()));
            game_write(CHECKED_RVA, LOWERED_BY.to_bits());
        }
        Ok(from)
    }
}

pub unsafe fn project_point(point: u32) -> Result<Projected, Refused> {
    unsafe {
        let offset = record_offset(point)?;
        let records: u32 = game_read(RECORDS_RVA);
        if records == 0 {
            return Err(BadInput("no records for the points").into());
        }
        let record = records.wrapping_add(offset);
        if (point as i32) < 0 {
            return Ok(Projected {
                record,
                eax: offset,
                ecx: None,
            });
        }
        let at = record as usize;
        if dword(at) == point {
            return Ok(Projected {
                record,
                eax: point,
                ecx: Some(game_read::<u32>(POINT_COUNT_RVA).wrapping_add(0x40)),
            });
        }
        let from = make_scene_record(point, record)?;
        Ok(Projected {
            record,
            eax: point,
            ecx: Some(from),
        })
    }
}

/// The view's screen from the game's variables.
pub fn screen() -> rc2k_core::scene::projection::Screen {
    unsafe {
        let f = |r: usize| f32::from_bits(game_read(r));
        rc2k_core::scene::projection::Screen {
            scale: f(SCALE_RVA),
            centre: [f(CENTRE_X_RVA), f(CENTRE_Y_RVA)],
            flat: (game_read::<u32>(FLAT_RVA) != 0).then(|| game_read(FLAT_DEPTH_RVA)),
        }
    }
}

/// The converted `make_scene_record` (plain floats).
// Converted: `rc2k_core::scene::projection::make_record`
pub(crate) unsafe fn make_scene_record_plain(point: u32, record: u32) -> Result<u32, BadInput> {
    unsafe {
        let at = record as usize;
        let points: u32 = game_read(POINTS_RVA);
        if points == 0 {
            return Err(BadInput("no points"));
        }
        let from = points.wrapping_add(point.wrapping_mul(12));
        let place = (from as *const [i32; 3]).read_unaligned();
        let matrix: [u32; 16] = game_read(MATRIX_RVA);
        let made = rc2k_core::scene::projection::make_record(
            place,
            &matrix.map(f32::from_bits),
            screen,
            game_read::<u32>(LOWERED_RVA) == 1,
        );
        put(at, point);
        for (k, v) in made.view.into_iter().enumerate() {
            put(at + 0x10 + 4 * k, v.to_bits());
        }
        put(at + 0xC, made.screen[0].to_bits());
        put(at + 0x1C, made.screen[1].to_bits());
        if let Some(checked) = made.checked {
            game_write(CHECKED_RVA, checked.to_bits());
        }
        Ok(from)
    }
}

/// The converted `project_point` (plain floats): the hook's, and `project_polygon_plain`'s.
/// The x87 version above stays for the Rust callers in unconverted code.
// Converted: `rc2k_core::scene::projection`
pub unsafe fn project_point_plain(point: u32) -> Result<Projected, Refused> {
    unsafe {
        let offset = record_offset(point)?;
        let records: u32 = game_read(RECORDS_RVA);
        if records == 0 {
            return Err(BadInput("no records for the points").into());
        }
        let record = records.wrapping_add(offset);
        if (point as i32) < 0 {
            return Ok(Projected {
                record,
                eax: offset,
                ecx: None,
            });
        }
        if dword(record as usize) == point {
            return Ok(Projected {
                record,
                eax: point,
                ecx: Some(game_read::<u32>(POINT_COUNT_RVA).wrapping_add(0x40)),
            });
        }
        let from = make_scene_record_plain(point, record)?;
        Ok(Projected {
            record,
            eax: point,
            ecx: Some(from),
        })
    }
}

/// The float places of `regions(point)`: the record's five floats and `dword_62400C`.
pub fn float_places(point: u32) -> Vec<(Region, crate::service::floats::Tolerance)> {
    use crate::service::floats::FLOAT;
    let mut places = vec![(at(CHECKED_RVA), FLOAT)];
    let records: u32 = unsafe { game_read(RECORDS_RVA) };
    if let (Ok(offset), true) = (unsafe { record_offset(point) }, records != 0) {
        places.push(((records.wrapping_add(offset + 0xC), 0x14), FLOAT));
    }
    places
}

// ---- hook ----

/// What a call for `point` may write.
pub fn regions(point: u32) -> Vec<Region> {
    let mut regions = vec![at(CHECKED_RVA)];
    let records: u32 = unsafe { game_read(RECORDS_RVA) };
    if let (Ok(offset), true) = (unsafe { record_offset(point) }, records != 0) {
        regions.push((records.wrapping_add(offset), RECORD_LEN));
    }
    regions
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let done = unsafe { project_point_plain(r.eax) }?;
    let left = Left::nothing().with(EAX, done.eax).with(EDX, done.record);
    Ok(match done.ecx {
        Some(ecx) => left.with(ECX, ecx),
        None => left,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    use crate::service::floats::{Floats, dispatch_state_floats};
    let places = float_places(r.eax);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(r.eax), &floats, &rust);
}

// ---- self-test: points, records and a view of the test's ----

/// The variables a test of the projection sets, to be put back afterwards.
pub fn inputs() -> Vec<Region> {
    let mut inputs: Vec<Region> = [
        RECORDS_RVA,
        POINTS_RVA,
        POINT_COUNT_RVA,
        OBJECT_POINTS_RVA,
        FLAT_RVA,
        FLAT_DEPTH_RVA,
        SCALE_RVA,
        CENTRE_X_RVA,
        CENTRE_Y_RVA,
        LOWERED_RVA,
        CHECKED_RVA,
    ]
    .map(at)
    .to_vec();
    inputs.push((game_addr(MATRIX_RVA) as u32, 0x40));
    inputs
}

/// Points, records and views for a test: the game's variables are pointed at them by
/// `set`, and `inputs()` is what to put back.
pub struct World {
    pub records: Vec<[u32; 8]>,
    pub points: Vec<[i32; 3]>,
}

pub const TEST_POINTS: u32 = 8;
pub const TEST_OBJECT_POINTS: u32 = 6;

impl World {
    pub fn new() -> Self {
        let mut points = vec![[0i32; 3]; TEST_POINTS as usize + 0x40];
        for (n, point) in points.iter_mut().enumerate() {
            let n = n as i32;
            *point = [n * 977 - 3000, 2500 - n * 613, n * 1201 - 1500];
        }
        // One far off, one of the largest numbers.
        points[5] = [123_456_789, -98_765_432, 7_654_321];
        points[6] = [i32::MAX, i32::MIN, 1];
        Self {
            records: vec![[0; 8]; 0x2000],
            points,
        }
    }

    /// The records as nobody made them, but for the objects' (which the function only
    /// finds) and one of the scene's that is there already; the variables for view
    /// `view` (0 .. 3), flat or not, lowered or not.
    pub unsafe fn set(&mut self, view: usize, flat: bool, lowered: bool) {
        let f = |value: f32| value.to_bits();
        self.records.fill([0xAAAA_AAAA; 8]);
        for n in 0..TEST_OBJECT_POINTS as usize {
            let depth = [100.0f32, 5000.0, 4096.0, 70000.0, f32::NAN, 12000.5][n];
            self.records[0x1000 + n] = [0, 0, 0, f(10.0 + n as f32), 0, 0, f(depth), f(20.0)];
        }
        self.records[2] = [2, 7, 7, f(1.5), f(2.5), f(3.5), f(9000.0), f(4.5)];
        let matrices: [[f32; 16]; 4] = [
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 9000.0, 0.0,
            ],
            [
                0.36, 0.48, -0.8, 0.0, -0.8, 0.6, 0.0, 0.0, 0.48, 0.64, 0.6, 0.0, 12.25, -7.5,
                20000.3, 0.0,
            ],
            [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 4096.0, 0.0,
            ],
            [
                0.1234567, 3.3333333, -0.7171068, 0.0, 2.7282818, -0.5, 0.3333333, 0.0, -1.4242135,
                0.0009765, 1.7320508, 0.0, -333.33334, 0.1, 6000.7, 0.0,
            ],
        ];
        unsafe {
            game_write(MATRIX_RVA, matrices[view % 4].map(f));
            game_write(RECORDS_RVA, self.records.as_mut_ptr() as u32);
            game_write(POINTS_RVA, self.points.as_ptr() as u32);
            game_write(POINT_COUNT_RVA, TEST_POINTS);
            game_write(OBJECT_POINTS_RVA, TEST_OBJECT_POINTS);
            game_write(FLAT_RVA, flat as u32);
            game_write(FLAT_DEPTH_RVA, 7999u32);
            game_write(SCALE_RVA, f(640.0 + view as f32 * 0.37));
            game_write(CENTRE_X_RVA, f(400.0));
            game_write(CENTRE_Y_RVA, f(300.5));
            game_write(LOWERED_RVA, lowered as u32);
            game_write(CHECKED_RVA, 0xAAAA_AAAAu32);
        }
    }

    /// Everything a projection may have written.
    pub fn state(&self) -> (Vec<[u32; 8]>, u32) {
        (self.records.clone(), unsafe { game_read(CHECKED_RVA) })
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Two states of the test's world the same, the records' floats (+0xC .. +0x1C) and the
/// checked value within `tolerance`.
pub fn same_world(
    theirs: &(Vec<[u32; 8]>, u32),
    mine: &(Vec<[u32; 8]>, u32),
    tolerance: crate::service::floats::Tolerance,
) -> bool {
    use crate::service::floats::close;
    theirs.0.len() == mine.0.len()
        && theirs
            .0
            .iter()
            .zip(&mine.0)
            .all(|(a, b)| a[..3] == b[..3] && (3..8).all(|k| close(a[k], b[k], tolerance)))
        && close(theirs.1, mine.1, tolerance)
}

fn self_test() -> (usize, usize) {
    use crate::service::floats::{FLOAT, Floats, at_test_precisions};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = inputs();
    let saved = capture(&inputs);
    let mut world = World::new();
    let last = TEST_POINTS + 0x3F;
    let places = [(at(CHECKED_RVA), FLOAT)];
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        let tolerance = held.memory[0].1;
        for view in 0..4 {
            for (flat, lowered) in [(false, false), (true, false), (false, true), (true, true)] {
                for point in [
                    0u32,
                    1,
                    2,
                    3,
                    5,
                    6,
                    7,
                    last,
                    -1i32 as u32,
                    -3i32 as u32,
                    -5i32 as u32,
                ] {
                    let before = Regs {
                        eax: point,
                        ..Regs::SENTINEL
                    };
                    unsafe { world.set(view, flat, lowered) };
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    let theirs = world.state();
                    unsafe { world.set(view, flat, lowered) };
                    let mine = rust(&before);
                    let state = world.state();
                    let regs_ok = mine.as_ref().is_ok_and(|left| left.matches(&before, &ro));
                    t.check(regs_ok && same_world(&theirs, &state, tolerance), || {
                        let n = theirs.0.iter().zip(&state.0).position(|(a, b)| a != b);
                        format!(
                            "control word {word:#x}, point {point:#x}, view {view}, flat {flat}, lowered {lowered}: regs_ok={regs_ok} original eax {:#x} ecx {:#x} edx {:#x}, rust {mine:x?}; checked {:#x} {:#x}; first different record {n:?}: {:x?} {:x?}",
                            ro.eax,
                            ro.ecx,
                            ro.edx,
                            theirs.1,
                            state.1,
                            n.map(|n| theirs.0[n]),
                            n.map(|n| state.0[n]),
                        )
                    });
                }
            }
        }
    });
    restore(&inputs, &saved);
    t.finish()
}
