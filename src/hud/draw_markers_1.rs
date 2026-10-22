//! `hud::draw_markers_1` (0x4B56E3): the two players' markers along the top in a split
//! layout: rectangle 0x5C of `dword_75B940` at the record's +0x6C (then +0x70) over +0x68
//! times `flt_75C0A8` plus `flt_75C0AC` (`fistp` through `dword_75C060`), coloured
//! `dword_75BF50` (then `dword_75BF54`), at rows 1 and 0xE, scaled 0x100 by 0x78 over
//! `flt_75C00C` (`hud::add_scaled_sprite`, the scales taken off after).
//!
//! In:  ebp the HUD's record
//! Out: eax 0x5C0, ebx the second's x, ecx its row, esi the rectangle
//!
//! Caller: layout 1's drawer (0x4B2BF4); `hud::draw_markers_2` is its twin (rows 0x11F,
//! 0x12C). The sprites through `part` (1 with the two scales), in the game
//! `add_scaled_sprite`'s Rust version. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to a stand-in reading the scales, against the Rust
//! version, shares by lot, at both test control words.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::draw_marker::RECTS_RVA;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::hud::quads;
use rc2k_core::math::extended::{from_f64, to_f64};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_markers_1", rva(0x4B56E3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const COLOURS_RVA: [usize; 2] = [rva(0x75BF50), rva(0x75BF54)];
const K_RVA: [usize; 2] = [rva(0x75C0A8), rva(0x75C0AC)];
const SCRATCH_RVA: usize = rva(0x75C060); // dword_75C060
const UV_SCALE_RVA: usize = rva(0x75C00C); // flt_75C00C
const RECT: u32 = 0x5C;

/// A call by place: 1 `add_scaled_sprite`, with its two scales (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// `fild i; fdiv k`, kept extended (`quads::int_over`: exact at the game's double
/// precision).
pub fn int_over_80(i: u32, k: u32) -> [u8; 10] {
    from_f64(quads::int_over(i, f32::from_bits(k)))
}

/// An 80-bit value taken off into a float (`fstp dword`).
pub fn taken_off(v: [u8; 10]) -> u32 {
    (to_f64(v) as f32).to_bits()
}

/// The two markers at rows `rows` (see the header).
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}`
pub unsafe fn markers_with(entry: &Regs, rows: [u32; 2], part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((entry.ebp + o) as *const u32).read_unaligned();
        let k: u32 = game_read(UV_SCALE_RVA);
        for (n, field) in [0x6Cu32, 0x70].into_iter().enumerate() {
            game_write(QUAD_RVA + 0x14, game_read::<u32>(COLOURS_RVA[n]));
            let f = f32::from_bits;
            let x = quads::marker_x(
                f(get(field)),
                f(get(0x68)),
                f(game_read(K_RVA[0])),
                f(game_read(K_RVA[1])),
            );
            game_write(SCRATCH_RVA, x);
            g.eax = RECT * 0x10;
            g.ebx = x;
            g.ecx = rows[n];
            g.esi = game_addr(RECTS_RVA) as u32 + g.eax;
            game_write(SCRATCH_RVA, 0x78u32);
            let st1 = int_over_80(0x78, k);
            game_write(SCRATCH_RVA, 0x100u32);
            let st0 = int_over_80(0x100, k);
            part(1, &mut g, &[st0, st1])?;
            // the two scales taken off into dword_75C060 (st(0), then st(1))
            game_write(SCRATCH_RVA, taken_off(st1));
        }
    }
    Ok(g)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::{marker_x, int_over}` (through `markers_with`)
pub unsafe fn draw_markers_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { markers_with(entry, [1, 0xE], part) }
}

/// In the game: `add_scaled_sprite`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::hud::add_scaled_sprite as s;
    *r = unsafe {
        s::add_scaled_sprite(
            r,
            [v[0], v[1]],
            &mut crate::hud::add_triangle::game_part_quad,
        )
    }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_markers_1(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_markers_1(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}

/// A call as the tests note it: the place, eax .. esi, the values.
pub type Noted = (u32, [u32; 5], Vec<[u8; 10]>);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

pub fn note(place: u32, r: &Regs, v: &[[u8; 10]]) {
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi], v.to_vec()));
}

pub fn take() -> Vec<Noted> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}

extern "C" fn note_kept(r: *const Regs, v: *const [u8; 10]) {
    note(1, unsafe { &*r }, unsafe {
        std::slice::from_raw_parts(v, 2)
    });
}

/// The tests' stand-in for `add_scaled_sprite`: its two values read, not popped.
#[unsafe(naked)]
pub unsafe extern "C" fn kept_stand_in() {
    core::arch::naked_asm!(
        "lea esp, [esp - 20]",
        "fld st(0)",
        "fstp tbyte ptr [esp]",
        "fld st(1)",
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
        note = sym note_kept,
    )
}

/// The test of the markers: a copy whose calls go to the stand-in, against `rust`.
pub fn test_markers(
    hook: &RawHook,
    address: usize,
    rust: &dyn Fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(address)),
            0xF5,
            // mov eax, dword_75BF50
            &[0xA1, 0x50, 0xBF, 0x75, 0x00],
            &[0x69, 0xE3],
            &|c| (c + 0x40_0000 == 0x4B716A).then_some(kept_stand_in as *const () as usize),
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![at(QUAD_RVA + 0x14), at(SCRATCH_RVA)];
    let mut vars = regions.clone();
    vars.extend(
        [
            COLOURS_RVA[0],
            COLOURS_RVA[1],
            K_RVA[0],
            K_RVA[1],
            UV_SCALE_RVA,
        ]
        .map(at),
    );
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for (word, case) in TEST_CONTROL_WORDS
        .into_iter()
        .flat_map(|w| (0..4u32).map(move |c| (w, c)))
    {
        let mut record = [0u32; 0x20];
        record[0x68 / 4] = 1000.0f32.to_bits();
        record[0x6C / 4] = (case as f32 * 333.3).to_bits();
        record[0x70 / 4] = (999.0 - case as f32 * 211.0).to_bits();
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(COLOURS_RVA[0], 0x00FF_8040u32 + case);
            game_write(COLOURS_RVA[1], 0x0040_80FFu32 + case);
            game_write(K_RVA[0], 560.0f32.to_bits());
            game_write(K_RVA[1], 40.0f32.to_bits());
            game_write(UV_SCALE_RVA, 256.0f32.to_bits());
        };
        prepare();
        take();
        let mut ro = before;
        with_control_word(word, &mut || unsafe {
            crate::framework::call_game(&mut ro, copy.addr())
        });
        let theirs = (Some(regs(&ro)), take(), capture(&regions));
        prepare();
        let mine = rust(&before, &mut |p, r, v| {
            note(p, r, v);
            Ok(())
        });
        let mine = (mine.as_ref().ok().map(regs), take(), capture(&regions));
        t.check(theirs == mine, || {
            format!(
                "control word {word:#x}, case {case}: original {:x?} {:?}; rust {:x?} {:?}; same state {}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.2 == mine.2
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}

fn self_test() -> (usize, usize) {
    test_markers(&HOOK, 0x4B56E3, &|r, p| unsafe { draw_markers_1(r, p) })
}
