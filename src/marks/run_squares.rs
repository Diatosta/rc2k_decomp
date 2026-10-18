//! `marks::run_squares` (0x4BFFEC): the squares' frame, with the x87 rounding set to chop
//! throughout (put back at the end): `marks::squares_setup`; the vertices' texture places
//! from `dword_8BC222` ..; with `dword_7652A0` `marks::squares_spawn`; then each live square
//! item: with `dword_7652A0` its age (+8) plus `g_marksFrameTicks`, ended at 0xFA0 or more;
//! ended when its cell's byte in `g_marksEdgeMask` is set (the cell from its place, chopped,
//! shifted right 3); else drawn by `marks::draw_square` (st(1) its x, st(0) `flt_8BC236`
//! less its z; colour `g_viewLight`'s low 24 bits under the alpha: not (age over
//! `dword_8BC232`), shifted left 24). Last `scene::depth_buffer_on`.
//!
//! In:  nothing
//! Out: esi past the items, edi `g_polygonVertices`; the rest as the walk leaves them
//!
//! Caller: `sub_4BF60F` (0x4BF7E9). The spawning is its Rust version; the set-up, the
//! drawing and the depth buffer through `part` (1 `squares_setup`, 2 `draw_square` with
//! its two values, 3 `depth_buffer_on`), in the game their Rust versions. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose three calls go to stand-ins
//! (the drawing's taking its values off), against the Rust version, on the game's items
//! (as `expire_squares`') with ages by lot; put back.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::marks::expire_squares::{ITEM_LEN, ITEMS, ITEMS_RVA, test_items};
use crate::marks::squares_spawn::squares_spawn;
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::run_squares", rva(0x4BFFEC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: usize = rva(0x8BC222); // dword_8BC222 .. dword_8BC22E
const RUNNING_RVA: usize = rva(0x7652A0); // dword_7652A0
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks
const EDGE_RVA: usize = rva(0x809BC3); // g_marksEdgeMask
const FADE_RVA: usize = rva(0x8BC232); // dword_8BC232
const LIGHT_RVA: usize = rva(0x6415E4); // g_viewLight
const FAR_RVA: usize = rva(0x8BC236); // flt_8BC236
const LIFE: i32 = 0xFA0;

/// A call by place (see the header), with its x87 values (st(0) first).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs, &[[u8; 10]]) -> Result<(), Refused>;

/// `fld k; fsub z`, kept extended.
fn less(k: u32, z: u32) -> [u8; 10] {
    let (v, mut out) = ([k, z], [0u8; 10]);
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v}]",
            "fsub dword ptr [{v} + 4]",
            "fstp tbyte ptr [{o}]",
            v = in(reg) v.as_ptr(),
            o = in(reg) out.as_mut_ptr(),
            out("st(0)") _,
            options(nostack),
        )
    };
    out
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn run_squares(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let old = x87::set_chop();
    let done = unsafe { run(entry, part) };
    x87::set_control(old);
    done
}

unsafe fn run(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        part(1, &mut g, &[])?;
        g.edi = game_addr(VERTICES_RVA) as u32;
        let [a, b, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, b), (c, b), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [a, b, c, d];
        if game_read::<u32>(RUNNING_RVA) != 0 {
            g = squares_spawn(&g);
        }
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            g.esi = e;
            let get = |o: u32| ((e + o) as *const u32).read_unaligned();
            if get(0) != 1 {
                continue;
            }
            if game_read::<u32>(RUNNING_RVA) != 0 {
                g.edx = game_read(TICKS_RVA);
                let age = get(8).wrapping_add(g.edx);
                ((e + 8) as *mut u32).write_unaligned(age);
                if age as i32 >= LIFE {
                    (e as *mut u32).write_unaligned(0);
                    continue;
                }
            }
            g.edx = get(8);
            let x = x87::to_int(get(0x20)) as u32;
            let z = x87::to_int(get(0x24)) as u32;
            g.eax = (x >> 3) & 0xFF;
            g.ebx = (((z >> 3) & 0x7F) << 8) + g.eax;
            if game_read::<u8>(EDGE_RVA + g.ebx as usize) != 0 {
                (e as *mut u32).write_unaligned(0);
                continue;
            }
            let alpha = x87::int_over_int_to_int(g.edx as i32, game_read(FADE_RVA)) as u32;
            g.ecx = game_read::<u32>(LIGHT_RVA) & 0xFF_FFFF;
            g.edx = ((!alpha) << 24) | g.ecx;
            let values = [less(game_read(FAR_RVA), get(0x24)), extended(get(0x20))];
            part(2, &mut g, &values)?;
            g.esi = e;
        }
        g.esi = base + ITEMS * ITEM_LEN;
        part(3, &mut g, &[])?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs, v: &[[u8; 10]]) -> Result<(), Refused> {
    use crate::marks::{draw_square, squares_setup};
    unsafe {
        match place {
            1 => *r = squares_setup::squares_setup(r, &mut squares_setup::game_part)?,
            2 => *r = draw_square::draw_square(r, [v[0], v[1]], &mut draw_square::game_part)?,
            _ => crate::scene::depth_buffer_on::depth_buffer(r, true),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { run_squares(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// A call as the test notes it: the place, eax .. esi, the values.
type Noted = (u32, [u32; 5], Vec<[u8; 10]>);

static NOTED: Mutex<Vec<Noted>> = Mutex::new(Vec::new());

fn note(place: u32, r: &Regs, v: &[[u8; 10]]) {
    NOTED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi], v.to_vec()));
}

fn note_plain(place: u32, r: &mut Regs) {
    note(place, r, &[]);
}

extern "C" fn note_two(r: *const Regs, v: *const [u8; 10]) {
    note(2, unsafe { &*r }, unsafe {
        std::slice::from_raw_parts(v, 2)
    });
}

/// The test's stand-in for `draw_square`: its two values taken off and noted.
#[unsafe(naked)]
unsafe extern "C" fn draw_stand_in() {
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
            game_addr(rva(0x4BFFEC)),
            0x134,
            // lea esp, [esp - 4]; fstcw [esp]
            &[0x8D, 0x64, 0x24, 0xFC, 0x9B, 0xD9, 0x3C, 0x24],
            &[0x23, 0x66, 0x112, 0x12E],
            &|c| match c + 0x40_0000 {
                0x4BFEAC => Some(stand_in(1)),
                0x4BF80F => Some(draw_stand_in as *const () as usize),
                0x44E41F => Some(stand_in(3)),
                _ => None,
            },
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut regions: Vec<Region> = vec![
        (game_addr(ITEMS_RVA) as u32, (ITEMS * ITEM_LEN) as usize),
        (game_addr(VERTICES_RVA) as u32, 0x80),
    ];
    // what squares_spawn writes
    regions.extend(
        [
            rva(0x89D166),
            rva(0x89CA30),
            rva(0x89CA38),
            rva(0x8BC252),
            rva(0x8BC256),
            crate::scene::vector3::STATUS_RVA,
        ]
        .map(at),
    );
    let mut vars = regions.clone();
    vars.extend(
        [
            RUNNING_RVA,
            TICKS_RVA,
            FADE_RVA,
            rva(0x82FC56),
            rva(0x833D7E),
        ]
        .map(at),
    );
    vars.push((game_addr(EDGE_RVA) as u32, 0x8000));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..4u32 {
        let prepare = || unsafe {
            restore(&vars, &saved);
            test_items(case);
            let base = game_addr(ITEMS_RVA) as u32;
            for n in 0..ITEMS {
                ((base + n * ITEM_LEN + 8) as *mut u32)
                    .write_unaligned((n * 97 + case * 300) % 0x1100);
            }
            game_write(RUNNING_RVA, case & 1);
            game_write(TICKS_RVA, 0x40 + case * 0x100);
            game_write(FADE_RVA, 16 + case);
            let edge = game_addr(EDGE_RVA) as *mut u8;
            for n in 0..0x8000usize {
                *edge.add(n) = u8::from(n % (7 + case as usize) == 0);
            }
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
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
            run_squares(&Regs::SENTINEL, &mut |p, r, v| {
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
            format!(
                "case {case}: original {:x?} {} calls; rust {:x?} {} calls; calls same {}; state first differs at {:?}",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len(),
                theirs.1 == mine.1,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b)
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
