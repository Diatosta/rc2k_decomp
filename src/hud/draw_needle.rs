//! `hud::draw_needle` (0x4B6ECD): a HUD needle: its value (`dword_75C2E0`, as a float
//! through `dword_75C060`) held to `flt_75C018` at least; with `dword_75BFAC` held to
//! `flt_75C2C8` at most and made an angle over `flt_75C2CC` plus `flt_75C2BC`; without,
//! held to `flt_75C2D4`, and below `flt_75C2C4` over `flt_75C2D0` plus `flt_75C2BC`, else
//! less `flt_75C2C4` over `flt_75C2CC` plus `flt_75C2C0`; times `flt_75C010` times
//! `flt_75C014` (`fistp`), times -16 the turn; the needle's four corners (x `flt_75C2A8` /
//! `flt_75C2B0`, y `flt_75C2AC` / `flt_75C2B4`) by `hud::turned_vertex`, then
//! `hud::add_polygon` on four. (The holds by the compares' C0, an unordered compare
//! counting as below.)
//!
//! In:  nothing
//! Out: edx the turn, then as `add_polygon` leaves them
//!
//! Callers: the needles' set-ups (`hud::needle_*`). The corners and the polygon through
//! `part` (1 a corner with its two values, 2 the polygon), in the game their Rust
//! versions. Shadow mode runs the original alone (it draws). Self-test: a copy whose calls
//! go to stand-ins (the corners' taking their values off), against the Rust version,
//! values by lot on both ways.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::batch::POLYGON_COUNT_RVA;
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_needle", rva(0x4B6ECD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KIND_RVA: usize = rva(0x75BFAC); // dword_75BFAC
const VALUE_RVA: usize = rva(0x75C2E0); // dword_75C2E0
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
const LEAST_RVA: usize = rva(0x75C018); // flt_75C018
/// flt_75C2BC .. flt_75C2D4: base, base 2, split, scale, scale 2, most, most 2
const K_RVA: usize = rva(0x75C2BC);
const TURN_K_RVA: [usize; 2] = [rva(0x75C010), rva(0x75C014)];
const CORNERS_RVA: [usize; 4] = [rva(0x75C2A8), rva(0x75C2AC), rva(0x75C2B0), rva(0x75C2B4)];
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices

fn k(n: usize) -> u32 {
    unsafe { game_read(K_RVA + 4 * n) }
}

/// A call by place (see the header), with its x87 values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// `fld b; fld a; fcompp`: C0 (a below b, or unordered).
fn below(a: u32, b: u32) -> bool {
    let (v, mut s) = ([a, b], 0u16);
    unsafe {
        core::arch::asm!("fld dword ptr [{v} + 4]", "fld dword ptr [{v}]", "fcompp", "fnstsw word ptr [{s}]",
            v = in(reg) v.as_ptr(), s = in(reg) &raw mut s, out("st(0)") _, out("st(1)") _, options(nostack))
    };
    s & 0x100 != 0
}

/// `fld v [; fsub less]; fdiv over; fadd plus; fmul k0; fmul k1; fistp`.
fn turn_of(v: u32, less: Option<u32>, over: u32, plus: u32, k: [u32; 2]) -> u32 {
    let (a, mut out) = ([v, less.unwrap_or(0), over, plus, k[0], k[1]], 0u32);
    unsafe {
        if less.is_some() {
            core::arch::asm!("fld dword ptr [{a}]", "fsub dword ptr [{a} + 4]", "fdiv dword ptr [{a} + 8]",
                "fadd dword ptr [{a} + 12]", "fmul dword ptr [{a} + 16]", "fmul dword ptr [{a} + 20]", "fistp dword ptr [{o}]",
                a = in(reg) a.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
        } else {
            core::arch::asm!("fld dword ptr [{a}]", "fdiv dword ptr [{a} + 8]",
                "fadd dword ptr [{a} + 12]", "fmul dword ptr [{a} + 16]", "fmul dword ptr [{a} + 20]", "fistp dword ptr [{o}]",
                a = in(reg) a.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
        }
    };
    out
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_needle(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let held = |most: u32| {
            let mut v = x87::from_int(game_read(VALUE_RVA));
            game_write(SCRATCH_RVA, v);
            let least: u32 = game_read(LEAST_RVA);
            if below(v, least) {
                v = x87::loaded(least);
                game_write(SCRATCH_RVA, v);
            }
            if !below(v, most) {
                v = x87::loaded(most);
                game_write(SCRATCH_RVA, v);
            }
            v
        };
        let tk = TURN_K_RVA.map(|r| game_read::<u32>(r));
        let turn = if game_read::<u32>(KIND_RVA) != 0 {
            let v = held(k(3));
            turn_of(v, None, k(4), k(0), tk)
        } else {
            let v = held(k(6));
            if below(v, k(2)) {
                turn_of(v, None, k(5), k(0), tk)
            } else {
                turn_of(v, Some(k(2)), k(4), k(1), tk)
            }
        };
        game_write(SCRATCH_RVA, turn);
        g.edx = (turn << 4).wrapping_neg();
        let [x0, y0, x1, y1] = CORNERS_RVA.map(|r| game_read::<u32>(r));
        for (n, (x, y)) in [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
            .into_iter()
            .enumerate()
        {
            g.edi = game_addr(VERTICES_RVA) as u32 + 0x20 * n as u32;
            part(1, &mut g, &[extended(y), extended(x)])?;
        }
        game_write(POLYGON_COUNT_RVA, 4u32);
        part(2, &mut g, &[])?;
    }
    Ok(g)
}

/// In the game: the corner's and the polygon's Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::hud::{add_polygon as ap, turned_vertex as tv};
    unsafe {
        *r = match place {
            1 => tv::turned_vertex(r, [v[0], v[1]])?,
            _ => ap::add_polygon(r, &mut ap::game_part)?,
        };
    }
    Ok(())
}

/// In the game, for the needles' set-ups: this function's Rust version.
pub fn game_part_needle(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { draw_needle(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_needle(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// A call as the test notes it: the place, eax .. edi, the values.
type Noted = (u32, [u32; 6], Vec<[u8; 10]>);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

fn note(place: u32, r: &Regs, v: &[[u8; 10]]) {
    NOTED.lock().unwrap().push((
        place,
        [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi],
        v.to_vec(),
    ));
}

fn note_plain(place: u32, r: &mut Regs) {
    note(place, r, &[]);
}

extern "C" fn note_two(r: *const Regs, v: *const [u8; 10]) {
    note(1, unsafe { &*r }, unsafe {
        std::slice::from_raw_parts(v, 2)
    });
}

/// The test's stand-in for `turned_vertex`: its two values taken off and noted.
#[unsafe(naked)]
unsafe extern "C" fn corner_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 20]",
        "fstp tbyte ptr [esp]",
        "fstp tbyte ptr [esp + 10]",
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
        "lea esp, [esp + 20]",
        "ret",
        note = sym note_two,
    )
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    set_handler(note_plain);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x4B6ECD)),
            0x18B,
            // cmp dword_75BFAC, 0
            &[0x83, 0x3D, 0xAC, 0xBF, 0x75, 0x00, 0x00],
            &[0x131, 0x148, 0x15F, 0x176, 0x185],
            &|c| match c + 0x40_0000 {
                0x4B7058 => Some(corner_stand_in as *const () as usize),
                0x4B7413 => Some(stand_in(2)),
                _ => None,
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![at(SCRATCH_RVA), at(POLYGON_COUNT_RVA)];
    let mut vars = regions.clone();
    vars.extend([KIND_RVA, VALUE_RVA, LEAST_RVA, TURN_K_RVA[0], TURN_K_RVA[1]].map(at));
    vars.push((game_addr(K_RVA) as u32, 0x1C));
    vars.push((game_addr(CORNERS_RVA[0]) as u32, 0x10));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..10u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            let f = |r: usize, v: f32| game_write(r, v.to_bits());
            game_write(KIND_RVA, case & 1);
            game_write(
                VALUE_RVA,
                [0u32, 30, 90, 150, 220, 400, 0xFFFF_FFF0, 75, 120, 9000][case as usize],
            );
            f(LEAST_RVA, 10.0);
            for (n, v) in [0.5f32, 1.25, 100.0, 200.0, 80.0, 40.0, 300.0]
                .into_iter()
                .enumerate()
            {
                f(K_RVA + 4 * n, v);
            }
            f(TURN_K_RVA[0], 0.7);
            f(TURN_K_RVA[1], 651.9);
            for (n, v) in [-4.0f32, -40.0, 4.0, 6.0].into_iter().enumerate() {
                f(CORNERS_RVA[n], v);
            }
        };
        prepare();
        NOTED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (
            Some(regs(&ro)),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        prepare();
        let mine = unsafe {
            draw_needle(&Regs::SENTINEL, &mut |p, r, v| {
                note(p, r, v);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            std::mem::take(&mut *NOTED.lock().unwrap()),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!("case {case}: original {:x?} {} calls; rust {:x?} {} calls; same calls {}; same state {}", theirs.0, theirs.1.len(), mine.0, mine.1.len(), theirs.1 == mine.1, theirs.2 == mine.2)
        });
    }
    restore(&vars, &saved);
    t.finish()
}
