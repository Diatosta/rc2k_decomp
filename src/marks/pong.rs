//! The pong game over the race (a reading: cheat bit 0x8000 and `dword_641678`; run by
//! `marks::run_pong`): its variables, the compares its hooks share, and a test stand-in
//! for `marks::draw_pong_box`. The ball at `flt_768688` (x, y), its way `flt_768694`, its
//! size `flt_7686A0`; the bat at `flt_7686B8`, its size `flt_7686D0`, its place before the
//! move `dword_7686AC`; the hits `dword_7687D0`, the best `dword_7687CC`, the ball's steps
//! a frame `g_pongSteps` (named before the game was seen), the score's place
//! `dword_7687D4` / `dword_7687D8`, the digits' 4 x 5 patterns `unk_768704` (0x14 each).

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::service::harness::Region;
use std::sync::Mutex;

pub const BALL: [usize; 2] = [rva(0x768688), rva(0x76868C)];
pub const BALL_WAY: [usize; 2] = [rva(0x768694), rva(0x768698)];
pub const BALL_SIZE: [usize; 2] = [rva(0x7686A0), rva(0x7686A4)];
pub const SAVED_BAT: [usize; 2] = [rva(0x7686AC), rva(0x7686B0)];
pub const BAT: [usize; 2] = [rva(0x7686B8), rva(0x7686BC)];
pub const BAT_SIZE: [usize; 2] = [rva(0x7686D0), rva(0x7686D4)];
pub const STEPS_RVA: usize = rva(0x768700); // g_pongSteps
pub const BEST_RVA: usize = rva(0x7687CC); // dword_7687CC
pub const HITS_RVA: usize = rva(0x7687D0); // dword_7687D0
pub const DIGIT_AT: [usize; 2] = [rva(0x7687D4), rva(0x7687D8)];
pub const PATTERNS_RVA: usize = rva(0x768704); // unk_768704: 10 of 0x14
pub const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
pub const SHOWN_RVA: usize = rva(0x641678); // dword_641678

/// `fld a; fadd b; fcomp c; fstsw ax; and eax, 700h`.
pub fn sum_compare(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut s) = ([a, b, c], 0u16);
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fadd dword ptr [{v} + 4]",
            "fcomp dword ptr [{v} + 8]",
            "fnstsw word ptr [{s}]",
            v = in(reg) v.as_ptr(),
            s = in(reg) &raw mut s,
            out("st(0)") _,
            options(nostack),
        )
    };
    u32::from(s) & 0x700
}

/// `fld x; fld k; fsub w [; fsub m]; fcompp; fstsw ax; and eax, 700h` (`m` with `Some`).
pub fn less_compare(x: u32, k: u32, w: u32, m: Option<u32>) -> u32 {
    let (v, mut s) = ([x, k, w, m.unwrap_or(0)], 0u16);
    unsafe {
        if m.is_some() {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fld dword ptr [{v} + 4]",
                "fsub dword ptr [{v} + 8]",
                "fsub dword ptr [{v} + 12]",
                "fcompp",
                "fnstsw word ptr [{s}]",
                v = in(reg) v.as_ptr(),
                s = in(reg) &raw mut s,
                out("st(0)") _, out("st(1)") _,
                options(nostack),
            )
        } else {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fld dword ptr [{v} + 4]",
                "fsub dword ptr [{v} + 8]",
                "fcompp",
                "fnstsw word ptr [{s}]",
                v = in(reg) v.as_ptr(),
                s = in(reg) &raw mut s,
                out("st(0)") _, out("st(1)") _,
                options(nostack),
            )
        }
    };
    u32::from(s) & 0x700
}

fn get(r: usize) -> u32 {
    unsafe { game_read(r) }
}

const CENTRE_RVA: usize = rva(0x7691C6); // flt_7691C6 .. flt_7691D2
const NUDGE_RVA: usize = rva(0x7691D6); // flt_7691D6
const WAY_SCALE_RVA: usize = rva(0x7691DA); // flt_7691DA, flt_7691DE
const LENGTH_RVA: usize = rva(0x76516C); // flt_76516C
pub const WALLS_RVA: usize = rva(0x7691A6); // flt_7691A6 .. flt_7691C2
pub const START_RVA: usize = rva(0x769182); // flt_769182 .. flt_76919E
const BALL_CENTRE: [usize; 2] = [rva(0x7686DC), rva(0x7686E0)];
const BAT_CENTRE: [usize; 2] = [rva(0x7686E8), rva(0x7686EC)];
const DIFFERENCE: [usize; 2] = [rva(0x7686F4), rva(0x7686F8)];

fn pair(r: [usize; 2]) -> [f32; 2] {
    r.map(|r| f32::from_bits(get(r)))
}

unsafe fn put_pair(r: [usize; 2], v: [f32; 2]) {
    for (r, v) in r.into_iter().zip(v) {
        unsafe { game_write(r, v.to_bits()) };
    }
}

/// The court (`rc2k_core::marks::pong::Court`) from the game's variables.
pub fn court() -> rc2k_core::marks::pong::Court {
    rc2k_core::marks::pong::Court {
        ball: pair(BALL),
        way: pair(BALL_WAY),
        ball_size: pair(BALL_SIZE),
        saved_bat: SAVED_BAT.map(get),
        bat: pair(BAT),
        bat_size: pair(BAT_SIZE),
        ball_centre: pair(BALL_CENTRE),
        bat_centre: pair(BAT_CENTRE),
        difference: pair(DIFFERENCE),
        steps: get(STEPS_RVA),
        hits: get(HITS_RVA),
    }
}

/// The court back into the game's variables: what differs from `before` (the rest is left
/// as it is, its bits untouched).
pub unsafe fn put_court(c: &rc2k_core::marks::pong::Court, before: &rc2k_core::marks::pong::Court) {
    unsafe {
        for (r, v, b) in [
            (BALL, c.ball, before.ball),
            (BALL_WAY, c.way, before.way),
            (BALL_SIZE, c.ball_size, before.ball_size),
            (BAT, c.bat, before.bat),
            (BAT_SIZE, c.bat_size, before.bat_size),
            (BALL_CENTRE, c.ball_centre, before.ball_centre),
            (BAT_CENTRE, c.bat_centre, before.bat_centre),
            (DIFFERENCE, c.difference, before.difference),
        ] {
            if v.map(f32::to_bits) != b.map(f32::to_bits) {
                put_pair(r, v);
            }
        }
        game_write(SAVED_BAT[0], c.saved_bat[0]);
        game_write(SAVED_BAT[1], c.saved_bat[1]);
        game_write(STEPS_RVA, c.steps);
        game_write(HITS_RVA, c.hits);
    }
}

/// The court's constants from the game.
pub fn rules() -> rc2k_core::marks::pong::Rules {
    let f = |r: usize| f32::from_bits(get(r));
    rc2k_core::marks::pong::Rules {
        centre: std::array::from_fn(|n| f(CENTRE_RVA + 4 * n)),
        nudge: f(NUDGE_RVA),
        way_scale: [f(WAY_SCALE_RVA), f(WAY_SCALE_RVA + 4)],
        length: f(LENGTH_RVA),
        walls: std::array::from_fn(|n| f(WALLS_RVA + 4 * n)),
        start: std::array::from_fn(|n| get(START_RVA + 4 * n)),
    }
}

/// Runs `step` on the court loaded from the game and puts it back.
pub fn with_court<T>(step: impl FnOnce(&mut rc2k_core::marks::pong::Court) -> T) -> T {
    let before = court();
    let mut c = before;
    let out = step(&mut c);
    unsafe { put_court(&c, &before) };
    out
}

/// The float places of `regions()` (the court's floats), for the comparisons.
pub fn float_places() -> Vec<(Region, crate::service::floats::Tolerance)> {
    use crate::service::floats::FLOAT;
    [
        BALL,
        BALL_WAY,
        BALL_SIZE,
        BAT,
        BAT_SIZE,
        BALL_CENTRE,
        BAT_CENTRE,
        DIFFERENCE,
    ]
    .into_iter()
    .flatten()
    .map(|r| ((game_addr(r) as u32, 4), FLOAT))
    .collect()
}

/// The four compares of the ball against the bat (`marks::pong_ball_hits`,
/// `marks::pong_bat_back`): whether they overlap (no compare gave C0 alone) and the
/// last status.
pub fn overlap() -> (bool, u32) {
    let tests = [
        (BALL[0], BALL_SIZE[0], BAT[0]),
        (BAT[0], BAT_SIZE[0], BALL[0]),
        (BALL[1], BALL_SIZE[1], BAT[1]),
        (BAT[1], BAT_SIZE[1], BALL[1]),
    ];
    let mut s = 0;
    for (a, b, c) in tests {
        s = sum_compare(get(a), get(b), get(c));
        if s == 0x100 {
            return (false, s);
        }
    }
    (true, s)
}

/// Everything the game's pong code writes but the box's work floats and the vertices.
pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(BALL[0]) as u32, 0x7C),
        (game_addr(BEST_RVA) as u32, 0x10),
    ]
}

/// The `dispatch` of a converted court hook: the court's floats and eax / ebx (a way's bits)
/// within `service::floats`' tolerances.
pub fn dispatch_court(
    hook: &'static crate::framework::RawHook,
    r: &mut Regs,
    call_original: crate::screens::harness::CallOriginal,
    rust: crate::service::harness::Rust,
) {
    use crate::framework::{EAX, EBX};
    use crate::service::floats::{FLOAT, Floats, dispatch_state_floats};
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[(EAX, FLOAT), (EBX, FLOAT)],
    };
    dispatch_state_floats(hook, r, call_original, &regions(), &floats, rust);
}

/// The self-test of a converted court hook: `cases` cases, each the game's variables put
/// back and `prepare(case)`, at both test precisions; put back afterwards.
pub fn test_court(
    t: &mut crate::framework::SelfTest,
    name: &'static str,
    call_original: crate::screens::harness::CallOriginal,
    cases: u32,
    prepare: &dyn Fn(u32),
    rust: crate::service::harness::Rust,
) {
    use crate::framework::{EAX, EBX};
    use crate::service::floats::{FLOAT, Floats, at_test_precisions, check_state_floats};
    use crate::service::harness::{capture, restore};
    let regions = regions();
    let saved = capture(&regions);
    let places = float_places();
    let tolerant = Floats {
        memory: &places,
        regs: &[(EAX, FLOAT), (EBX, FLOAT)],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for case in 0..cases {
            check_state_floats(
                t,
                name,
                &format!("control word {word:#x}, case {case}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    restore(&regions, &saved);
                    prepare(case);
                },
                &regions,
                held,
                rust,
            );
        }
    });
    restore(&regions, &saved);
}

/// Ball and bat by lot for the tests (the limits stay the game's).
pub unsafe fn test_values(case: u32) {
    let k = case as f32;
    let put = |r: usize, v: f32| unsafe { game_write(r, v.to_bits()) };
    put(BALL[0], 290.0 + 4.0 * k);
    put(BALL[1], 95.0 + 13.0 * k);
    put(BALL_WAY[0], 3.0 - k);
    put(BALL_WAY[1], 1.5 - 0.75 * k);
    put(BALL_SIZE[0], 6.0);
    put(BALL_SIZE[1], 6.0);
    put(BAT[0], 300.0);
    put(BAT[1], 100.0);
    put(BAT_SIZE[0], 8.0);
    put(BAT_SIZE[1], 60.0);
    put(SAVED_BAT[0], 299.0 - k);
    put(SAVED_BAT[1], 98.0);
    unsafe {
        game_write(HITS_RVA, case * 60);
        game_write(BEST_RVA, 100u32);
        game_write(STEPS_RVA, 1 + case);
    }
}

/// A call as the tests note it: the place, eax .. esi, the x87 values.
pub type Noted = (u32, [u32; 5], Vec<[u8; 10]>);

pub static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

pub fn note(place: u32, r: &Regs, v: &[[u8; 10]]) {
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi], v.to_vec()));
}

pub fn take() -> Vec<Noted> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

/// For `install::stand_ins::set_handler`: a plain call noted.
pub fn note_plain(place: u32, r: &mut Regs) {
    note(place, r, &[]);
}

/// The place the box stand-in notes.
pub const BOX_PLACE: u32 = 2;

extern "C" fn note_four(r: *const Regs, v: *const [u8; 10]) {
    note(BOX_PLACE, unsafe { &*r }, unsafe {
        std::slice::from_raw_parts(v, 4)
    });
}

/// The tests' stand-in for `draw_pong_box`: its four values taken off and noted.
#[unsafe(naked)]
pub unsafe extern "C" fn box_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 40]",
        "fstp tbyte ptr [esp]",
        "fstp tbyte ptr [esp + 10]",
        "fstp tbyte ptr [esp + 20]",
        "fstp tbyte ptr [esp + 30]",
        "pushfd",
        "pushad",
        "mov eax, esp",
        "lea ecx, [esp + 36]",
        "push ecx",
        "push eax",
        "call {note}",
        "add esp, 8",
        "popad",
        "popfd",
        "lea esp, [esp + 40]",
        "ret",
        note = sym note_four,
    )
}
