//! `race::face_hidden` (0x454D92): whether a course face is left out (carry).
//!
//! In:  esi = the face (0x40 bytes), eax
//! Out: CF (read by `jb` in `sub_454C99`); eax the face's +2 word & `dword_64167C` when
//!      that test is reached and gives 0 or carry; the rest kept
//!
//! With the shown view's (`g_car` +0x684) `dword_606478` entry 0 and `dword_621480` set,
//! carry when the face's +2 word has a bit of `dword_64167C`. Otherwise, inside `pusha`, the
//! first point's (+4) scene record (made when not there, as `sub_44CF33`): `dword_635004` =
//! |x| and `dword_635010` = depth + `flt_63F188` (`fistp`), carry when the depth is below
//! |x| (the sign of the difference). See `race/frame_draw.rs`. Refuses (before anything)
//! no point records or points. Shadow mode: by state (the two dwords, the record,
//! `dword_62400C`) and the carry. Self-test: a test face on the game's points 0 .. 3 (records
//! cleared so they are made, and kept), the hiding test on and off; all put back
//! (skipped without points).

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::scene::project_point::{
    POINT_COUNT_RVA, POINTS_RVA, RECORDS_RVA, regions as point_regions,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_hidden", rva(0x454D92), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VIEW_HIDES: usize = rva(0x606478); // dword_606478[]
const SHOWN_VIEW: usize = rva(0x71A65C); // g_car +0x684
const HIDING: usize = rva(0x621480); // dword_621480
const MASK: usize = rva(0x64167C); // dword_64167C

fn regions(esi: u32) -> Vec<Region> {
    let mut v = vec![at(rva(0x635004)), at(rva(0x635010))];
    let point = unsafe { (esi.wrapping_add(4) as *const u32).read_unaligned() };
    v.extend(point_regions(point));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (eax, carry) = unsafe { face_hidden(r.esi, r.eax) }?;
    Ok(Left::carry(carry).with(EAX, eax))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.esi), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let there = unsafe {
        game_read::<u32>(RECORDS_RVA) != 0
            && game_read::<u32>(POINTS_RVA) != 0
            && game_read::<u32>(POINT_COUNT_RVA) >= 4
    };
    if !there {
        return t.skip("no scene points (no race loaded)");
    }
    let mut face = vec![0u8; 0x40];
    let f = face.as_mut_ptr() as u32;
    let view: u32 = unsafe { game_read(SHOWN_VIEW) };
    let mut kept: Vec<Region> = vec![at(HIDING), at(MASK), at(VIEW_HIDES + 4 * view as usize)];
    for p in 0..4 {
        kept.extend(point_regions(p));
    }
    kept.extend([at(rva(0x635004)), at(rva(0x635010))]);
    let saved = capture(&kept);
    for (point, hides, hiding, mask, word, made) in [
        (0u32, 0u32, 1u32, 0xFFFFu32, 0x0003u16, true),
        (1, 0, 1, 0x0004, 0x0003, true),
        (2, 1, 1, 0xFFFF, 0x0003, false),
        (3, 0, 0, 0xFFFF, 0x0003, true),
        (1, 0, 0, 0, 0, false),
    ] {
        unsafe { ((f + 4) as *mut u32).write_unaligned(point) };
        let regions = regions(f);
        check_state(
            &mut t,
            &format!(
                "point {point} hides {hides} hiding {hiding} mask {mask:#x} word {word:#x} made {made}"
            ),
            call_original,
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                game_write(VIEW_HIDES + 4 * view as usize, hides);
                game_write(HIDING, hiding);
                game_write(MASK, mask);
                ((f + 2) as *mut u16).write_unaligned(word);
                ((f + 4) as *mut u32).write_unaligned(point);
                if made {
                    let record = ((point & 0xFFF) << 5) + game_read::<u32>(RECORDS_RVA);
                    (record as *mut u32).write_unaligned(u32::MAX);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(face);
    t.finish()
}
