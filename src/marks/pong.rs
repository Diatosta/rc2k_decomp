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
