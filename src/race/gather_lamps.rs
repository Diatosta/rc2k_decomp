//! `race::gather_lamps` (0x454E08): the lamps among the lit faces (object 0x3B), at most
//! 16, into `g_lamps` (+0 the face's first point, +4 its +0x11 byte, +8 0xFFFFFF), each with
//! a depth entry (`add_lamp_entry`, 0x4573FA: the point through `g_viewMatrix`, its depth
//! rounded from the x87 into the key, the entry's second word `g_lampCount` + 0x3000000,
//! `dword_641590` + 1).
//!
//! In:  nothing
//! Out: the registers as it leaves them (ecx the lit faces' count; ebx, esi, eax, edi the
//!      last face's) (a `call` follows)
//!
//! See `race/frame_draw.rs`. `sub_4573FA` is covered here: its only caller is this
//! function. Caller: `sub_4557AB` (0x455821), each view drawn. Refuses (before anything)
//! more than 0x100 lit faces (`g_litFaces` holds 0x100) or a depth list without room for
//! 16. Shadow mode: by state (the lamps, the lamp count, the depth list and count, the
//! key, `dword_63501C` .. 0x635024, `dword_641590`). Self-test: test faces
//! (`g_courseFaces` pointed at them) on the game's points, lit-face lists with lamps and
//! not, more than 16; all put back (skipped without points).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::gather_lamps", rva(0x454E08), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x63E05C, 0x100),
        at(LAMP_COUNT_RVA),
        at(DEPTH_COUNT_RVA),
        g(0x9D3730, 0x8000),
        at(FACE_KEY_RVA),
        g(0x63501C, 0xC),
        g(0x641590, 4),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { gather_lamps(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

const FACES: usize = rva(0x5CCCBC); // g_courseFaces
const LIT_COUNT: usize = rva(0x6D3620); // g_litFaceCount
const LIT: usize = rva(0x6D3220); // g_litFaces

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let points: u32 = unsafe { game_read(crate::scene::project_point::POINTS_RVA) };
    if points == 0 || unsafe { game_read::<u32>(crate::scene::project_point::POINT_COUNT_RVA) } < 20
    {
        return t.skip("no scene points (no race loaded)");
    }
    let mut faces = vec![0u8; 0x40 * 20];
    let base = faces.as_mut_ptr() as u32;
    for f in 0..20u32 {
        let at = base + 0x40 * f;
        unsafe {
            ((at + 1) as *mut u8).write(if f % 3 == 0 { 0x22 } else { 0x3B });
            ((at + 4) as *mut u32).write_unaligned(f);
            ((at + 0x11) as *mut u8).write(f as u8 * 7);
            ((at + 0x2C) as *mut u32).write_unaligned(0x1000 + f);
        }
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(FACES), at(LIT_COUNT), (game_addr(LIT) as u32, 0x100)]);
    let saved = capture(&kept);
    for (count, depth) in [(0u32, 0u32), (5, 0x10), (40, 0), (19, 0xFE0)] {
        check_state(
            &mut t,
            &format!("lit faces {count} depth entries {depth:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(FACES, base);
                game_write(LIT_COUNT, count);
                game_write(DEPTH_COUNT_RVA, depth);
                for k in 0..count {
                    game_write(LIT + 4 * k as usize, (k * 7) % 20);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(faces);
    t.finish()
}
