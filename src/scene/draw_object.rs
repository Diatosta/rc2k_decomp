//! `scene::draw_object` (0x45045D): draws one object, in seven steps.
//!
//! In:  nothing (`g_objectNumber` the object)
//! Out: the registers as the seven steps leave them, the last being
//!      `scene::draw_object_faces`
//!
//! eax = `g_objectNumber`; `scene::set_object_light`; `scene::set_object_material`;
//! `scene::set_object_fade` (which may make `g_objectNumber` another object);
//! ebx = `g_objectNumber`; `scene::place_object_points_any` (0x45BDE1, inlined: with
//! `g_use3dNow` `sub_45BF89`, else with `g_useSse` `sub_45C04E`, neither ported, else
//! `scene::place_object_points`); `scene::list_object_faces`;
//! `scene::sort_listed_faces`; `scene::draw_object_faces`.
//! One caller, `sub_4503A5`, which reads no register after it.
//! The Rust versions of the light, the material, the fade and the sorting keep the
//! registers their originals change, so what is left differs from the original's there.
//! A refusal of a later step comes with the earlier ones done; the original then does
//! them again.
//! Shadow mode runs the original alone (each step is compared in its own hook).
//! Self-test: a redirected copy whose seven calls note themselves.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_object_face::draw_object_face_in_game;
use crate::scene::draw_object_faces::draw_object_faces;
use crate::scene::list_object_face::list_object_face;
use crate::scene::list_object_faces::list_object_faces;
use crate::scene::place_object_points::place_object_points;
use crate::scene::set_object_fade::set_object_fade;
use crate::scene::set_object_light::set_object_light_in_game;
use crate::scene::set_object_material::set_object_material;
use crate::scene::sort_listed_faces::sort_listed_faces;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

/// Counts the calls that came through the hook: `scene::draw_shown_object`'s shadow
/// comparison asks it whether the original drew.
pub static ENTERED: AtomicU32 = AtomicU32::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_object", rva(0x45045D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const USE_3DNOW_RVA: usize = rva(0x621A5C); // g_use3dNow
const USE_SSE_RVA: usize = rva(0x621A58); // g_useSse
const PLACE_3DNOW_RVA: usize = rva(0x45BF89); // sub_45BF89
const PLACE_SSE_RVA: usize = rva(0x45C04E); // sub_45C04E

/// The seven calls, in their order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Light,
    Material,
    Fade,
    Points,
    List,
    Sort,
    Faces,
}

const STEPS: [(usize, Step); 7] = [
    (rva(0x45143A), Step::Light),
    (rva(0x45170E), Step::Material),
    (rva(0x451609), Step::Fade),
    (rva(0x45BDE1), Step::Points),
    (rva(0x45048C), Step::List),
    (rva(0x45098C), Step::Sort),
    (rva(0x450A4B), Step::Faces),
];

// ---- Rust version ----

/// `step`: what stands for each call, with the registers it is entered with.
pub unsafe fn draw_object(
    r: &mut Regs,
    step: &mut dyn FnMut(Step, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        r.eax = game_read(OBJECT_RVA);
        step(Step::Light, r)?;
        step(Step::Material, r)?;
        step(Step::Fade, r)?;
        r.ebx = game_read(OBJECT_RVA);
        step(Step::Points, r)?;
        step(Step::List, r)?;
        step(Step::Sort, r)?;
        step(Step::Faces, r)
    }
}

/// `scene::place_object_points_any` (0x45BDE1).
unsafe fn place_object_points_any(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(USE_3DNOW_RVA) != 0 {
            call_game(r, game_addr(PLACE_3DNOW_RVA));
        } else if game_read::<u32>(USE_SSE_RVA) != 0 {
            call_game(r, game_addr(PLACE_SSE_RVA));
        } else {
            place_object_points(r)?.apply(r);
        }
        Ok(())
    }
}

/// With the steps' Rust versions.
pub unsafe fn draw_object_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        draw_object(r, &mut |step, r| {
            match step {
                Step::Light => set_object_light_in_game()?,
                Step::Material => set_object_material()?,
                Step::Fade => set_object_fade()?,
                Step::Points => place_object_points_any(r)?,
                Step::List => list_object_faces(&mut |face| list_object_face(face))?.apply(r),
                Step::Sort => sort_listed_faces()?,
                Step::Faces => {
                    let entry = *r;
                    draw_object_faces(&mut |face| draw_object_face_in_game(face, &entry))?.apply(r)
                }
            }
            Ok(())
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    ENTERED.fetch_add(1, Ordering::Relaxed);
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { draw_object_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x2F;
const ENTRY_BYTES: [u8; 5] = [0xA1, 0xFC, 0x14, 0x62, 0x00];
const CALLS: [usize; 7] = [0x05, 0x0A, 0x0F, 0x1A, 0x1F, 0x24, 0x29];

/// Each call: its place, the object's number by then, and eax, ebx, ecx, esi as it
/// was entered with.
static CALLED: Mutex<Vec<[u32; 6]>> = Mutex::new(Vec::new());
/// What the stand-in of the fade makes the object's number (0: left alone).
static OTHER: Mutex<u32> = Mutex::new(0);

fn handler(place: u32, r: &mut Regs) {
    unsafe {
        CALLED
            .lock()
            .unwrap()
            .push([place, game_read(OBJECT_RVA), r.eax, r.ebx, r.ecx, r.esi]);
        let other = *OTHER.lock().unwrap();
        if place == 2 && other != 0 {
            game_write(OBJECT_RVA, other);
        }
        // Each leaves registers changed in its own way, as the real ones do.
        r.eax = r.eax.wrapping_mul(3).wrapping_add(place);
        r.ecx ^= 0x1111_1111 << (place % 4);
        r.esi = r.esi.wrapping_add(place * 0x10);
        if place % 2 == 1 {
            r.ebx = r.ebx.rotate_left(place);
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x45045D)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                STEPS
                    .iter()
                    .position(|&(address, _)| address == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let object = [at(OBJECT_RVA)];
    let saved = capture(&object);
    for number in [0u32, 1, 0x50, 0xF3, 0xFFFF_FFFF] {
        for other in [0u32, 7, 0x1234] {
            let run = |rust: bool| unsafe {
                game_write(OBJECT_RVA, number);
                *OTHER.lock().unwrap() = other;
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let mut r = Regs::SENTINEL;
                if rust {
                    let done = draw_object(&mut r, &mut |step, r| {
                        act(step as u32, r);
                        Ok(())
                    });
                    if done.is_err() {
                        r = Regs::default();
                    }
                } else {
                    call_game(&mut r, copy.addr());
                }
                (
                    [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                    game_read::<u32>(OBJECT_RVA),
                    CALLED.lock().unwrap().clone(),
                )
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.2.len() == 7, || {
                format!(
                    "object {number:#x}, other {other:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    restore(&object, &saved);
    t.finish()
}
