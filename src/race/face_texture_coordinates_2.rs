//! `race::face_texture_coordinates_2` (0x45349F): the second triangle's texture working values (size word,
//! mapping, the four corners' u, v; see `face_texture_coordinates` in
//! `race/course_faces.rs`).
//!
//! In:  nothing (`dword_63DA38`, `dword_63DA3C`, the corners)
//! Out: eax the size word, ebx the texture record (a `mov` / `call` follows)
//!
//! Caller: `sub_452C6F`. Shadow mode: by state. Self-test: the race's textures, each
//! order, the record's size and mapping set for the cases (put back), corners on test
//! scene points (one before them for a missing fourth); skipped without a race.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::race::course_faces::{SECOND_SET, face_texture_coordinates};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::face_texture_coordinates_2",
    rva(0x45349F),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let out = unsafe { face_texture_coordinates(&SECOND_SET, r) };
    Ok(Left::nothing().with(EAX, out.eax).with(EBX, out.ebx))
}

fn regions() -> Vec<Region> {
    vec![
        at(rva(SECOND_SET.size)),
        at(rva(SECOND_SET.mapping)),
        at(rva(SECOND_SET.plain)),
        (game_addr(rva(SECOND_SET.uv)) as u32, 0x20),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x621480)) } == 0 {
        return t.skip("no race loaded");
    }
    let regions = regions();
    let infos = game_addr(rva(0x979F30)) as u32;
    let texture = 3u32;
    let info = infos + (texture << 6);
    let mut kept = regions.clone();
    kept.extend([
        at(rva(SECOND_SET.texture)),
        at(rva(SECOND_SET.order)),
        at(rva(0x5CCCC0)),
        (game_addr(rva(0x63D9EC)) as u32, 0x10),
        (info + 0x20, 4),
        (info + 0x2C, 4),
    ]);
    let saved = capture(&kept);
    let mut points = vec![0u32; 3 * 6];
    for (k, v) in points.iter_mut().enumerate() {
        *v = (k as u32).wrapping_mul(0x0123_4567) ^ 0x00AB_CDEF;
    }
    let p = points.as_ptr() as u32 + 12;
    for (order, size, mapping, fourth) in [
        (0u32, 0x100u32, 0i8, 3u32),
        (1, 0x190, 0, 3),
        (2, 0x31F, -1, 3),
        (3, 0x320, 1, 3),
        (5, 0x200, 2, u32::MAX),
        (6, 0x80, 7, 2),
        (7, 0x400, 0, 3),
    ] {
        check_state(
            &mut t,
            &format!("order {order} size {size:#x} mapping {mapping} fourth {fourth:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(SECOND_SET.texture), texture);
                game_write(rva(SECOND_SET.order), order);
                game_write(rva(0x5CCCC0), p);
                ((info + 0x20) as *mut u32).write_unaligned(size);
                ((info + 0x2D) as *mut i8).write(mapping);
                for (n, v) in [0u32, 1, 2, fourth].iter().enumerate() {
                    game_write(rva(0x63D9EC + 4 * n), *v);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(points);
    t.finish()
}
